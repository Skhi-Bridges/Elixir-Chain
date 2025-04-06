//! DeFi pallet for ELXR (Kombucha) chain
//! 
//! This pallet implements DeFi functionality for the ELXR chain,
//! allowing users to obtain loans based on their kombucha production value.
//! The loan values are calculated based on the fixed price of $20 per gallon
//! and standard vessel size of 50 gallons.

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

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance;

/// Configuration trait for the DeFi pallet
pub trait Config: system::Config {
    /// The overarching event type
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    
    /// Currency mechanism
    type Currency: Currency<Self::AccountId>;
    
    /// Minimum collateral ratio required (e.g. 125%)
    type MinCollateralRatio: Get<u64>;
    
    /// Maximum loan-to-value ratio (e.g. 80%)
    type MaxLoanToValue: Get<u64>;
    
    /// Liquidation threshold for collateral ratio (e.g. 110%)
    type LiquidationThreshold: Get<u64>;
    
    /// Treasury account for fee collection
    type TreasuryAccount: Get<Self::AccountId>;
}

/// Loan state enum
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum LoanState {
    /// Loan is active and in good standing
    Active,
    /// Loan is at risk of liquidation
    AtRisk,
    /// Loan has been liquidated
    Liquidated,
    /// Loan has been fully repaid
    Repaid,
}

/// Loan data structure
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Loan<AccountId, Balance, BlockNumber> {
    /// Loan ID
    pub id: u64,
    /// Borrower account
    pub borrower: AccountId,
    /// Initial loan amount
    pub initial_amount: Balance,
    /// Current outstanding amount (including interest)
    pub current_amount: Balance,
    /// Batch ID used as collateral
    pub collateral_batch_id: Vec<u8>,
    /// Collateral value at loan initiation
    pub collateral_value: Balance,
    /// Current loan-to-value ratio (in basis points, 1/100 of a percent)
    pub current_ltv: u64,
    /// Loan creation block
    pub created_at: BlockNumber,
    /// Loan due date block
    pub due_by: BlockNumber,
    /// Last interest applied block
    pub last_interest_block: BlockNumber,
    /// Interest rate (in basis points, 1/100 of a percent, per 10000 blocks)
    pub interest_rate: u64,
    /// Loan state
    pub state: LoanState,
}

/// Vessel data validated from telemetry
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct VesselData {
    /// Batch ID
    pub batch_id: Vec<u8>,
    /// Facility ID
    pub facility_id: Vec<u8>,
    /// Recipe ID
    pub recipe_id: Vec<u8>,
    /// Fermentation stage
    pub fermentation_stage: Vec<u8>,
    /// Quality score (0-100)
    pub quality_score: u8,
    /// Fermentation day
    pub fermentation_day: u32,
    /// Current fill percentage (0-100)
    pub fill_percentage: u8,
    /// Certifications
    pub certifications: Vec<Vec<u8>>,
    /// Region code
    pub region: Vec<u8>,
    /// SCOBY health
    pub scoby_health: Vec<u8>,
}

decl_storage! {
    trait Store for Module<T: Config> as ElxrDefi {
        /// Next loan ID to be assigned
        pub NextLoanId get(fn next_loan_id): u64 = 1;
        
        /// Mapping from loan ID to loan data
        pub Loans get(fn loan): map hasher(twox_64_concat) u64 => Option<Loan<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
        
        /// Mapping from account to loan IDs
        pub AccountLoans get(fn account_loans): map hasher(blake2_128_concat) T::AccountId => Vec<u64>;
        
        /// Validated vessel data
        pub Vessels get(fn vessel): map hasher(twox_64_concat) Vec<u8> => Option<VesselData>;
        
        /// Current interest rate for new loans (in basis points)
        pub BaseInterestRate get(fn base_interest_rate): u64 = 400; // 4% per 10000 blocks
        
        /// Liquidation penalty rate (in basis points)
        pub LiquidationPenalty get(fn liquidation_penalty): u64 = 1000; // 10%
        
        /// Origination fee for new loans (in basis points)
        pub OriginationFee get(fn origination_fee): u64 = 100; // 1%
    }
}

