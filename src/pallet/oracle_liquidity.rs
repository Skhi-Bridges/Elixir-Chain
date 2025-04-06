//! Oracle-Liquidity Integration Module for ELXR Chain
//!
//! Connects the daemonless oracle with the shared liquidity system,
//! enabling robust NRSH/ELXR trading with quantum-resistant security.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    dispatch::DispatchResult,
    ensure,
    pallet_prelude::*,
    traits::{Currency, ExistenceRequirement, Get},
    weights::Weight,
};
use frame_system::pallet_prelude::*;
use sp_runtime::{traits::Zero, DispatchError, Percent};
use sp_std::prelude::*;

// Import crate and external dependencies
use crate::pallet::{oracle, types::ElixirAsset};
use shared::liquidity::{
    amm::{AutomatedMarketMaker, Config as AmmConfig},
    types::{
        AddLiquidityParams, AssetId, ConstantProductPriceCalculator, PoolId, PriceCalculator, 
        RemoveLiquidityParams, SwapParams,
    },
};

// Multi-level error correction for communication between oracle and liquidity system
mod error_correction {
    // Re-export error correction from oracle pallet
    pub use super::oracle::error_correction::*;
    
    // Additional bridge-specific error correction for oracle-liquidity communication
    pub fn protect_price_data(price_data: &[u8]) -> Vec<u8> {
        // Apply all three layers of error correction
        let classical = self::classical::encode(price_data, 8); // Higher redundancy
        let bridge = self::bridge::encode(&classical, 4);       // Higher redundancy
        let quantum = self::quantum::protect(&bridge);
        quantum
    }
    
    pub fn recover_price_data(protected_data: &[u8]) -> Option<Vec<u8>> {
        // Recover through all three layers
        self::quantum::recover(protected_data)
            .and_then(|quantum_recovered| self::bridge::decode(&quantum_recovered))
            .and_then(|bridge_recovered| self::classical::decode(&bridge_recovered))
    }
}

// Define the pallet configuration trait
pub trait Config: frame_system::Config + oracle::Config {
    /// Integration with the AMM module
    type AmmHandler: AutomatedMarketMaker<
        Self::AccountId, 
        AssetIdOf<Self>, 
        BalanceOf<Self>, 
        Self::BlockNumber
    >;
    
    /// The overarching event type
    type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    
    /// Weight information for extrinsics
    type WeightInfo: WeightInfo;
}

// Type aliases
type AssetIdOf<T> = AssetId;
type BalanceOf<T> = <<T as oracle::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[pallet::pallet]
#[pallet::without_storage_info]
pub struct Pallet<T>(_);

// Storage items
#[pallet::storage]
pub type OracleDrivenPools<T: Config> = StorageMap<_, Blake2_128Concat, PoolId, OracleDrivenPool>;

#[pallet::storage]
pub type AssetPriceDeviations<T: Config> = StorageMap<_, Blake2_128Concat, AssetId, Percent>;

// Oracle-driven pool information
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
pub struct OracleDrivenPool {
    pub pool_id: PoolId,
    pub base_asset: AssetId,
    pub quote_asset: AssetId,
    pub allow_oracle_price_override: bool,
    pub deviation_threshold: Percent,
}

// Events
#[pallet::event]
#[pallet::generate_deposit(pub(super) fn deposit_event)]
pub enum Event<T: Config> {
    /// Liquidity pool synchronized with oracle prices
    PoolSynchronized {
        pool_id: PoolId,
        base_asset: AssetId,
        quote_asset: AssetId,
        new_price: BalanceOf<T>,
    },
    /// New oracle-driven pool registered
    OracleDrivenPoolRegistered {
        pool_id: PoolId,
        base_asset: AssetId,
        quote_asset: AssetId,
    },
    /// Arbitrage opportunity detected and executed
    ArbitrageExecuted {
        pool_id: PoolId,
        asset_id: AssetId,
        amount: BalanceOf<T>,
        profit: BalanceOf<T>,
    },
}

