use sp_core::{Pair, Public, sr25519, H160, Bytes};
use reef_runtime::{
	AccountId, CurrencyId,
	BabeConfig, BalancesConfig, GenesisConfig, GrandpaConfig, SudoConfig, SystemConfig,
	IndicesConfig, EVMConfig, StakingConfig, SessionConfig,
	WASM_BINARY, Signature,
	TokenSymbol, TokensConfig, REEF,
	StakerStatus,
	opaque::SessionKeys,
};
use sp_consensus_babe::AuthorityId as BabeId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::{ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;

use sp_std::{collections::btree_map::BTreeMap, str::FromStr};
use sc_chain_spec::ChainSpecExtension;

use serde::{Deserialize, Serialize};

use hex_literal::hex;
use sp_core::{crypto::UncheckedInto, bytes::from_hex};

use reef_primitives::{AccountPublic, Balance, Nonce};
use module_evm::GenesisAccount;

// The URL for the telemetry server.
const TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<reef_primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<reef_primitives::Block>,
}

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

fn get_session_keys(grandpa: GrandpaId, babe: BabeId) -> SessionKeys {
	SessionKeys { grandpa, babe }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an authority keys.
pub fn get_authority_keys_from_seed(seed: &str) -> (AccountId, AccountId, GrandpaId, BabeId) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
	)
}

