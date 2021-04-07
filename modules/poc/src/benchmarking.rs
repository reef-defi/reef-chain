#![cfg(feature = "runtime-benchmarks")]

use crate::{*, Module as Poc};
use frame_benchmarking::{benchmarks, account, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use primitives::{Balance, currency::REEF, time::DAYS};
use sp_std::prelude::*;

benchmarks! {
	where_clause { where BalanceOf<T>: From<u128> }

	start_candidacy {
		let alice: T::AccountId = account("alice", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(1_000_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

	}: _(RawOrigin::Signed(alice))

	stop_candidacy {
		let alice: T::AccountId = account("alice", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(1_000_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		Pallet::<T>::start_candidacy(
			RawOrigin::Signed(alice.clone()).into(),
		);

	}: _(RawOrigin::Signed(alice))

	commit {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(100_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);
	}: _(RawOrigin::Signed(alice), amount, LockDuration::OneYear, bob)


	add_funds {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(200_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);

		// she makes initial commitment
		Pallet::<T>::commit(
			RawOrigin::Signed(alice.clone()).into(),
			amount,
			LockDuration::OneYear,
			bob
		);

	}: _(RawOrigin::Signed(alice), amount)

	unbond {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(200_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);

		// she makes initial commitment
		Pallet::<T>::commit(
			RawOrigin::Signed(alice.clone()).into(),
			amount,
			LockDuration::OneYear,
			bob
		);

	}: _(RawOrigin::Signed(alice))

	withdraw {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(200_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);

		// she makes initial commitment
		Pallet::<T>::commit(
			RawOrigin::Signed(alice.clone()).into(),
			amount,
			LockDuration::OneMonth,
			bob
		);

		// she unbonds
		Pallet::<T>::unbond(
			RawOrigin::Signed(alice.clone()).into(),
		);

		// skip 1 month
		frame_system::Module::<T>::set_block_number((31 * DAYS).into());

	}: _(RawOrigin::Signed(alice))

	vote_candidate {
		let alice: T::AccountId = account("alice", 0, 0);
		let bob: T::AccountId = account("bob", 0, 0);
		let charlie: T::AccountId = account("charlie", 0, 0);

		// alice needs funds
		let deposit: BalanceOf<T> = BalanceOf::<T>::from(200_001 * REEF);
		T::Currency::deposit_creating(&alice, deposit);

		let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);

		// she makes initial commitment
		Pallet::<T>::commit(
			RawOrigin::Signed(alice.clone()).into(),
			amount,
			LockDuration::OneYear,
			bob
		);

	}: _(RawOrigin::Signed(alice), charlie)

}

// auto-generate benchmark tests
impl_benchmark_test_suite!(Pallet, mock::new_test_ext(), mock::Runtime);

