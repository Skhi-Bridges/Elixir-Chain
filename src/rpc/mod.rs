//! RPC module for ELXR chain
//!
//! This module provides RPC endpoints for the ELXR chain,
//! including fermentation telemetry data access and chain-specific operations.

pub mod telemetry;
pub mod eigenlayer;
pub mod quantum_nft;

use jsonrpsee::RpcModule;
use std::error::Error;
use std::sync::Arc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;

use self::telemetry::{TelemetryRpc, TelemetryRpcImpl, TelemetryRpcServer};
use self::eigenlayer::{ElxrEigenlayerRpc, ElxrEigenlayerRpcImpl, ElxrEigenlayerRpcServer};
use self::quantum_nft::{ElxrQuantumNftRpc, ElxrQuantumNft};

/// Create and configure the RPC module with all ELXR endpoints
pub fn create_full_rpc_module<C, B>(client: Arc<C>) -> Result<RpcModule<()>, Box<dyn Error>> 
where
    B: BlockT,
    C: HeaderBackend<B> + ProvideRuntimeApi<B> + Send + Sync + 'static,
    C::Api: telemetry::TelemetryRuntimeApi<B> + 
           eigenlayer::ElxrEigenlayerRuntimeApi<B, crate::runtime::AccountId, crate::runtime::Balance, crate::runtime::BlockNumber> + 
           crate::runtime_api::ELXRRuntimeApi<B>,
{
    let mut module = RpcModule::new(());
    
    // Create and register the telemetry RPC implementation
    let telemetry_rpc = TelemetryRpcImpl::new(client.clone());
    module.merge(telemetry_rpc.into_rpc())?;
    
    // Create and register the Eigenlayer RPC implementation
    let eigenlayer_rpc = ElxrEigenlayerRpcImpl::new(client.clone());
    module.merge(eigenlayer_rpc.into_rpc())?;
    
    // Create and register the SBX-Q999 Quantum NFT RPC implementation
    let quantum_nft_handler = ElxrQuantumNft::new(client.clone());
    module.merge(ElxrQuantumNftRpcServer::into_rpc(quantum_nft_handler))?;
    
    // Register other RPC implementations here as needed
    
    Ok(module)
}

/// Initialize all RPC services
/// This should be called during node startup
pub fn initialize() -> Result<(), Box<dyn Error>> {
    // Initialize the telemetry system
    crate::telemetry::initialize()?;
    
    // Initialize the Eigenlayer system
    crate::pallet::eigenlayer::initialize()?;
    
    Ok(())
}
