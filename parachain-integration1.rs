// NRSH and ELXR Parachain Integration Module
// Connects Arduino telemetry devices to Polkadot-based parachains
// Target: Rococo testnet for initial demonstration
// Copyright © 2025 NRSH/ELXR

use clap::Parser;
use codec::{Decode, Encode};
use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult,
    ensure, traits::Get, Parameter,
};
use frame_system::{self as system, ensure_signed};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::AccountId32, H256};
use sp_runtime::{traits::Hash, RuntimeDebug};
use sp_std::{prelude::*, vec::Vec, convert::TryInto, fmt::Debug};

// Data structures for telemetry data
#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Serialize, Deserialize)]
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

#[derive(Clone, PartialEq, Eq, Encode, Decode, RuntimeDebug, TypeInfo, Serialize, Deserialize)]
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

// Pallet definitions
pub trait NrshConfig: system::Config {
    type Event: From<NrshEvent<Self>> + Into<<Self as system::Config>::Event>;
    type TelemetryId: Member + Parameter + Default + Copy + Decode + Encode + TypeInfo;
    type MaxDeviceIdLength: Get<u32>;
    type MaxBatchIdLength: Get<u32>;
    type MaxSignatureLength: Get<u32>;
}

pub trait ElxrConfig: system::Config {
    type Event: From<ElxrEvent<Self>> + Into<<Self as system::Config>::Event>;
    type TelemetryId: Member + Parameter + Default + Copy + Decode + Encode + TypeInfo;
    type MaxDeviceIdLength: Get<u32>;
    type MaxSignatureLength: Get<u32>;
}

// NRSH Pallet
decl_storage! {
    trait Store for NrshModule<T: NrshConfig> as NrshTelemetry {
        // Storage for NRSH spirulina telemetry
        pub SpirulinaTelemetry get(fn spirulina_telemetry):
            map hasher(blake2_128_concat) T::TelemetryId => Option<NrshTelemetry<T::AccountId, T::BlockNumber>>;
        
        // Mapping from device ID to latest telemetry ID
        pub DeviceLatestTelemetry get(fn device_latest_telemetry):
            map hasher(blake2_128_concat) Vec<u8> => Option<T::TelemetryId>;
        
        // Next available telemetry ID
        pub NextTelemetryId get(fn next_telemetry_id): T::TelemetryId;
        
        // Authorized devices list
        pub AuthorizedDevices get(fn authorized_devices):
            map hasher(blake2_128_concat) Vec<u8> => Option<T::AccountId>;
        
        // Optimal ranges for verification (min/max for each parameter)
        pub OptimalRanges get(fn optimal_ranges): (
            (u32, u32), // pH min/max (scaled by 100)
            (u32, u32), // temperature min/max (scaled by 100)
            (u32, u32), // light min/max (scaled by 10)
            (u32, u32), // density min/max (scaled by 1000)
            (u32, u32), // dissolved_oxygen min/max (scaled by 100)
            (u32, u32), // nitrate min/max (scaled by 10)
            (u32, u32)  // salinity min/max (scaled by 10)
        );
    }
}

// ELXR Pallet
decl_storage! {
    trait Store for ElxrModule<T: ElxrConfig> as ElxrTelemetry {
        // Storage for ELXR kombucha telemetry
        pub KombuchaTelemetry get(fn kombucha_telemetry):
            map hasher(blake2_128_concat) T::TelemetryId => Option<ElxrTelemetry<T::AccountId, T::BlockNumber>>;
        
        // Mapping from device ID to latest telemetry ID
        pub DeviceLatestTelemetry get(fn device_latest_telemetry):
            map hasher(blake2_128_concat) Vec<u8> => Option<T::TelemetryId>;
        
        // Next available telemetry ID
        pub NextTelemetryId get(fn next_telemetry_id): T::TelemetryId;
        
        // Authorized devices list
        pub AuthorizedDevices get(fn authorized_devices):
            map hasher(blake2_128_concat) Vec<u8> => Option<T::AccountId>;
        
        // Optimal ranges for verification (min/max for each parameter)
        pub OptimalRanges get(fn optimal_ranges): (
            (u32, u32), // pH min/max (scaled by 100)
            (u32, u32), // temperature min/max (scaled by 100)
            (u32, u32), // light min/max (scaled by 10)
            (u32, u32), // density min/max (scaled by 1000)
            (u32, u32), // co2 min/max (scaled by 10)
            (u32, u32)  // fermentation min/max (scaled by 1000)
        );
    }
}

