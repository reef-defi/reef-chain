use crate::Balance;
use codec::{Decode, Encode};
use evm::ExitReason;
use sp_core::{H160, U256};
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub use evm::backend::{Basic as Account, Log};
pub use evm::Config;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};


/// Evm Address.
pub type EvmAddress = sp_core::H160;

#[derive(Clone, Eq, PartialEq, Encode, Decode, Default)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
/// External input from the transaction.
pub struct Vicinity {
	/// Current transaction gas price.
	pub gas_price: U256,
	/// Origin of the transaction.
	pub origin: EvmAddress,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CreateInfo {
	pub exit_reason: ExitReason,
	pub address: EvmAddress,
	pub output: Vec<u8>,
	pub used_gas: U256,
	pub used_storage: i32,
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct CallInfo {
	pub exit_reason: ExitReason,
	pub output: Vec<u8>,
	pub used_gas: U256,
	pub used_storage: i32,
}
/// A mapping between `AccountId` and `EvmAddress`.
pub trait AddressMapping<AccountId> {
	fn get_account_id(evm: &EvmAddress) -> AccountId;
	fn get_evm_address(account_id: &AccountId) -> Option<EvmAddress>;
	fn get_or_create_evm_address(account_id: &AccountId) -> EvmAddress;
	fn get_default_evm_address(account_id: &AccountId) -> EvmAddress;
	fn is_linked(account_id: &AccountId, evm: &EvmAddress) -> bool;
}

#[derive(Clone, Eq, PartialEq, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct EstimateResourcesRequest {
	/// From
	pub from: Option<H160>,
	/// To
	pub to: Option<H160>,
	/// Gas Limit
	pub gas_limit: Option<u64>,
	/// Storage Limit
	pub storage_limit: Option<u32>,
	/// Value
	pub value: Option<Balance>,
	/// Data
	pub data: Option<Vec<u8>>,
}
