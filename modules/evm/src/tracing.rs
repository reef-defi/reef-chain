use evm_runtime::tracing::{using as runtime_using, EventListener as RuntimeListener, Event};
use sp_std::{cell::RefCell, rc::Rc, vec::Vec};

struct ListenerProxy<T>(pub Rc<RefCell<T>>);
impl<T: RuntimeListener> RuntimeListener for ListenerProxy<T> {
	fn event(&mut self, event: Event) {
		self.0.borrow_mut().event(event);
	}
}

pub struct EvmTracer;

impl EvmTracer {
	pub fn new() -> Self {
		Self {}
	}

	pub fn trace<R, F: FnOnce() -> R>(self, f: F) -> R {
		let wrapped = Rc::new(RefCell::new(self));
		let mut runtime = ListenerProxy(Rc::clone(&wrapped));
		let f = || runtime_using(&mut runtime, f);
		f()
	}
}

impl RuntimeListener for EvmTracer {
	/// Proxies `evm_runtime::tracing::Event` to the host.
	fn event(&mut self, event: Event) {
		log::debug!(
			target: "evm-tracing",
			"Runtime event: {:?}",
			event
		);
	}
}
