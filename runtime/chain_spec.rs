use sc_service::ChainType;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use elxr_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
    SudoConfig, SystemConfig, WASM_BINARY, Signature, 
    DaemonlessOracleConfig, UnifiedLiquidityPoolConfig, 
    ZeroSpreadDexConfig, TelemetryConfig
};

// Import error correction components
use elxr_runtime::error_correction::{
    ClassicalErrorCorrectionConfig,
    BridgeErrorCorrectionConfig, 
    QuantumErrorCorrectionConfig
};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate authority keys for Aura and Grandpa.
pub fn authority_keys_from_seed(seed: &str) -> (AuraId, GrandpaId) {
    (
        get_from_seed::<AuraId>(seed),
        get_from_seed::<GrandpaId>(seed),
    )
}

pub fn development_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "elxr_dev",
        ChainType::Development,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        Some(
            serde_json::from_str(
                r#"{
                    "tokenDecimals": 12,
                    "tokenSymbol": "ELXR",
                    "ss58Format": 42
                }"#,
            )
            .expect("valid json; qed"),
        ),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Elixir Chain Local Testnet",
        // ID
        "elxr_local_testnet",
        ChainType::Local,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    authority_keys_from_seed("Alice"),
                    authority_keys_from_seed("Bob"),
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
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Fork ID
        None,
        // Properties
        Some(
            serde_json::from_str(
                r#"{
                    "tokenDecimals": 12,
                    "tokenSymbol": "ELXR",
                    "ss58Format": 42
                }"#,
            )
            .expect("valid json; qed"),
        ),
        // Extensions
        None,
    ))
}

pub fn jam_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Elixir Chain JAM Testnet",
        // ID
        "elxr_jam_testnet",
        ChainType::Live,
        move || {
            testnet_genesis(
                wasm_binary,
                // Initial PoA authorities (would be actual validator keys in production)
                vec![
                    authority_keys_from_seed("ElixirValidator1"),
                    authority_keys_from_seed("ElixirValidator2"),
                    authority_keys_from_seed("ElixirValidator3"),
                ],
                // Sudo account (governance controlled in production)
                get_account_id_from_seed::<sr25519::Public>("ElixirAdmin"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("ElixirTreasury"),
                    get_account_id_from_seed::<sr25519::Public>("ElixirValidator1"),
                    get_account_id_from_seed::<sr25519::Public>("ElixirValidator2"),
                    get_account_id_from_seed::<sr25519::Public>("ElixirValidator3"),
                    get_account_id_from_seed::<sr25519::Public>("ElixirAdmin"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        Some("elxr-jam-testnet"),
        // Fork ID
        None,
        // Properties
        Some(
            serde_json::from_str(
                r#"{
                    "tokenDecimals": 12,
                    "tokenSymbol": "ELXR",
                    "ss58Format": 42
                }"#,
            )
            .expect("valid json; qed"),
        ),
        // Extensions
        None,
    ))
}

/// Configure initial storage state for FRAME modules.
fn testnet_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 million ELXR.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 1_000_000_000_000_000))
                .collect(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities
                .iter()
                .map(|x| (x.1.clone(), 1))
                .collect(),
        },
        sudo: SudoConfig { key: Some(root_key) },
        
        // ELXR specific configurations
        daemonless_oracle: DaemonlessOracleConfig {
            initial_data_providers: vec![],
            initial_feed_ids: vec![],
        },
        unified_liquidity_pool: UnifiedLiquidityPoolConfig {
            initial_pools: vec![],
            initial_asset_ids: vec![],
        },
        zero_spread_dex: ZeroSpreadDexConfig {
            trading_pairs: vec![],
            min_liquidity: 1000,
        },
        telemetry: TelemetryConfig {
            authorized_endpoints: vec![],
        },
        
        // Error correction configurations
        classical_error_correction: ClassicalErrorCorrectionConfig {
            redundancy_level: 3,
            recovery_threshold: 2,
        },
        bridge_error_correction: BridgeErrorCorrectionConfig {
            verification_rounds: 2,
            timeout_period: 1000,
        },
        quantum_error_correction: QuantumErrorCorrectionConfig {
            code_distance: 3,
            syndrome_measurements: 4,
        },
    }
}
