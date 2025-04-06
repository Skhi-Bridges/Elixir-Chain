//! Tests for the ELXR Eigenlayer RPC module
//!
//! This module contains unit tests for the Eigenlayer RPC functionality in the ELXR chain,
//! including kombucha-specific features like fermentation stage and SCOBY health metrics.

#[cfg(test)]
mod tests {
    use super::super::*;
    use mockall::predicate::*;
    use mockall::*;
    use sp_core::H256;
    use sp_runtime::testing::TestXt;
    use std::sync::Arc;
    use jsonrpsee::core::RpcResult;
    use common::quantum_crypto::{QuantumCrypto, EigenlayerProof};
    use crate::{
        runtime::{AccountId, Balance, BlockNumber},
        pallet::eigenlayer::PositionData,
    };

    // Mock the client
    #[automock]
    pub trait RuntimeApiMock {
        fn get_staking_params(&self, at: Option<H256>) -> Result<StakingParams, sp_runtime::DispatchError>;
        fn get_staking_positions(&self, account_id: AccountId, at: Option<H256>) -> Result<Vec<KombuchaStakingPosition>, sp_runtime::DispatchError>;
        fn calculate_potential_rewards(&self, amount: Balance, quality_score: u8, fermentation_stage: u8, scoby_health: u8, days: u32, at: Option<H256>) -> Result<PotentialRewards, sp_runtime::DispatchError>;
        fn get_scoby_score(&self, batch_id: Vec<u8>, at: Option<H256>) -> Result<ScobyData, sp_runtime::DispatchError>;
        fn verify_quantum_proof(&self, position_id: u64, proof: EigenlayerProof, at: Option<H256>) -> Result<bool, sp_runtime::DispatchError>;
    }

    // Mock client implementation
    mock! {
        ClientMock<B> {}
        impl<B: sp_runtime::traits::Block> HeaderBackend<B> for ClientMock<B> {
            fn header(&self, hash: B::Hash) -> sp_blockchain::Result<Option<B::Header>>;
            fn info(&self) -> sp_blockchain::Info<B>;
            fn status(&self, hash: B::Hash) -> sp_blockchain::Result<sp_blockchain::BlockStatus>;
            fn number(&self, hash: B::Hash) -> sp_blockchain::Result<Option<sp_blockchain::NumberFor<B>>>;
            fn hash(&self, number: sp_blockchain::NumberFor<B>) -> sp_blockchain::Result<Option<B::Hash>>;
        }
        
        impl<B: sp_runtime::traits::Block> ProvideRuntimeApi<B> for ClientMock<B> {
            type Api = MockRuntimeApi;
            fn runtime_api(&self) -> sp_api::ApiRef<Self::Api>;
        }
    }

    // Test block type
    type Block = sp_runtime::generic::Block<
        sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
        TestXt<()>
    >;

    // Setup test RPC
    fn setup_test_rpc() -> (MockClientMock<Block>, ElxrEigenlayerRpcImpl<Block, MockClientMock<Block>>) {
        let mock_client = MockClientMock::<Block>::default();
        let mock_runtime_api = MockRuntimeApi::default();
        
        // Setup expectations for runtime_api()
        mock_client.expect_runtime_api()
            .returning(move || {
                sp_api::ApiRef::from(Arc::new(mock_runtime_api.clone()) as Arc<_>)
            });
        
        let rpc = ElxrEigenlayerRpcImpl::new(Arc::new(mock_client.clone()));
        
        (mock_client, rpc)
    }

    #[test]
    fn test_get_staking_params() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Expected staking params (specific to kombucha)
        let expected_params = StakingParams {
            min_stake: 100,  // Lower for kombucha
            max_stake: 500000,
            unbonding_period: 14400, // 1 day in blocks (faster for kombucha)
            base_apy: 15.0,  // Higher base APY for kombucha
            quality_multiplier: 2.5,
            fermentation_multiplier: 2.0, // Kombucha-specific
            scoby_health_multiplier: 1.5, // Kombucha-specific
        };
        
