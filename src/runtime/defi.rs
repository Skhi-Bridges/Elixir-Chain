//! ELXR DeFi Runtime Configuration
//!
//! This module configures the DeFi pallet for the ELXR chain runtime,
//! setting up the necessary parameters and types for kombucha-based loans.

use crate::pallet::defi;
use frame_support::{
    parameter_types,
    weights::Weight,
};
use sp_runtime::Perbill;
use sp_core::H256;
use sp_std::marker::PhantomData;

use crate::runtime::{Runtime, Balances, DOLLARS, System};

// Minimum collateral ratio (125%)
parameter_types! {
    pub const MinCollateralRatio: u64 = 12500;
}

// Maximum loan-to-value ratio (80%)
parameter_types! {
    pub const MaxLoanToValue: u64 = 8000;
}

// Liquidation threshold (110%)
parameter_types! {
    pub const LiquidationThreshold: u64 = 11000;
}

// Treasury account for fee collection
parameter_types! {
    pub const TreasuryAccount: H256 = H256::repeat_byte(0xF1);
}

/// Configure the ELXR DeFi pallet
impl defi::Config for Runtime {
    type Event = crate::runtime::Event;
    type Currency = Balances;
    type MinCollateralRatio = MinCollateralRatio;
    type MaxLoanToValue = MaxLoanToValue;
    type LiquidationThreshold = LiquidationThreshold;
    type TreasuryAccount = TreasuryAccount;
}

/// Add this line to your Runtime implementation
/// pub type ElxrDefi = defi::Module<Runtime>;