// Events for NRSH Pallet
decl_event! {
    pub enum NrshEvent<T> where 
        AccountId = <T as system::Config>::AccountId,
        TelemetryId = <T as NrshConfig>::TelemetryId
    {
        /// New telemetry data recorded [device_id, telemetry_id]
        NewTelemetryRecorded(Vec<u8>, TelemetryId),
        /// Device authorization updated [device_id, account_id]
        DeviceAuthorizationUpdated(Vec<u8>, AccountId),
        /// Optimal ranges updated
        OptimalRangesUpdated,
        /// Harvest readiness detected [device_id, batch_id]
        HarvestReadinessDetected(Vec<u8>, Vec<u8>),
        /// Anomaly detected [device_id, anomaly_type]
        AnomalyDetected(Vec<u8>, Vec<u8>),
    }
}

// Events for ELXR Pallet
decl_event! {
    pub enum ElxrEvent<T> where 
        AccountId = <T as system::Config>::AccountId,
        TelemetryId = <T as ElxrConfig>::TelemetryId
    {
        /// New telemetry data recorded [device_id, telemetry_id]
        NewTelemetryRecorded(Vec<u8>, TelemetryId),
        /// Device authorization updated [device_id, account_id]
        DeviceAuthorizationUpdated(Vec<u8>, AccountId),
        /// Optimal ranges updated
        OptimalRangesUpdated,
        /// Fermentation completion detected [device_id]
        FermentationCompleted(Vec<u8>),
        /// Anomaly detected [device_id, anomaly_type]
        AnomalyDetected(Vec<u8>, Vec<u8>),
    }
}

