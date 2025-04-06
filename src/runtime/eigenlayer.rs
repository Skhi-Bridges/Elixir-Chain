//! ELXR Eigenlayer Runtime Configuration
//!
//! This module configures the Eigenlayer pallet for the ELXR chain runtime,
//! setting up the necessary parameters and types for kombucha-based staking
//! with quantum-resistant security.

use crate::pallet::eigenlayer;
use frame_support::{
    parameter_types,
    weights::Weight,
};
use sp_runtime::Perbill;
use sp_core::H256;
use sp_std::marker::PhantomData;

use crate::runtime::{Runtime, Balances, DOLLARS, System};

// Blocks per year (assuming 6 second blocks)
parameter_types! {
    pub const BlocksPerYear: u64 = 5_256_000; // 6s blocks, 365 days
}

// Minimum staking amount (100 DOLLARS - lower than NRSH due to smaller vessel size)
parameter_types! {
    pub const MinStakingAmount: u128 = 100 * DOLLARS;
}

// Maximum staking amount (5_000_000 DOLLARS)
parameter_types! {
    pub const MaxStakingAmount: u128 = 5_000_000 * DOLLARS;
}

// Treasury account for fee collection
parameter_types! {
    pub const TreasuryAccount: H256 = H256::repeat_byte(0xE1);
}

/// Configure the ELXR Eigenlayer pallet
impl eigenlayer::Config for Runtime {
    type Event = crate::runtime::Event;
    type Currency = Balances;
    type BlocksPerYear = BlocksPerYear;
    type MinStakingAmount = MinStakingAmount;
    type MaxStakingAmount = MaxStakingAmount;
    type TreasuryAccount = TreasuryAccount;
}

/// Add this line to your Runtime implementation
/// pub type ElxrEigenlayer = eigenlayer::Module<Runtime>;
