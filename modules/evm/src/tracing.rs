use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener, Event};
use evm_runtime::Opcode;
use sp_std::{cell::RefCell, rc::Rc, vec::Vec};

struct ListenerProxy<T>(pub Rc<RefCell<T>>);
impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
	fn event(&mut self, event: Event) {
		self.0.borrow_mut().event(event);
	}
}

pub struct EvmTracer{
	stack: Vec<u32>,
	capture_result_flag: bool
}

impl EvmTracer {
	pub fn new() -> Self {
		Self { stack: Vec::new(), capture_result_flag: false }
	}

	pub fn trace<R, F: FnOnce() -> R>(self, f: F) -> R {
		let wrapped = Rc::new(RefCell::new(self));
		let mut runtime = ListenerProxy(Rc::clone(&wrapped));
		let f = || runtime_using(&mut runtime, f);
		f()
	}
}

/// `CREATE`
pub const CREATE: Opcode = Opcode(0xf0);
/// `CREATE2`
pub const CREATE2: Opcode = Opcode(0xf5);

impl RuntimeListener for EvmTracer {
	/// Proxies `evm_runtime::tracing::Event` to the host.
	fn event(&mut self, event: Event) {
		if self.capture_result_flag {
			self.stack.push(2u32);
			self.capture_result_flag = false;
			log::debug!(
				target: "evm-tracing",
				"result captured {:?}",
				event
			);
			return;
		}

		match event {
			Event::Step{context: _, opcode, position: _, stack: _, memory: _} => {
				match opcode {
					CREATE | CREATE2 => {
						log::debug!(
							target: "evm-tracing",
							"CREATE opcode matched {:?}",
							event
						);
						self.stack.push(1u32);
						self.capture_result_flag = true;
					},
					_ => {}
				}
			},
			_ => {}
		}
	}
}