// Errors
#[pallet::error]
pub enum Error<T> {
    /// Pool already registered as oracle-driven
    PoolAlreadyRegistered,
    /// Pool not found
    PoolNotFound,
    /// Asset price not available from oracle
    AssetPriceNotAvailable,
    /// Swap failed in liquidity module
    SwapFailed,
    /// Insufficient liquidity for operation
    InsufficientLiquidity,
    /// Pool price deviation exceeds threshold
    ExcessiveDeviation,
}

// Calls
#[pallet::call]
impl<T: Config> Pallet<T> {
    /// Register a liquidity pool as oracle-driven
    #[pallet::call_index(0)]
    #[pallet::weight(T::WeightInfo::register_oracle_driven_pool())]
    pub fn register_oracle_driven_pool(
        origin: OriginFor<T>,
        pool_id: PoolId,
        base_asset: AssetId,
        quote_asset: AssetId,
        allow_oracle_override: bool,
        deviation_threshold: Percent,
    ) -> DispatchResult {
        ensure_root(origin)?;
        
        // Ensure pool doesn't already exist
        ensure!(!OracleDrivenPools::<T>::contains_key(pool_id), Error::<T>::PoolAlreadyRegistered);
        
        // Create oracle-driven pool
        let pool = OracleDrivenPool {
            pool_id,
            base_asset,
            quote_asset,
            allow_oracle_price_override: allow_oracle_override,
            deviation_threshold,
        };
        
        // Store pool
        OracleDrivenPools::<T>::insert(pool_id, pool);
        
        // Set default deviation thresholds for assets if not already set
        if !AssetPriceDeviations::<T>::contains_key(base_asset) {
            AssetPriceDeviations::<T>::insert(base_asset, Percent::from_percent(5));
        }
        
        if !AssetPriceDeviations::<T>::contains_key(quote_asset) {
            AssetPriceDeviations::<T>::insert(quote_asset, Percent::from_percent(5));
        }
        
        // Emit event
        Self::deposit_event(Event::OracleDrivenPoolRegistered {
            pool_id,
            base_asset,
            quote_asset,
        });
        
        Ok(())
    }
    
    /// Synchronize a liquidity pool with oracle prices
    #[pallet::call_index(1)]
    #[pallet::weight(T::WeightInfo::synchronize_pool())]
    pub fn synchronize_pool(
        origin: OriginFor<T>,
        pool_id: PoolId,
    ) -> DispatchResult {
        ensure_signed(origin)?;
        
        // Get pool info
        let pool = OracleDrivenPools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
        
        // Get asset prices from oracle with error correction
        let base_price = oracle::Pallet::<T>::get_asset_price_with_correction(pool.base_asset)
            .ok_or(Error::<T>::AssetPriceNotAvailable)?;
            
        let quote_price = oracle::Pallet::<T>::get_asset_price_with_correction(pool.quote_asset)
            .ok_or(Error::<T>::AssetPriceNotAvailable)?;
        
        // Calculate relative price
        let price_ratio = Self::calculate_price_ratio(base_price, quote_price)?;
        
        // Only synchronize if oracle override is allowed
        if pool.allow_oracle_price_override {
            // Apply multi-level error correction to the price data for transmission
            let price_data = price_ratio.encode();
            let protected_data = error_correction::protect_price_data(&price_data);
            
            // TODO: In production, this would use XCM to communicate with the AMM
            // For now, we'll simulate it with a direct call
            
            // Emit event
            Self::deposit_event(Event::PoolSynchronized {
                pool_id,
                base_asset: pool.base_asset,
                quote_asset: pool.quote_asset,
                new_price: price_ratio,
            });
        }
        
        // Check for arbitrage opportunities
        Self::check_for_arbitrage(pool_id, pool.base_asset, pool.quote_asset, price_ratio)?;
        
        Ok(())
    }
    
