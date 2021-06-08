#![cfg(test)]

use codec::Encode;
use frame_support::{
	assert_noop, assert_ok,
	traits::{GenesisBuild, OnFinalize, OnInitialize},
};
use reef_runtime::{
	get_all_module_accounts,
	AccountId, AuthoritysOriginId,
	Balance, Balances, Call,
	CurrencyId,
	Event, EvmAccounts, GetNativeCurrencyId,
	NativeTokenExistentialDeposit, Origin,
	Perbill, Runtime, System,
	TokenSymbol, Evm,
};
use module_support::{Price};
use sp_io::hashing::keccak_256;
use sp_runtime::{
	traits::{BadOrigin},
	DispatchError, FixedPointNumber, MultiAddress,
};

use primitives::currency::*;

const ALICE: [u8; 32] = [4u8; 32];
const BOB: [u8; 32] = [5u8; 32];

pub type SystemModule = frame_system::Module<Runtime>;
pub type AuthorityModule = orml_authority::Module<Runtime>;
pub type Currencies = module_currencies::Module<Runtime>;
pub type SchedulerModule = pallet_scheduler::Module<Runtime>;

fn run_to_block(n: u32) {
	while SystemModule::block_number() < n {
		SchedulerModule::on_finalize(SystemModule::block_number());
		SystemModule::set_block_number(SystemModule::block_number() + 1);
		SchedulerModule::on_initialize(SystemModule::block_number());
	}
}

fn last_event() -> Event {
	SystemModule::events().pop().expect("Event expected").event
}

pub struct ExtBuilder {
	endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			endowed_accounts: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn balances(mut self, endowed_accounts: Vec<(AccountId, CurrencyId, Balance)>) -> Self {
		self.endowed_accounts = endowed_accounts;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let native_currency_id = GetNativeCurrencyId::get();
		let existential_deposit = NativeTokenExistentialDeposit::get();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: self
				.endowed_accounts
				.clone()
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id == native_currency_id)
				.map(|(account_id, _, initial_balance)| (account_id, initial_balance))
				.chain(
					get_all_module_accounts()
						.iter()
						.map(|x| (x.clone(), existential_deposit)),
				)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		orml_tokens::GenesisConfig::<Runtime> {
			endowed_accounts: self
				.endowed_accounts
				.into_iter()
				.filter(|(_, currency_id, _)| *currency_id != native_currency_id)
				.collect::<Vec<_>>(),
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| SystemModule::set_block_number(1));
		ext
	}
}

pub fn origin_of(account_id: AccountId) -> <Runtime as frame_system::Config>::Origin {
	<Runtime as frame_system::Config>::Origin::signed(account_id)
}

fn amount(amount: u128) -> u128 {
	amount.saturating_mul(Price::accuracy())
}

