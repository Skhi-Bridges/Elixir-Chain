//! Client implementation for interacting with Eigenlayer contracts
use crate::eigenlayer::{
    config::EigenConfig,
    types::{RestakeInfo, QuorumInfo, OperatorStatus, address_to_bytes, bytes_to_address},
};
use anyhow::{Result, Context};
use eigensdk::{
    eigen_client_avsregistry::AvsRegistryClient,
    eigen_client_elcontracts::ELContracts,
    eigen_types::{
        Quorum, QuorumHeaderHash, QuorumNumbers, OperatorId, OperatorStateRetriever,
        RegistryCoordinatorClient, StakerStateRetriever, ZERO_ADDRESS,
    },
    eigen_crypto_bls::{BlsKeyPair, PublicKey, SecretKey}
};
use ethers::{
    providers::{Http, Provider, Middleware},
    signers::{LocalWallet, Signer, Wallet},
};
use alloy_primitives::{Address, U256};
use tokio::runtime::Runtime;
use std::{sync::Arc, str::FromStr, collections::HashMap};
use log::{info, error, debug};

/// Client for interacting with Eigenlayer contracts
pub struct EigenlayerClient {
    /// Ethereum provider
    provider: Arc<Provider<Http>>,
    
    /// Signer wallet
    wallet: Wallet<LocalWallet>,
    
    /// EL contracts client
    el_contracts: ELContracts<Provider<Http>, Wallet<LocalWallet>>,
    
    /// AVS registry client
    avs_registry: AvsRegistryClient<Provider<Http>, Wallet<LocalWallet>>,
    
    /// Configuration
    config: EigenConfig,
    
    /// Tokio runtime for async operations
    runtime: Arc<Runtime>,
}

impl EigenlayerClient {
    /// Create a new client instance
    pub fn new(config: EigenConfig) -> Result<Self> {
        // Create runtime for async operations
        let runtime = Arc::new(Runtime::new()?);
        
        // Use the runtime to get signer and provider
        let (wallet, provider) = runtime.block_on(async {
            config.get_signer_and_provider()
                .context("Failed to create signer and provider")
        })?;
        
        let provider = Arc::new(provider);
        
        // Create registry coordinator address from the config
        let registry_coordinator_addr = Address::from_str(&config.contract_addresses.elxr_registry_coordinator)
            .context("Invalid registry coordinator address")?;
            
        // Create BLS public key compendium address from the config
        let bls_pk_compendium_addr = Address::from_str(&config.contract_addresses.bls_public_key_compendium)
            .context("Invalid BLS public key compendium address")?;
            
        // Create registry client
        let avs_registry = runtime.block_on(async {
            AvsRegistryClient::new(
                provider.clone(),
                wallet.clone(),
                registry_coordinator_addr,
                bls_pk_compendium_addr,
            )
            .await
            .context("Failed to create AVS registry client")
        })?;
        
        // Create EL contracts client with addresses from config
        let el_contracts = runtime.block_on(async {
            let delegation_manager_addr = Address::from_str(&config.contract_addresses.delegation_manager)
                .context("Invalid delegation manager address")?;
                
            let avs_directory_addr = Address::from_str(&config.contract_addresses.avs_directory)
                .context("Invalid AVS directory address")?;
                
            let strategy_manager_addr = Address::from_str(&config.contract_addresses.strategy_manager)
                .context("Invalid strategy manager address")?;
                
            let slasher_addr = Address::from_str(&config.contract_addresses.slasher)
                .context("Invalid slasher address")?;
                
            ELContracts::new(
                provider.clone(),
                wallet.clone(),
                delegation_manager_addr,
                avs_directory_addr,
                strategy_manager_addr,
                slasher_addr,
            )
            .await
            .context("Failed to create EL contracts client")
        })?;
        
        Ok(Self {
            provider,
            wallet,
            el_contracts,
            avs_registry,
            config,
            runtime,
        })
    }
    
