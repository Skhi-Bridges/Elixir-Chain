//! Eigenlayer RPC module for ELXR chain
//!
//! This module provides RPC endpoints for interacting with the Eigenlayer pallet,
//! allowing users to stake, unstake, and query staking information for kombucha production.

use frame_support::traits::Currency;
use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    RpcModule,
};
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::{error::Error, sync::Arc};

use crate::pallet::eigenlayer;
use crate::runtime::{AccountId, Balance, BlockNumber, Runtime};
use common::quantum_crypto::EigenlayerProof;

/// JSON-RPC spec for Eigenlayer RPC methods for ELXR chain
#[rpc(server, client)]
pub trait ElxrEigenlayerRpc<BlockHash> {
    /// Get staking parameters
    #[method(name = "eigenlayer_getStakingParams")]
    fn get_staking_params(&self, at: Option<BlockHash>) -> RpcResult<StakingParams>;
    
    /// Get a staking position by ID
    #[method(name = "eigenlayer_getPosition")]
    fn get_position(&self, position_id: u64, at: Option<BlockHash>) -> RpcResult<Option<StakingPositionInfo>>;
    
    /// Get all staking positions for an account
    #[method(name = "eigenlayer_getAccountPositions")]
    fn get_account_positions(&self, account: AccountId, at: Option<BlockHash>) -> RpcResult<Vec<StakingPositionInfo>>;
    
    /// Calculate potential staking rewards for a given amount and quality score
    #[method(name = "eigenlayer_calculatePotentialRewards")]
    fn calculate_potential_rewards(
        &self, 
        amount: Balance, 
        quality_score: u8, 
        fermentation_stage: u8, // Kombucha-specific parameter
        scoby_health: u8,      // Kombucha-specific parameter
        days: u32, 
        at: Option<BlockHash>
    ) -> RpcResult<PotentialRewards>;
    
    /// Verify a staking position's quantum proof
    #[method(name = "eigenlayer_verifyQuantumProof")]
    fn verify_quantum_proof(
        &self,
        position_id: u64,
        proof: EigenlayerProof,
        at: Option<BlockHash>
    ) -> RpcResult<bool>;
    
    /// Get SCOBY score for a specific batch
    #[method(name = "eigenlayer_getScobyScore")]
    fn get_scoby_score(
        &self,
        batch_id: Vec<u8>,
        at: Option<BlockHash>
    ) -> RpcResult<Option<ScobyScoreInfo>>;
}

/// Eigenlayer RPC implementation for ELXR
pub struct ElxrEigenlayerRpcImpl<C, B> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<B>,
}

/// Staking parameters returned by RPC
#[derive(Serialize, Deserialize)]
pub struct StakingParams {
    pub min_staking_amount: Balance,
    pub max_staking_amount: Balance,
    pub base_apr: u32,            // Base APR in basis points (e.g., 500 = 5%)
    pub quality_multiplier: u32,   // Multiplier for quality bonus in basis points
    pub fermentation_multiplier: u32, // Kombucha-specific: multiplier for fermentation stage
    pub scoby_health_multiplier: u32, // Kombucha-specific: multiplier for SCOBY health
    pub unstaking_period: u32,     // Unstaking period in days
}

/// Staking position information returned by RPC
#[derive(Serialize, Deserialize)]
pub struct StakingPositionInfo {
    pub position_id: u64,
    pub staker: AccountId,
    pub amount: Balance,
    pub batch_id: Vec<u8>,
    pub quality_score: u8,
    pub fermentation_stage: u8,    // Kombucha-specific
    pub scoby_health: u8,          // Kombucha-specific
    pub current_apr: u32,          // Current APR in basis points
    pub rewards_earned: Balance,
    pub stake_date: BlockNumber,
    pub unlock_date: Option<BlockNumber>,
    pub status: String,           // "Active", "Unlocking", "Unstaked"
}

/// Potential rewards calculation returned by RPC
#[derive(Serialize, Deserialize)]
pub struct PotentialRewards {
    pub principal: Balance,
    pub estimated_rewards: Balance,
    pub effective_apr: u32,       // Effective APR in basis points
    pub quality_bonus: u32,       // Quality bonus in basis points
    pub fermentation_bonus: u32,  // Fermentation stage bonus in basis points
    pub scoby_bonus: u32,         // SCOBY health bonus in basis points
}

/// SCOBY score information
#[derive(Serialize, Deserialize)]
pub struct ScobyScoreInfo {
    pub batch_id: Vec<u8>,
    pub scoby_health: u8,
    pub age_days: u32,
    pub thickness_mm: u32,
    pub culture_density: u8,
    pub ph_level: f32,
}

impl<C, B> ElxrEigenlayerRpcImpl<C, B> {
    /// Create a new Eigenlayer RPC implementation for ELXR
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: std::marker::PhantomData,
        }
    }
}

/// Error types specific to Eigenlayer RPC
#[derive(Debug, thiserror::Error)]
pub enum ElxrEigenlayerError {
    #[error("API error: {0}")]
    ApiError(String),
    
    #[error("Position not found")]
    PositionNotFound,
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Quantum verification failed")]
    QuantumVerificationFailed,
    