pub fn development_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			],
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(reef_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || testnet_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				get_authority_keys_from_seed("Alice"),
				get_authority_keys_from_seed("Bob"),
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave"),
				get_account_id_from_seed::<sr25519::Public>("Eve"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie"),
				get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
				get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
				get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
				get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
			],
		),
		// Bootnodes
		vec![],
		// Telemetry
		// TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		None,
		// Protocol ID
		Some("reef_local_testnet"),
		// Properties
		Some(reef_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn public_testnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Reef Testnet",
		// ID
		"reef_testnet",
		ChainType::Live,
		move || testnet_genesis(
			wasm_binary,
			// Initial authorities
			vec![
				(
					hex!["b2902b07056f7365bc22bf7e69c4e4fdba03e6af9c73ca6eb1703ccbc0248857"].into(),
					hex!["cc2ea454844cc1a2e821198d9e0ce1de1aee7d014af5dd3404fc8199df89f821"].into(),
					hex!["607712f6581e191b69046427a7e33c4713e96b4ae4654e2467c74279dc20beb2"].unchecked_into(),
					hex!["ba630d2df03743a6441ab9221a25fc00a62e6f3b56c6920634eebb72a15fc90f"].unchecked_into(),
				),
				(
					hex!["06ee8fc0e34e40f6f2c98328d70874c6dd7d7989159634c8c87301efbcbe4470"].into(),
					hex!["9cf9f939c16ef458e677472ff113af53e7fb9139244fcfa6fccb765aa8831019"].into(),
					hex!["db6d2cb33abebdc024a14ef7bfbc68823660be8d1acac66770e406e484de3184"].unchecked_into(),
					hex!["d09f879b3273d2cedab83fa741cdac328679c98914dc8dc07e359e19f0379844"].unchecked_into(),
				),
				(
					hex!["48267bffea5e524f1c0e06cce77f0ef920be7ed9a7dd47705e181edad64f532a"].into(),
					hex!["38594d7640612c49337f3a0bc7b39232b86f9c9c4fedec3f8b00e45d3f073a2d"].into(),
					hex!["c8996b17688cab9bcda8dafb4dde9bab4d9b1dc81c71419fca46fedcba74a14e"].unchecked_into(),
					hex!["568c17ce5ef308bd9544e7b16f34089a2c2329193f31577a830ffe8a023a6874"].unchecked_into(),
				),
			],
			// Sudo
			hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
			// Endowed accounts
			vec![
				hex!["0c994e7589709a85128a6695254af16227f7873816ae0269aa705861c315ba1e"].into(),
				hex!["9e42365c1a43fe7bd886118f49a2247aabda7079c3e4c5288f41afadd7bb1963"].into(),
				hex!["6c1371ce4b06b8d191d6f552d716c00da31aca08a291ccbdeaf0f7aeae51201b"].into(),
			],
		),
		// Bootnodes
		vec!["/dns/bootnode-t1.reefscan.com/tcp/30334/p2p/12D3KooWKmFtS7BFtkkKWrP5ZcCpPFokmST2JFXFSsVBNeW5SXWg".parse().unwrap()],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("reef_testnet"),
		// Properties
		Some(reef_properties()),
		// Extensions
		Default::default(),
	))
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "WASM binary not available".to_string())?;
	Ok(ChainSpec::from_genesis(
		// Name
		"Reef Mainnet",
		// ID
		"reef_mainnet",
		ChainType::Live,
		move || mainnet_genesis(
			wasm_binary,
			// Initial authorities
			vec![
				(
					hex!["6c08c1f8e0cf1e200b24b43fca4c4e407b963b6b1e459d1aeff80c566a1da469"].into(),
					hex!["864eff3160ff8609c030316867630850a9d6e35c47d3efe54de44264fef7665e"].into(),
					hex!["dc563fb4641b61381ee79b98d54270e798515dac2193eef871ff18912f9a2960"].unchecked_into(),
					hex!["da9d50d8b4122743227cc7af017bd6fea7da3f2d86f786cf29605264935e6f3a"].unchecked_into(),
				),
				(
					hex!["5c22097b5c8b5912ce28b72ba4de52c3da8aca9379c748c1356a6642107d4c4a"].into(),
					hex!["543fd4fd9a284c0f955bb083ae6e0fe7a584eb6f6e72b386071a250b94f99a59"].into(),
					hex!["2ef0e32340579ca51b755ef8fa3fa4b7b0ada7214a7dc1fc96f84fc9d36b4a0c"].unchecked_into(),
					hex!["b66040bbbe3440f508b9275c404d38ee73f5e22eaaf89a55819d52d52d85ed56"].unchecked_into(),
				),
				(
					hex!["a67f388c1b8d68287fb3288b5aa36f069875c15ebcb9b1e4e62678aad6b24b44"].into(),
					hex!["ec912201d98911842b1a8e82983f71f2116dd8b898798ece4e1d210590de7d60"].into(),
					hex!["5031a3fbf6119e72d02d95d30fed820a3048ff9b80c44ff6215dee5aa6b8a233"].unchecked_into(),
					hex!["9603161d3d5c2ad0e334a789c408b5828c84291c395aff2faaec882ebd34dc17"].unchecked_into(),
				),
			],
			// Sudo
			hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into(),
			// Endowed accounts
			vec![
				// Investors
				(hex!["3c483acc759b79f8b12fa177e4bdfa0448a6ea03c389cf4db2b4325f0fc8f84a"].into(), 4_340_893_656 as u128),
				// Liquidity bridge reserves
				(hex!["5adebb35eb317412b58672db0434e4b112fcd27abaf28039f07c0db155b26650"].into(), 2_000_000_000 as u128),
				// Lockup & core nominators
				(hex!["746db342d3981b230804d1a187245e565f8eb3a2897f83d0d841cc52282e324c"].into(), 500_000_000 as u128),
				(hex!["da512d1335a62ad6f79baecfe87578c5d829113dc85dbb984d90a83f50680145"].into(), 500_000_000 as u128),
				(hex!["b493eacad9ca9d7d8dc21b940966b4db65dfbe01084f73c1eee2793b1b0a1504"].into(), 500_000_000 as u128),
				(hex!["849cf6f8a093c28fd0f699b47383767b0618f06aad9df61c4a9aff4af5809841"].into(), 250_000_000 as u128),
				(hex!["863bd6a38c7beb526be033068ac625536cd5d8a83cd51c1577a1779fab41655c"].into(), 250_000_000 as u128),
				(hex!["c2d2d7784e9272ef1785f92630dbce167a280149b22f2ae3b0262435e478884d"].into(), 250_000_000 as u128),
				// Sudo
				(hex!["9c48c0498bdf1d716f4544fc21f050963409f2db8154ba21e5233001202cbf08"].into(), 100_000_000 as u128),
				// Developer pool & faucet
				(hex!["1acc4a5c6361770eac4da9be1c37ac37ea91a55f57121c03240ceabf0b7c1c5e"].into(), 10_000_000 as u128),
			],
		),
		// Bootnodes
		vec![
			"/dns/bootnode-1.reefscan.com/tcp/30333/p2p/12D3KooWFHSc9cUcyNtavUkLg4VBAeBnYNgy713BnovUa9WNY5pp".parse().unwrap(),
			"/dns/bootnode-2.reefscan.com/tcp/30333/p2p/12D3KooWAQqcXvcvt4eVEgogpDLAdGWgR5bY1drew44We6FfJAYq".parse().unwrap(),
			"/dns/bootnode-3.reefscan.com/tcp/30333/p2p/12D3KooWCT7rnUmEK7anTp7svwr4GTs6k3XXnSjmgTcNvdzWzgWU".parse().unwrap(),
		],
		// Telemetry
		TelemetryEndpoints::new(vec![(TELEMETRY_URL.into(), 0)]).ok(),
		// Protocol ID
		Some("reef_mainnet"),
		// Properties
		Some(reef_properties()),
		// Extensions
		Default::default(),
	))
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	const INITIAL_BALANCE: u128 = 100_000_000 * REEF;
	const INITIAL_STAKING: u128 =   1_000_000 * REEF;

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), INITIAL_STAKING))
		.chain(endowed_accounts.iter().cloned().map(|k| (k, INITIAL_BALANCE)))
		// .chain(
		// 	get_all_module_accounts()
		// 		.iter()
		// 		.map(|x| (x.clone(), existential_deposit)),
		// )
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_balances: Some(BalancesConfig { balances }),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), get_session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		orml_tokens: Some(TokensConfig {
			endowed_accounts: endowed_accounts
				.iter()
				.flat_map(|x| {
					vec![
						(x.clone(), CurrencyId::Token(TokenSymbol::RUSD), INITIAL_BALANCE),
					]
				})
				.collect(),
		}),
		module_evm: Some(EVMConfig {
			accounts: evm_genesis_accounts,
		}),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_collective_Instance1: Some(Default::default()),
	}
}

