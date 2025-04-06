//! Eigenlayer pallet for ELXR (Kombucha) chain
//! 
//! This pallet implements Eigenlayer integration for the ELXR chain,
//! allowing users to stake assets backed by kombucha production
//! and earn quantum-secure rewards based on quality metrics.

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchResult, DispatchError},
    ensure, traits::{Currency, ExistenceRequirement, WithdrawReasons, Get},
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::{
    traits::{Zero, StaticLookup, CheckedSub, CheckedAdd},
    DispatchError as SpRuntimeError,
};
use codec::{Decode, Encode};
use sp_std::prelude::*;
use serde::{Deserialize, Serialize};

use crate::telemetry::{self, api::TelemetrySummary, FermentationStage};
use common::pricing::{self, PricingCalculator, ProductType};
use common::eigenlayer::{EigenlayerCalculator, EigenlayerConfig, StakingPosition};

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// Configuration trait for the Eigenlayer pallet
pub trait Config: system::Config {
    /// The overarching event type
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    
    /// Currency mechanism
    type Currency: Currency<Self::AccountId>;
    
    /// Blocks per year, used for APR calculations
    type BlocksPerYear: Get<u64>;
    
    /// Minimum staking amount
    type MinStakingAmount: Get<BalanceOf<Self>>;
    
    /// Maximum staking amount
    type MaxStakingAmount: Get<BalanceOf<Self>>;
    
    /// Treasury account for fee collection
    type TreasuryAccount: Get<Self::AccountId>;
}

/// Staking status enum
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum StakingStatus {
    /// Staking position is active
    Active,
    /// Staking position is unstaking (in cooldown)
    Unstaking,
    /// Staking position is completed
    Completed,
    /// Staking position is slashed
    Slashed,
}

