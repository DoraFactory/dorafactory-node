use cumulus_primitives_core::ParaId;
use dorafactory_node_runtime::{
    AccountId, Signature, SudoConfig, TokensConfig, EXISTENTIAL_DEPOSIT,
};
use hex_literal::hex;
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{
    crypto::{Ss58Codec, UncheckedInto},
    sr25519, Pair, Public,
};
use sp_runtime::{
    traits::{IdentifyAccount, Verify},
    AccountId32,
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec =
    sc_service::GenericChainSpec<dorafactory_node_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_public_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/dora-ksm-mainnet.json")[..])
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// The relay chain of the Parachain.
    pub relay_chain: String,
    /// The id of the Parachain.
    pub para_id: u32,
}

impl Extensions {
    /// Try to get the extension from the given `ChainSpec`.
    pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
        sc_chain_spec::get_extension(chain_spec.extensions())
    }
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate collator keys from seed.
///
/// This function's return type must always match the session keys of the chain in tuple format.
pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
    get_public_from_seed::<AuraId>(seed)
}

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_public_from_seed::<TPublic>(seed)).into_account()
}

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn template_session_keys(keys: AuraId) -> dorafactory_node_runtime::SessionKeys {
    dorafactory_node_runtime::SessionKeys { aura: keys }
}

pub fn staging_config() -> ChainSpec {
    let mainnet_para_id: u32 = 2115;
    ChainSpec::from_genesis(
        // Name
        "Dorafactory Network",
        // ID
        "dorafactory",
        ChainType::Live,
        move || {
            dorafactory_genesis(
                // subkey inspect "$SECRET"
                get_root(),
                // initial collators.
                vec![
                    (
                        hex!["123c0de5a0486486e3df5740e92527ab79a6d57556696c91406272e940f1144a"]
                            .into(),
                        hex!["123c0de5a0486486e3df5740e92527ab79a6d57556696c91406272e940f1144a"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["804d98125209e39771eaab2bc62a5f54f6a84f429e59f41c591b593b06ba5027"]
                            .into(),
                        hex!["804d98125209e39771eaab2bc62a5f54f6a84f429e59f41c591b593b06ba5027"]
                            .unchecked_into(),
                    ),
                    (
                        hex!["f0a9fe6b6df079bb61eb750bd49f12483c9a0d64c8dc8f3f565a7768fef0556b"]
                            .into(),
                        hex!["f0a9fe6b6df079bb61eb750bd49f12483c9a0d64c8dc8f3f565a7768fef0556b"]
                            .unchecked_into(),
                    ),
                ],
                vec![get_root()],
                mainnet_para_id.into(),
            )
        },
        // Bootnodes
        Vec::new(),
        // Telemetry
        None,
        // Protocol ID
        Some("DORA KSM Parachain"),
        // Fork ID
        None,
        // Properties
        Some(get_properties()),
        // Extensions
        Extensions {
            relay_chain: "kusama".into(), // You MUST set this to the correct network!
            para_id: mainnet_para_id.into(),
        },
    )
}

pub fn development_config() -> ChainSpec {
    let dev_para_id: u32 = 2115;
    ChainSpec::from_genesis(
        // Name
        "Dorafactory Network",
        // ID
        "dorafactory",
        ChainType::Local,
        move || {
            dorafactory_genesis(
                // subkey inspect "$SECRET"
                hex!["34c63c6b3213570b0513c706f6c49a4ce253570ac213e53c919d2cd6f8913a07"].into(),
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
                vec![
                    hex!["34c63c6b3213570b0513c706f6c49a4ce253570ac213e53c919d2cd6f8913a07"].into(),
                ],
                dev_para_id.into(),
            )
        },
        // Bootnodes
        Vec::new(),
        // Telemetry
        None,
        // Protocol ID
        Some("DORA KSM Parachain"),
        // Fork ID
        None,
        // Properties
        Some(get_properties()),
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: dev_para_id.into(),
        },
    )
}

pub fn local_testnet_config() -> ChainSpec {
    // Give your base currency a unit name and decimal places
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DORA".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 42.into());

    ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            dorafactory_genesis(
                // subkey inspect "$SECRET"
                hex!["34c63c6b3213570b0513c706f6c49a4ce253570ac213e53c919d2cd6f8913a07"].into(),
                // initial collators.
                vec![
                    (
                        get_account_id_from_seed::<sr25519::Public>("Alice"),
                        get_collator_keys_from_seed("Alice"),
                    ),
                    (
                        get_account_id_from_seed::<sr25519::Public>("Bob"),
                        get_collator_keys_from_seed("Bob"),
                    ),
                ],
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
                2000.into(),
            )
        },
        // Bootnodes
        Vec::new(),
        // Telemetry
        None,
        // Protocol ID
        Some("dorafactory-node"),
        // Fork ID
        None,
        // Properties
        Some(properties),
        // Extensions
        Extensions {
            relay_chain: "rococo-local".into(), // You MUST set this to the correct network!
            para_id: 2000,
        },
    )
}

fn dorafactory_genesis(
    root_key: AccountId,
    invulnerables: Vec<(AccountId, AuraId)>,
    endowed_accounts: Vec<AccountId>,
    id: ParaId,
) -> dorafactory_node_runtime::GenesisConfig {
    dorafactory_node_runtime::GenesisConfig {
        system: dorafactory_node_runtime::SystemConfig {
            code: dorafactory_node_runtime::WASM_BINARY
                .expect("WASM binary was not build, please build it!")
                .to_vec(),
        },
        balances: dorafactory_node_runtime::BalancesConfig {
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 30_000_000_000_000_000))
                .collect(),
        },
        parachain_info: dorafactory_node_runtime::ParachainInfoConfig { parachain_id: id },
        collator_selection: dorafactory_node_runtime::CollatorSelectionConfig {
            invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
            candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
            ..Default::default()
        },
        session: dorafactory_node_runtime::SessionConfig {
            keys: invulnerables
                .into_iter()
                .map(|(acc, aura)| {
                    (
                        acc.clone(),                 // account id
                        acc,                         // validator id
                        template_session_keys(aura), // session keys
                    )
                })
                .collect(),
        },
        // no need to pass anything to aura, in fact it will panic if we do. Session will take care
        // of this.
        aura: Default::default(),
        aura_ext: Default::default(),
        parachain_system: Default::default(),
        polkadot_xcm: Default::default(),
        tokens: TokensConfig { balances: vec![] },
        sudo: SudoConfig {
            key: Some(root_key),
        },
        dora_rewards: dorafactory_node_runtime::DoraRewardsConfig {
            // set the funds
            funded_amount: 0,
        },
    }
}

fn get_properties() -> Properties {
    let mut properties = sc_chain_spec::Properties::new();
    properties.insert("tokenSymbol".into(), "DORA".into());
    properties.insert("tokenDecimals".into(), 12.into());
    properties.insert("ss58Format".into(), 128.into());
    properties
}

fn get_root() -> AccountId {
    // KSM Sudo Account
    AccountId32::from_string("5Ci36kbH533VyL5iYyFQ8QkR3eEc5Dwu1V8LxX8QcniJqxyb").unwrap()
}
