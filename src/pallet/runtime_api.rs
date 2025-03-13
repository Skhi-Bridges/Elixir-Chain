//! Runtime API definition for elixir pallet.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

/// The Elixir runtime API used to access kombucha batch verification data.
sp_api::decl_runtime_apis! {
    pub trait ElixirRuntimeApi<AccountId, Balance> where
        AccountId: Codec,
        Balance: Codec,
    {
        /// Get details of a specific kombucha batch by ID
        fn get_batch_details(batch_id: Vec<u8>) -> Option<BatchDetails<AccountId, Balance>>;
        
        /// Check the verification status of a kombucha batch
        fn batch_verification_status(batch_id: Vec<u8>) -> Option<VerificationStatus>;
        
        /// Get all batch IDs produced by a specific brewer
        fn get_brewer_batches(brewer: AccountId) -> Vec<Vec<u8>>;
        
        /// Check the fermentation status of a batch
        fn check_fermentation_status(batch_id: Vec<u8>) -> Option<FermentationStatus>;
    }
}

/// Details of a kombucha production batch
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct BatchDetails<AccountId, Balance> {
    /// Identifier of the batch
    pub id: Vec<u8>,
    /// Account ID of the brewer
    pub brewer: AccountId,
    /// Quantity produced in liters
    pub quantity: u64,
    /// Production timestamp
    pub timestamp: u64,
    /// Current verification status
    pub status: VerificationStatus,
    /// Fermentation metrics of the batch
    pub metrics: FermentationMetrics,
    /// Production facility ID
    pub facility_id: Vec<u8>,
    /// Current reward value
    pub reward_value: Balance,
    /// Verification result data from oracles
    pub verification_data: Vec<VerificationData>,
    /// Recipe ID used for this batch
    pub recipe_id: Vec<u8>,
}

/// Status of a kombucha batch verification
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub enum VerificationStatus {
    /// Batch is pending verification
    Pending,
    /// Batch is currently being verified
    InProgress,
    /// Batch has been verified as valid
    Verified,
    /// Batch verification failed
    Failed,
    /// Batch verification has been disputed
    Disputed,
}

/// Status of fermentation process
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub enum FermentationStatus {
    /// Fermentation has not started
    NotStarted,
    /// Fermentation is in progress
    InProgress,
    /// Primary fermentation is complete
    PrimaryComplete,
    /// Secondary fermentation is complete
    SecondaryComplete,
    /// Fermentation failed or was contaminated
    Failed,
}

/// Fermentation metrics for a kombucha batch
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct FermentationMetrics {
    /// pH level (scaled by 10, e.g. 3.5 = 35)
    pub ph_level: u8,
    /// Sugar content in percentage
    pub sugar_content: u8,
    /// Alcohol content in percentage (scaled by 10)
    pub alcohol_content: u8,
    /// Acidity in percentage (scaled by 10)
    pub acidity: u8,
    /// Probiotic count (log scale)
    pub probiotic_count: u8,
    /// Overall quality score (0-100)
    pub quality_score: u8,
    /// Fermentation time in hours
    pub fermentation_time: u32,
}

/// Verification data from an oracle
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub struct VerificationData {
    /// Oracle account ID
    pub oracle: Vec<u8>,
    /// Verification timestamp
    pub timestamp: u64,
    /// Verification result (0-100)
    pub score: u8,
    /// Optional comments
    pub comments: Vec<u8>,
    /// Signature using CRYSTALS-Dilithium
    pub signature: Vec<u8>,
}
