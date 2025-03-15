//! Weights for the ELXR pallet

use frame_support::weights::{
    constants::RocksDbWeight as DbWeight, constants::WEIGHT_PER_SECOND, Weight,
};

/// Weight functions for ELXR pallet operations
pub trait WeightInfo {
    fn register_product() -> Weight;
    fn create_batch() -> Weight;
    fn update_fermentation_stage() -> Weight;
    fn verify_batch() -> Weight;
    fn generate_quantum_key() -> Weight;
}

/// Weights for ELXR pallet using the Substrate node and recommended hardware.
impl WeightInfo for () {
    fn register_product() -> Weight {
        // Reading and writing to storage with relatively simple data
        DbWeight::get().reads(1) // Reading next_product_id
            .saturating_add(DbWeight::get().writes(2)) // Writing product and next_product_id
            .saturating_add(25_000_000) // Base computational complexity
    }

    fn create_batch() -> Weight {
        // Multiple reads and writes with more complex operations
        DbWeight::get().reads(2) // Reading product and next_batch_id
            .saturating_add(DbWeight::get().writes(2)) // Writing batch and next_batch_id
            .saturating_add(35_000_000) // Base computational complexity
    }

    fn update_fermentation_stage() -> Weight {
        // Reading and validating batch data, then updating it
        DbWeight::get().reads(1) // Reading batch
            .saturating_add(DbWeight::get().writes(1)) // Writing updated batch
            .saturating_add(40_000_000) // Base computational complexity including validation
    }

    fn verify_batch() -> Weight {
        // Complex operation with multiple reads and writes
        DbWeight::get().reads(2) // Reading batch and existing verification
            .saturating_add(DbWeight::get().writes(1)) // Writing verification
            .saturating_add(60_000_000) // Higher computational complexity due to verification processing
    }

    fn generate_quantum_key() -> Weight {
        // Cryptographic operation with high computational requirements
        DbWeight::get().reads(1) // Reading account info
            .saturating_add(DbWeight::get().writes(1)) // Writing key data
            .saturating_add(150_000_000) // High computational complexity for quantum crypto
    }
}