fn alice() -> secp256k1::SecretKey {
	secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn bob() -> secp256k1::SecretKey {
	secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

pub fn alice_account_id() -> AccountId {
	let address = EvmAccounts::eth_address(&alice());
	let mut data = [0u8; 32];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);
	AccountId::from(Into::<[u8; 32]>::into(data))
}

pub fn bob_account_id() -> AccountId {
	let address = EvmAccounts::eth_address(&bob());
	let mut data = [0u8; 32];
	data[0..4].copy_from_slice(b"evm:");
	data[4..24].copy_from_slice(&address[..]);
	AccountId::from(Into::<[u8; 32]>::into(data))
}

#[cfg(not(feature = "with-ethereum-compatibility"))]
use sp_core::H160;
#[cfg(not(feature = "with-ethereum-compatibility"))]
fn deploy_contract(account: AccountId) -> Result<H160, DispatchError> {
	// pragma solidity ^0.5.0;
	//
	// contract Factory {
	//     Contract[] newContracts;
	//
	//     function createContract () public payable {
	//         Contract newContract = new Contract();
	//         newContracts.push(newContract);
	//     }
	// }
	//
	// contract Contract {}
	let contract = hex_literal::hex!("608060405234801561001057600080fd5b5061016f806100206000396000f3fe608060405260043610610041576000357c0100000000000000000000000000000000000000000000000000000000900463ffffffff168063412a5a6d14610046575b600080fd5b61004e610050565b005b600061005a6100e2565b604051809103906000f080158015610076573d6000803e3d6000fd5b50905060008190806001815401808255809150509060018203906000526020600020016000909192909190916101000a81548173ffffffffffffffffffffffffffffffffffffffff021916908373ffffffffffffffffffffffffffffffffffffffff1602179055505050565b6040516052806100f28339019056fe6080604052348015600f57600080fd5b50603580601d6000396000f3fe6080604052600080fdfea165627a7a7230582092dc1966a8880ddf11e067f9dd56a632c11a78a4afd4a9f05924d427367958cc0029a165627a7a723058202b2cc7384e11c452cdbf39b68dada2d5e10a632cc0174a354b8b8c83237e28a40029").to_vec();

	Evm::create(Origin::signed(account), contract, 0, 1000000000, 1000000000)
		.map_or_else(|e| Err(e.error), |_| Ok(()))?;

	if let Event::module_evm(module_evm::Event::Created(address)) = System::events().iter().last().unwrap().event {
		Ok(address)
	} else {
		Err("deploy_contract failed".into())
	}
}


#[test]
fn test_authority_module() {
	const _AUTHORITY_ORIGIN_ID: u8 = 10u8;

	ExtBuilder::default()
		.balances(vec![
			(
				AccountId::from(ALICE),
				CurrencyId::Token(TokenSymbol::RUSD),
				amount(1000),
			),
		])
		.build()
		.execute_with(|| {
			let ensure_root_call = Call::System(frame_system::Call::fill_block(Perbill::one()));
			let _call = Call::Authority(orml_authority::Call::dispatch_as(
				AuthoritysOriginId::Root,
				Box::new(ensure_root_call.clone()),
			));

			// dispatch_as
			assert_ok!(AuthorityModule::dispatch_as(
				Origin::root(),
				AuthoritysOriginId::Root,
				Box::new(ensure_root_call.clone())
			));

			assert_noop!(
				AuthorityModule::dispatch_as(
					Origin::signed(AccountId::from(BOB)),
					AuthoritysOriginId::Root,
					Box::new(ensure_root_call.clone())
				),
				BadOrigin
			);

			// schedule_dispatch
			run_to_block(1);
			// // DSWF transfer
			// let transfer_call = Call::Currencies(module_currencies::Call::transfer(
			// 	AccountId::from(BOB).into(),
			// 	CurrencyId::Token(TokenSymbol::RUSD),
			// 	amount(500),
			// ));
			// let dswf_call = Call::Authority(orml_authority::Call::dispatch_as(
			// 	AuthoritysOriginId::DSWF,
			// 	Box::new(transfer_call.clone()),
			// ));
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(2),
			// 	0,
			// 	true,
			// 	Box::new(dswf_call.clone())
			// ));
            //
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(2),
			// 	0,
			// 	true,
			// 	Box::new(call.clone())
			// ));
            //
			// let event = Event::orml_authority(orml_authority::Event::Scheduled(
			// 	OriginCaller::orml_authority(DelayedOrigin {
			// 		delay: 1,
			// 		origin: Box::new(OriginCaller::system(RawOrigin::Root)),
			// 	}),
			// 	1,
			// ));
			// assert_eq!(last_event(), event);
            //
			// run_to_block(2);
			// assert_eq!(
			// 	Currencies::free_balance(
			// 		CurrencyId::Token(TokenSymbol::RUSD),
			// 		&DSWFModuleId::get().into_account()
			// 	),
			// 	amount(500)
			// );
			// assert_eq!(
			// 	Currencies::free_balance(CurrencyId::Token(TokenSymbol::RUSD), &AccountId::from(BOB)),
			// 	amount(500)
			// );
            //
			// // delay < SevenDays
			// let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
			// 	(2, 1),
			// 	Some([AUTHORITY_ORIGIN_ID, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0].to_vec()),
			// 	Err(DispatchError::BadOrigin),
			// ));
			// assert_eq!(last_event(), event);
            //
			// // delay = SevenDays
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(SevenDays::get() + 2),
			// 	0,
			// 	true,
			// 	Box::new(call.clone())
			// ));
            //
			// run_to_block(SevenDays::get() + 2);
			// let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
			// 	(151202, 0),
			// 	Some([AUTHORITY_ORIGIN_ID, 160, 78, 2, 0, 0, 0, 2, 0, 0, 0].to_vec()),
			// 	Ok(()),
			// ));
			// assert_eq!(last_event(), event);
            //
			// // with_delayed_origin = false
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(SevenDays::get() + 3),
			// 	0,
			// 	false,
			// 	Box::new(call.clone())
			// ));
			// let event = Event::orml_authority(orml_authority::Event::Scheduled(
			// 	OriginCaller::system(RawOrigin::Root),
			// 	3,
			// ));
			// assert_eq!(last_event(), event);
            //
			// run_to_block(SevenDays::get() + 3);
			// let event = Event::pallet_scheduler(pallet_scheduler::RawEvent::Dispatched(
			// 	(151203, 0),
			// 	Some([0, 0, 3, 0, 0, 0].to_vec()),
			// 	Ok(()),
			// ));
			// assert_eq!(last_event(), event);
            //
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(SevenDays::get() + 4),
			// 	0,
			// 	false,
			// 	Box::new(call.clone())
			// ));
            //
			// // fast_track_scheduled_dispatch
			// assert_ok!(AuthorityModule::fast_track_scheduled_dispatch(
			// 	Origin::root(),
			// 	frame_system::RawOrigin::Root.into(),
			// 	4,
			// 	DispatchTime::At(SevenDays::get() + 5),
			// ));
            //
			// // delay_scheduled_dispatch
			// assert_ok!(AuthorityModule::delay_scheduled_dispatch(
			// 	Origin::root(),
			// 	frame_system::RawOrigin::Root.into(),
			// 	4,
			// 	4,
			// ));
            //
			// // cancel_scheduled_dispatch
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(SevenDays::get() + 4),
			// 	0,
			// 	true,
			// 	Box::new(call.clone())
			// ));
			// let event = Event::orml_authority(orml_authority::Event::Scheduled(
			// 	OriginCaller::orml_authority(DelayedOrigin {
			// 		delay: 1,
			// 		origin: Box::new(OriginCaller::system(RawOrigin::Root)),
			// 	}),
			// 	5,
			// ));
			// assert_eq!(last_event(), event);
            //
			// let schedule_origin = {
			// 	let origin: <Runtime as orml_authority::Config>::Origin = From::from(Origin::root());
			// 	let origin: <Runtime as orml_authority::Config>::Origin = From::from(DelayedOrigin::<
			// 		BlockNumber,
			// 		<Runtime as orml_authority::Config>::PalletsOrigin,
			// 	> {
			// 		delay: 1,
			// 		origin: Box::new(origin.caller().clone()),
			// 	});
			// 	origin
			// };
            //
			// let pallets_origin = schedule_origin.caller().clone();
			// assert_ok!(AuthorityModule::cancel_scheduled_dispatch(
			// 	Origin::root(),
			// 	pallets_origin,
			// 	5
			// ));
			// let event = Event::orml_authority(orml_authority::Event::Cancelled(
			// 	OriginCaller::orml_authority(DelayedOrigin {
			// 		delay: 1,
			// 		origin: Box::new(OriginCaller::system(RawOrigin::Root)),
			// 	}),
			// 	5,
			// ));
			// assert_eq!(last_event(), event);
            //
			// assert_ok!(AuthorityModule::schedule_dispatch(
			// 	Origin::root(),
			// 	DispatchTime::At(SevenDays::get() + 5),
			// 	0,
			// 	false,
			// 	Box::new(call.clone())
			// ));
			// let event = Event::orml_authority(orml_authority::Event::Scheduled(
			// 	OriginCaller::system(RawOrigin::Root),
			// 	6,
			// ));
			// assert_eq!(last_event(), event);
            //
			// assert_ok!(AuthorityModule::cancel_scheduled_dispatch(
			// 	Origin::root(),
			// 	frame_system::RawOrigin::Root.into(),
			// 	6
			// ));
			// let event = Event::orml_authority(orml_authority::Event::Cancelled(
			// 	OriginCaller::system(RawOrigin::Root),
			// 	6,
			// ));
			// assert_eq!(last_event(), event);
		});
}


#[test]
fn test_evm_accounts_module() {
	ExtBuilder::default()
		.balances(vec![(
			bob_account_id(),
			CurrencyId::Token(TokenSymbol::REEF),
			amount(1000),
		)])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(AccountId::from(ALICE)), 0);
			assert_eq!(Balances::free_balance(bob_account_id()), 1000000000000000000000);
			assert_ok!(EvmAccounts::claim_account(
				Origin::signed(AccountId::from(ALICE)),
				EvmAccounts::eth_address(&alice()),
				EvmAccounts::eth_sign(&alice(), &AccountId::from(ALICE).encode(), &[][..])
			));
			let event = Event::module_evm_accounts(module_evm_accounts::Event::ClaimAccount(
				AccountId::from(ALICE),
				EvmAccounts::eth_address(&alice()),
			));
			assert_eq!(last_event(), event);

			// claim another eth address
			assert_noop!(
				EvmAccounts::claim_account(
					Origin::signed(AccountId::from(ALICE)),
					EvmAccounts::eth_address(&alice()),
					EvmAccounts::eth_sign(&alice(), &AccountId::from(ALICE).encode(), &[][..])
				),
				module_evm_accounts::Error::<Runtime>::AccountIdHasMapped
			);
			assert_noop!(
				EvmAccounts::claim_account(
					Origin::signed(AccountId::from(BOB)),
					EvmAccounts::eth_address(&alice()),
					EvmAccounts::eth_sign(&alice(), &AccountId::from(BOB).encode(), &[][..])
				),
				module_evm_accounts::Error::<Runtime>::EthAddressHasMapped
			);
		});
}

