//! Operator management for Eigenlayer integration
use crate::eigenlayer::{
    client::EigenlayerClient,
    types::{RestakeInfo, QuorumInfo, OperatorStatus},
};
use anyhow::{Result, Context};
use log::{info, error, warn, debug};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};
use tokio::runtime::Runtime;

/// Information about an operator in the Eigenlayer ecosystem
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct OperatorInfo {
    /// Basic restaking information
    pub restake_info: RestakeInfo,
    
    /// Last time this operator was updated
    pub last_updated: u64,
    
    /// Reliability score (0-100)
    pub reliability_score: u8,
    
    /// Number of blocks validated
    pub blocks_validated: u64,
    
    /// Number of slashes received
    pub slashes: u32,
}

/// Manages a set of operators for a specific chain
pub struct OperatorManager {
    /// Client for interacting with Eigenlayer contracts
    client: Arc<EigenlayerClient>,
    
    /// Cache of operator information
    operators: RwLock<HashMap<Vec<u8>, OperatorInfo>>,
    
    /// Cache of quorum information
    quorums: RwLock<HashMap<u8, QuorumInfo>>,
    
    /// Tokio runtime for async operations
    runtime: Arc<Runtime>,
    
    /// Last time the operators were refreshed
    last_refresh: Mutex<u64>,
    
    /// Maximum age of operator information before refresh (in seconds)
    max_cache_age: u64,
}

impl OperatorManager {
    /// Create a new operator manager
    pub fn new(client: Arc<EigenlayerClient>) -> Result<Self> {
        // Default cache age - 5 minutes
        const DEFAULT_CACHE_AGE: u64 = 300;
        
        // Create runtime for async operations
        let runtime = Arc::new(Runtime::new()?);
        
        Ok(Self {
            client,
            operators: RwLock::new(HashMap::new()),
            quorums: RwLock::new(HashMap::new()),
            runtime,
            last_refresh: Mutex::new(0),
            max_cache_age: DEFAULT_CACHE_AGE,
        })
    }
    
    /// Set the maximum age of cached operator information
    pub fn set_max_cache_age(&mut self, age_seconds: u64) {
        self.max_cache_age = age_seconds;
    }
    
    /// Get the current timestamp
    fn current_time() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
    
    /// Check if the cache needs refreshing
    fn needs_refresh(&self) -> bool {
        let now = Self::current_time();
        let last = *self.last_refresh.lock().unwrap();
        
        now - last > self.max_cache_age
    }
    
    /// Update the cached operator information
    pub fn refresh_operators(&self) -> Result<()> {
        if !self.needs_refresh() {
            debug!("Using cached operator information");
            return Ok(());
        }
        
        info!("Refreshing operator information from Eigenlayer");
        
        // Get all operators from the client
        let operators = self.client.get_all_operators()?;
        
        // Update the cache
        let mut cache = self.operators.write().unwrap();
        for op_info in operators {
            let operator_id = op_info.operator_address.clone();
            
            // If we already have this operator, update it while preserving stats
            if let Some(existing) = cache.get(&operator_id) {
                cache.insert(operator_id, OperatorInfo {
                    restake_info: op_info,
                    last_updated: Self::current_time(),
                    reliability_score: existing.reliability_score,
                    blocks_validated: existing.blocks_validated,
                    slashes: existing.slashes,
                });
            } else {
                // New operator
                cache.insert(operator_id, OperatorInfo {
                    restake_info: op_info,
                    last_updated: Self::current_time(),
                    reliability_score: 100, // Start with perfect score
                    blocks_validated: 0,
                    slashes: 0,
                });
            }
        }
        
        // Update last refresh time
        *self.last_refresh.lock().unwrap() = Self::current_time();
        
        info!("Cached {} operators", cache.len());
        
        Ok(())
    }
    
    /// Update the cached quorum information
    pub fn refresh_quorums(&self) -> Result<()> {
        if !self.needs_refresh() {
            debug!("Using cached quorum information");
            return Ok(());
        }
        
        info!("Refreshing quorum information from Eigenlayer");
        
        // Get all quorums from the client
        let quorums = self.client.get_all_quorums()?;
        
        // Update the cache
        let mut cache = self.quorums.write().unwrap();
        for quorum_info in quorums {
            cache.insert(quorum_info.quorum_id, quorum_info);
        }
        
        info!("Cached {} quorums", cache.len());
        
        Ok(())
    }
    
