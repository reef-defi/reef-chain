//! # Proof of Commitment
//!
//! Stake tokens with extremely long unbonding period,
//! to obtain the Technical Council election voting rights.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

use frame_support::{
	pallet_prelude::*,
	traits::{Currency, ReservableCurrency, IsType, WithdrawReasons, ExistenceRequirement},
	weights::Weight,
	ensure,
	transactional,
};
use frame_support::sp_runtime::traits::{CheckedAdd, CheckedDiv};
use frame_system::pallet_prelude::*;

#[cfg(feature = "std")]
pub use serde::{Deserialize, Serialize};

mod mock;
mod tests;

pub use module::*;

/// How many members does the technical council have?
pub const COUNCIL_SIZE: usize = 3;
/// How long (in block count) is the era
pub const ERA_DURATION: u32 = 7 * primitives::time::DAYS;
/// How many eras per year there are (roughly)?
pub const ERAS_PER_YEAR: u32 = 52;
/// Yearly returns: notional / YEARLY_RETURNS_DENOM
pub const YEARLY_RETURNS_DENOM: u32 = 10;
/// Fixed rate per-era rewards for Council members
pub const ERA_COUNCIL_REWARDS: u32 = 100; //TODO: * primitives::currency::DOLLARS;



#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq)]
pub enum LockState<BlockNumber> {
	/// Locked /w voting power
	Committed,
	/// BlockNumber when Unbonding period started
	Unbonding(BlockNumber),
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Debug, PartialEq)]
pub enum LockDuration {
	OneMonth,
	OneYear,
	TenYears,
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


#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, Default)]
pub struct Era<BlockNumber> {
	pub index: EraIndex,
	pub start: BlockNumber,
}


pub type EraIndex = u32;
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
	// TODO: is std use allowed?
	use std::iter::FromIterator;
	use std::collections::BTreeMap;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Account already has an active commitment
		AlreadyCommitted,
		/// Cannot operate on a non existing commitment
		CommitmentNotFound,
		/// The commitment is not active
		NotCommitted,
		/// Funds are still locked and cannot be withdrawn
		CannotWithdrawLocked,
		/// Bonded amount is too small
		InsufficientAmount,
	}

	#[pallet::event]
	#[pallet::generate_deposit(fn deposit_event)]
	#[pallet::metadata(T::AccountId = "AccountId", BalanceOf<T> = "BalanceOf")]
	pub enum Event<T: Config> {
		/// Created a new committment
		Committed(T::AccountId),
		/// Add more funds to existing commitment
		FundsAdded(T::AccountId),
		/// Voter,Candidate,VotingPower
		Voted(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Voter,Reward
		VoterRewarded(EraIndex, T::AccountId, BalanceOf<T>),
	}

	#[pallet::type_value]
	pub(super) fn FirstEra<T: Config>() -> Era<T::BlockNumber> {
		Era {
			index: (0 as u32).into(),
			start: (0 as u32).into(),
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn current_era)]
	pub(super) type CurrentEra<T: Config> =
		StorageValue<_, Era<T::BlockNumber>, ValueQuery, FirstEra<T>>;

	#[pallet::storage]
	#[pallet::getter(fn voter_rewards)]
	pub(crate) type VoterRewards<T: Config> = StorageDoubleMap<_,
		Blake2_128Concat, EraIndex,
		Blake2_128Concat, T::AccountId, BalanceOf<T>,
		ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn commitments)]
	pub(crate) type Commitments<T: Config> = StorageMap<_,
		Blake2_128Concat, T::AccountId, CommitmentOf<T>, ValueQuery>;

	#[pallet::pallet]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<T::BlockNumber> for Pallet<T> {
		fn on_finalize(n: T::BlockNumber) {
			let current_era = <CurrentEra<T>>::get();
			let era_duration: T::BlockNumber = ERA_DURATION.into();

			if current_era.start + era_duration >= n {
				// move the era forward
				let new_era = Era{index: current_era.index, start: n};
				<CurrentEra<T>>::set(new_era);

				// clear old voter rewards (to save space)
				<VoterRewards<T>>::remove_prefix(&current_era.index);

				// set winners on new era
				let mut counter: BTreeMap<T::AccountId, BalanceOf<T>> = BTreeMap::new();

				for (_, c) in <Commitments<T>>::iter() {
					if counter.contains_key(&c.candidate) {
						let acc_w = *counter.get(&c.candidate).unwrap();
						counter.insert(c.candidate.clone(), Self::voting_weight(&c) + acc_w);
					} else {
						counter.insert(c.candidate.clone(), Self::voting_weight(&c));
					}
				}
				let mut sorted = Vec::from_iter(counter);
				sorted.sort_by(|&(_, a), &(_, b)| b.cmp(&a));

				let mut winners: Vec<T::AccountId> = Vec::new();
				for (candidate, _weight) in sorted.iter().take(COUNCIL_SIZE) {
					winners.push(candidate.to_owned());
				}

				// TODO: assign winners to the quorum

				// distribute winners rewards
				let zero = BalanceOf::<T>::from(0 as u32);
				let rewards = BalanceOf::<T>::from(ERA_COUNCIL_REWARDS);
				let reward = rewards.checked_div(&BalanceOf::<T>::from(COUNCIL_SIZE as u32)).unwrap_or(zero);
				if reward > zero {
					for winner in winners.iter() {
						// ignore failed cases
						T::Currency::deposit_into_existing(&winner, reward).ok();
					}
				}

			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {

		#[pallet::weight(10_000)]
		pub fn commit(
			origin: OriginFor<T>,
			#[pallet::compact] amount: BalanceOf<T>,
			duration: LockDuration,
			candidate: T::AccountId,
			) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(!<Commitments<T>>::contains_key(&origin), Error::<T>::AlreadyCommitted);

			// TODO: consider imposing a minimum bond size (to make on_finalize faster)
			ensure!(amount >= BalanceOf::<T>::from(0 as u32), Error::<T>::InsufficientAmount);

			// TODO check if at totalSupply capacity
			// ensure!(T::Currency::free_balance(&origin) >= amount &&
			// 		T::Currency::total_issuance() * 0.1 < Self::totalBonded.checked_add(amount).unwrap(), Error::<T>::OverQuota);

			T::Currency::withdraw(
				&origin, amount,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive)?;

			// create a new commitment
			<Commitments<T>>::insert(&origin, Commitment {
				amount: amount,
				duration: duration,
				candidate: candidate,
				..Default::default()
			});
			Self::deposit_event(Event::Committed(origin));
			Ok(().into())
		}


		#[pallet::weight(10_000)]
		pub fn add_funds(origin: OriginFor<T>, #[pallet::compact] amount: BalanceOf<T>) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(<Commitments<T>>::contains_key(&origin), Error::<T>::CommitmentNotFound);
			let mut commitment = <Commitments<T>>::get(&origin);

			ensure!(amount >= BalanceOf::<T>::from(0 as u32), Error::<T>::InsufficientAmount);
			// TODO check if at totalSupply capacity

			T::Currency::withdraw(
				&origin, amount,
				WithdrawReasons::RESERVE,
				ExistenceRequirement::KeepAlive)?;
			commitment.amount = commitment.amount.checked_add(&amount).ok_or("currency overflow")?;

			// always re-commit
			commitment.state = LockState::Committed;

			// save the commitment
			<Commitments<T>>::insert(&origin, commitment);


			Ok(().into())
		}


		#[pallet::weight(10_000)]
		pub fn unbond(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(<Commitments<T>>::contains_key(&origin), Error::<T>::CommitmentNotFound);
			let mut commitment = <Commitments<T>>::get(&origin);
			ensure!(commitment.state == LockState::Committed, Error::<T>::NotCommitted);

			// record the unbonding block number
			let current_block: T::BlockNumber = frame_system::Module::<T>::block_number();
			commitment.state = LockState::Unbonding(current_block);

			<Commitments<T>>::insert(&origin, commitment);
			Ok(().into())
		}


		#[pallet::weight(10_000)]
		pub fn withdraw(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(<Commitments<T>>::contains_key(&origin), Error::<T>::CommitmentNotFound);
			let commitment = <Commitments<T>>::get(&origin);
			ensure!(commitment.state != LockState::Committed, Error::<T>::AlreadyCommitted);

			// check if Unbonding period is over
			// WARN: if block times are altered, this calculation will become invalid
			if let LockState::Unbonding(start_block) = commitment.state {

				let lock_period = match commitment.duration {
					LockDuration::OneMonth => 30,
					LockDuration::OneYear  => 365,
					LockDuration::TenYears => 3650,
				} * primitives::time::DAYS;
				let lock_period: T::BlockNumber = lock_period.into();
				let current_block: T::BlockNumber = frame_system::Module::<T>::block_number();

				if start_block + lock_period <= current_block {
					// credit the user his funds
					T::Currency::deposit_into_existing(&origin, commitment.amount)?;

					// delete the commitment
					<Commitments<T>>::remove(&origin);

					return Ok(().into());
				}
			}
			Err(Error::<T>::CannotWithdrawLocked.into())
		}


		#[pallet::weight(10_000)]
		#[transactional]
		pub fn set_candidate(
			origin: OriginFor<T>,
			candidate: T::AccountId,
			) -> DispatchResultWithPostInfo {
			let origin = ensure_signed(origin)?;

			ensure!(<Commitments<T>>::contains_key(&origin), Error::<T>::CommitmentNotFound);
			let mut commitment = <Commitments<T>>::get(&origin);
			ensure!(commitment.state == LockState::Committed, Error::<T>::NotCommitted);

			if commitment.candidate != candidate {
				commitment.candidate = candidate.clone();
				<Commitments<T>>::insert(&origin, &commitment);
				Self::deposit_event(Event::Voted(origin.clone(), candidate, Self::voting_weight(&commitment)));
			}

			let apy = Self::apy_reward(&commitment);
			let zero = BalanceOf::<T>::from(0 as u32);
			let reward = apy.checked_div(&BalanceOf::<T>::from(ERAS_PER_YEAR)).unwrap_or(zero);
			if reward > zero {
				// check if nominator has been rewarded already for this era
				let current_era = <CurrentEra<T>>::get();
				if !<VoterRewards<T>>::contains_key(&current_era.index, &origin) {
					<VoterRewards<T>>::insert(&current_era.index, &origin, &reward);
					Self::deposit_event(Event::VoterRewarded(current_era.index, origin, reward));
				}
			}

			Ok(().into())
		}


	}
}