decl_storage! {
    trait Store for Module<T: Config> as ElxrEigenlayer {
        /// Next staking position ID to be assigned
        pub NextPositionId get(fn next_position_id): u64 = 1;
        
        /// Mapping from position ID to staking position
        pub StakingPositions get(fn staking_position): map hasher(twox_64_concat) 
            u64 => Option<StakingPosition<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
        
        /// Mapping from account to position IDs
        pub AccountPositions get(fn account_positions): map hasher(blake2_128_concat) 
            T::AccountId => Vec<u64>;
        
        /// Mapping from batch ID to position ID (each batch can only have one position)
        pub BatchPositions get(fn batch_position): map hasher(twox_64_concat) 
            Vec<u8> => Option<u64>;
        
        /// Eigenlayer configuration
        pub Config get(fn config): EigenlayerConfig = EigenlayerConfig::default();
        
        /// Total staked amount
        pub TotalStaked get(fn total_staked): BalanceOf<T> = Zero::zero();
        
        /// Protocol fee for staking (in basis points)
        pub ProtocolFee get(fn protocol_fee): u64 = 50; // 0.5%
        
        /// Operator fee for restaking (in basis points)
        pub OperatorFee get(fn operator_fee): u64 = 100; // 1%
        
        /// SCOBY health bonus multiplier (in basis points)
        pub ScobyHealthMultiplier get(fn scoby_health_multiplier): u64 = 50; // 0.5% additional reward per health level
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Position created: [position_id, staker, amount]
        PositionCreated(u64, AccountId, Balance),
        
        /// Position unstaked: [position_id, staker, amount]
        PositionUnstaked(u64, AccountId, Balance),
        
        /// Rewards claimed: [position_id, staker, amount]
        RewardsClaimed(u64, AccountId, Balance),
        
        /// Position slashed: [position_id, staker, amount]
        PositionSlashed(u64, AccountId, Balance),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Position not found
        PositionNotFound,
        
        /// Unauthorized access to position
        Unauthorized,
        
        /// Batch already staked
        BatchAlreadyStaked,
        
        /// Invalid staking amount
        InvalidStakingAmount,
        
        /// Batch not verified
        BatchNotVerified,
        
        /// Staking position still in lock period
        StakingLocked,
        
        /// Arithmetic overflow
        ArithmeticOverflow,
        
        /// Invalid unstaking operation
        InvalidUnstaking,
        
        /// Telemetry retrieval failed
        TelemetryRetrievalFailed,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initialize errors
        type Error = Error<T>;
        
        // Initialize events
        fn deposit_event() = default;
        
        /// Create a staking position with Eigenlayer
        #[weight = 10_000]
        pub fn create_position(
            origin,
            batch_id: Vec<u8>,
            amount: BalanceOf<T>,
            lock_blocks: T::BlockNumber
        ) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            
            // Ensure amount is within limits
            ensure!(
                amount >= T::MinStakingAmount::get() && amount <= T::MaxStakingAmount::get(),
                Error::<T>::InvalidStakingAmount
            );
            
            // Ensure batch is not already staked
            ensure!(
                !<BatchPositions<T>>::contains_key(&batch_id),
                Error::<T>::BatchAlreadyStaked
            );
            
            // Retrieve telemetry data for the batch
            let telemetry_api = telemetry::get_api()
                .ok_or(Error::<T>::TelemetryRetrievalFailed)?;
                
            let batch_id_str = sp_std::str::from_utf8(&batch_id)
                .map_err(|_| Error::<T>::TelemetryRetrievalFailed)?;
                
            let batch_data = telemetry_api.get_detailed_telemetry(batch_id_str)
                .ok_or(Error::<T>::TelemetryRetrievalFailed)?;
                
            // Validate batch quality score
            let quality_score = batch_data.summary.quality_score;
            
            // Check fermentation stage - kombucha has additional bonuses for later stages
            let fermentation_stage = batch_data.summary.fermentation_stage.as_str();
            let stage_bonus = match fermentation_stage {
                "Primary" => 0,
                "Secondary" => 50,  // 0.5% bonus for secondary fermentation
                "Bottling" => 100,  // 1% bonus for bottling stage
                "Aging" => 150,     // 1.5% bonus for aging
                _ => 0,
            };
            
            // Check SCOBY health for additional bonus
            let scoby_health = batch_data.summary.scoby_health.as_str();
            let scoby_bonus = match scoby_health {
                "Excellent" => 4 * Self::scoby_health_multiplier(),
                "Very Good" => 3 * Self::scoby_health_multiplier(),
                "Good" => 2 * Self::scoby_health_multiplier(),
                "Fair" => 1 * Self::scoby_health_multiplier(),
                _ => 0,
            };
            
            // Calculate APR based on quality and kombucha-specific factors
            let eigenlayer_calc = EigenlayerCalculator::new(Self::config());
            let base_apr = eigenlayer_calc.calculate_apr(quality_score);
            
            // Add kombucha-specific bonuses
            let apr = base_apr.saturating_add(stage_bonus).saturating_add(scoby_bonus);
            
            // Apply protocol fee
            let protocol_fee = amount
                .checked_mul(&(Self::protocol_fee() as u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(&(10000u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            let stake_amount = amount.checked_sub(&protocol_fee)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Calculate staking timeframe
            let current_block = <system::Pallet<T>>::block_number();
            let unstake_at = current_block.saturating_add(lock_blocks);
            
            // Get next position ID
            let position_id = Self::next_position_id();
            <NextPositionId<T>>::mutate(|id| *id = id.saturating_add(1));
            
            // Create staking position
            let position = StakingPosition {
                id: position_id,
                staker: staker.clone(),
                amount: stake_amount,
                product_type: ProductType::Kombucha,
                batch_id: batch_id.clone(),
                quality_score,
                current_apr: apr,
                staked_at: current_block,
                unstake_at,
                last_claim: current_block,
                unclaimed_rewards: Zero::zero(),
            };
            
            // Store position
            <StakingPositions<T>>::insert(position_id, position);
            
            // Update account positions
            <AccountPositions<T>>::mutate(&staker, |positions| {
                positions.push(position_id);
            });
            
            // Map batch to position
            <BatchPositions<T>>::insert(&batch_id, position_id);
            
            // Update total staked
            <TotalStaked<T>>::mutate(|total| {
                *total = total.saturating_add(&stake_amount);
            });
            
            // Transfer protocol fee to treasury
            let treasury = T::TreasuryAccount::get();
            T::Currency::transfer(
                &staker,
                &treasury,
                protocol_fee,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Transfer stake amount from user to pallet account
            T::Currency::transfer(
                &staker,
                &treasury, // Treasury also holds staked funds
                stake_amount,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Create quantum commitment for Eigenlayer using the quantum-resistant cryptography
            let commitment = eigenlayer_calc.generate_quantum_commitment(&batch_id);
            
            // Emit event
            Self::deposit_event(RawEvent::PositionCreated(position_id, staker, stake_amount));
            
            Ok(())
        }
        
        /// Unstake a position
        #[weight = 10_000]
        pub fn unstake_position(origin, position_id: u64) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            
            // Retrieve position
            let mut position = Self::staking_position(position_id).ok_or(Error::<T>::PositionNotFound)?;
            
            // Ensure caller is the staker
            ensure!(position.staker == staker, Error::<T>::Unauthorized);
            
            // Check if position is still locked
            let current_block = <system::Pallet<T>>::block_number();
            let is_early = current_block < position.unstake_at;
            
            // Calculate final rewards
            Self::calculate_and_update_rewards(&mut position, current_block)?;
            
            // Calculate amount to return
            let mut return_amount = position.amount;
            
            // Apply early unstaking penalty if applicable
            if is_early {
                let eigenlayer_calc = EigenlayerCalculator::new(Self::config());
                let penalty = eigenlayer_calc.calculate_early_unstake_penalty(position.amount);
                
                return_amount = return_amount.checked_sub(&penalty)
                    .ok_or(Error::<T>::ArithmeticOverflow)?;
                    
                // Penalty goes to treasury
                let treasury = T::TreasuryAccount::get();
                T::Currency::deposit_creating(&treasury, penalty);
            }
            
            // Add unclaimed rewards
            let total_return = return_amount.checked_add(&position.unclaimed_rewards)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Update total staked
            <TotalStaked<T>>::mutate(|total| {
                *total = total.saturating_sub(&position.amount);
            });
            
            // Remove batch mapping
            <BatchPositions<T>>::remove(&position.batch_id);
            
            // Mark position as completed
            position.current_apr = 0;
            position.unclaimed_rewards = Zero::zero();
            
            // Store updated position
            <StakingPositions<T>>::insert(position_id, position);
            
            // Transfer funds back to staker
            let treasury = T::TreasuryAccount::get();
            T::Currency::transfer(
                &treasury,
                &staker,
                total_return,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Emit event
            Self::deposit_event(RawEvent::PositionUnstaked(position_id, staker, total_return));
            
            Ok(())
        }
        
        /// Claim rewards from a staking position
        #[weight = 10_000]
        pub fn claim_rewards(origin, position_id: u64) -> DispatchResult {
            let staker = ensure_signed(origin)?;
            
            // Retrieve position
            let mut position = Self::staking_position(position_id).ok_or(Error::<T>::PositionNotFound)?;
            
            // Ensure caller is the staker
            ensure!(position.staker == staker, Error::<T>::Unauthorized);
            
            // Calculate rewards
            let current_block = <system::Pallet<T>>::block_number();
            Self::calculate_and_update_rewards(&mut position, current_block)?;
            
            // Ensure there are rewards to claim
            ensure!(!position.unclaimed_rewards.is_zero(), Error::<T>::InvalidUnstaking);
            
            let rewards = position.unclaimed_rewards;
            position.unclaimed_rewards = Zero::zero();
            position.last_claim = current_block;
            
            // Store updated position
            <StakingPositions<T>>::insert(position_id, position);
            
            // Transfer rewards to staker
            let treasury = T::TreasuryAccount::get();
            T::Currency::transfer(
                &treasury,
                &staker,
                rewards,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Emit event
            Self::deposit_event(RawEvent::RewardsClaimed(position_id, staker, rewards));
            
            Ok(())
        }
        
        /// Update Eigenlayer configuration
        #[weight = 10_000]
        pub fn update_config(origin, new_config: EigenlayerConfig) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure parameters are reasonable
            ensure!(new_config.base_apr <= 5000, Error::<T>::ArithmeticOverflow); // Max 50%
            ensure!(new_config.max_apr <= 10000, Error::<T>::ArithmeticOverflow); // Max 100%
            
            <Config<T>>::put(new_config);
            
            Ok(())
        }
        
        /// Update protocol fee
        #[weight = 10_000]
        pub fn update_protocol_fee(origin, new_fee: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure fee is reasonable
            ensure!(new_fee <= 500, Error::<T>::ArithmeticOverflow); // Max 5%
            
            <ProtocolFee<T>>::put(new_fee);
            
            Ok(())
        }
        
        /// Update operator fee
        #[weight = 10_000]
        pub fn update_operator_fee(origin, new_fee: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure fee is reasonable
            ensure!(new_fee <= 1000, Error::<T>::ArithmeticOverflow); // Max 10%
            
            <OperatorFee<T>>::put(new_fee);
            
            Ok(())
        }
        
        /// Update SCOBY health multiplier
        #[weight = 10_000]
        pub fn update_scoby_health_multiplier(origin, new_multiplier: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure multiplier is reasonable
            ensure!(new_multiplier <= 200, Error::<T>::ArithmeticOverflow); // Max 2%
            
            <ScobyHealthMultiplier<T>>::put(new_multiplier);
            
            Ok(())
        }
        
        /// Periodic updates (called by BlockNumberProvider)
        fn on_finalize(n: T::BlockNumber) {
            // Update positions every 1000 blocks
            if (n % 1000.into()).is_zero() {
                Self::update_all_positions();
            }
        }
    }
}

impl<T: Config> Module<T> {
    /// Calculate and update rewards for a position
    fn calculate_and_update_rewards(
        position: &mut StakingPosition<T::AccountId, BalanceOf<T>, T::BlockNumber>,
        current_block: T::BlockNumber
    ) -> Result<BalanceOf<T>, DispatchError> {
        // Calculate blocks since last claim
        let blocks_elapsed = current_block.saturating_sub(position.last_claim);
        
        // Skip if no blocks elapsed
        if blocks_elapsed.is_zero() {
            return Ok(Zero::zero());
        }
        
        // Convert to u64 for calculation
        let blocks_elapsed_u64: u64 = blocks_elapsed.saturated_into::<u64>();
        
        // Create calculator
        let eigenlayer_calc = EigenlayerCalculator::new(Self::config());
        
        // Calculate rewards for period
        let rewards = eigenlayer_calc.calculate_rewards(
            position.amount,
            position.current_apr,
            blocks_elapsed_u64,
            T::BlocksPerYear::get()
        );
        
        // Update position
        position.unclaimed_rewards = position.unclaimed_rewards
            .checked_add(&rewards)
            .ok_or(Error::<T>::ArithmeticOverflow)?;
            
        position.last_claim = current_block;
        
        Ok(rewards)
    }
    
    /// Update all active positions
    fn update_all_positions() {
        // Get all position IDs
        let position_count = Self::next_position_id();
        let current_block = <system::Pallet<T>>::block_number();
        
        for position_id in 1..position_count {
            if let Some(mut position) = Self::staking_position(position_id) {
                // Only update active positions
                if position.current_apr > 0 {
                    if let Err(e) = Self::calculate_and_update_rewards(&mut position, current_block) {
                        log::error!("Failed to update rewards for position {}: {:?}", position_id, e);
                        continue;
                    }
                    
                    // Update position
                    <StakingPositions<T>>::insert(position_id, position);
                }
            }
        }
    }
}
