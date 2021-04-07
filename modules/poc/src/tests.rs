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
		run_blocks(7 * HOURS + 1);
		assert_eq!(Poc::current_era().index, 1);
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
		assert_eq!(Poc::locked_amount(), 101_000 as u64);
	});
}

#[test]
fn withdrawals() {
	new_test_ext().execute_with(|| {
		// alice commits for a month
		let alice = 0 as u64;
		let bob = 1 as u64;
		assert_ok!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneMonth,
				bob,
			)
		);

		// she cannot withdraw an active commitment
		assert_err!(
			Poc::withdraw(Origin::signed(alice)),
			crate::Error::<Runtime>::AlreadyCommitted
		);

		// she starts the unbonding
		assert_ok!(Poc::unbond(Origin::signed(alice)));

		// her voting power is now 0
		assert_eq!(
			Poc::voting_weight(&Poc::commitments(&alice)),
			0
		);

		// still to early to withdraw
		assert_err!(
			Poc::withdraw(Origin::signed(alice)),
			crate::Error::<Runtime>::CannotWithdrawLocked
		);

		// after the unboding period we can withdraw
		skip_blocks(31 * DAYS);
		assert_ok!(Poc::withdraw(Origin::signed(alice)));

		// the funds are returned to alice
		let balance = Balances::free_balance(&alice);
		assert_eq!(balance, 1_000_000 as u64);

		// storage checks
		assert_eq!(Poc::locked_amount(), 0 as u64);
		assert_eq!(Poc::commitments(&alice).amount, 0 as u64);

		// alice can make a new commitment
		assert_ok!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneMonth,
				bob,
			)
		);
		assert_eq!(Poc::commitments(&alice).amount, 100_000 as u64);
		let balance = Balances::free_balance(&alice);
		assert_eq!(balance, 900_000 as u64);
	});
}

#[test]
fn voting_rewards() {
	new_test_ext().execute_with(|| {
		let alice = 0 as u64;
		let bob = 1 as u64;
		let charlie = 2 as u64;

		// alice commits for a month
		assert_ok!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneMonth,
				bob,
			)
		);

		// bob commits for a year
		assert_ok!(
			Poc::commit(
				Origin::signed(bob),
				(100_000 as u64).into(),
				crate::LockDuration::OneYear,
				bob,
			)
		);

		// charlie commits for a 10 years
		assert_ok!(
			Poc::commit(
				Origin::signed(charlie),
				(100_000 as u64).into(),
				crate::LockDuration::TenYears,
				bob,
			)
		);

		// all 3 vote
		assert_ok!(Poc::vote_candidate(Origin::signed(alice), bob));
		assert_ok!(Poc::vote_candidate(Origin::signed(bob), bob));
		assert_ok!(Poc::vote_candidate(Origin::signed(charlie), bob));

		// alice should not have received a reward
		let balance = Balances::free_balance(&alice);
		assert_eq!(balance, 900_000 as u64);

		// bob should receive 10% APY
		// In [1]: (7/(24*365)) * 10000
		// Out[1]: 7.990867579908676
		let balance = Balances::free_balance(&bob);
		assert_eq!(balance, 900_008 as u64);

		// charlie should also receive 10% APY
		let balance = Balances::free_balance(&charlie);
		assert_eq!(balance, 900_008 as u64);

		// charlie starts unbonding
		assert_ok!(Poc::unbond(Origin::signed(charlie)));

		// so he can no longer vote
		assert_err!(
			Poc::vote_candidate(Origin::signed(charlie), bob),
			crate::Error::<Runtime>::NotCommitted
		);

		// voting twice in the same era does not double rewards
		assert_ok!(Poc::vote_candidate(Origin::signed(bob), bob));
		let balance = Balances::free_balance(&bob);
		assert_eq!(balance, 900_008 as u64);

		// voting in the next era yields more rewards
		run_blocks(7 * HOURS + 1);
		assert_ok!(Poc::vote_candidate(Origin::signed(bob), bob));
		let balance = Balances::free_balance(&bob);
		assert_eq!(balance, 900_016 as u64);
	});
}

#[test]
fn candidacy() {
	new_test_ext().execute_with(|| {
		let alice = 0 as u64;

		// alice starts candidacy and bonds 250k REEF
		assert_ok!(Poc::start_candidacy(Origin::signed(alice)));
		assert_eq!(Balances::free_balance(&alice), 750_000 as u64);
		assert_eq!(Balances::reserved_balance(&alice), 250_000 as u64);
		assert_eq!(Poc::n_candidates(), 1);
		assert_eq!(Poc::candidates(alice), 250_000);

		// she cannot become a candidate twice
		assert_err!(
			Poc::start_candidacy(Origin::signed(alice)),
			crate::Error::<Runtime>::AlreadyCandidate
		);

		// alice stops candidacy and gets her funds back
		assert_ok!(Poc::stop_candidacy(Origin::signed(alice)));
		assert_eq!(Balances::free_balance(&alice), 1_000_000 as u64);
		assert_eq!(Balances::reserved_balance(&alice), 0 as u64);
		assert_eq!(Poc::n_candidates(), 0);
		assert_eq!(Poc::candidates(alice), 0);
	});
}

