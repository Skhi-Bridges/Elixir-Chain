//! Type definitions for the Elixir pallet.

use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use sp_std::prelude::*;

/// Status of a kombucha batch verification
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, Default)]
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
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VerificationData<AccountId> {
    /// Oracle account ID
    pub oracle: AccountId,
    /// Verification timestamp
    pub timestamp: u64,
    /// Verification result (0-100)
    pub score: u8,
    /// Optional comments
    pub comments: Vec<u8>,
    /// Signature using CRYSTALS-Dilithium
    pub signature: Vec<u8>,
}

/// Details of a kombucha production batch
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
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
    pub verification_data: Vec<VerificationData<AccountId>>,
    /// Recipe ID used for this batch
    pub recipe_id: Vec<u8>,
}

/// Registration information for a production facility
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct FacilityInfo<AccountId> {
    /// Facility ID
    pub id: Vec<u8>,
    /// Facility name
    pub name: Vec<u8>,
    /// Geographic location (latitude, longitude)
    pub location: (i32, i32),
    /// Owner account
    pub owner: AccountId,
    /// Maximum production capacity in liters per month
    pub capacity: u64,
    /// Registration timestamp
    pub registered_at: u64,
    /// ISO certification IDs
    pub certifications: Vec<Vec<u8>>,
    /// Whether the facility is currently active
    pub active: bool,
}

/// Registration information for a verifier
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct VerifierInfo<AccountId, Balance> {
    /// Verifier account
    pub account: AccountId,
    /// Verifier name
    pub name: Vec<u8>,
    /// Verification records count
    pub verification_count: u64,
    /// Success rate (0-100)
    pub success_rate: u8,
    /// Staked balance
    pub stake: Balance,
    /// Whether the verifier is currently active
    pub active: bool,
}

/// Recipe for kombucha production
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct KombuchaRecipe {
    /// Recipe ID
    pub id: Vec<u8>,
    /// Recipe name
    pub name: Vec<u8>,
    /// Creator account ID
    pub creator: Vec<u8>,
    /// Tea varieties and ratios
    pub tea_blend: Vec<(Vec<u8>, u8)>,
    /// Sugar amount in grams per liter
    pub sugar_amount: u16,
    /// Fermentation temperature range in Celsius (min, max)
    pub temperature_range: (u8, u8),
    /// Primary fermentation time in days
    pub primary_fermentation_time: u8,
    /// Secondary fermentation time in days
    pub secondary_fermentation_time: u8,
    /// Additional ingredients
    pub additional_ingredients: Vec<Vec<u8>>,
    /// Expected final pH range (min, max)
    pub ph_range: (u8, u8),
    /// Public recipe (true if shared with community)
    pub is_public: bool,
}
