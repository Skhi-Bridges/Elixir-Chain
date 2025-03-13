//! # Eigenlayer Integration for ELXR
//! 
//! This module implements integration with Eigenlayer for the Elixir Chain,
//! enabling validator restaking and middleware security.
//! 
//! It leverages the official eigensdk-rs library to interact with Eigenlayer contracts.

mod client;
mod config;
mod operator;
mod service;
mod types;

pub use client::EigenlayerClient;
pub use config::EigenConfig;
pub use operator::{OperatorInfo, OperatorManager};
pub use service::EigenlayerService;
pub use types::{RestakeInfo, QuorumInfo, StakeAmount, AVSIdentifier};
