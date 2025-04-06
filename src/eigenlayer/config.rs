//! Configuration for Eigenlayer integration
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use eigensdk::eigen_common::getters::{get_signer_and_provider, SignerConfig};
use anyhow::Result;

/// Configuration for Eigenlayer integration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EigenConfig {
    /// Ethereum RPC URL
    pub eth_rpc_url: String,
    
    /// Chain ID of the target Ethereum network
    pub chain_id: u64,
    
    /// Registry addresses for Eigenlayer contracts
    pub contract_addresses: ContractAddresses,
    
    /// BLS private key path (if using BLS signatures)
    pub bls_private_key_path: Option<PathBuf>,
    
    /// ECDSA key configuration
    pub ecdsa_config: SignerConfigWrapper,
    
    /// Configuration for this AVS
    pub avs_config: AVSConfig,
    
    /// Gas price in gwei
    pub gas_price_gwei: Option<u64>,
    
    /// Whether to use mainnet or testnet
    pub is_mainnet: bool,
}

/// Wrapper for Eigensdk's SignerConfig
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignerConfigWrapper {
    /// Use keystore file authentication
    pub keystore: Option<KeystoreConfig>,
    
    /// Use private key authentication
    pub private_key: Option<String>,
    
    /// Use Fireblocks authentication
    pub fireblocks: Option<FireblocksConfig>,
}

/// Configuration for keystore-based authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeystoreConfig {
    /// Path to the keystore file
    pub path: PathBuf,
    
    /// Password for the keystore
    pub password: String,
}

/// Configuration for Fireblocks-based authentication
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FireblocksConfig {
    /// API key for Fireblocks
    pub api_key: String,
    
    /// Path to the private key file
    pub private_key_path: PathBuf,
    
    /// Fireblocks API URL
    pub api_url: String,
}

/// Addresses of Eigenlayer contracts
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContractAddresses {
    /// Address of the delegation manager contract
    pub delegation_manager: String,
    
    /// Address of the AVS directory contract
    pub avs_directory: String,
    
    /// Address of the staking strategy manager contract
    pub strategy_manager: String,
    
    /// Address of the Elixir Chain registry coordinator
    pub elxr_registry_coordinator: String,
    
    /// Address of the ELXR BLS public key compendium
    pub bls_public_key_compendium: String,
    
    /// Address of the slasher contract
    pub slasher: String,
}

/// Configuration specific to this AVS
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AVSConfig {
    /// Service name
    pub name: String,
    
    /// Version of the AVS software
    pub version: String,
    
    /// Metadata URL for the AVS
    pub metadata_url: Option<String>,
    
    /// Required quorums for operation
    pub required_quorums: Vec<u8>,
    
    /// Minimum operator stake (in ETH)
    pub min_operator_stake_eth: f64,
}

impl EigenConfig {
    /// Load configuration from a file
    pub fn from_file(path: &str) -> Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: EigenConfig = serde_json::from_str(&config_str)?;
        Ok(config)
    }
    
    /// Get a signer and provider using this configuration
    pub fn get_signer_and_provider(&self) -> Result<(ethers::signers::Wallet<ethers::signers::LocalWallet>, ethers::providers::Provider<ethers::providers::Http>)> {
        // Convert to the SDK's signer config format
        let signer_config = self.to_signer_config()?;
        
        // Use the SDK's built-in function
        let (signer, provider) = get_signer_and_provider(
            &signer_config,
            &self.eth_rpc_url,
            self.chain_id,
        )?;
        
        Ok((signer, provider))
    }
    
    /// Convert our wrapper to the SDK's format
    fn to_signer_config(&self) -> Result<SignerConfig> {
        let signer_config = if let Some(keystore) = &self.ecdsa_config.keystore {
            SignerConfig::Keystore {
                path: keystore.path.to_string_lossy().to_string(),
                password: keystore.password.clone(),
            }
        } else if let Some(private_key) = &self.ecdsa_config.private_key {
            SignerConfig::PrivateKey(private_key.clone())
        } else if let Some(fireblocks) = &self.ecdsa_config.fireblocks {
            SignerConfig::Fireblocks {
                api_key: fireblocks.api_key.clone(),
                private_key_path: fireblocks.private_key_path.to_string_lossy().to_string(),
                api_url: fireblocks.api_url.clone(),
            }
        } else {
            return Err(anyhow::anyhow!("No valid signer configuration found"));
        };
        
        Ok(signer_config)
    }
}
