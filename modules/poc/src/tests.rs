#![cfg(test)]

use crate::mock::*;
use frame_support::{assert_ok, assert_err};

#[test]
fn test_setup() {
	new_test_ext().execute_with(|| {
		// dummy accounts are setup
		let alice = 0 as u64;
		let balance = Balances::free_balance(&alice);
		assert_eq!(balance, 1_000_000 as u64);

		// TechCouncil membership is empty
		assert!(TechCouncil::members().is_empty());

		// we start at era 0
		assert!(Poc::current_era().index == 0);

		// skipping to new era works
		skip_blocks(7 * HOURS + 1);
		assert!(Poc::current_era().index == 1);
	});
}

#[test]
fn commits() {
	new_test_ext().execute_with(|| {
		let alice = 0 as u64;
		let bob = 1 as u64;
		let nobody = 42 as u64;

		// cannot commit with insufficient funds
		assert_err!(
			Poc::commit(
				Origin::signed(nobody),
				(100_000 as u64).into(),
				crate::LockDuration::OneYear,
				bob,
			),
			pallet_balances::Error::<Runtime>::InsufficientBalance
		);

		// alice commits 100k and votes for bob
		assert_ok!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneYear,
				bob,
			)
		);
		assert!(Poc::commitments(alice).state == crate::LockState::Committed);

		// cannot commit again
		assert_err!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneYear,
				bob,
			),
			crate::Error::<Runtime>::AlreadyCommitted
		);

		// we can however add more funds
		assert_ok!(
			Poc::add_funds(
				Origin::signed(alice),
				(1_000 as u64).into(),
			)
		);
		let balance = Poc::commitments(alice).amount;
		assert!(balance as u64 == 101_000 as u64);
	});
}

#[test]
fn cannot_vote_noncandidate() {
	new_test_ext().execute_with(|| {
	});
}

fn skip_blocks(n: u32) {
	use frame_support::traits::OnInitialize;
	for _ in 0..n {
		Poc::on_initialize(System::block_number());
		System::set_block_number(System::block_number() + 1);
	}
}