    #[error("SCOBY data not found")]
    ScobyDataNotFound,
}

impl From<ElxrEigenlayerError> for JsonRpseeError {
    fn from(e: ElxrEigenlayerError) -> Self {
        JsonRpseeError::Custom(format!("ELXR Eigenlayer error: {}", e))
    }
}

impl<C, B> ElxrEigenlayerRpc<<B as BlockT>::Hash> for ElxrEigenlayerRpcImpl<C, B>
where
    B: BlockT,
    C: HeaderBackend<B> + ProvideRuntimeApi<B> + Send + Sync + 'static,
    C::Api: eigenlayer::ElxrEigenlayerRuntimeApi<B, AccountId, Balance, BlockNumber>,
{
    fn get_staking_params(&self, at: Option<<B as BlockT>::Hash>) -> RpcResult<StakingParams> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let params = api.get_staking_params(&at)
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to get staking params: {:?}", e)))?;
        
        Ok(StakingParams {
            min_staking_amount: params.min_staking_amount,
            max_staking_amount: params.max_staking_amount,
            base_apr: params.base_apr,
            quality_multiplier: params.quality_multiplier,
            fermentation_multiplier: params.fermentation_multiplier,
            scoby_health_multiplier: params.scoby_health_multiplier,
            unstaking_period: params.unstaking_period,
        })
    }
    
    fn get_position(&self, position_id: u64, at: Option<<B as BlockT>::Hash>) -> RpcResult<Option<StakingPositionInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let position = api.get_position(&at, position_id)
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to get position: {:?}", e)))?;
        
        Ok(position.map(|p| StakingPositionInfo {
            position_id: p.position_id,
            staker: p.staker,
            amount: p.amount,
            batch_id: p.batch_id,
            quality_score: p.quality_score,
            fermentation_stage: p.fermentation_stage,
            scoby_health: p.scoby_health,
            current_apr: p.current_apr,
            rewards_earned: p.rewards_earned,
            stake_date: p.stake_date,
            unlock_date: p.unlock_date,
            status: p.status,
        }))
    }
    
    fn get_account_positions(&self, account: AccountId, at: Option<<B as BlockT>::Hash>) -> RpcResult<Vec<StakingPositionInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let positions = api.get_account_positions(&at, account)
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to get account positions: {:?}", e)))?;
        
        let result = positions.into_iter()
            .map(|p| StakingPositionInfo {
                position_id: p.position_id,
                staker: p.staker,
                amount: p.amount,
                batch_id: p.batch_id,
                quality_score: p.quality_score,
                fermentation_stage: p.fermentation_stage,
                scoby_health: p.scoby_health,
                current_apr: p.current_apr,
                rewards_earned: p.rewards_earned,
                stake_date: p.stake_date,
                unlock_date: p.unlock_date,
                status: p.status,
            })
            .collect();
        
        Ok(result)
    }
    
    fn calculate_potential_rewards(
        &self, 
        amount: Balance, 
        quality_score: u8, 
        fermentation_stage: u8,
        scoby_health: u8,
        days: u32, 
        at: Option<<B as BlockT>::Hash>
    ) -> RpcResult<PotentialRewards> {
        if quality_score > 100 {
            return Err(ElxrEigenlayerError::InvalidParameters("Quality score must be between 0 and 100".into()).into());
        }
        
        if fermentation_stage > 5 {
            return Err(ElxrEigenlayerError::InvalidParameters("Fermentation stage must be between 0 and 5".into()).into());
        }
        
        if scoby_health > 100 {
            return Err(ElxrEigenlayerError::InvalidParameters("SCOBY health must be between 0 and 100".into()).into());
        }
        
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let rewards = api.calculate_potential_rewards(&at, amount, quality_score, fermentation_stage, scoby_health, days)
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to calculate rewards: {:?}", e)))?;
        
        Ok(PotentialRewards {
            principal: rewards.principal,
            estimated_rewards: rewards.estimated_rewards,
            effective_apr: rewards.effective_apr,
            quality_bonus: rewards.quality_bonus,
            fermentation_bonus: rewards.fermentation_bonus,
            scoby_bonus: rewards.scoby_bonus,
        })
    }
    
    fn verify_quantum_proof(
        &self,
        position_id: u64,
        proof: EigenlayerProof,
        at: Option<<B as BlockT>::Hash>
    ) -> RpcResult<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let is_valid = api.verify_quantum_proof(&at, position_id, proof)
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to verify proof: {:?}", e)))?;
        
        Ok(is_valid)
    }
    
    fn get_scoby_score(
        &self,
        batch_id: Vec<u8>,
        at: Option<<B as BlockT>::Hash>
    ) -> RpcResult<Option<ScobyScoreInfo>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        let scoby_data = api.get_scoby_score(&at, batch_id.clone())
            .map_err(|e| ElxrEigenlayerError::ApiError(format!("Failed to get SCOBY score: {:?}", e)))?;
        
        Ok(scoby_data.map(|d| ScobyScoreInfo {
            batch_id,
            scoby_health: d.scoby_health,
            age_days: d.age_days,
            thickness_mm: d.thickness_mm,
            culture_density: d.culture_density,
            ph_level: d.ph_level,
        }))
    }
}