        // Setup runtime API mock to return the expected params
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_get_staking_params()
            .returning(move |_| Ok(expected_params.clone()));
        
        // Call RPC method
        let result = futures::executor::block_on(rpc.get_staking_params(None)).unwrap();
        
        // Verify result including kombucha-specific multipliers
        assert_eq!(result.min_stake, expected_params.min_stake);
        assert_eq!(result.max_stake, expected_params.max_stake);
        assert_eq!(result.unbonding_period, expected_params.unbonding_period);
        assert_eq!(result.base_apy, expected_params.base_apy);
        assert_eq!(result.quality_multiplier, expected_params.quality_multiplier);
        assert_eq!(result.fermentation_multiplier, expected_params.fermentation_multiplier);
        assert_eq!(result.scoby_health_multiplier, expected_params.scoby_health_multiplier);
    }

    #[test]
    fn test_get_staking_positions() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Mock account ID
        let account_id = 1;
        
        // Expected kombucha staking positions with fermentation and SCOBY data
        let expected_positions = vec![
            KombuchaStakingPosition {
                id: 1,
                batch_id: b"batch-1".to_vec(),
                amount: 2000,
                quality_score: 90,
                fermentation_stage: 5,  // Perfect fermentation
                scoby_health: 95,       // Excellent SCOBY health
                rewards_earned: 150,    // Higher rewards due to excellent parameters
                status: 1, // Active
                start_date: 10000,
                end_date: None,
            },
            KombuchaStakingPosition {
                id: 2,
                batch_id: b"batch-2".to_vec(),
                amount: 1500,
                quality_score: 75,
                fermentation_stage: 3,  // Middle fermentation
                scoby_health: 80,       // Good SCOBY health
                rewards_earned: 70,
                status: 2, // Unlocking
                start_date: 9000,
                end_date: Some(15000),
            },
        ];
        
        // Setup runtime API mock
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_get_staking_positions()
            .with(eq(account_id), any())
            .returning(move |_, _| Ok(expected_positions.clone()));
        
        // Call RPC method
        let result = futures::executor::block_on(rpc.get_staking_positions(account_id, None)).unwrap();
        
        // Verify result with kombucha-specific fields
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, 1);
        assert_eq!(result[0].batch_id, b"batch-1".to_vec());
        assert_eq!(result[0].amount, 2000);
        assert_eq!(result[0].fermentation_stage, 5);
        assert_eq!(result[0].scoby_health, 95);
        assert_eq!(result[1].id, 2);
        assert_eq!(result[1].fermentation_stage, 3);
        assert_eq!(result[1].scoby_health, 80);
        assert_eq!(result[1].status, 2);
    }

    #[test]
    fn test_calculate_potential_rewards() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Input parameters including kombucha-specific ones
        let amount = 2000;
        let quality_score = 90;
        let fermentation_stage = 5;  // Perfect fermentation
        let scoby_health = 95;       // Excellent SCOBY health
        let days = 30;
        
        // Expected rewards with added bonuses for fermentation and SCOBY
        let expected_rewards = PotentialRewards {
            daily_reward: 1.15,
            total_reward: 34.5, // 30 days
            apy_percentage: 21.0,  // Higher due to bonuses
            quality_bonus: 3.0,
            fermentation_bonus: 5.0, // Bonus for perfect fermentation
            scoby_health_bonus: 3.0, // Bonus for excellent SCOBY
        };
        
        // Setup runtime API mock
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_calculate_potential_rewards()
            .with(eq(amount), eq(quality_score), eq(fermentation_stage), eq(scoby_health), eq(days), any())
            .returning(move |_, _, _, _, _, _| Ok(expected_rewards.clone()));
        
        // Call RPC method with kombucha-specific parameters
        let result = futures::executor::block_on(rpc.calculate_potential_rewards(
            amount, quality_score, fermentation_stage, scoby_health, days, None
        )).unwrap();
        
        // Verify result including kombucha-specific bonuses
        assert_eq!(result.daily_reward, expected_rewards.daily_reward);
        assert_eq!(result.total_reward, expected_rewards.total_reward);
        assert_eq!(result.apy_percentage, expected_rewards.apy_percentage);
        assert_eq!(result.quality_bonus, expected_rewards.quality_bonus);
        assert_eq!(result.fermentation_bonus, expected_rewards.fermentation_bonus);
        assert_eq!(result.scoby_health_bonus, expected_rewards.scoby_health_bonus);
    }

    #[test]
    fn test_get_scoby_score() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Test batch ID
        let batch_id = b"batch-1".to_vec();
        
        // Expected SCOBY data
        let expected_scoby_data = ScobyData {
            batch_id: batch_id.clone(),
            scoby_health: 95,
            age_days: 28,
            thickness_mm: 12,
            culture_density: 85,
            ph_level: 3.2,
        };
        
        // Setup runtime API mock
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_get_scoby_score()
            .with(eq(batch_id.clone()), any())
            .returning(move |_, _| Ok(expected_scoby_data.clone()));
        
        // Call RPC method
        let result = futures::executor::block_on(rpc.get_scoby_score(batch_id.clone(), None)).unwrap();
        
        // Verify detailed SCOBY data
        assert_eq!(result.scoby_health, 95);
        assert_eq!(result.age_days, 28);
        assert_eq!(result.thickness_mm, 12);
        assert_eq!(result.culture_density, 85);
        assert_eq!(result.ph_level, 3.2);
    }

    #[test]
    fn test_calculate_rewards_with_different_fermentation_stages() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Base parameters
        let amount = 2000;
        let quality_score = 90;
        let scoby_health = 95;
        let days = 30;
        
        // Test with perfect fermentation (stage 5)
        let fermentation_stage_5 = 5;
        let expected_rewards_stage_5 = PotentialRewards {
            daily_reward: 1.15,
            total_reward: 34.5,
            apy_percentage: 21.0,
            quality_bonus: 3.0,
            fermentation_bonus: 5.0, // Maximum bonus
            scoby_health_bonus: 3.0,
        };
        
        // Setup runtime API mock for stage 5
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_calculate_potential_rewards()
            .with(eq(amount), eq(quality_score), eq(fermentation_stage_5), eq(scoby_health), eq(days), any())
            .returning(move |_, _, _, _, _, _| Ok(expected_rewards_stage_5.clone()));
        
        // Call RPC method with perfect fermentation
        let result_stage_5 = futures::executor::block_on(rpc.calculate_potential_rewards(
            amount, quality_score, fermentation_stage_5, scoby_health, days, None
        )).unwrap();
        
        // Verify result for perfect fermentation
        assert_eq!(result_stage_5.fermentation_bonus, 5.0);
        assert_eq!(result_stage_5.apy_percentage, 21.0);
        
        // Test with early fermentation (stage 1)
        let fermentation_stage_1 = 1;
        let expected_rewards_stage_1 = PotentialRewards {
            daily_reward: 0.8,
            total_reward: 24.0,
            apy_percentage: 15.0, // Lower APY
            quality_bonus: 3.0,
            fermentation_bonus: 1.0, // Minimal bonus
            scoby_health_bonus: 3.0,
        };
        
        // Setup runtime API mock for stage 1
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_calculate_potential_rewards()
            .with(eq(amount), eq(quality_score), eq(fermentation_stage_1), eq(scoby_health), eq(days), any())
            .returning(move |_, _, _, _, _, _| Ok(expected_rewards_stage_1.clone()));
        
        // Call RPC method with early fermentation
        let result_stage_1 = futures::executor::block_on(rpc.calculate_potential_rewards(
            amount, quality_score, fermentation_stage_1, scoby_health, days, None
        )).unwrap();
        
        // Verify result for early fermentation
        assert_eq!(result_stage_1.fermentation_bonus, 1.0);
        assert_eq!(result_stage_1.apy_percentage, 15.0);
    }

    #[test]
    fn test_calculate_rewards_with_different_scoby_health() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Base parameters
        let amount = 2000;
        let quality_score = 90;
        let fermentation_stage = 5;
        let days = 30;
        
        // Test with excellent SCOBY health
        let scoby_health_95 = 95;
        let expected_rewards_scoby_95 = PotentialRewards {
            daily_reward: 1.15,
            total_reward: 34.5,
            apy_percentage: 21.0,
            quality_bonus: 3.0,
            fermentation_bonus: 5.0,
            scoby_health_bonus: 3.0, // Maximum bonus
        };
        
        // Setup runtime API mock for excellent SCOBY
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_calculate_potential_rewards()
            .with(eq(amount), eq(quality_score), eq(fermentation_stage), eq(scoby_health_95), eq(days), any())
            .returning(move |_, _, _, _, _, _| Ok(expected_rewards_scoby_95.clone()));
        
        // Call RPC method with excellent SCOBY
        let result_scoby_95 = futures::executor::block_on(rpc.calculate_potential_rewards(
            amount, quality_score, fermentation_stage, scoby_health_95, days, None
        )).unwrap();
        
        // Verify result for excellent SCOBY
        assert_eq!(result_scoby_95.scoby_health_bonus, 3.0);
        
        // Test with poor SCOBY health
        let scoby_health_40 = 40;
        let expected_rewards_scoby_40 = PotentialRewards {
            daily_reward: 0.95,
            total_reward: 28.5,
            apy_percentage: 17.0, // Lower APY
            quality_bonus: 3.0,
            fermentation_bonus: 5.0,
            scoby_health_bonus: 1.0, // Minimal bonus
        };
        
        // Setup runtime API mock for poor SCOBY
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_calculate_potential_rewards()
            .with(eq(amount), eq(quality_score), eq(fermentation_stage), eq(scoby_health_40), eq(days), any())
            .returning(move |_, _, _, _, _, _| Ok(expected_rewards_scoby_40.clone()));
        
        // Call RPC method with poor SCOBY
        let result_scoby_40 = futures::executor::block_on(rpc.calculate_potential_rewards(
            amount, quality_score, fermentation_stage, scoby_health_40, days, None
        )).unwrap();
        
        // Verify result for poor SCOBY
        assert_eq!(result_scoby_40.scoby_health_bonus, 1.0);
        assert_eq!(result_scoby_40.apy_percentage, 17.0);
    }

    #[test]
    fn test_verify_quantum_proof() {
        let (mock_client, rpc) = setup_test_rpc();
        
        // Create a test keypair for proof verification
        let keypair = QuantumCrypto::generate_keypair().expect("Failed to generate keypair");
        
        // Create test data and proof with kombucha-specific fields
        let position_id = 1;
        let batch_id = b"batch-1".to_vec();
        let position_data = format!("position:{}:batch:{:?}:staker:1:amount:2000:fermentation_stage:5:scoby_health:95", 
                                    position_id, batch_id).into_bytes();
        
        // Create a valid proof
        let valid_proof = QuantumCrypto::create_eigenlayer_proof(
            position_id,
            &batch_id,
            position_data.clone(),
            &keypair.signing_private_key
        ).expect("Failed to create proof");
        
        // Setup runtime API mock for valid proof
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_verify_quantum_proof()
            .with(eq(position_id), any(), any())
            .returning(|_, _, _| Ok(true));
        
        // Call RPC method with valid proof
        let result = futures::executor::block_on(rpc.verify_quantum_proof(
            position_id, valid_proof, None
        )).unwrap();
        
        // Verify result
        assert!(result);
        
        // Create an invalid proof
        let invalid_data = b"tampered_position_data";
        let invalid_proof = QuantumCrypto::create_eigenlayer_proof(
            position_id,
            &batch_id,
            invalid_data,
            &keypair.signing_private_key
        ).expect("Failed to create proof");
        
        // Setup runtime API mock for invalid proof
        let mock_runtime_api = mock_client.runtime_api();
        mock_runtime_api.expect_verify_quantum_proof()
            .with(eq(position_id), any(), any())
            .returning(|_, _, _| Ok(false));
        
        // Call RPC method with invalid proof
        let result = futures::executor::block_on(rpc.verify_quantum_proof(
            position_id, invalid_proof, None
        )).unwrap();
        
        // Verify result
        assert!(!result);
    }
}