decl_event!(
    pub enum Event<T> where
        <T as system::Config>::AccountId,
        Balance = BalanceOf<T>,
    {
        /// Loan created: [loan_id, borrower, loan_amount]
        LoanCreated(u64, AccountId, Balance),
        
        /// Loan repaid: [loan_id, borrower, amount_repaid]
        LoanRepaid(u64, AccountId, Balance),
        
        /// Loan liquidated: [loan_id, borrower, liquidated_amount]
        LoanLiquidated(u64, AccountId, Balance),
        
        /// Loan updated: [loan_id]
        LoanUpdated(u64),
        
        /// Vessel verified: [batch_id]
        VesselVerified(Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Loan not found
        LoanNotFound,
        
        /// Unauthorized access to loan
        Unauthorized,
        
        /// Insufficient collateral
        InsufficientCollateral,
        
        /// Invalid loan amount (too small or too large)
        InvalidLoanAmount,
        
        /// Invalid loan term (too short or too long)
        InvalidLoanTerm,
        
        /// Loan already repaid
        LoanAlreadyRepaid,
        
        /// Vessel not verified
        VesselNotVerified,
        
        /// Vessel already used as collateral
        VesselAlreadyUsed,
        
        /// Arithmetic overflow
        ArithmeticOverflow,
        
        /// Loan is in liquidation state
        LoanLiquidated,
        
        /// Insufficient repayment amount
        InsufficientRepayment,
        
        /// Failed to retrieve telemetry data
        TelemetryRetrievalFailed,
        
        /// Invalid fermentation stage for loan
        InvalidFermentationStage,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Initialize errors
        type Error = Error<T>;
        
        // Initialize events
        fn deposit_event() = default;
        
        /// Verify a kombucha vessel and prepare it for use as collateral
        #[weight = 10_000]
        pub fn verify_vessel(origin, batch_id: Vec<u8>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Retrieve telemetry data for the batch
            let telemetry_api = telemetry::get_api()
                .ok_or(Error::<T>::TelemetryRetrievalFailed)?;
                
            let batch_id_str = sp_std::str::from_utf8(&batch_id)
                .map_err(|_| Error::<T>::TelemetryRetrievalFailed)?;
                
            let batch_data = telemetry_api.get_detailed_telemetry(batch_id_str)
                .ok_or(Error::<T>::TelemetryRetrievalFailed)?;
                
            // Extract vessel data from telemetry
            let vessel_data = VesselData {
                batch_id: batch_id.clone(),
                facility_id: batch_data.summary.facility_id.as_bytes().to_vec(),
                recipe_id: batch_data.summary.recipe_id.as_bytes().to_vec(),
                fermentation_stage: batch_data.summary.fermentation_stage.as_bytes().to_vec(),
                quality_score: batch_data.summary.quality_score,
                fermentation_day: batch_data.summary.fermentation_day,
                fill_percentage: 100, // Assuming full vessel for simplicity
                certifications: vec![], // Extract from metadata if available
                region: "global".as_bytes().to_vec(), // Default region
                scoby_health: batch_data.summary.scoby_health.as_bytes().to_vec(),
            };
            
            // Store vessel data
            <Vessels<T>>::insert(&batch_id, vessel_data);
            
            // Emit event
            Self::deposit_event(RawEvent::VesselVerified(batch_id));
            
            Ok(())
        }
        
        /// Create a loan using a kombucha vessel as collateral
        #[weight = 10_000]
        pub fn create_loan(
            origin,
            batch_id: Vec<u8>,
            requested_amount: BalanceOf<T>,
            loan_term_blocks: T::BlockNumber
        ) -> DispatchResult {
            let borrower = ensure_signed(origin)?;
            
            // Check if vessel exists and is verified
            let vessel_data = Self::vessel(&batch_id).ok_or(Error::<T>::VesselNotVerified)?;
            
            // Check if vessel is already used as collateral
            for loan_id in Self::account_loans(&borrower) {
                if let Some(loan) = Self::loan(loan_id) {
                    if loan.collateral_batch_id == batch_id &&
                       (loan.state == LoanState::Active || loan.state == LoanState::AtRisk) {
                        return Err(Error::<T>::VesselAlreadyUsed.into());
                    }
                }
            }
            
            // Check fermentation stage (only allow loans on Primary and Secondary stages)
            let stage = sp_std::str::from_utf8(&vessel_data.fermentation_stage)
                .map_err(|_| Error::<T>::TelemetryRetrievalFailed)?;
                
            ensure!(
                stage == "Primary" || stage == "Secondary",
                Error::<T>::InvalidFermentationStage
            );
            
            // Create pricing calculator
            let calculator = PricingCalculator::default();
            
            // Convert vessel data to format expected by calculator
            let certifications: Vec<String> = vessel_data.certifications.iter()
                .filter_map(|c| sp_std::str::from_utf8(c).ok().map(|s| s.to_string()))
                .collect();
                
            let region = sp_std::str::from_utf8(&vessel_data.region)
                .unwrap_or("global")
                .to_string();
                
            // Calculate vessel value
            let vessel_value = calculator.calculate_vessel_value(
                ProductType::Kombucha,
                vessel_data.quality_score,
                &certifications,
                &region,
                vessel_data.fill_percentage as f64 / 100.0,
            );
            
            // Convert to blockchain balance unit
            let vessel_value_balance = Self::float_to_balance(vessel_value);
            
            // Calculate max loan amount based on LTV
            let max_ltv = T::MaxLoanToValue::get();
            let max_loan_amount = vessel_value_balance
                .checked_mul(&(max_ltv as u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(&(10000u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Ensure requested amount is valid
            ensure!(requested_amount <= max_loan_amount, Error::<T>::InvalidLoanAmount);
            ensure!(!requested_amount.is_zero(), Error::<T>::InvalidLoanAmount);
            
            // Calculate current LTV ratio
            let current_ltv = Self::calculate_ltv(requested_amount, vessel_value_balance)?;
            
            // Apply origination fee
            let fee_rate = Self::origination_fee();
            let fee_amount = requested_amount
                .checked_mul(&(fee_rate as u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(&(10000u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            let amount_after_fee = requested_amount
                .checked_sub(&fee_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Calculate interest rate (base rate + premium based on LTV, quality, and SCOBY health)
            let base_rate = Self::base_interest_rate();
            
            // Higher LTV = higher interest
            let ltv_premium = current_ltv / 100;
            
            // Higher quality = lower interest
            let quality_discount = vessel_data.quality_score as u64;
            
            // SCOBY health affects interest rate
            let scoby_health = sp_std::str::from_utf8(&vessel_data.scoby_health)
                .unwrap_or("Unknown");
                
            let scoby_discount = match scoby_health {
                "Excellent" => 50,
                "Very Good" => 40,
                "Good" => 30,
                "Fair" => 10,
                _ => 0,
            };
            
            let interest_rate = base_rate
                .saturating_add(ltv_premium)
                .saturating_sub(quality_discount / 2)
                .saturating_sub(scoby_discount);
            
            // Calculate loan due date
            let current_block = <system::Pallet<T>>::block_number();
            let due_by = current_block.saturating_add(loan_term_blocks);
            
            // Get next loan ID
            let loan_id = Self::next_loan_id();
            <NextLoanId<T>>::mutate(|id| *id = id.saturating_add(1));
            
            // Create loan
            let loan = Loan {
                id: loan_id,
                borrower: borrower.clone(),
                initial_amount: requested_amount,
                current_amount: requested_amount,
                collateral_batch_id: batch_id,
                collateral_value: vessel_value_balance,
                current_ltv,
                created_at: current_block,
                due_by,
                last_interest_block: current_block,
                interest_rate,
                state: LoanState::Active,
            };
            
            // Store loan
            <Loans<T>>::insert(loan_id, loan);
            
            // Update account loans
            <AccountLoans<T>>::mutate(&borrower, |loans| {
                loans.push(loan_id);
            });
            
            // Transfer fee to treasury
            let treasury = T::TreasuryAccount::get();
            T::Currency::transfer(
                &borrower,
                &treasury,
                fee_amount,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Transfer loan amount to borrower
            T::Currency::deposit_creating(&borrower, amount_after_fee);
            
            // Emit event
            Self::deposit_event(RawEvent::LoanCreated(loan_id, borrower, requested_amount));
            
            Ok(())
        }
        
        /// Repay a loan (fully or partially)
        #[weight = 10_000]
        pub fn repay_loan(origin, loan_id: u64, amount: BalanceOf<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Retrieve loan
            let mut loan = Self::loan(loan_id).ok_or(Error::<T>::LoanNotFound)?;
            
            // Ensure caller is the borrower
            ensure!(loan.borrower == who, Error::<T>::Unauthorized);
            
            // Ensure loan is not already repaid or liquidated
            ensure!(loan.state != LoanState::Repaid, Error::<T>::LoanAlreadyRepaid);
            ensure!(loan.state != LoanState::Liquidated, Error::<T>::LoanLiquidated);
            
            // Apply accrued interest
            Self::apply_interest(&mut loan)?;
            
            // Ensure repayment amount is valid
            ensure!(!amount.is_zero(), Error::<T>::InsufficientRepayment);
            ensure!(amount <= loan.current_amount, Error::<T>::ArithmeticOverflow);
            
            // Process repayment
            loan.current_amount = loan.current_amount
                .checked_sub(&amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Update loan state if fully repaid
            if loan.current_amount.is_zero() {
                loan.state = LoanState::Repaid;
            } else {
                // Recalculate LTV
                loan.current_ltv = Self::calculate_ltv(loan.current_amount, loan.collateral_value)?;
                
                // Update loan state based on new LTV
                if loan.current_ltv <= T::LiquidationThreshold::get() {
                    loan.state = LoanState::Active;
                } else {
                    loan.state = LoanState::AtRisk;
                }
            }
            
            // Update loan
            <Loans<T>>::insert(loan_id, loan.clone());
            
            // Transfer repayment from user
            T::Currency::transfer(
                &who,
                &T::TreasuryAccount::get(),
                amount,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Emit event
            Self::deposit_event(RawEvent::LoanRepaid(loan_id, who, amount));
            
            Ok(())
        }
        
        /// Liquidate a loan that has fallen below the required collateral ratio
        #[weight = 10_000]
        pub fn liquidate_loan(origin, loan_id: u64) -> DispatchResult {
            let liquidator = ensure_signed(origin)?;
            
            // Retrieve loan
            let mut loan = Self::loan(loan_id).ok_or(Error::<T>::LoanNotFound)?;
            
            // Apply accrued interest
            Self::apply_interest(&mut loan)?;
            
            // Recalculate LTV
            loan.current_ltv = Self::calculate_ltv(loan.current_amount, loan.collateral_value)?;
            
            // Ensure loan is eligible for liquidation
            ensure!(
                loan.current_ltv > T::LiquidationThreshold::get() || 
                <system::Pallet<T>>::block_number() > loan.due_by,
                Error::<T>::Unauthorized
            );
            
            // Ensure loan is not already liquidated
            ensure!(loan.state != LoanState::Liquidated, Error::<T>::LoanLiquidated);
            
            // Calculate liquidation amount (loan amount + penalty)
            let penalty_amount = loan.current_amount
                .checked_mul(&(Self::liquidation_penalty() as u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?
                .checked_div(&(10000u32).into())
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            let liquidation_amount = loan.current_amount
                .checked_add(&penalty_amount)
                .ok_or(Error::<T>::ArithmeticOverflow)?;
                
            // Ensure liquidator has enough funds
            ensure!(
                T::Currency::free_balance(&liquidator) >= liquidation_amount,
                Error::<T>::InsufficientRepayment
            );
            
            // Transfer liquidation amount to treasury
            T::Currency::transfer(
                &liquidator,
                &T::TreasuryAccount::get(),
                liquidation_amount,
                ExistenceRequirement::KeepAlive
            )?;
            
            // Update loan state
            loan.state = LoanState::Liquidated;
            <Loans<T>>::insert(loan_id, loan.clone());
            
            // Emit event
            Self::deposit_event(RawEvent::LoanLiquidated(
                loan_id,
                loan.borrower.clone(),
                liquidation_amount
            ));
            
            Ok(())
        }
        
        /// Update the base interest rate
        #[weight = 10_000]
        pub fn update_base_rate(origin, new_rate: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure new rate is reasonable
            ensure!(new_rate <= 5000, Error::<T>::ArithmeticOverflow); // Max 50%
            
            <BaseInterestRate<T>>::put(new_rate);
            
            Ok(())
        }
        
        /// Update the liquidation penalty
        #[weight = 10_000]
        pub fn update_liquidation_penalty(origin, new_penalty: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure new penalty is reasonable
            ensure!(new_penalty <= 5000, Error::<T>::ArithmeticOverflow); // Max 50%
            
            <LiquidationPenalty<T>>::put(new_penalty);
            
            Ok(())
        }
        
        /// Update the origination fee
        #[weight = 10_000]
        pub fn update_origination_fee(origin, new_fee: u64) -> DispatchResult {
            ensure_root(origin)?;
            
            // Ensure new fee is reasonable
            ensure!(new_fee <= 1000, Error::<T>::ArithmeticOverflow); // Max 10%
            
            <OriginationFee<T>>::put(new_fee);
            
            Ok(())
        }
        
        /// Periodic updates (called by BlockNumberProvider)
        fn on_finalize(n: T::BlockNumber) {
            // Update loans every 100 blocks
            if (n % 100.into()).is_zero() {
                Self::update_loans();
            }
        }
    }
}

impl<T: Config> Module<T> {
    /// Calculate loan-to-value ratio in basis points (1/100 of a percent)
    fn calculate_ltv(loan_amount: BalanceOf<T>, collateral_value: BalanceOf<T>) -> Result<u64, DispatchError> {
        // LTV = (loan_amount / collateral_value) * 10000
        let ltv_raw = loan_amount
            .checked_mul(&(10000u32).into())
            .ok_or(Error::<T>::ArithmeticOverflow)?
            .checked_div(&collateral_value)
            .ok_or(Error::<T>::ArithmeticOverflow)?;
            
        // Convert balance to u64 for simplified math
        let ltv = ltv_raw.saturated_into::<u64>();
        
        Ok(ltv)
    }
    
    /// Convert float value to Balance
    fn float_to_balance(value: f64) -> BalanceOf<T> {
        // Assuming 12 decimal places in the native token
        let amount_raw = (value * 1_000_000_000_000.0) as u128;
        amount_raw.saturated_into::<BalanceOf<T>>()
    }
    
    /// Apply accrued interest to a loan
    fn apply_interest(loan: &mut Loan<T::AccountId, BalanceOf<T>, T::BlockNumber>) -> Result<(), DispatchError> {
        let current_block = <system::Pallet<T>>::block_number();
        
        // Calculate blocks since last interest update
        let blocks_elapsed = current_block.saturating_sub(loan.last_interest_block);
        
        // Skip if no blocks elapsed
        if blocks_elapsed.is_zero() {
            return Ok(());
        }
        
        // Calculate interest
        let blocks_elapsed_u64: u64 = blocks_elapsed.saturated_into::<u64>();
        
        // Interest = loan_amount * interest_rate * blocks_elapsed / 10000 / 10000
        // (interest_rate is per 10000 blocks)
        let interest = loan.current_amount
            .checked_mul(&(loan.interest_rate as u32).into())
            .ok_or(Error::<T>::ArithmeticOverflow)?
            .checked_mul(&(blocks_elapsed_u64 as u32).into())
            .ok_or(Error::<T>::ArithmeticOverflow)?
            .checked_div(&(10000u32).into())  // Basis points
            .ok_or(Error::<T>::ArithmeticOverflow)?
            .checked_div(&(10000u32).into())  // Per 10000 blocks
            .ok_or(Error::<T>::ArithmeticOverflow)?;
            
        // Add interest to current amount
        loan.current_amount = loan.current_amount
            .checked_add(&interest)
            .ok_or(Error::<T>::ArithmeticOverflow)?;
            
        // Update last interest block
        loan.last_interest_block = current_block;
        
        // Recalculate LTV
        loan.current_ltv = Self::calculate_ltv(loan.current_amount, loan.collateral_value)?;
        
        // Update loan state based on new LTV
        if loan.current_ltv > T::LiquidationThreshold::get() {
            loan.state = LoanState::AtRisk;
        }
        
        Ok(())
    }
    
    /// Update all active loans (for interest and state)
    fn update_loans() {
        // Get all loan IDs
        let loan_count = Self::next_loan_id();
        
        for loan_id in 1..loan_count {
            if let Some(mut loan) = Self::loan(loan_id) {
                // Skip if loan is not active
                if loan.state != LoanState::Active && loan.state != LoanState::AtRisk {
                    continue;
                }
                
                // Apply interest
                if let Err(e) = Self::apply_interest(&mut loan) {
                    log::error!("Failed to apply interest to loan {}: {:?}", loan_id, e);
                    continue;
                }
                
                // Update loan
                <Loans<T>>::insert(loan_id, loan);
                
                // Emit event
                Self::deposit_event(RawEvent::LoanUpdated(loan_id));
            }
        }
    }
}
