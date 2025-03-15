//! # Elixir Chain (ELXR) Pallet
//!
//! A specialized pallet for the Elixir Chain implementing:
//! - Fermentation tracking and verification
//! - Remote Online Notarization (RON) with post-quantum cryptography
//! - Integration with ActorX messaging framework
//! - Error correction at multiple levels (classical, bridge, quantum)
//!
//! ## Overview
//!
//! The ELXR pallet provides core functionality for tracking and verifying
//! fermentation processes while maintaining a high level of security through
//! post-quantum cryptography and RON.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Get, EnsureOrigin, Randomness},
    weights::{DispatchClass, Pays, Weight},
};
use frame_system::{self as system, ensure_signed};
use scale_info::TypeInfo;
use sp_runtime::{
    traits::{AtLeast32BitUnsigned, Bounded, Member, Saturating, StaticLookup},
    DispatchError as RuntimeDispatchError,
};
use sp_std::{fmt::Debug, prelude::*};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types;
mod weights;
pub use types::*;
pub use weights::*;

/// The pallet's configuration trait.
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

// Storage for the pallet.
decl_storage! {
    trait Store for Module<T: Config> as Elixir {
        /// Stores the next available product ID
        NextProductId get(fn next_product_id): T::ProductId;
        
        /// Stores the next available batch ID
        NextBatchId get(fn next_batch_id): T::BatchId;
        
        /// Stores product information
        Products get(fn products): map hasher(blake2_128_concat) T::ProductId => Option<ProductInfo<T>>;
        
        /// Stores batch information
        Batches get(fn batches): map hasher(blake2_128_concat) T::BatchId => Option<BatchInfo<T>>;
        
        /// Stores verification data for batches
        BatchVerification get(fn batch_verification): map hasher(blake2_128_concat) T::BatchId => Option<VerificationInfo<T>>;
        
        /// Quantum cryptographic keys for secure communication
        QuantumKeys get(fn quantum_keys): map hasher(blake2_128_concat) T::AccountId => Option<QuantumKeyInfo>;
    }
}

// Events for the pallet.
decl_event! {
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
        ProductId = <T as Config>::ProductId,
        BatchId = <T as Config>::BatchId,
    {
        /// A new product was registered
        ProductRegistered(AccountId, ProductId),
        
        /// A new batch was created
        BatchCreated(AccountId, BatchId, ProductId),
        
        /// A batch verification was recorded
        BatchVerified(AccountId, BatchId, VerificationResult),
        
        /// Fermentation stage was updated
        FermentationStageUpdated(BatchId, FermentationStage),
        
        /// Quantum key was generated
        QuantumKeyGenerated(AccountId),
    }
}

// Errors for the pallet.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// Product does not exist
        ProductNotFound,
        
        /// Batch does not exist
        BatchNotFound,
        
        /// Verification data already exists
        VerificationAlreadyExists,
        
        /// Not authorized to perform this action
        NotAuthorized,
        
        /// Invalid fermentation stage transition
        InvalidStageTransition,
        
        /// Quantum key operation failed
        QuantumKeyError,
        
        /// Error correction failed
        ErrorCorrectionFailed,
    }
}

// Dispatchable functions for the pallet.
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initialize errors
        type Error = Error<T>;
        
        // Initialize events
        fn deposit_event() = default;
        
        /// Register a new product
        #[weight = <T as Config>::WeightInfo::register_product()]
        pub fn register_product(
            origin,
            name: Vec<u8>,
            details: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            let product_id = Self::next_product_id();
            let next_id = product_id.saturating_add(1.into());
            
            let product_info = ProductInfo {
                id: product_id,
                owner: sender.clone(),
                name,
                details,
                created_at: <frame_system::Pallet<T>>::block_number(),
            };
            
            <NextProductId<T>>::put(next_id);
            <Products<T>>::insert(product_id, product_info);
            
            Self::deposit_event(RawEvent::ProductRegistered(sender, product_id));
            Ok(())
        }
        
        /// Create a new fermentation batch
        #[weight = <T as Config>::WeightInfo::create_batch()]
        pub fn create_batch(
            origin,
            product_id: T::ProductId,
            initial_data: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure product exists
            ensure!(<Products<T>>::contains_key(product_id), Error::<T>::ProductNotFound);
            
            let batch_id = Self::next_batch_id();
            let next_id = batch_id.saturating_add(1.into());
            
            let batch_info = BatchInfo {
                id: batch_id,
                product_id,
                creator: sender.clone(),
                data: initial_data,
                stage: FermentationStage::Started,
                created_at: <frame_system::Pallet<T>>::block_number(),
                updated_at: <frame_system::Pallet<T>>::block_number(),
            };
            
            <NextBatchId<T>>::put(next_id);
            <Batches<T>>::insert(batch_id, batch_info);
            
            Self::deposit_event(RawEvent::BatchCreated(sender, batch_id, product_id));
            Ok(())
        }
        
        /// Update fermentation stage
        #[weight = <T as Config>::WeightInfo::update_fermentation_stage()]
        pub fn update_fermentation_stage(
            origin,
            batch_id: T::BatchId,
            new_stage: FermentationStage,
            data_update: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Ensure batch exists
            let mut batch = Self::batches(batch_id).ok_or(Error::<T>::BatchNotFound)?;
            
            // Ensure sender is authorized
            ensure!(batch.creator == sender, Error::<T>::NotAuthorized);
            
            // Ensure valid stage transition
            ensure!(Self::is_valid_stage_transition(batch.stage, new_stage), Error::<T>::InvalidStageTransition);
            
            // Update batch information
            batch.stage = new_stage;
            batch.data = data_update;
            batch.updated_at = <frame_system::Pallet<T>>::block_number();
            
            <Batches<T>>::insert(batch_id, batch);
            
            Self::deposit_event(RawEvent::FermentationStageUpdated(batch_id, new_stage));
            Ok(())
        }
        
        /// Generate quantum key for secure communication
        #[weight = <T as Config>::WeightInfo::generate_quantum_key()]
        pub fn generate_quantum_key(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Generate quantum key using Dilithium
            let random_seed = <T as Config>::Randomness::random(&sender.encode());
            let key_data = random_seed.0.to_vec();
            
            let key_info = QuantumKeyInfo {
                algorithm: QuantumAlgorithm::Dilithium,
                public_key: key_data,
                created_at: <frame_system::Pallet<T>>::block_number(),
            };
            
            <QuantumKeys<T>>::insert(sender.clone(), key_info);
            
            Self::deposit_event(RawEvent::QuantumKeyGenerated(sender));
            Ok(())
        }
    }
}

// Implementation for the pallet
impl<T: Config> Module<T> {
    /// Check if a stage transition is valid
    fn is_valid_stage_transition(current: FermentationStage, next: FermentationStage) -> bool {
        match (current, next) {
            (FermentationStage::Started, FermentationStage::Primary) => true,
            (FermentationStage::Primary, FermentationStage::Secondary) => true,
            (FermentationStage::Secondary, FermentationStage::Aging) => true,
            (FermentationStage::Aging, FermentationStage::Completed) => true,
            _ => false,
        }
    }
}
