#![cfg(feature = "runtime-benchmarks")]

use crate::{*};
use frame_benchmarking::{benchmarks, account, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use primitives::{currency::REEF, time::DAYS};

benchmarks! {
	where_clause { where BalanceOf<T>: From<u128> }

	on_initialize_empty {
	}: {
		// trigger regular block without era change
		Pallet::<T>::on_initialize((1 as u32).into());
	}

	on_initialize_era {
		// benchmark election worst-case (everyone votes for everyone else who is a valid candidate)
		// TODO: commitments are weakly capped - 1k is a conservative ceiling. re-run if surpassed
		// TODO: if MaxMembers changes (number of winners), this benchmark should re-run
		let c in 0..1_000;
		for i in 0..1_000 {
			let voter: T::AccountId = account("voter", i, 0);
			let candidate: T::AccountId = account("candidate", i, 0);
			T::Currency::deposit_creating(&voter, BalanceOf::<T>::from(100_001 * REEF));
			T::Currency::deposit_creating(&candidate, BalanceOf::<T>::from(1_000_001 * REEF));

			Pallet::<T>::start_candidacy(
				RawOrigin::Signed(candidate.clone()).into()
			);

			let amount: BalanceOf<T> = BalanceOf::<T>::from(100_000 * REEF);
			Pallet::<T>::commit(
				RawOrigin::Signed(voter.clone()).into(),
				amount,
				LockDuration::OneYear,
				candidate
			);
		}

	}: {
		// trigger the era change block
		Pallet::<T>::on_initialize((7 * DAYS).into());
	}

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

