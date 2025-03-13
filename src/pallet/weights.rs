//! Weights for the Elixir pallet

use frame_support::weights::{
    constants::{RocksDbWeight, WEIGHT_PER_SECOND},
    Weight,
};

/// Weight functions for elixir_pallet.
pub trait WeightInfo {
    fn register_brewer() -> Weight;
    fn register_facility() -> Weight;
    fn register_verifier() -> Weight;
    fn register_batch() -> Weight;
    fn verify_batch() -> Weight;
    fn dispute_verification() -> Weight;
    fn update_facility() -> Weight;
    fn deactivate_facility() -> Weight;
    fn update_verifier() -> Weight;
    fn deactivate_verifier() -> Weight;
    fn register_recipe() -> Weight;
    fn update_recipe() -> Weight;
    fn claim_rewards() -> Weight;
    fn update_fermentation_status() -> Weight;
}

/// Weights for elixir_pallet using the Substrate node and recommended hardware.
impl WeightInfo for () {
    fn register_brewer() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn register_facility() -> Weight {
        WEIGHT_PER_SECOND / 10
    }

    fn register_verifier() -> Weight {
        WEIGHT_PER_SECOND / 10
    }

    fn register_batch() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn verify_batch() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn dispute_verification() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn update_facility() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn deactivate_facility() -> Weight {
        WEIGHT_PER_SECOND / 50
    }

    fn update_verifier() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn deactivate_verifier() -> Weight {
        WEIGHT_PER_SECOND / 50
    }

    fn register_recipe() -> Weight {
        WEIGHT_PER_SECOND / 15
    }
    
    fn update_recipe() -> Weight {
        WEIGHT_PER_SECOND / 20
    }

    fn claim_rewards() -> Weight {
        WEIGHT_PER_SECOND / 20
    }
    
    fn update_fermentation_status() -> Weight {
        WEIGHT_PER_SECOND / 20
    }
}
