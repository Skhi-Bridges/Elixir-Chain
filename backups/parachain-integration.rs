// NRSH and ELXR Parachain Integration Module
// Connects Arduino telemetry devices to Polkadot-based parachains
// Target: Rococo testnet for initial demonstration
// Copyright © 2025 NRSH/ELXR

use clap::Parser;
use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    ensure, traits::Get,
};
use frame_system::{self as system, ensure_signed};
use serde::{Deserialize, Serialize};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{traits::Hash, RuntimeDebug};
use sp_std::prelude::*;

// Command-line interface for integration testing
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Rococo testnet endpoint
    #[clap(short, long, default_value = "wss://rococo-rpc.polkadot.io")]
    endpoint: String,

    /// Serial port for Arduino connection
    #[clap(short, long)]
    serial_port: Option<String>,

    /// Use simulated data instead of real device
    #[clap(short, long)]
    simulate: bool,

    /// Project selection (nrsh or elxr)
    #[clap(short, long, default_value = "nrsh")]
    project: String,
}

// Data structures for telemetry data
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, Serialize, Deserialize)]
pub struct NrshTelemetry<AccountId, Moment> {
    pub device_id: Vec<u8>,
    pub timestamp: Moment,
    pub batch_id: Vec<u8>,
    pub ph: u32,           // scaled by 100
    pub temperature: u32,  // scaled by 100
    pub light: u32,        // scaled by 10
    pub density: u32,      // scaled by 1000
    pub dissolved_oxygen: u32, // scaled by 100
    pub nitrate: u32,      // scaled by 10
    pub salinity: u32,     // scaled by 10
    pub battery: u32,      // scaled by 10
    pub overall_health: u32, // scaled by 10
    pub harvest_ready: bool,
    pub reporter: AccountId,
    pub quantum_signature: Vec<u8>,
}

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, Serialize, Deserialize)]
pub struct ElxrTelemetry<AccountId, Moment> {
    pub device_id: Vec<u8>,
    pub timestamp: Moment,
    pub ph: u32,           // scaled by 100
    pub temperature: u32,  // scaled by 100
    pub light: u32,        // scaled by 10
    pub density: u32,      // scaled by 1000
    pub co2: u32,          // scaled by 10
    pub fermentation: u32, // scaled by 1000
    pub battery: u32,      // scaled by 10
    pub reporter: AccountId,
    pub quantum_signature: Vec<u8>,
}

// Pallet definition for NRSH telemetry
pub trait Config: system::Config {
    type Event: From<Event<Self>> + Into<<Self as system::Config>::Event>;
    type TelemetryId: Member + Parameter + Default + Copy;
    type MaxDeviceIdLength: Get<u32>;
    type MaxBatchIdLength: Get<u32>;
    type MaxSignatureLength: Get<u32>;
}

decl_storage! {
    trait Store for Module<T: Config> as NrshTelemetry {
        // Storage for NRSH spirulina telemetry
        pub SpirulinaTelemetry