    /// Get information about a specific operator
    pub fn get_operator_info(&self, operator_address: &[u8]) -> Result<RestakeInfo> {
        let operator_addr = bytes_to_address(operator_address)
            .context("Invalid operator address")?;
            
        // Run the async operations in the runtime
        self.runtime.block_on(async {
            // Get the operator's status
            let is_registered = self.avs_registry.is_operator_registered(operator_addr).await?;
            
            if !is_registered {
                return Err(anyhow::anyhow!("Operator is not registered"));
            }
            
            // Get quorums the operator is registered for
            let quorum_numbers = self.avs_registry.get_operator_quorum_bits_at_block_number(
                operator_addr,
                None, // Use latest block
            ).await?;
            
            // Convert quorum numbers to vector of quorum IDs
            let quorum_ids = quorum_numbers.get_quorum_ids();
            
            // Get operator's BLS public key
            let public_key = self.avs_registry.get_operator_pubkey_hash(operator_addr)
                .await?
                .to_vec();
                
            // Get restaked amount from EL contracts
            let operator_shares = self.el_contracts.get_operator_shares(operator_addr).await?;
            
            // Calculate total restaked amount by summing all shares
            let mut restaked_amount: u128 = 0;
            for (_, amount) in operator_shares.iter() {
                restaked_amount += amount.to::<u128>();
            }
            
            // Determine operator status
            let status = if self.avs_registry.is_operator_registered(operator_addr).await? {
                OperatorStatus::Active
            } else {
                OperatorStatus::Removed
            };
            
            Ok(RestakeInfo {
                operator_address: address_to_bytes(&operator_addr),
                restaked_amount,
                public_key,
                quorum_ids: quorum_ids.into_iter().map(|id| id as u8).collect(),
                status,
            })
        })
    }
    
    /// Get information about a specific quorum
    pub fn get_quorum_info(&self, quorum_id: u8) -> Result<QuorumInfo> {
        self.runtime.block_on(async {
            // Get operators in this quorum
            let operators = self.avs_registry.get_operators_in_quorum_at_block_number(
                quorum_id as u8,
                None, // Use latest block
            ).await?;
            
            let operator_count = operators.len() as u32;
            
            // Get minimum stake for this quorum
            let quorum_param = self.avs_registry.get_quorum_params(quorum_id as u8).await?;
            let min_stake = quorum_param.minimum_stake.to::<u128>();
            
            // Calculate total stake in this quorum
            let mut total_stake: u128 = 0;
            for operator in operators {
                let operator_shares = self.el_contracts.get_operator_shares(operator).await?;
                for (_, amount) in operator_shares.iter() {
                    total_stake += amount.to::<u128>();
                }
            }
            
            Ok(QuorumInfo {
                quorum_id,
                operator_count,
                total_stake,
                min_stake,
            })
        })
    }
    
    /// Register a new operator with Eigenlayer
    pub fn register_operator(&self, 
                             operator_address: &[u8], 
                             bls_public_key: &[u8], 
                             bls_signature: &[u8]) -> Result<()> {
        let operator_addr = bytes_to_address(operator_address)
            .context("Invalid operator address")?;
            
        // Convert BLS public key to the format expected by the SDK
        let public_key = PublicKey::from_bytes(bls_public_key)
            .context("Invalid BLS public key")?;
            
        // This would normally come from the operator's signed registration
        // For this example, we're converting from the provided signature bytes
        let signature = self.runtime.block_on(async {
            self.avs_registry.register_operator(
                operator_addr, 
                public_key,
                self.config.avs_config.required_quorums.clone(),
                // In a real implementation, we would use the operator's actual signature
                // For now, we'll use a placeholder
                [0u8; 64].to_vec()
            ).await
        })?;
        
        info!("Registered operator: {:?}", operator_addr);
        
        Ok(())
    }
    
    /// Get all active operators and their information
    pub fn get_all_operators(&self) -> Result<Vec<RestakeInfo>> {
        self.runtime.block_on(async {
            // Get all operators registered with the AVS
            let operators = self.avs_registry.get_all_operators().await?;
            
            let mut operator_infos = Vec::new();
            for operator_addr in operators {
                // Skip invalid operators
                match self.get_operator_info(&address_to_bytes(&operator_addr)) {
                    Ok(info) => operator_infos.push(info),
                    Err(e) => error!("Error getting operator info for {:?}: {:?}", operator_addr, e),
                }
            }
            
            Ok(operator_infos)
        })
    }
    
    /// Get information for all quorums
    pub fn get_all_quorums(&self) -> Result<Vec<QuorumInfo>> {
        self.runtime.block_on(async {
            // Get total number of quorums
            let quorum_count = self.avs_registry.get_quorum_count().await?;
            
            let mut quorum_infos = Vec::new();
            for i in 0..quorum_count {
                match self.get_quorum_info(i as u8) {
                    Ok(info) => quorum_infos.push(info),
                    Err(e) => error!("Error getting quorum info for {}: {:?}", i, e),
                }
            }
            
            Ok(quorum_infos)
        })
    }
    
    /// Get the current address of the signer
    pub fn get_signer_address(&self) -> Address {
        self.wallet.address()
    }
    
    /// Get the current blockchain ID
    pub fn get_chain_id(&self) -> u64 {
        self.config.chain_id
    }
}
