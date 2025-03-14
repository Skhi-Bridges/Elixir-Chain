//! Elixir Chain pallet for Kombucha production and verification.

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement, Get, ReservableCurrency},
};
use frame_system::{ensure_signed, pallet_prelude::*};
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

mod types;
pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

pub mod weights;
pub use weights::*;

pub mod runtime_api;
pub use runtime_api::*;

type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

/// Configuration trait for Elixir pallet.
pub trait Config: frame_system::Config {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

    /// The currency mechanism, used for paying for deposits and rewards.
    type Currency: ReservableCurrency<Self::AccountId>;

    /// The minimum amount that should be reserved for a brewer to register.
    type MinBrewerStake: Get<BalanceOf<Self>>;

    /// The period (in blocks) during which a batch should be validated.
    type FermentationPeriod: Get<Self::BlockNumber>;

    /// The maximum number of verifiers per batch.
    type MaxVerifiers: Get<u32>;

    /// Weight information for extrinsics in this pallet.
    type WeightInfo: WeightInfo;
}

// The main implementation will go here