impl<T: Config> Pallet<T> {
	/// Voting shares based on currently committed amount.
	/// Monthly locks have 1x voting power, yearly 10x and 10 yearly 100x.
	pub fn voting_weight(commitment: &Commitment<T::AccountId, BalanceOf<T>, T::BlockNumber>) -> BalanceOf<T> {
		if commitment.state != LockState::Committed {
			return BalanceOf::<T>::from(0 as u32);
		}
		let multiplier = match commitment.duration {
			LockDuration::OneMonth => 1,
			LockDuration::OneYear  => 10,
			LockDuration::TenYears => 100,
		};
		commitment.amount * BalanceOf::<T>::from(multiplier as u32)
	}

	/// Yearly reward amount based on currently committed amount.
	/// Montly locks yield 0% APY. Yearly and 10 Year locks yield 10% APY.
	pub fn apy_reward(commitment: &Commitment<T::AccountId, BalanceOf<T>, T::BlockNumber>) -> BalanceOf<T> {
		let zero = BalanceOf::<T>::from(0 as u32);

		if commitment.state != LockState::Committed {
			return zero;
		}

		match commitment.duration {
			LockDuration::OneMonth => {
				zero
			},
			_ => {
				commitment.amount
					.checked_div(&BalanceOf::<T>::from(YEARLY_RETURNS_DENOM))
					.unwrap_or(zero)
			}
		}
	}


}