// NRSH Pallet Implementation
decl_module! {
    pub struct NrshModule<T: NrshConfig> for enum Call where origin: T::Origin {
        // Initialize events
        fn deposit_event() = default;

        /// Submit new spirulina telemetry data
        #[weight = 10_000]
        pub fn submit_telemetry(
            origin,
            device_id: Vec<u8>,
            batch_id: Vec<u8>,
            ph: u32,
            temperature: u32,
            light: u32,
            density: u32,
            dissolved_oxygen: u32,
            nitrate: u32,
            salinity: u32,
            battery: u32,
            overall_health: u32,
            harvest_ready: bool,
            quantum_signature: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Validate device is authorized
            ensure!(
                Self::authorized_devices(&device_id) == Some(sender.clone()),
                "Device not authorized for this account"
            );
            
            // Validate data lengths
            ensure!(
                device_id.len() <= T::MaxDeviceIdLength::get() as usize,
                "Device ID too long"
            );
            ensure!(
                batch_id.len() <= T::MaxBatchIdLength::get() as usize,
                "Batch ID too long"
            );
            ensure!(
                quantum_signature.len() <= T::MaxSignatureLength::get() as usize,
                "Quantum signature too long"
            );
            
            // Verify quantum signature (simplified - would be implemented with Kyber-Dilithium)
            // In production, use actual post-quantum verification
            Self::verify_quantum_signature(&device_id, &quantum_signature)?;
            
            // Get next telemetry ID
            let telemetry_id = Self::next_telemetry_id();
            let next_id = telemetry_id.checked_add(&Default::default())
                .ok_or("Telemetry ID overflow")?;
            
            // Create telemetry record
            let telemetry = NrshTelemetry {
                device_id: device_id.clone(),
                timestamp: <frame_system::Pallet<T>>::block_number(),
                batch_id: batch_id.clone(),
                ph,
                temperature,
                light,
                density,
                dissolved_oxygen,
                nitrate,
                salinity,
                battery,
                overall_health,
                harvest_ready,
                reporter: sender,
                quantum_signature,
            };
            
            // Store telemetry data
            <SpirulinaTelemetry<T>>::insert(telemetry_id, telemetry);
            <DeviceLatestTelemetry<T>>::insert(&device_id, telemetry_id);
            <NextTelemetryId<T>>::put(next_id);
            
            // Check for anomalies and harvest readiness
            Self::check_anomalies(&device_id, ph, temperature, light, density, dissolved_oxygen, nitrate, salinity)?;
            if harvest_ready {
                Self::deposit_event(NrshEvent::HarvestReadinessDetected(device_id.clone(), batch_id));
            }
            
            // Emit event
            Self::deposit_event(NrshEvent::NewTelemetryRecorded(device_id, telemetry_id));
            
            Ok(())
        }
        
        /// Authorize a device for telemetry submission
        #[weight = 10_000]
        pub fn authorize_device(
            origin,
            device_id: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Validate device ID length
            ensure!(
                device_id.len() <= T::MaxDeviceIdLength::get() as usize,
                "Device ID too long"
            );
            
            // Register device authorization
            <AuthorizedDevices<T>>::insert(&device_id, sender.clone());
            
            // Emit event
            Self::deposit_event(NrshEvent::DeviceAuthorizationUpdated(device_id, sender));
            
            Ok(())
        }
        
        /// Update optimal ranges for spirulina cultivation
        #[weight = 10_000]
        pub fn update_optimal_ranges(
            origin,
            ph_min: u32,
            ph_max: u32,
            temp_min: u32,
            temp_max: u32,
            light_min: u32,
            light_max: u32,
            density_min: u32,
            density_max: u32,
            dissolved_oxygen_min: u32,
            dissolved_oxygen_max: u32,
            nitrate_min: u32,
            nitrate_max: u32,
            salinity_min: u32,
            salinity_max: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            // Validate ranges
            ensure!(ph_min < ph_max, "Invalid pH range");
            ensure!(temp_min < temp_max, "Invalid temperature range");
            ensure!(light_min < light_max, "Invalid light range");
            ensure!(density_min < density_max, "Invalid density range");
            ensure!(dissolved_oxygen_min < dissolved_oxygen_max, "Invalid dissolved oxygen range");
            ensure!(nitrate_min < nitrate_max, "Invalid nitrate range");
            ensure!(salinity_min < salinity_max, "Invalid salinity range");
            
            // Update optimal ranges
            <OptimalRanges<T>>::put(
                ((ph_min, ph_max),
                (temp_min, temp_max),
                (light_min, light_max),
                (density_min, density_max),
                (dissolved_oxygen_min, dissolved_oxygen_max),
                (nitrate_min, nitrate_max),
                (salinity_min, salinity_max))
            );
            
            // Emit event
            Self::deposit_event(NrshEvent::OptimalRangesUpdated);
            
            Ok(())
        }
    }
}

