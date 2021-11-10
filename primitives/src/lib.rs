#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unnecessary_cast)]
#![allow(clippy::upper_case_acronyms)]

pub mod evm;
pub mod mocks;

use crate::evm::EvmAddress;

use codec::{Decode, Encode};
use sp_runtime::{
	generic,
	traits::{BlakeTwo256, IdentifyAccount, Verify},
	MultiSignature, RuntimeDebug,
};
use sp_std::convert::{Into, TryFrom, TryInto};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// System contracts
/// 0x00000000000...
pub const SYSTEM_CONTRACT_ADDRESS_PREFIX: [u8; 11] = [0u8; 11];
/// Ethereum precompiles
/// 0 - 0x400
/// Reef precompiles
/// 0x400 - 0x800
pub const PRECOMPILE_ADDRESS_START: u64 = 0x400;
/// Predeployed system contracts
/// 0x800 - 0x1000
pub const PREDEPLOY_ADDRESS_START: u64 = 0x800;
/// Network contracts
/// 0x1000 - 0x01000000
pub const NETWORK_CONTRACT_START: u64 = 0x1000;
/// Mirrored Tokens
/// 0x01000000
pub const MIRRORED_TOKENS_ADDRESS_START: u64 = 0x01000000;

/// Amounts
pub mod currency {
	use super::Balance;

	pub const DOLLARS: Balance = 1_000_000_000_000_000_000;
	pub const CENTS: Balance = DOLLARS / 100;

	pub const REEF: Balance = DOLLARS;
	pub const MILLI_REEF: Balance = REEF / 1_000;
	pub const MICRO_REEF: Balance = REEF / 1_000_000;

	// TODO validate deposit fucntion logic
	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		items as Balance * 20 * REEF + (bytes as Balance) * 100 * MILLI_REEF
	}
}

/// Time and blocks.
pub mod time {
	use super::{BlockNumber, Moment};

	/// 10 second block times
	pub const SECS_PER_BLOCK: Moment = 10;
	pub const MILLISECS_PER_BLOCK: Moment = SECS_PER_BLOCK * 1000;

	// These time units are defined in number of blocks.
	pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

	// 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
	pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

	pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 1 * HOURS;
	pub const EPOCH_DURATION_IN_SLOTS: u64 = {
		const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

		(EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
	};
}


/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on
/// the chain.
pub type Signature = MultiSignature;

/// Alias to the public key used for this chain, actually a `MultiSigner`. Like
/// the signature, this also isn't a fixed size when encoded, as different
/// cryptos have different size public keys.
pub type AccountPublic = <Signature as Verify>::Signer;

/// Alias to the opaque account ID type for this chain, actually a
/// `AccountId32`. This is always 32 bytes.
pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of
/// them.
pub type AccountIndex = u32;

/// Index of a transaction in the chain. 32-bit should be plenty.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// An instant or duration in time.
pub type Moment = u64;

/// Counter for the number of eras that have passed.
pub type EraIndex = u32;

/// Balance of an account.
pub type Balance = u128;

/// Signed version of Balance
pub type Amount = i128;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenSymbol {
	REEF = 0,
	RUSD = 1,
}

impl TryFrom<u8> for TokenSymbol {
	type Error = ();

	fn try_from(v: u8) -> Result<Self, Self::Error> {
		match v {
			0 => Ok(TokenSymbol::REEF),
			1 => Ok(TokenSymbol::RUSD),
			_ => Err(()),
		}
	}
}

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum CurrencyId {
	Token(TokenSymbol),
	DEXShare(TokenSymbol, TokenSymbol),
	ERC20(EvmAddress),
}

impl CurrencyId {
	pub fn is_token_currency_id(&self) -> bool {
		matches!(self, CurrencyId::Token(_))
	}

	pub fn is_dex_share_currency_id(&self) -> bool {
		matches!(self, CurrencyId::DEXShare(_, _))
	}

	pub fn split_dex_share_currency_id(&self) -> Option<(Self, Self)> {
		match self {
			CurrencyId::DEXShare(token_symbol_0, token_symbol_1) => {
				Some((CurrencyId::Token(*token_symbol_0), CurrencyId::Token(*token_symbol_1)))
			}
			_ => None,
		}
	}

	pub fn join_dex_share_currency_id(currency_id_0: Self, currency_id_1: Self) -> Option<Self> {
		match (currency_id_0, currency_id_1) {
			(CurrencyId::Token(token_symbol_0), CurrencyId::Token(token_symbol_1)) => {
				Some(CurrencyId::DEXShare(token_symbol_0, token_symbol_1))
			}
			_ => None,
		}
	}
}

/// Note the pre-deployed ERC20 contracts depend on `CurrencyId` implementation,
/// and need to be updated if any change.
impl TryFrom<[u8; 32]> for CurrencyId {
	type Error = ();

	fn try_from(v: [u8; 32]) -> Result<Self, Self::Error> {
		if !v.starts_with(&[0u8; 29][..]) {
			return Err(());
		}

		// token
		if v[29] == 0 && v[31] == 0 {
			return v[30].try_into().map(CurrencyId::Token);
		}

		// DEX share
		if v[29] == 1 {
			let left = v[30].try_into()?;
			let right = v[31].try_into()?;
			return Ok(CurrencyId::DEXShare(left, right));
		}

		Err(())
	}
}

/// Note the pre-deployed ERC20 contracts depend on `CurrencyId` implementation,
/// and need to be updated if any change.
impl From<CurrencyId> for [u8; 32] {
	fn from(val: CurrencyId) -> Self {
		let mut bytes = [0u8; 32];
		match val {
			CurrencyId::Token(token) => {
				bytes[30] = token as u8;
			}
			CurrencyId::DEXShare(left, right) => {
				bytes[29] = 1;
				bytes[30] = left as u8;
				bytes[31] = right as u8;
			}
			_ => {}
		}
		bytes
	}
}


#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum AuthoritysOriginId {
	Root,
}
