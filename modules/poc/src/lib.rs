//! # Proof of Commitment
//!
//! Stake tokens with extremely long unbonding period,
//! to obtain the Technical Council election voting rights.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, IsType, },
	weights::Weight,
	ensure,
	transactional,
};
use frame_system::pallet_prelude::*;

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

mod mock;
mod tests;

pub use module::*;

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone)]
pub enum LockState<BlockNumber> {
	Committed,
	Unbonding(BlockNumber),
	Free
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone)]
pub enum LockDuration {
	TenYears,
	OneYear,
	OneMonth,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Commitment<AccountId, BalanceOf, BlockNumber> {
	pub state: LockState<BlockNumber>,
	pub duration: LockDuration,
	pub amount: BalanceOf,
	pub candidate: AccountId,
}

impl<BlockNumber> Default for LockState<BlockNumber> {
	fn default() -> Self {
		Self::Committed
	}
}

impl Default for LockDuration {
	fn default() -> Self {
		Self::OneMonth
	}
}


// config
// % of total supply that can be locked

// voting
// only LockState::Committed funds can vote
// lock duration multiplier

// council
// how does council get paid?
// membership pallet?
// how often are elections?

pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type CommitmentOf<T> =
	Commitment<
		<T as frame_system::Config>::AccountId,
		BalanceOf<T>,
		<T as frame_system::Config>::BlockNumber,
	>;

#[frame_support::pallet]
pub mod module {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Some wrong behavior
		Wrong,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId")]
	pub enum Event<T: Config> {
		/// Dummy event, just here so there's a generic type that's used.
		Dummy(T::AccountId),
	}

	#[pallet::storage]
	#[pallet::getter(fn commitments)]
	pub(crate) type Commitments<T: Config> = StorageMap<_,
		Blake2_128Concat, T::AccountId, CommitmentOf<T>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_initialize(_n: T::BlockNumber) -> Weight {
			// Dummy::<T>::put(T::Balance::from(10));
			10
		}

		fn on_finalize(_n: T::BlockNumber) {
			// Dummy::<T>::put(T::Balance::from(11));
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// #[pallet::weight(<T::Balance as Into<Weight>>::into(new_value.clone()))]
		#[pallet::weight(10_000)]
		pub fn commit(origin: OriginFor<T>, #[pallet::compact] amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			Ok(().into())
		}
	}
}

// impl<T: Config> Pallet<T> {
// 	pub fn do_set_bar(who: &T::AccountId, amount: T::Balance) {
// 		Bar::<T>::insert(who, amount);
// 	}
// }