// ELXR Pallet Implementation
decl_module! {
    pub struct ElxrModule<T: ElxrConfig> for enum Call where origin: T::Origin {
        // Initialize events
        fn deposit_event() = default;

        /// Submit new kombucha telemetry data
        #[weight = 10_000]
        pub fn submit_telemetry(
            origin,
            device_id: Vec<u8>,
            ph: u32,
            temperature: u32,
            light: u32,
            density: u32,
            co2: u32,
            fermentation: u32,
            battery: u32,
            quantum_signature: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Validate device is authorized
            ensure!(
                Self::authorized_devices(&device_id) == Some(sender.clone()),
                "Device not authorized for this account"
            );
            
            // Validate data lengths
            ensure!(
                device_id.len() <= T::MaxDeviceIdLength::get() as usize,
                "Device ID too long"
            );
            ensure!(
                quantum_signature.len() <= T::MaxSignatureLength::get() as usize,
                "Quantum signature too long"
            );
            
            // Verify quantum signature (simplified - would be implemented with Kyber-Dilithium)
            // In production, use actual post-quantum verification
            Self::verify_quantum_signature(&device_id, &quantum_signature)?;
            
            // Get next telemetry ID
            let telemetry_id = Self::next_telemetry_id();
            let next_id = telemetry_id.checked_add(&Default::default())
                .ok_or("Telemetry ID overflow")?;
            
            // Create telemetry record
            let telemetry = ElxrTelemetry {
                device_id: device_id.clone(),
                timestamp: <frame_system::Pallet<T>>::block_number(),
                ph,
                temperature,
                light,
                density,
                co2,
                fermentation,
                battery,
                reporter: sender,
                quantum_signature,
            };
            
            // Store telemetry data
            <KombuchaTelemetry<T>>::insert(telemetry_id, telemetry);
            <DeviceLatestTelemetry<T>>::insert(&device_id, telemetry_id);
            <NextTelemetryId<T>>::put(next_id);
            
            // Check for anomalies and fermentation completion
            Self::check_anomalies(&device_id, ph, temperature, light, density, co2, fermentation)?;
            if fermentation >= 800 { // 80% fermentation completion (scaled by 1000)
                Self::deposit_event(ElxrEvent::FermentationCompleted(device_id.clone()));
            }
            
            // Emit event
            Self::deposit_event(ElxrEvent::NewTelemetryRecorded(device_id, telemetry_id));
            
            Ok(())
        }
        
        /// Authorize a device for telemetry submission
        #[weight = 10_000]
        pub fn authorize_device(
            origin,
            device_id: Vec<u8>,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            
            // Validate device ID length
            ensure!(
                device_id.len() <= T::MaxDeviceIdLength::get() as usize,
                "Device ID too long"
            );
            
            // Register device authorization
            <AuthorizedDevices<T>>::insert(&device_id, sender.clone());
            
            // Emit event
            Self::deposit_event(ElxrEvent::DeviceAuthorizationUpdated(device_id, sender));
            
            Ok(())
        }
        
        /// Update optimal ranges for kombucha fermentation
        #[weight = 10_000]
        pub fn update_optimal_ranges(
            origin,
            ph_min: u32,
            ph_max: u32,
            temp_min: u32,
            temp_max: u32,
            light_min: u32,
            light_max: u32,
            density_min: u32,
            density_max: u32,
            co2_min: u32,
            co2_max: u32,
            fermentation_min: u32,
            fermentation_max: u32,
        ) -> DispatchResult {
            ensure_signed(origin)?;
            
            // Validate ranges
            ensure!(ph_min < ph_max, "Invalid pH range");
            ensure!(temp_min < temp_max, "Invalid temperature range");
            ensure!(light_min < light_max, "Invalid light range");
            ensure!(density_min < density_max, "Invalid density range");
            ensure!(co2_min < co2_max, "Invalid CO2 range");
            ensure!(fermentation_min < fermentation_max, "Invalid fermentation range");
            
            // Update optimal ranges
            <OptimalRanges<T>>::put(
                ((ph_min, ph_max),
                (temp_min, temp_max),
                (light_min, light_max),
                (density_min, density_max),
                (co2_min, co2_max),
                (fermentation_min, fermentation_max))
            );
            
            // Emit event
            Self::deposit_event(ElxrEvent::OptimalRangesUpdated);
            
            Ok(())
        }
    }
}

