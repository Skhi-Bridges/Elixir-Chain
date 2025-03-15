//! Type definitions for the ELXR pallet

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::pallet_prelude::*;
use scale_info::TypeInfo;
use sp_std::prelude::*;

/// Fermentation stage for tracking the process
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum FermentationStage {
    /// Initial stage when fermentation batch is created
    Started,
    /// Primary fermentation stage
    Primary,
    /// Secondary fermentation stage
    Secondary,
    /// Aging stage
    Aging,
    /// Completed fermentation process
    Completed,
}

/// Verification result for a batch
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum VerificationResult {
    /// Verification passed
    Passed,
    /// Verification failed
    Failed,
    /// Verification pending additional data
    Pending,
}

/// Quantum cryptographic algorithm
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum QuantumAlgorithm {
    /// CRYSTALS-Dilithium signature algorithm
    Dilithium,
    /// CRYSTALS-Kyber key encapsulation mechanism
    Kyber,
    /// Falcon signature algorithm
    Falcon,
}

/// Error correction strategy
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub enum ErrorCorrectionStrategy {
    /// Classical error correction using Reed-Solomon
    Classical,
    /// Bridge error correction for classical-quantum interface
    Bridge,
    /// Quantum error correction using Surface codes
    Quantum,
    /// Comprehensive error correction at all levels
    Comprehensive,
}

/// Product information stored on-chain
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ProductInfo<T: Config> {
    /// Product identifier
    pub id: T::ProductId,
    /// Product owner
    pub owner: T::AccountId,
    /// Product name
    pub name: Vec<u8>,
    /// Product details
    pub details: Vec<u8>,
    /// Block when product was created
    pub created_at: T::BlockNumber,
}

/// Batch information stored on-chain
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct BatchInfo<T: Config> {
    /// Batch identifier
    pub id: T::BatchId,
    /// Associated product ID
    pub product_id: T::ProductId,
    /// Batch creator
    pub creator: T::AccountId,
    /// Batch data
    pub data: Vec<u8>,
    /// Current fermentation stage
    pub stage: FermentationStage,
    /// Block when batch was created
    pub created_at: T::BlockNumber,
    /// Block when batch was last updated
    pub updated_at: T::BlockNumber,
}

/// Verification information for a batch
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct VerificationInfo<T: Config> {
    /// Batch identifier
    pub batch_id: T::BatchId,
    /// Verifier account
    pub verifier: T::AccountId,
    /// Verification result
    pub result: VerificationResult,
    /// Verification data
    pub data: Vec<u8>,
    /// Error correction strategy used
    pub error_correction: ErrorCorrectionStrategy,
    /// Block when verification was performed
    pub verified_at: T::BlockNumber,
}

/// Quantum key information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct QuantumKeyInfo {
    /// Quantum algorithm used
    pub algorithm: QuantumAlgorithm,
    /// Public key data
    pub public_key: Vec<u8>,
    /// Block when key was created
    pub created_at: T::BlockNumber,
}

/// Trait for handling ELXR pallet configuration
pub trait Config: frame_system::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    
    /// The type for recording product identifiers
    type ProductId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + Debug;
    
    /// The type for fermentation batch identifiers
    type BatchId: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + Debug;
    
    /// Weight information for the extrinsics in this pallet.
    type WeightInfo: WeightInfo;
    
    /// Random number generator for cryptographic operations
    type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
}
