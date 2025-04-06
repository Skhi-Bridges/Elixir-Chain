//! ELXR Runtime API definitions
//!
//! This module defines the Runtime API interfaces for the ELXR chain,
//! including support for the SBX-Q999 Quantum NFT system with kombucha-specific features.

use sp_api::{decl_runtime_apis, impl_runtime_apis};
use sp_runtime::traits::Block as BlockT;
use sp_std::prelude::*;

use common::nft::types::{
    TokenId, NFTMetadata, NFTQueryParams, NFTQueryResult, KombuchaMetadata,
};

/// Runtime API for ELXR quantum NFT operations
decl_runtime_apis! {
    pub trait ELXRRuntimeApi<Block> where
        Block: BlockT,
    {
        /// Get NFT by token ID
        fn get_nft(token_id: TokenId) -> Option<NFTMetadata>;
        
        /// Query NFTs by parameters
        fn query_nfts(params: NFTQueryParams) -> NFTQueryResult;
        
        /// Get kombucha-specific metadata directly
        fn get_kombucha_metadata(token_id: TokenId) -> Option<KombuchaMetadata>;
    }
}
