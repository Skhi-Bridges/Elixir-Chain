//! RPC-Q999 Quantum NFT RPC implementation for ELXR chain
//!
//! This module provides RPC endpoints for interacting with quantum NFTs
//! representing verified kombucha cultures with SCOBY health metrics.

use jsonrpsee::{
    core::{Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorObject},
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

use common::nft::types::{
    TokenId, NFTMetadata, NFTCreationParams, NFTTransferParams,
    NFTQueryParams, NFTQueryResult, NFTSummary, ProductType,
    KombuchaMetadata, TeaBase,
};
use common::pallet::quantum_nft::rpc::QuantumNftRpcServer;
use crate::runtime_api::ELXRRuntimeApi;

/// Error type for quantum NFT RPC
pub enum Error {
    /// The NFT was not found
    NftNotFound,
    /// Runtime error
    Runtime(String),
    /// RPC error
    Rpc(JsonRpseeError),
}

impl From<Error> for JsonRpseeError {
    fn from(error: Error) -> Self {
        match error {
            Error::NftNotFound => CallError::Custom(ErrorObject::owned(
                1001,
                "NFT not found",
                None::<()>,
            ))
            .into(),
            Error::Runtime(message) => CallError::Custom(ErrorObject::owned(
                1002,
                format!("Runtime error: {}", message),
                None::<()>,
            ))
            .into(),
            Error::Rpc(error) => error,
        }
    }
}

#[rpc(client, server)]
pub trait ElxrQuantumNftRpc<BlockHash> {
    /// Get NFT by token ID
    #[method(name = "rpcq999_getNFT")]
    fn get_nft(&self, token_id: TokenId, at: Option<BlockHash>) -> RpcResult<Option<NFTMetadata>>;
    
    /// Query NFTs by parameters
    #[method(name = "rpcq999_queryNFTs")]
    fn query_nfts(&self, params: NFTQueryParams, at: Option<BlockHash>) -> RpcResult<NFTQueryResult>;
    
    /// Get Kombucha-specific NFT data
    #[method(name = "rpcq999_getKombuchaData")]
    fn get_kombucha_data(&self, token_id: TokenId, at: Option<BlockHash>) -> RpcResult<Option<KombuchaMetadata>>;
    
    /// Query NFTs by fermentation stage
    #[method(name = "rpcq999_queryByFermentationStage")]
    fn query_by_fermentation_stage(&self, stage: u8, at: Option<BlockHash>) -> RpcResult<NFTQueryResult>;
    
    /// Query NFTs by SCOBY health
    #[method(name = "rpcq999_queryByScobyHealth")]
    fn query_by_scoby_health(&self, min_health: u8, at: Option<BlockHash>) -> RpcResult<NFTQueryResult>;
    
    /// Query NFTs by tea base type
    #[method(name = "rpcq999_queryByTeaBase")]
    fn query_by_tea_base(&self, tea_base: TeaBase, at: Option<BlockHash>) -> RpcResult<NFTQueryResult>;
    
    /// Get total kombucha NFT count
    #[method(name = "rpcq999_getTotalKombuchaCount")]
    fn get_total_kombucha_count(&self, at: Option<BlockHash>) -> RpcResult<u64>;
    
    /// Get highest quality kombucha NFTs
    #[method(name = "rpcq999_getHighestQualityKombucha")]
    fn get_highest_quality_kombucha(&self, limit: u32, at: Option<BlockHash>) -> RpcResult<Vec<NFTSummary>>;
}

/// Quantum NFT RPC implementation for ELXR chain
pub struct ElxrQuantumNft<C, Block> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<Block>,
}

