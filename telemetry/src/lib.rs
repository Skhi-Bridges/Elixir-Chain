//! ELXR Kombucha Telemetry System
//! 
//! Provides quantum-secured telemetry for the ELXR (Elixir) chain,
//! focused on kombucha fermentation monitoring and blockchain integration.
//! Implements comprehensive error correction at classical, bridge, and quantum levels.
//!
//! Copyright 2025 Matrix-Magiq ELXR Chain

#![cfg_attr(not(test), warn(clippy::unwrap_used))]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Sensor implementations for telemetry data collection
pub mod sensor {
    /// Core sensor traits and types
    pub mod core;
    
    /// Alcohol sensor for measuring kombucha fermentation progress
    pub mod alcohol;
    
    /// Turbidity sensor for monitoring kombucha clarity
    pub mod turbidity;
    
    /// Acetic acid sensor for measuring vinegar conversion
    pub mod acetic_acid;
    
    /// pH sensor for monitoring kombucha acidity
    pub mod ph;
    
    /// Temperature sensor for monitoring fermentation environment
    pub mod temperature;
    
    /// Light sensor for monitoring ambient conditions
    pub mod light;
    
    /// CO2 sensor for monitoring fermentation activity
    pub mod co2;
}

/// Hardware interface implementations
pub mod hardware {
    /// Security management for hardware components
    pub mod security;
    
    /// Hardware abstraction layer
    pub mod hal;
    
    /// Sensor calibration utilities
    pub mod calibration;
}

/// Cryptographic components for quantum security
pub mod crypto;

/// Error types and handling
pub mod error;

/// Time management with quantum security
pub mod time;

/// ActorX framework integration
pub mod actorx_integration;

/// Integrated monitoring systems
pub mod monitoring;

// Re-export commonly used components
pub use sensor::core::{QuantumSecuredSensor, TelemetryReading};
pub use error::TelemetryResult;
pub use crypto::QuantumCrypto;
pub use time::QuantumSecureClock;
pub use std::sync::Arc;
pub use tokio::sync::RwLock;
pub use monitoring::{KombuchaMonitor, MonitoringThresholds, AlertSeverity};

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Create a new AlcoholSensor with default configuration
/// 
/// This is a convenience function for quickly setting up a new alcohol sensor.
pub async fn create_alcohol_sensor() -> TelemetryResult<Arc<sensor::alcohol::AlcoholSensor>> {
    let security_manager = Arc::new(RwLock::new(hardware::security::HardwareSecurityManager::new()?));
    let quantum_crypto = Arc::new(crypto::QuantumCrypto::new()?);
    let quantum_clock = Arc::new(RwLock::new(time::QuantumSecureClock::new(quantum_crypto.clone())?));
    
    // Create sensor connection info
    let connection_info = sensor::core::SensorConnectionInfo {
        address: "0xA0:20:D2:1C:FF:23".to_string(),
        port: 8033,
        protocol: "esp32s3-secure".to_string(),
        sensor_type: "alcohol".to_string(),
    };
    
    // Create sensor hardware
    let hardware = sensor::core::SensorHardware::new(
        "ELXR-ALC-001".to_string(),
        "ESP32S3-ALCOHOL-SENSOR".to_string(),
        connection_info,
    );
    
    // Create and initialize the alcohol sensor
    let sensor = Arc::new(sensor::alcohol::AlcoholSensor::new(
        hardware,
        security_manager,
        quantum_clock,
    ).await?);
    
    Ok(sensor)
}

/// Create a new TurbiditySensor with default configuration
/// 
/// This is a convenience function for quickly setting up a new turbidity sensor.
pub async fn create_turbidity_sensor() -> TelemetryResult<Arc<sensor::turbidity::TurbiditySensor>> {
    let security_manager = Arc::new(RwLock::new(hardware::security::HardwareSecurityManager::new()?));
    let quantum_crypto = Arc::new(crypto::QuantumCrypto::new()?);
    let quantum_clock = Arc::new(RwLock::new(time::QuantumSecureClock::new(quantum_crypto.clone())?));
    
    // Create sensor connection info
    let connection_info = sensor::core::SensorConnectionInfo {
        address: "0xA0:20:D2:1C:FF:24".to_string(),
        port: 8034,
        protocol: "esp32s3-secure".to_string(),
        sensor_type: "turbidity".to_string(),
    };
    
    // Create sensor hardware
    let hardware = sensor::core::SensorHardware::new(
        "ELXR-TRB-001".to_string(),
        "ESP32S3-TURBIDITY-SENSOR".to_string(),
        connection_info,
    );
    
    // Create and initialize the turbidity sensor
    let sensor = Arc::new(sensor::turbidity::TurbiditySensor::new(
        hardware,
        security_manager,
        quantum_clock,
    ).await?);
    
    Ok(sensor)
}

/// Create a new ElxrSensorActorFactory with default configuration
///
/// This is a convenience function for quickly setting up the actor factory.
pub async fn create_sensor_actor_factory() -> TelemetryResult<actorx_integration::ElxrSensorActorFactory> {
    actorx_integration::ElxrSensorActorFactory::new().await
}

/// Create a new KombuchaMonitor with default configuration
///
/// This is a convenience function for quickly setting up a complete
/// kombucha monitoring system with all sensors and ActorX integration.
pub async fn create_kombucha_monitor() -> TelemetryResult<monitoring::KombuchaMonitor> {
    monitoring::create_default_monitor().await
}