// Implementation for NRSH Pallet
impl<T: NrshConfig> NrshModule<T> {
    // Verify quantum signature (simplified)
    fn verify_quantum_signature(device_id: &[u8], signature: &[u8]) -> DispatchResult {
        // In production implementation, this would use actual Kyber-Dilithium verification
        // For now, we accept all signatures with at least 64 bytes for demo purposes
        ensure!(signature.len() >= 64, "Invalid quantum signature length");
        Ok(())
    }
    
    // Check for anomalies in telemetry data
    fn check_anomalies(
        device_id: &[u8],
        ph: u32,
        temperature: u32,
        light: u32,
        density: u32,
        dissolved_oxygen: u32,
        nitrate: u32,
        salinity: u32,
    ) -> DispatchResult {
        let ranges = Self::optimal_ranges();
        
        // Check each parameter against optimal ranges
        if ph < ranges.0.0 || ph > ranges.0.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"pH_out_of_range".to_vec()));
        }
        
        if temperature < ranges.1.0 || temperature > ranges.1.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"temperature_out_of_range".to_vec()));
        }
        
        if light < ranges.2.0 || light > ranges.2.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"light_out_of_range".to_vec()));
        }
        
        if density < ranges.3.0 || density > ranges.3.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"density_out_of_range".to_vec()));
        }
        
        if dissolved_oxygen < ranges.4.0 || dissolved_oxygen > ranges.4.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"dissolved_oxygen_out_of_range".to_vec()));
        }
        
        if nitrate < ranges.5.0 || nitrate > ranges.5.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"nitrate_out_of_range".to_vec()));
        }
        
        if salinity < ranges.6.0 || salinity > ranges.6.1 {
            Self::deposit_event(NrshEvent::AnomalyDetected(device_id.to_vec(), b"salinity_out_of_range".to_vec()));
        }
        
        Ok(())
    }
}

// Implementation for ELXR Pallet
impl<T: ElxrConfig> ElxrModule<T> {
    // Verify quantum signature (simplified)
    fn verify_quantum_signature(device_id: &[u8], signature: &[u8]) -> DispatchResult {
        // In production implementation, this would use actual Kyber-Dilithium verification
        // For now, we accept all signatures with at least 64 bytes for demo purposes
        ensure!(signature.len() >= 64, "Invalid quantum signature length");
        Ok(())
    }
    
    // Check for anomalies in telemetry data
    fn check_anomalies(
        device_id: &[u8],
        ph: u32,
        temperature: u32,
        light: u32,
        density: u32,
        co2: u32,
        fermentation: u32,
    ) -> DispatchResult {
        let ranges = Self::optimal_ranges();
        
        // Check each parameter against optimal ranges
        if ph < ranges.0.0 || ph > ranges.0.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"pH_out_of_range".to_vec()));
        }
        
        if temperature < ranges.1.0 || temperature > ranges.1.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"temperature_out_of_range".to_vec()));
        }
        
        if light < ranges.2.0 || light > ranges.2.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"light_out_of_range".to_vec()));
        }
        
        if density < ranges.3.0 || density > ranges.3.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"density_out_of_range".to_vec()));
        }
        
        if co2 < ranges.4.0 || co2 > ranges.4.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"co2_out_of_range".to_vec()));
        }
        
        if fermentation < ranges.5.0 || fermentation > ranges.5.1 {
            Self::deposit_event(ElxrEvent::AnomalyDetected(device_id.to_vec(), b"fermentation_out_of_range".to_vec()));
        }
        
        Ok(())
    }
}

// Command-line Rococo testnet integration for demonstration
// This would be a separate binary for interacting with the parachain
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct TestnetConnector {
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

// Note: In a real implementation, this binary would:
// 1. Connect to the specified serial port to read Arduino data
// 2. Parse the JSON telemetry data
// 3. Generate Kyber-Dilithium signatures
// 4. Connect to the Rococo testnet using the Substrate client library
// 5. Submit extrinsics to the respective pallet
// 6. Monitor events from the parachain
