//! Common types for Eigenlayer integration
use eigensdk::eigen_types::{
    Quorum, QuorumHeaderHash, QuorumNumbers, OperatorId, OperatorStateRetriever,
    RegistryCoordinatorClient, StakerStateRetriever, ZERO_ADDRESS,
};
use eigensdk::eigen_crypto_bls::{BlsKeyPair, PublicKey, SecretKey};
use alloy_primitives::{Address, U256};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;
use std::collections::HashMap;

/// Identifies an AVS (Actively Validated Service) in the Eigenlayer ecosystem
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct AVSIdentifier {
    /// The Ethereum address of the service registry
    pub registry_address: Vec<u8>,
    /// Unique service identifier
    pub service_id: u32,
    /// Human-readable name of the service
    pub name: String,
}

/// Information about operator restaking and participation
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct RestakeInfo {
    /// Operator Ethereum address
    pub operator_address: Vec<u8>,
    /// Total ETH restaked (in wei)
    pub restaked_amount: u128,
    /// BLS Public Key of the operator
    pub public_key: Vec<u8>,
    /// Quorums the operator is participating in
    pub quorum_ids: Vec<u8>,
    /// Operator status (active, pending, etc.)
    pub status: OperatorStatus,
}

/// The status of an operator in the Eigenlayer system
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub enum OperatorStatus {
    /// Operator is active and can participate in consensus
    Active,
    /// Operator is registered but not yet active
    Pending,
    /// Operator is temporarily disabled
    Paused,
    /// Operator has been removed/slashed
    Removed,
}

/// Information about a quorum in Eigenlayer
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct QuorumInfo {
    /// ID of the quorum
    pub quorum_id: u8,
    /// Number of operators in the quorum
    pub operator_count: u32, 
    /// Total amount staked in this quorum (in wei)
    pub total_stake: u128,
    /// Minimum stake required to join this quorum (in wei)
    pub min_stake: u128,
}

/// Represents a stake amount for a specific token
#[derive(Clone, Debug, Serialize, Deserialize, Encode, Decode, TypeInfo, PartialEq, Eq)]
pub struct StakeAmount {
    /// Ethereum token address (zero address for ETH)
    pub token_address: Vec<u8>,
    /// Amount staked in smallest token unit
    pub amount: u128,
}

/// Converts an Ethereum address to a byte vector
pub fn address_to_bytes(address: &Address) -> Vec<u8> {
    address.to_vec()
}

/// Converts a byte vector to an Ethereum address
pub fn bytes_to_address(bytes: &[u8]) -> Option<Address> {
    if bytes.len() != 20 {
        return None;
    }
    let mut address_bytes = [0u8; 20];
    address_bytes.copy_from_slice(bytes);
    Some(Address::from(address_bytes))
}

/// Adds domain-specific functionality to convert between library types and our types
pub trait EigenlayerConversions {
    /// Convert from Eigenlayer library types to our local types
    fn to_local(&self) -> Self;
    
    /// Convert to Eigenlayer library types from our local types
    fn to_eigen(&self) -> Self;
}

impl EigenlayerConversions for RestakeInfo {
    fn to_local(&self) -> Self {
        // Implementation would convert from library types
        self.clone()
    }
    
    fn to_eigen(&self) -> Self {
        // Implementation would convert to library types
        self.clone()
    }
}
