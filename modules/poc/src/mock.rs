#![cfg(test)]

use crate as module_poc;
use frame_support::{construct_runtime, parameter_types};
use sp_runtime::Perbill;
pub use primitives::{BlockNumber, currency::*, time::*};

type Balance = u64;
type TechCouncilInstance = pallet_collective::Instance1;

parameter_types!(
	pub const BlockHashCount: u32 = 250;
);
impl frame_system::Config for Runtime {
	type BaseCallFilter = ();
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Call = Call;
	type Hash = sp_runtime::testing::H256;
	type Hashing = sp_runtime::traits::BlakeTwo256;
	type AccountId = u64;
	type Lookup = sp_runtime::traits::IdentityLookup<Self::AccountId>;
	type Header = sp_runtime::testing::Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}


parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}
impl pallet_balances::Config for Runtime {
	type MaxLocks = ();
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
}

parameter_types! {
	pub const TechCouncilMotionDuration: BlockNumber = 7 * HOURS;
	pub const TechCouncilMaxProposals: u32 = 100;
	pub const TechCouncilMaxMembers: u32 = 3;
	pub const TechCouncilMaxCandidates: u32 = 100;
}

impl pallet_collective::Config<TechCouncilInstance> for Runtime {
	type Origin = Origin;
	type Proposal = Call;
	type Event = Event;
	type MotionDuration = TechCouncilMotionDuration;
	type MaxProposals = TechCouncilMaxProposals;
	type MaxMembers = TechCouncilMaxMembers;
	type DefaultVote = pallet_collective::MoreThanMajorityThenPrimeDefaultVote;
	type WeightInfo = ();
}

parameter_types! {
	pub const EraDuration: BlockNumber = 7 * HOURS;
	pub const NominatorAPY: Perbill = Perbill::from_percent(10);
	pub const CouncilInflation: Perbill = Perbill::from_percent(1);
	pub const CandidacyDeposit: Balance = 250_000;
	pub const MinLockAmount: Balance = 100;
	pub const TotalLockedCap: Balance = 10_000_000;
}

impl module_poc::Config for Runtime {
	type Event = Event;
	type Currency = Balances;
	type EraDuration = EraDuration;
	type NominatorAPY = NominatorAPY;
	type CouncilInflation = CouncilInflation;
	type CandidacyDeposit = CandidacyDeposit;
	type MinLockAmount = MinLockAmount;
	type TotalLockedCap = TotalLockedCap;
	type MaxCandidates = TechCouncilMaxCandidates;
	type MaxMembers = TechCouncilMaxMembers;
	type MembershipChanged = TechCouncil;
	type WeightInfo = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Module, Call, Storage, Event<T>},
		TechCouncil: pallet_collective::<Instance1>::{Module, Call, Storage, Origin<T>, Event<T>, Config<T>},
		Poc: module_poc::{Module, Call, Storage, Event<T>},
	}
);

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	// inject test balances
	pallet_balances::GenesisConfig::<Runtime>{
		balances: vec![
			(0, 1_000_000), // alice
			(1, 1_000_000), // bob
			(2, 1_000_000), // charlie
			(3, 1_000_000), // eve
		],
	}.assimilate_storage(&mut t).unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));

	ext
}
