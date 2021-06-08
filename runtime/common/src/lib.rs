//! Common runtime code

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, WEIGHT_PER_SECOND},
		DispatchClass, Weight,
	},
};
use frame_system::limits;
use primitives::{PRECOMPILE_ADDRESS_START, PREDEPLOY_ADDRESS_START};
use sp_core::H160;
use sp_runtime::{
	traits::{Convert},
	Perbill,
	FixedU128
};

use static_assertions::const_assert;

pub use module_support::{PrecompileCallerFilter};

pub mod precompile;
pub use precompile::{
	AllPrecompiles,
	MultiCurrencyPrecompile,
	ScheduleCallPrecompile,
	StateRentPrecompile,
};

pub type Price = FixedU128;
pub type Ratio = FixedU128;
pub type Rate = FixedU128;

pub const SYSTEM_CONTRACT_LEADING_ZERO_BYTES: usize = 12;

/// Check if the given `address` is a system contract.
///
/// It's system contract if the address starts with 12 zero bytes.
pub fn is_system_contract(address: H160) -> bool {
	address[..SYSTEM_CONTRACT_LEADING_ZERO_BYTES] == [0u8; SYSTEM_CONTRACT_LEADING_ZERO_BYTES]
}

pub fn is_core_precompile(address: H160) -> bool {
	address >= H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)
		&& address < H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)
}

/// The call is allowed only if caller is a system contract.
pub struct SystemContractsFilter;
impl PrecompileCallerFilter for SystemContractsFilter {
	fn is_allowed(caller: H160) -> bool {
		is_system_contract(caller)
	}
}

/// Convert gas to weight
pub struct GasToWeight;
impl Convert<u64, Weight> for GasToWeight {
	fn convert(a: u64) -> u64 {
		a as Weight
	}
}

pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_perthousand(25);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be
/// used by  Operational  extrinsics.
const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 5 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

const_assert!(NORMAL_DISPATCH_RATIO.deconstruct() >= AVERAGE_ON_INITIALIZE_RATIO.deconstruct());

parameter_types! {
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: limits::BlockLength =
		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	/// Block weights base values and limits.
	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have an extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
}

parameter_types! {
	/// A limit for off-chain phragmen unsigned solution submission.
	///
	/// We want to keep it as high as possible, but can't risk having it reject,
	/// so we always subtract the base block execution weight.
	pub OffchainSolutionWeightLimit: Weight = BlockWeights::get()
		.get(DispatchClass::Normal)
		.max_extrinsic
		.expect("Normal extrinsics have weight limit configured by default; qed")
		.saturating_sub(BlockExecutionWeight::get());
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn system_contracts_filter_works() {
		assert!(SystemContractsFilter::is_allowed(H160::from_low_u64_be(1)));

		let mut max_allowed_addr = [0u8; 20];
		max_allowed_addr[SYSTEM_CONTRACT_LEADING_ZERO_BYTES] = 127u8;
		assert!(SystemContractsFilter::is_allowed(max_allowed_addr.into()));

		let mut min_blocked_addr = [0u8; 20];
		min_blocked_addr[SYSTEM_CONTRACT_LEADING_ZERO_BYTES - 1] = 1u8;
		assert!(!SystemContractsFilter::is_allowed(min_blocked_addr.into()));
	}

	#[test]
	fn is_system_contract_works() {
		assert!(is_system_contract(H160::from_low_u64_be(0)));
		assert!(is_system_contract(H160::from_low_u64_be(u64::max_value())));

		let mut bytes = [0u8; 20];
		bytes[SYSTEM_CONTRACT_LEADING_ZERO_BYTES - 1] = 1u8;

		assert!(!is_system_contract(bytes.into()));

		bytes = [0u8; 20];
		bytes[0] = 1u8;

		assert!(!is_system_contract(bytes.into()));
	}

	#[test]
	fn is_core_precompile_works() {
		assert!(!is_core_precompile(H160::from_low_u64_be(0)));
		assert!(!is_core_precompile(H160::from_low_u64_be(
			PRECOMPILE_ADDRESS_START - 1
		)));
		assert!(is_core_precompile(H160::from_low_u64_be(PRECOMPILE_ADDRESS_START)));
		assert!(is_core_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START - 1)));
		assert!(!is_core_precompile(H160::from_low_u64_be(PREDEPLOY_ADDRESS_START)));
		assert!(!is_core_precompile([1u8; 20].into()));
	}
}