#[cfg(not(feature = "with-ethereum-compatibility"))]
#[test]
fn test_evm_module() {
	ExtBuilder::default()
		.balances(vec![
			(alice_account_id(), CurrencyId::Token(TokenSymbol::REEF), amount(1 * MILLI_REEF)),
			(bob_account_id(), CurrencyId::Token(TokenSymbol::REEF), amount(1 * MILLI_REEF)),
		])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(alice_account_id()), amount(1 * MILLI_REEF));
			assert_eq!(Balances::free_balance(bob_account_id()), amount(1 * MILLI_REEF));

			let _alice_address = EvmAccounts::eth_address(&alice());
			let bob_address = EvmAccounts::eth_address(&bob());

			let contract = deploy_contract(alice_account_id()).unwrap();
			let event = Event::module_evm(module_evm::Event::Created(contract));
			assert_eq!(last_event(), event);

			assert_ok!(Evm::transfer_maintainer(
				Origin::signed(alice_account_id()),
				contract,
				bob_address
			));
			let event = Event::module_evm(module_evm::Event::TransferredMaintainer(contract, bob_address));
			assert_eq!(last_event(), event);

			// test EvmAccounts Lookup
			assert_eq!(Balances::free_balance(alice_account_id()), 999999999999989633000000000000000);
			assert_eq!(Balances::free_balance(bob_account_id()), amount(1 * MILLI_REEF));
			let to = EvmAccounts::eth_address(&alice());
			assert_ok!(Currencies::transfer(
				Origin::signed(bob_account_id()),
				MultiAddress::Address20(to.0),
				CurrencyId::Token(TokenSymbol::REEF),
				amount(10 * UREEF)
			));
			assert_eq!(Balances::free_balance(alice_account_id()), 1009999999999989633000000000000000);
			assert_eq!(Balances::free_balance(bob_account_id()), amount(1 * MILLI_REEF) - amount(10 * UREEF));
		});
}