    /// Get information about a specific operator
    pub fn get_operator(&self, operator_address: &[u8]) -> Result<OperatorInfo> {
        // Refresh if needed
        self.refresh_operators()?;
        
        // Try to get from cache first
        {
            let cache = self.operators.read().unwrap();
            if let Some(info) = cache.get(operator_address) {
                return Ok(info.clone());
            }
        }
        
        // Not in cache, try to get directly
        let op_info = self.client.get_operator_info(operator_address)?;
        
        // Create and cache a new operator info
        let info = OperatorInfo {
            restake_info: op_info,
            last_updated: Self::current_time(),
            reliability_score: 100, // Start with perfect score
            blocks_validated: 0,
            slashes: 0,
        };
        
        // Update cache
        let mut cache = self.operators.write().unwrap();
        cache.insert(operator_address.to_vec(), info.clone());
        
        Ok(info)
    }
    
    /// Get all operators
    pub fn get_all_operators(&self) -> Result<Vec<OperatorInfo>> {
        // Refresh if needed
        self.refresh_operators()?;
        
        // Get from cache
        let cache = self.operators.read().unwrap();
        let operators: Vec<OperatorInfo> = cache.values().cloned().collect();
        
        Ok(operators)
    }
    
    /// Get active operators only (filtered by status)
    pub fn get_active_operators(&self) -> Result<Vec<OperatorInfo>> {
        let all_ops = self.get_all_operators()?;
        
        // Filter to only active operators
        let active_ops = all_ops
            .into_iter()
            .filter(|op| op.restake_info.status == OperatorStatus::Active)
            .collect();
            
        Ok(active_ops)
    }
    
    /// Get operators for a specific quorum
    pub fn get_operators_in_quorum(&self, quorum_id: u8) -> Result<Vec<OperatorInfo>> {
        let all_ops = self.get_all_operators()?;
        
        // Filter to operators in this quorum
        let quorum_ops = all_ops
            .into_iter()
            .filter(|op| op.restake_info.quorum_ids.contains(&quorum_id))
            .collect();
            
        Ok(quorum_ops)
    }
    
    /// Get information about a specific quorum
    pub fn get_quorum(&self, quorum_id: u8) -> Result<QuorumInfo> {
        // Refresh if needed
        self.refresh_quorums()?;
        
        // Try to get from cache first
        {
            let cache = self.quorums.read().unwrap();
            if let Some(info) = cache.get(&quorum_id) {
                return Ok(info.clone());
            }
        }
        
        // Not in cache, try to get directly
        let quorum_info = self.client.get_quorum_info(quorum_id)?;
        
        // Update cache
        let mut cache = self.quorums.write().unwrap();
        cache.insert(quorum_id, quorum_info.clone());
        
        Ok(quorum_info)
    }
    
    /// Get all quorums
    pub fn get_all_quorums(&self) -> Result<Vec<QuorumInfo>> {
        // Refresh if needed
        self.refresh_quorums()?;
        
        // Get from cache
        let cache = self.quorums.read().unwrap();
        let quorums: Vec<QuorumInfo> = cache.values().cloned().collect();
        
        Ok(quorums)
    }
    
    /// Register a new operator
    pub fn register_operator(&self, 
                            operator_address: &[u8], 
                            bls_public_key: &[u8], 
                            bls_signature: &[u8]) -> Result<()> {
        // Call client to register
        self.client.register_operator(operator_address, bls_public_key, bls_signature)?;
        
        // Force refresh to get the new operator
        *self.last_refresh.lock().unwrap() = 0;
        self.refresh_operators()?;
        
        Ok(())
    }
    
    /// Record a successful block validation by an operator
    pub fn record_successful_validation(&self, operator_address: &[u8]) -> Result<()> {
        let mut cache = self.operators.write().unwrap();
        
        if let Some(mut info) = cache.get_mut(operator_address) {
            info.blocks_validated += 1;
            
            // Increase reliability score if it's not already perfect
            if info.reliability_score < 100 {
                info.reliability_score = (info.reliability_score + 1).min(100);
            }
            
            info.last_updated = Self::current_time();
        } else {
            warn!("Tried to record validation for unknown operator: {:?}", operator_address);
        }
        
        Ok(())
    }
    
    /// Record a slash event for an operator
    pub fn record_slash(&self, operator_address: &[u8], severity: u8) -> Result<()> {
        let mut cache = self.operators.write().unwrap();
        
        if let Some(mut info) = cache.get_mut(operator_address) {
            info.slashes += 1;
            
            // Decrease reliability score based on severity (1-100)
            let decrease = severity.min(100);
            info.reliability_score = info.reliability_score.saturating_sub(decrease);
            
            info.last_updated = Self::current_time();
        } else {
            warn!("Tried to record slash for unknown operator: {:?}", operator_address);
        }
        
        Ok(())
    }
}