fn mainnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AccountId, GrandpaId, BabeId)>,
	root_key: AccountId,
	endowed_accounts: Vec<(AccountId, Balance)>,
) -> GenesisConfig {

	let evm_genesis_accounts = evm_genesis();

	const INITIAL_STAKING: u128 = 1_000_000 * REEF;

	let balances = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), INITIAL_STAKING))
		.chain(endowed_accounts)
		.fold(
			BTreeMap::<AccountId, Balance>::new(),
			|mut acc, (account_id, amount)| {
				if let Some(balance) = acc.get_mut(&account_id) {
					*balance = balance
						.checked_add(amount)
						.expect("balance cannot overflow when building genesis");
				} else {
					acc.insert(account_id.clone(), amount);
				}
				acc
			},
		)
		.into_iter()
		.collect::<Vec<(AccountId, Balance)>>();

	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_indices: Some(IndicesConfig { indices: vec![] }),
		pallet_balances: Some(BalancesConfig { balances }),
		pallet_session: Some(SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.0.clone(), get_session_keys(x.2.clone(), x.3.clone())))
				.collect::<Vec<_>>(),
		}),
		pallet_staking: Some(StakingConfig {
			validator_count: initial_authorities.len() as u32 * 2,
			minimum_validator_count: initial_authorities.len() as u32,
			stakers: initial_authorities
				.iter()
				.map(|x| (x.0.clone(), x.1.clone(), INITIAL_STAKING, StakerStatus::Validator))
				.collect(),
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: sp_runtime::Perbill::from_percent(10),
			..Default::default()
		}),
		pallet_babe: Some(BabeConfig { authorities: vec![] }),
		pallet_grandpa: Some(GrandpaConfig { authorities: vec![] }),
		orml_tokens: Some(TokensConfig {
			endowed_accounts: vec![]
		}),
		module_evm: Some(EVMConfig {
			accounts: evm_genesis_accounts,
		}),
		pallet_sudo: Some(SudoConfig { key: root_key }),
		pallet_collective_Instance1: Some(Default::default()),
	}
}


/// Token
pub fn reef_properties() -> Properties {
	let mut p = Properties::new();
	p.insert("ss58format".into(), 42.into());
	p.insert("tokenDecimals".into(), 18.into());
	p.insert("tokenSymbol".into(), "REEF".into());
	p
}


/// Predeployed contract addresses
pub fn evm_genesis() -> BTreeMap<H160, module_evm::GenesisAccount<Balance, Nonce>> {
	let contracts_json = &include_bytes!("../../assets/bytecodes.json")[..];
	let contracts: Vec<(String, String, String)> = serde_json::from_slice(contracts_json).unwrap();
	let mut accounts = BTreeMap::new();
	for (_, address, code_string) in contracts {
		let account = module_evm::GenesisAccount {
			nonce: 0,
			balance: 0u128,
			storage: Default::default(),
			code: Bytes::from_str(&code_string).unwrap().0,
		};
		let addr = H160::from_slice(
			from_hex(address.as_str())
				.expect("predeploy-contracts must specify address")
				.as_slice(),
		);
		accounts.insert(addr, account);
	}
	accounts
}
