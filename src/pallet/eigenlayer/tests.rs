//! Tests for the ELXR Eigenlayer pallet
//!
//! This module contains unit tests for the Eigenlayer functionality in the ELXR chain,
//! including staking, unstaking, and rewards calculations with quantum resistance,
//! adapted for kombucha production characteristics.

#[cfg(test)]
mod tests {
    use super::super::*;
    use frame_support::{
        assert_ok, assert_noop,
        parameter_types,
        traits::{GenesisBuild, OnFinalize, OnInitialize},
    };
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        DispatchError,
    };
    use frame_system as system;
    use common::{
        pricing::{PricingCalculator, ProductType},
        quantum_crypto::{QuantumCrypto, EigenlayerProof},
    };
    use mock_telemetry::{MockTelemetry, MockTelemetryData};

    type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
    type Block = frame_system::mocking::MockBlock<TestRuntime>;

    // Configure a mock runtime to test the pallet
    frame_support::construct_runtime!(
        pub enum TestRuntime where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic,
        {
            System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
            Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
            ElxrEigenlayer: super::Module::<TestRuntime>,
        }
    );

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const SS58Prefix: u8 = 42;
    }

    impl system::Config for TestRuntime {
        type BaseCallFilter = ();
        type BlockWeights = ();
        type BlockLength = ();
        type Origin = Origin;
        type Call = Call;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = Event;
        type BlockHashCount = BlockHashCount;
        type DbWeight = ();
        type Version = ();
        type PalletInfo = PalletInfo;
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
        type SS58Prefix = SS58Prefix;
        type OnSetCode = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 1;
        pub const MaxLocks: u32 = 50;
    }

    impl pallet_balances::Config for TestRuntime {
        type MaxLocks = MaxLocks;
        type Balance = u64;
        type Event = Event;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
        type MaxReserves = ();
        type ReserveIdentifier = [u8; 8];
    }

    parameter_types! {
        pub const BlocksPerYear: u64 = 5_256_000; // 6s blocks, 365 days
        pub const MinStakingAmount: u64 = 100;  // Lower for kombucha (smaller vessel size)
        pub const MaxStakingAmount: u64 = 500_000;
        pub const TreasuryAccount: H256 = H256::repeat_byte(0xE1);
    }

    impl Config for TestRuntime {
        type Event = Event;
        type Currency = Balances;
        type BlocksPerYear = BlocksPerYear;
        type MinStakingAmount = MinStakingAmount;
        type MaxStakingAmount = MaxStakingAmount;
        type TreasuryAccount = TreasuryAccount;
    }

    // Mock telemetry module for testing
    mod mock_telemetry {
        use super::*;

        pub struct MockTelemetry;
        
        pub struct MockTelemetryData {
            pub quality_score: u8,
            pub fill_percentage: u8,
            pub fermentation_stage: u8,  // Kombucha-specific
            pub scoby_health: u8,        // Kombucha-specific
            pub certified_organic: bool,
        }

        impl MockTelemetry {
            pub fn inject_telemetry_data(batch_id: &[u8], data: MockTelemetryData) {
                let batch_id_str = sp_std::str::from_utf8(batch_id).unwrap_or("unknown");
                MOCK_TELEMETRY_DATA.with(|m| {
                    m.borrow_mut().insert(batch_id_str.to_string(), data);
                });
            }

            pub fn get_telemetry_data(batch_id: &str) -> Option<MockTelemetryData> {
                MOCK_TELEMETRY_DATA.with(|m| {
                    m.borrow().get(batch_id).cloned()
                })
            }
        }

        thread_local! {
            pub static MOCK_TELEMETRY_DATA: std::cell::RefCell<std::collections::HashMap<String, MockTelemetryData>> = 
                std::cell::RefCell::new(std::collections::HashMap::new());
        }
    }

    // Helper function to build test externalities
    fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = frame_system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
        
        pallet_balances::GenesisConfig::<TestRuntime> {
            balances: vec![
                (1, 10000), // User 1 with 10000 tokens
                (2, 20000), // User 2 with 20000 tokens
                (3, 30000), // User 3 with 30000 tokens
                (999, 100000), // Treasury with 100000 tokens
            ],
        }.assimilate_storage(&mut t).unwrap();
        
        t.into()
    }

    // Mocking telemetry API for testing
    fn mock_telemetry_api() {
        // Mock batch 1 - Perfect kombucha, optimal stage and SCOBY
        MockTelemetry::inject_telemetry_data(
            b"batch-1", 
            MockTelemetryData { 
                quality_score: 90, 
                fill_percentage: 100,
                fermentation_stage: 5,   // Perfect fermentation stage
                scoby_health: 95,        // Excellent SCOBY health
                certified_organic: true,
            }
        );
        
        // Mock batch 2 - Good kombucha, middle stage
        MockTelemetry::inject_telemetry_data(
            b"batch-2", 
            MockTelemetryData { 
                quality_score: 75, 
                fill_percentage: 90,
                fermentation_stage: 3,   // Middle fermentation stage
                scoby_health: 80,        // Good SCOBY health
                certified_organic: true,
            }
        );
        
        // Mock batch 3 - Low quality kombucha, early stage
        MockTelemetry::inject_telemetry_data(
            b"batch-3", 
            MockTelemetryData { 
                quality_score: 50, 
                fill_percentage: 80,
                fermentation_stage: 1,   // Early fermentation stage
                scoby_health: 60,        // Mediocre SCOBY health
                certified_organic: false,
            }
        );
    }

    #[test]
    fn verify_batch_works() {
        new_test_ext().execute_with(|| {
            // Set up mock telemetry
            mock_telemetry_api();
            
            // Mock telemetry API access
            let batch_id = b"batch-1".to_vec();
            
            // Override the telemetry API access for testing
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Now try to verify the batch
            assert_ok!(ElxrEigenlayer::verify_batch(Origin::signed(1), batch_id.clone()));
            
            // Batch should be verified and stored
            assert!(ElxrEigenlayer::batch(&batch_id).is_some());
        });
    }

    #[test]
    fn stake_works() {
        new_test_ext().execute_with(|| {
            // Set up mock telemetry and verified vessel
            mock_telemetry_api();
            
            let batch_id = b"batch-1".to_vec();
            
            // Inject a verified batch
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Try to create a staking position
            let stake_amount = 2000; // 2000 tokens (lower for kombucha)
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_id.clone(),
                stake_amount
            ));
            
            // Check staking position was created
            let position_id = 1; // First position would have ID 1
            let position = ElxrEigenlayer::position(position_id);
            
            assert!(position.is_some());
            let position = position.unwrap();
            
            // Verify staking position details
            assert_eq!(position.staker, 1);
            assert_eq!(position.amount, stake_amount);
            assert_eq!(position.batch_id, batch_id);
            assert_eq!(position.status, StakingStatus::Active);
            
            // Kombucha-specific details
            assert_eq!(position.fermentation_stage, 5);
            assert_eq!(position.scoby_health, 95);
            
            // Check account positions mapping
            let account_positions = ElxrEigenlayer::account_positions(1);
            assert_eq!(account_positions.len(), 1);
            assert_eq!(account_positions[0], position_id);
            
            // Check staker's balance reduced by staked amount
            assert_eq!(Balances::free_balance(1), 10000 - stake_amount);
        });
    }

    #[test]
    fn unstake_works() {
        new_test_ext().execute_with(|| {
            // Set up a staking position first
            mock_telemetry_api();
            
            let batch_id = b"batch-1".to_vec();
            
            // Inject a verified batch
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Create a staking position
            let stake_amount = 2000; // 2000 tokens
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_id.clone(),
                stake_amount
            ));
            
            // Now try to unstake
            assert_ok!(ElxrEigenlayer::request_unstake(
                Origin::signed(1),
                1 // position_id
            ));
            
            // Check position was updated
            let position = ElxrEigenlayer::position(1).unwrap();
            
            // Position should be in Unlocking state
            assert_eq!(position.status, StakingStatus::Unlocking);
            assert!(position.unlock_date.is_some());
            
            // Fast forward to unlock date
            if let Some(unlock_date) = position.unlock_date {
                run_to_block(unlock_date);
                
                // Complete unstaking
                assert_ok!(ElxrEigenlayer::complete_unstake(
                    Origin::signed(1),
                    1 // position_id
                ));
                
                // Check position is unstaked
                let position = ElxrEigenlayer::position(1).unwrap();
                assert_eq!(position.status, StakingStatus::Unstaked);
                
                // Check funds returned to account (minus any early unstaking fee)
                assert_eq!(Balances::free_balance(1), 10000);
            }
        });
    }

    #[test]
    fn claim_rewards_works() {
        new_test_ext().execute_with(|| {
            // Set up a staking position
            mock_telemetry_api();
            
            let batch_id = b"batch-1".to_vec();
            
            // Inject a verified batch
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Create a staking position
            let stake_amount = 2000; // 2000 tokens
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_id.clone(),
                stake_amount
            ));
            
            // Advance blocks to accrue rewards
            run_to_block(10000);
            
            // Update rewards (simulating on_finalize)
            ElxrEigenlayer::update_positions();
            
            // Get position with accumulated rewards
            let position = ElxrEigenlayer::position(1).unwrap();
            let rewards_amount = position.rewards_earned;
            
            // Rewards should be higher due to high fermentation stage and SCOBY health
            assert!(rewards_amount > 0);
            
            // Claim rewards
            assert_ok!(ElxrEigenlayer::claim_rewards(
                Origin::signed(1),
                1 // position_id
            ));
            
            // Check rewards have been claimed
            let updated_position = ElxrEigenlayer::position(1).unwrap();
            assert_eq!(updated_position.rewards_earned, 0);
            
            // Check rewards added to account balance
            assert_eq!(Balances::free_balance(1), 10000 - stake_amount + rewards_amount);
        });
    }

    #[test]
    fn fermentation_stage_bonus_works() {
        new_test_ext().execute_with(|| {
            // Set up staking positions for different fermentation stages
            mock_telemetry_api();
            
            // Batch 1 - Perfect kombucha (stage 5)
            let batch_1 = b"batch-1".to_vec();
            ElxrEigenlayer::test_inject_batch(
                batch_1.clone(),
                BatchData {
                    batch_id: batch_1.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Batch 3 - Early stage kombucha (stage 1)
            let batch_3 = b"batch-3".to_vec();
            ElxrEigenlayer::test_inject_batch(
                batch_3.clone(),
                BatchData {
                    batch_id: batch_3.clone(),
                    facility_id: b"facility-3".to_vec(),
                    quality_score: 50,
                    fermentation_day: 4,
                    fermentation_stage: 1,
                    scoby_health: 60,
                    fill_percentage: 80,
                    certifications: vec![],
                    region: b"global".to_vec(),
                }
            );
            
            // Create staking positions for both
            let stake_amount = 2000; // Same amount for fair comparison
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_1.clone(),
                stake_amount
            ));
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(2),
                batch_3.clone(),
                stake_amount
            ));
            
            // Advance blocks to accrue rewards
            run_to_block(10000);
            
            // Update rewards
            ElxrEigenlayer::update_positions();
            
            // Get positions with accumulated rewards
            let position_1 = ElxrEigenlayer::position(1).unwrap(); // Optimal fermentation
            let position_3 = ElxrEigenlayer::position(2).unwrap(); // Early fermentation
            
            // Higher fermentation stage should yield more rewards
            assert!(position_1.rewards_earned > position_3.rewards_earned);
            
            // Calculate expected bonus ratio (not exact due to compounding)
            let expected_ratio = (position_1.rewards_earned as f64) / (position_3.rewards_earned as f64);
            
            // The ratio should be significant (at least 1.5x more rewards for perfect fermentation)
            assert!(expected_ratio > 1.5);
        });
    }

    #[test]
    fn scoby_health_bonus_works() {
        new_test_ext().execute_with(|| {
            // Set up staking positions for different SCOBY health levels
            mock_telemetry_api();
            
            // Batch 1 - Excellent SCOBY health (95)
            let batch_1 = b"batch-1".to_vec();
            ElxrEigenlayer::test_inject_batch(
                batch_1.clone(),
                BatchData {
                    batch_id: batch_1.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Create a second batch with identical parameters except SCOBY health
            let batch_modified = b"batch-modified".to_vec();
            ElxrEigenlayer::test_inject_batch(
                batch_modified.clone(),
                BatchData {
                    batch_id: batch_modified.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 50, // Lower SCOBY health
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Create staking positions for both
            let stake_amount = 2000; // Same amount for fair comparison
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_1.clone(),
                stake_amount
            ));
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(2),
                batch_modified.clone(),
                stake_amount
            ));
            
            // Advance blocks to accrue rewards
            run_to_block(10000);
            
            // Update rewards
            ElxrEigenlayer::update_positions();
            
            // Get positions with accumulated rewards
            let position_1 = ElxrEigenlayer::position(1).unwrap(); // Excellent SCOBY health
            let position_2 = ElxrEigenlayer::position(2).unwrap(); // Mediocre SCOBY health
            
            // Higher SCOBY health should yield more rewards
            assert!(position_1.rewards_earned > position_2.rewards_earned);
            
            // The difference should be significant
            assert!(position_1.rewards_earned > position_2.rewards_earned * 12 / 10); // At least 20% more
        });
    }

    #[test]
    fn get_scoby_score_works() {
        new_test_ext().execute_with(|| {
            // Set up a batch with SCOBY data
            mock_telemetry_api();
            
            let batch_id = b"batch-1".to_vec();
            
            // Inject a verified batch with SCOBY data
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Inject SCOBY-specific data
            ElxrEigenlayer::test_inject_scoby_data(
                batch_id.clone(),
                ScobyData {
                    batch_id: batch_id.clone(),
                    scoby_health: 95,
                    age_days: 28,
                    thickness_mm: 12,
                    culture_density: 85,
                    ph_level: 3.2,
                }
            );
            
            // Query the SCOBY score
            let scoby_data = ElxrEigenlayer::get_scoby_score(
                Origin::signed(1),
                batch_id.clone()
            ).unwrap();
            
            // Verify data
            assert_eq!(scoby_data.scoby_health, 95);
            assert_eq!(scoby_data.age_days, 28);
            assert_eq!(scoby_data.thickness_mm, 12);
            assert_eq!(scoby_data.culture_density, 85);
            assert_eq!(scoby_data.ph_level, 3.2);
        });
    }

    #[test]
    fn quantum_proof_verification_works() {
        new_test_ext().execute_with(|| {
            // Set up a staking position
            mock_telemetry_api();
            
            let batch_id = b"batch-1".to_vec();
            
            // Inject a verified batch
            ElxrEigenlayer::test_inject_batch(
                batch_id.clone(),
                BatchData {
                    batch_id: batch_id.clone(),
                    facility_id: b"facility-1".to_vec(),
                    quality_score: 90,
                    fermentation_day: 14,
                    fermentation_stage: 5,
                    scoby_health: 95,
                    fill_percentage: 100,
                    certifications: vec![b"organic".to_vec()],
                    region: b"US-West".to_vec(),
                }
            );
            
            // Create a staking position
            let stake_amount = 2000; // 2000 tokens
            
            assert_ok!(ElxrEigenlayer::stake(
                Origin::signed(1),
                batch_id.clone(),
                stake_amount
            ));
            
            // Generate a quantum keypair for testing
            let keypair = QuantumCrypto::generate_keypair().expect("Failed to generate keypair");
            
            // Create position data for proof including kombucha-specific parameters
            let position_data = format!("position:1:batch:{:?}:staker:1:amount:{}:fermentation_stage:5:scoby_health:95", 
                                        batch_id, stake_amount).into_bytes();
            
            // Create a quantum proof
            let proof = QuantumCrypto::create_eigenlayer_proof(
                1, // position_id
                &batch_id,
                position_data.clone(),
                &keypair.signing_private_key
            ).expect("Failed to create proof");
            
            // Inject the proof for testing
            ElxrEigenlayer::test_store_quantum_proof(1, proof.clone());
            
            // Verify the quantum proof
            let verification_result = ElxrEigenlayer::verify_quantum_proof(
                Origin::signed(1),
                1, // position_id
                proof
            );
            
            assert_ok!(verification_result);
            assert!(verification_result.unwrap());
            
            // Create an invalid proof with different data
            let invalid_data = b"tampered_position_data";
            let invalid_proof = QuantumCrypto::create_eigenlayer_proof(
                1,
                &batch_id,
                invalid_data,
                &keypair.signing_private_key
            ).expect("Failed to create proof");
            
            // Verification should fail for invalid proof
            let invalid_result = ElxrEigenlayer::verify_quantum_proof(
                Origin::signed(1),
                1,
                invalid_proof
            );
            
            assert_ok!(invalid_result);
            assert!(!invalid_result.unwrap());
        });
    }

    // Helper to run to a specific block
    fn run_to_block(n: u64) {
        while System::block_number() < n {
            let block_number = System::block_number();
            System::on_finalize(block_number);
            System::set_block_number(block_number + 1);
            System::on_initialize(block_number + 1);
        }
    }

    // Add test methods to the Module for testing purposes
    impl<T: Config> Module<T> {
        // Test-only function to inject a batch for testing
        pub fn test_inject_batch(batch_id: Vec<u8>, batch_data: BatchData) {
            <Batches<T>>::insert(&batch_id, batch_data);
        }
        
        // Test-only function to store a quantum proof for testing
        pub fn test_store_quantum_proof(position_id: u64, proof: EigenlayerProof) {
            <QuantumProofs<T>>::insert(position_id, proof);
        }
        
        // Test-only function to inject SCOBY data
        pub fn test_inject_scoby_data(batch_id: Vec<u8>, scoby_data: ScobyData) {
            <ScobyScores<T>>::insert(&batch_id, scoby_data);
        }
    }
}