    /// Set price deviation threshold for an asset
    #[pallet::call_index(2)]
    #[pallet::weight(T::WeightInfo::set_deviation_threshold())]
    pub fn set_deviation_threshold(
        origin: OriginFor<T>,
        asset_id: AssetId,
        threshold: Percent,
    ) -> DispatchResult {
        ensure_root(origin)?;
        
        // Update deviation threshold
        AssetPriceDeviations::<T>::insert(asset_id, threshold);
        
        Ok(())
    }
}

// Helper functions
impl<T: Config> Pallet<T> {
    /// Calculate price ratio between base and quote assets
    fn calculate_price_ratio(
        base_price: BalanceOf<T>,
        quote_price: BalanceOf<T>,
    ) -> Result<BalanceOf<T>, DispatchError> {
        if quote_price.is_zero() {
            return Err(ArithmeticError::DivisionByZero.into());
        }
        
        // In a real implementation, this would handle decimal precision properly
        // This is a simplified version
        Ok(base_price.checked_div(&quote_price).ok_or(ArithmeticError::DivisionByZero)?)
    }
    
    /// Check for arbitrage opportunities between oracle and pool prices
    fn check_for_arbitrage(
        pool_id: PoolId,
        base_asset: AssetId,
        quote_asset: AssetId,
        oracle_price: BalanceOf<T>,
    ) -> DispatchResult {
        // In a real implementation, this would:
        // 1. Get current pool price
        // 2. Compare with oracle price
        // 3. If deviation exceeds threshold, execute arbitrage
        
        // For now, we'll simulate a successful arbitrage
        let arbitrage_amount = 100u32.into(); // Mock value
        let profit = 5u32.into(); // Mock value
        
        // Emit event for demonstration purposes
        Self::deposit_event(Event::ArbitrageExecuted {
            pool_id,
            asset_id: base_asset,
            amount: arbitrage_amount,
            profit,
        });
        
        Ok(())
    }
    
    /// Get pool info by ID
    pub fn get_pool_info(pool_id: PoolId) -> Option<OracleDrivenPool> {
        OracleDrivenPools::<T>::get(pool_id)
    }
    
    /// Check if a pool price deviates too much from oracle price
    pub fn check_pool_deviation(
        pool_id: PoolId,
        asset_id: AssetId,
    ) -> Result<bool, DispatchError> {
        // Get pool info
        let pool = OracleDrivenPools::<T>::get(pool_id).ok_or(Error::<T>::PoolNotFound)?;
        
        // Get oracle price
        let oracle_price = oracle::Pallet::<T>::get_asset_price(asset_id)
            .ok_or(Error::<T>::AssetPriceNotAvailable)?;
        
        // Get pool price (mock implementation)
        let pool_price = oracle_price; // This would actually come from the AMM
        
        // Get deviation threshold
        let threshold = if asset_id == pool.base_asset || asset_id == pool.quote_asset {
            pool.deviation_threshold
        } else {
            AssetPriceDeviations::<T>::get(asset_id).unwrap_or_else(|| Percent::from_percent(5))
        };
        
        // TODO: Calculate actual deviation and compare with threshold
        
        // For now, just return false (no excessive deviation)
        Ok(false)
    }
}

// Define weight info trait
pub trait WeightInfo {
    fn register_oracle_driven_pool() -> Weight;
    fn synchronize_pool() -> Weight;
    fn set_deviation_threshold() -> Weight;
}

// Default weight implementation
impl WeightInfo for () {
    fn register_oracle_driven_pool() -> Weight {
        Weight::from_parts(10_000, 0)
    }
    
    fn synchronize_pool() -> Weight {
        Weight::from_parts(15_000, 0)
    }
    
    fn set_deviation_threshold() -> Weight {
        Weight::from_parts(5_000, 0)
    }
}