#[test]
fn elections() {
	new_test_ext().execute_with(|| {
		let alice = 0 as u64;
		let bob = 1 as u64;
		let charlie = 2 as u64;
		let eve = 3 as u64;
		let nobody = 42 as u64;

		// all but nobody are candidates
		assert_ok!(Poc::start_candidacy(Origin::signed(alice)));
		assert_ok!(Poc::start_candidacy(Origin::signed(bob)));
		assert_ok!(Poc::start_candidacy(Origin::signed(charlie)));
		assert_ok!(Poc::start_candidacy(Origin::signed(eve)));

		// alice commits for a month
		assert_ok!(
			Poc::commit(
				Origin::signed(alice),
				(100_000 as u64).into(),
				crate::LockDuration::OneMonth,
				alice,
			)
		);
		// she gets 1x voting power
		assert_eq!(
			Poc::voting_weight(&Poc::commitments(&alice)),
			100_000,
		);

		// bob commits for a year
		assert_ok!(
			Poc::commit(
				Origin::signed(bob),
				(100_000 as u64).into(),
				crate::LockDuration::OneYear,
				bob,
			)
		);
		// he gets 10x voting power
		assert_eq!(
			Poc::voting_weight(&Poc::commitments(&bob)),
			10 * 100_000,
		);

		// charlie commits for 10 years
		assert_ok!(
			Poc::commit(
				Origin::signed(charlie),
				(100_000 as u64).into(),
				crate::LockDuration::TenYears,
				charlie,
			)
		);
		// he gets 100x voting power
		assert_eq!(
			Poc::voting_weight(&Poc::commitments(&charlie)),
			100 * 100_000,
		);

		// eve votes most heavily for non-candidate
		assert_ok!(
			Poc::commit(
				Origin::signed(eve),
				(200_000 as u64).into(),
				crate::LockDuration::TenYears,
				nobody,
			)
		);
		assert_eq!(
			Poc::voting_weight(&Poc::commitments(&eve)),
			100 * 200_000,
		);

		// check current supply for rewards = 4m - 500k committed
		let total_supply = Balances::total_issuance();
		assert_eq!(total_supply, 3_500_000 as u64);

		// run the elections
		run_blocks(7 * HOURS);

		// winners (non-candidate not included)
		assert_eq!(
			Poc::members(),
			vec![alice, bob, charlie]
		);
		assert_eq!(
			TechCouncil::members(),
			vec![alice, bob, charlie]
		);

		// check rewards
		// In [1]: (7/(24*365)) * (3_500_000 * 0.01)
		// Out[1]: 27.968036529680365
		// In [2]: _ / 3
		// Out[2]: 9.322678843226788
		// per winner reward ^^^
		assert_eq!(Balances::free_balance(&alice), 650_009 as u64);
		assert_eq!(Balances::free_balance(&bob), 650_009 as u64);
		assert_eq!(Balances::free_balance(&charlie), 650_009 as u64);
		assert_eq!(Balances::free_balance(&nobody), 0 as u64);

		// eve votes herself (and gets voting reward)
		assert_eq!(Balances::free_balance(&eve), 550_000 as u64);
		assert_ok!(Poc::vote_candidate(Origin::signed(eve), eve));
		assert_eq!(Balances::free_balance(&eve), 550_016 as u64);

		// run the elections
		run_blocks(7 * HOURS);

		// winners (alice falls out)
		assert_eq!(
			Poc::members(),
			vec![bob, charlie, eve]
		);
		assert_eq!(
			TechCouncil::members(),
			vec![bob, charlie, eve]
		);

		// rewards
		assert_eq!(Balances::free_balance(&alice), 650_009 as u64);
		assert_eq!(Balances::free_balance(&bob), 650_018 as u64);
		assert_eq!(Balances::free_balance(&charlie), 650_018 as u64);
		assert_eq!(Balances::free_balance(&eve), 550_025 as u64);

		// TODO: cannot vote empty / too small quorum
		// assert_ok!(Poc::unbond(Origin::signed(alice)));
		// assert_ok!(Poc::unbond(Origin::signed(bob)));
	});
}


fn run_blocks(n: u32) {
	use frame_support::traits::OnInitialize;
	for _ in 0..n {
		Poc::on_initialize(System::block_number());
		System::set_block_number(System::block_number() + 1);
	}
}

fn skip_blocks(n: u32) {
	System::set_block_number(System::block_number() + n as u64);
}