impl<C, Block> ElxrQuantumNft<C, Block> {
    /// Create a new RPC handler
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block> ElxrQuantumNftRpcServer<<Block as BlockT>::Hash> for ElxrQuantumNft<C, Block>
where
    Block: BlockT,
    C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
    C::Api: ELXRRuntimeApi<Block>,
{
    fn get_nft(&self, token_id: TokenId, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Option<NFTMetadata>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        api.get_nft(&at, token_id)
            .map_err(|e| Error::Runtime(format!("Error getting NFT: {:?}", e)).into())
    }
    
    fn query_nfts(&self, params: NFTQueryParams, at: Option<<Block as BlockT>::Hash>) -> RpcResult<NFTQueryResult> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())
    }
    
    fn get_kombucha_data(&self, token_id: TokenId, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Option<KombuchaMetadata>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // First get the NFT
        let nft = api.get_nft(&at, token_id)
            .map_err(|e| Error::Runtime(format!("Error getting NFT: {:?}", e)).into())?;
        
        // Extract kombucha data if present
        match nft {
            Some(metadata) => match metadata.product_data {
                common::nft::types::ProductMetadata::Kombucha(data) => Ok(Some(data)),
                _ => Ok(None),
            },
            None => Ok(None),
        }
    }
    
    fn query_by_fermentation_stage(&self, stage: u8, at: Option<<Block as BlockT>::Hash>) -> RpcResult<NFTQueryResult> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // Get all kombucha NFTs
        let params = NFTQueryParams {
            owner: None,
            product_type: Some(ProductType::Kombucha),
            min_quality: None,
            certified_organic_only: false,
        };
        
        let all_nfts = api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())?;
        
        // Filter by fermentation stage
        let mut filtered_nfts = Vec::new();
        let mut total_count = 0;
        
        for summary in all_nfts.nfts {
            // Get full NFT data
            if let Ok(Some(nft)) = api.get_nft(&at, summary.token_id) {
                if let common::nft::types::ProductMetadata::Kombucha(kombucha_data) = nft.product_data {
                    if kombucha_data.fermentation_stage == stage {
                        filtered_nfts.push(summary);
                        total_count += 1;
                    }
                }
            }
        }
        
        Ok(NFTQueryResult {
            nfts: filtered_nfts,
            total_count,
        })
    }
    
    fn query_by_scoby_health(&self, min_health: u8, at: Option<<Block as BlockT>::Hash>) -> RpcResult<NFTQueryResult> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // Get all kombucha NFTs
        let params = NFTQueryParams {
            owner: None,
            product_type: Some(ProductType::Kombucha),
            min_quality: None,
            certified_organic_only: false,
        };
        
        let all_nfts = api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())?;
        
        // Filter by SCOBY health
        let mut filtered_nfts = Vec::new();
        let mut total_count = 0;
        
        for summary in all_nfts.nfts {
            // Get full NFT data
            if let Ok(Some(nft)) = api.get_nft(&at, summary.token_id) {
                if let common::nft::types::ProductMetadata::Kombucha(kombucha_data) = nft.product_data {
                    if kombucha_data.scoby_health >= min_health {
                        filtered_nfts.push(summary);
                        total_count += 1;
                    }
                }
            }
        }
        
        Ok(NFTQueryResult {
            nfts: filtered_nfts,
            total_count,
        })
    }
    
    fn query_by_tea_base(&self, tea_base: TeaBase, at: Option<<Block as BlockT>::Hash>) -> RpcResult<NFTQueryResult> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // Get all kombucha NFTs
        let params = NFTQueryParams {
            owner: None,
            product_type: Some(ProductType::Kombucha),
            min_quality: None,
            certified_organic_only: false,
        };
        
        let all_nfts = api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())?;
        
        // Filter by tea base
        let mut filtered_nfts = Vec::new();
        let mut total_count = 0;
        
        for summary in all_nfts.nfts {
            // Get full NFT data
            if let Ok(Some(nft)) = api.get_nft(&at, summary.token_id) {
                if let common::nft::types::ProductMetadata::Kombucha(kombucha_data) = nft.product_data {
                    if matches!(
                        (&kombucha_data.tea_base, &tea_base),
                        (TeaBase::Black, TeaBase::Black)
                        | (TeaBase::Green, TeaBase::Green)
                        | (TeaBase::White, TeaBase::White)
                        | (TeaBase::Oolong, TeaBase::Oolong)
                        | (TeaBase::Herbal, TeaBase::Herbal)
                    ) {
                        filtered_nfts.push(summary);
                        total_count += 1;
                    } else if let (TeaBase::Custom(a), TeaBase::Custom(b)) = (&kombucha_data.tea_base, &tea_base) {
                        if a == b {
                            filtered_nfts.push(summary);
                            total_count += 1;
                        }
                    }
                }
            }
        }
        
        Ok(NFTQueryResult {
            nfts: filtered_nfts,
            total_count,
        })
    }
    
    fn get_total_kombucha_count(&self, at: Option<<Block as BlockT>::Hash>) -> RpcResult<u64> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // Create query params focusing on kombucha
        let params = NFTQueryParams {
            owner: None,
            product_type: Some(ProductType::Kombucha),
            min_quality: None,
            certified_organic_only: false,
        };
        
        let result = api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())?;
        
        Ok(result.total_count)
    }
    
    fn get_highest_quality_kombucha(&self, limit: u32, at: Option<<Block as BlockT>::Hash>) -> RpcResult<Vec<NFTSummary>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));
        
        // Get all kombucha NFTs
        let params = NFTQueryParams {
            owner: None,
            product_type: Some(ProductType::Kombucha),
            min_quality: None,
            certified_organic_only: false,
        };
        
        let mut all_nfts = api.query_nfts(&at, params)
            .map_err(|e| Error::Runtime(format!("Error querying NFTs: {:?}", e)).into())?
            .nfts;
        
        // Sort by quality score (descending)
        all_nfts.sort_by(|a, b| b.quality_score.cmp(&a.quality_score));
        
        // Take requested limit
        let limit = std::cmp::min(limit as usize, all_nfts.len());
        
        Ok(all_nfts.into_iter().take(limit).collect())
    }
}