#[cfg(feature = "with-ethereum-compatibility")]
#[test]
fn test_evm_module() {
	ExtBuilder::default()
		.balances(vec![
			(alice_account_id(), CurrencyId::Token(TokenSymbol::REEF), amount(1 * MILLI_REEF)),
			(bob_account_id(), CurrencyId::Token(TokenSymbol::REEF), amount(1 * MILLI_REEF)),
		])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::free_balance(alice_account_id()), amount(1 * MILLI_REEF));
			assert_eq!(Balances::free_balance(bob_account_id()), amount(1 * MILLI_REEF));

			use std::fs::{self, File};
			use std::io::Read;

			let paths = fs::read_dir("../../runtime/mandala/tests/solidity_test").unwrap();
			let file_names = paths
				.filter_map(|entry| entry.ok().and_then(|e| e.path().to_str().map(|s| String::from(s))))
				.collect::<Vec<String>>();

			for file in file_names {
				let mut f = File::open(&file).expect("File not found");
				let mut contents = String::new();
				f.read_to_string(&mut contents)
					.expect("Something went wrong reading the file.");
				let json: serde_json::Value = serde_json::from_str(&contents).unwrap();

				let bytecode_str = serde_json::to_string(&json["bytecode"]).unwrap();
				let bytecode_str = bytecode_str.replace("\"", "");

				let bytecode = hex::decode(bytecode_str).unwrap();
				assert_ok!(Evm::create(
					Origin::signed(alice_account_id()),
					bytecode,
					0,
					u64::MAX,
					u32::MAX
				));

				match System::events().iter().last().unwrap().event {
					Event::module_evm(module_evm::Event::Created(_)) => {}
					_ => {
						println!(
							"contract {:?} create failed, event: {:?}",
							file,
							System::events().iter().last().unwrap().event
						);
						assert!(false);
					}
				};
			}
		});
}
