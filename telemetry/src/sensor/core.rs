//! Core sensor traits and types
//!
//! Defines the core interfaces and data structures for quantum-secured sensors
//! in the ELXR chain telemetry system.

use crate::error::TelemetryResult;
use crate::crypto::QuantumCrypto;
use crate::time::QuantumSecureClock;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, SystemTime};
use serde::{Serialize, Deserialize};

/// Connection information for a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorConnectionInfo {
    /// Physical or logical address of the sensor
    pub address: String,
    
    /// Port number for communication
    pub port: u16,
    
    /// Protocol used for communication
    pub protocol: String,
    
    /// Type of sensor
    pub sensor_type: String,
}

/// Hardware information for a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHardware {
    /// Unique identifier for the sensor
    pub id: String,
    
    /// Model name of the sensor
    pub model: String,
    
    /// Connection information
    pub connection: SensorConnectionInfo,
    
    /// Firmware version
    pub firmware_version: Option<String>,
    
    /// Hardware version
    pub hardware_version: Option<String>,
}

impl SensorHardware {
    /// Create a new SensorHardware instance
    pub fn new(id: String, model: String, connection: SensorConnectionInfo) -> Self {
        Self {
            id,
            model,
            connection,
            firmware_version: None,
            hardware_version: None,
        }
    }
    
    /// Set firmware version
    pub fn with_firmware_version(mut self, version: String) -> Self {
        self.firmware_version = Some(version);
        self
    }
    
    /// Set hardware version
    pub fn with_hardware_version(mut self, version: String) -> Self {
        self.hardware_version = Some(version);
        self
    }
}

/// A telemetry reading with quantum security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryReading<T> {
    /// The actual sensor reading
    pub value: T,
    
    /// Timestamp when the reading was taken (nanoseconds since epoch)
    pub timestamp_ns: u128,
    
    /// Quantum signature of the reading
    pub quantum_signature: Vec<u8>,
    
    /// Reading sequence number
    pub sequence: u64,
    
    /// Hardware information
    pub hardware: SensorHardware,
    
    /// Error margin of the reading
    pub error_margin: Option<f64>,
}

/// A forecast for future readings with quantum security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryForecast<T> {
    /// The forecasted readings
    pub forecasted_values: Vec<(u128, T)>,
    
    /// Baseline reading used for forecast
    pub baseline_reading: TelemetryReading<T>,
    
    /// Confidence level (0.0 to 1.0)
    pub confidence: f64,
    
    /// Quantum signature of the forecast
    pub quantum_signature: Vec<u8>,
    
    /// Algorithm used for forecasting
    pub algorithm: String,
}

/// A history of telemetry readings with time series information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryHistory<T> {
    /// The historical readings
    pub readings: Vec<TelemetryReading<T>>,
    
    /// Time series analysis results
    pub time_series_analysis: Option<TimeSeriesAnalysis>,
    
    /// Quantum signature of the history
    pub quantum_signature: Vec<u8>,
}

/// Time series analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesAnalysis {
    /// Trend direction (positive, negative, or neutral)
    pub trend: TrendDirection,
    
    /// Seasonality information
    pub seasonality: Option<Seasonality>,
    
    /// Autocorrelation factor (-1.0 to 1.0)
    pub autocorrelation: f64,
    
    /// Variance
    pub variance: f64,
    
    /// Analysis algorithm used
    pub algorithm: String,
}

/// Trend direction for time series analysis
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// Upward trend
    Positive,
    
    /// Downward trend
    Negative,
    
    /// No clear trend
    Neutral,
}

/// Seasonality information for time series analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Seasonality {
    /// Period length in nanoseconds
    pub period_ns: u128,
    
    /// Strength of seasonality (0.0 to 1.0)
    pub strength: f64,
}

/// Trait for quantum-secured sensors
#[async_trait::async_trait]
pub trait QuantumSecuredSensor {
    /// The type of reading produced by this sensor
    type ReadingType: Clone + Send + Sync + 'static;
    
    /// Take a reading from the sensor
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>>;
    
    /// Get the hardware information for this sensor
    fn hardware(&self) -> &SensorHardware;
    
    /// Check if the sensor is quantum-secured
    async fn is_quantum_secured(&self) -> TelemetryResult<bool>;
    
    /// Get a forecast of future readings
    async fn forecast(&self, hours: u32) -> TelemetryResult<TelemetryForecast<Self::ReadingType>>;
    
    /// Get historical readings
    async fn history(&self, count: usize) -> TelemetryResult<TelemetryHistory<Self::ReadingType>>;
    
    /// Verify that a reading is authentic using quantum verification
    async fn verify_reading(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<bool>;
    
    /// Commit a reading to the blockchain
    async fn commit_to_blockchain(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<String>;
}

/// Base implementation for sensors with common functionality
pub struct BaseSensor<T> {
    /// Hardware information
    pub hardware: SensorHardware,
    
    /// Security manager for hardware security
    pub security_manager: Arc<RwLock<crate::hardware::security::HardwareSecurityManager>>,
    
    /// Quantum-secured clock for timestamping
    pub quantum_clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Phantom data for the reading type
    pub _phantom: std::marker::PhantomData<T>,
}

impl<T> BaseSensor<T> {
    /// Create a new BaseSensor
    pub fn new(
        hardware: SensorHardware, 
        security_manager: Arc<RwLock<crate::hardware::security::HardwareSecurityManager>>,
        quantum_clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> Self {
        Self {
            hardware,
            security_manager,
            quantum_clock,
            _phantom: std::marker::PhantomData,
        }
    }
    
    /// Get current timestamp from quantum-secured clock
    pub async fn quantum_timestamp(&self) -> TelemetryResult<u128> {
        let clock = self.quantum_clock.read().await;
        Ok(clock.quantum_secured_timestamp_ns())
    }
    
    /// Check if security is compromised
    pub async fn is_security_compromised(&self) -> TelemetryResult<bool> {
        let security = self.security_manager.read().await;
        security.is_security_compromised().await
    }
    
    /// Create a quantum signature for data
    pub async fn create_quantum_signature(&self, data: &[u8]) -> TelemetryResult<Vec<u8>> {
        // In a real implementation, this would use the QuantumCrypto to create a signature
        // For this sketch, return dummy signature
        let clock = self.quantum_clock.read().await;
        let timestamp = clock.quantum_secured_timestamp_ns();
        
        // Create a dummy signature based on time and data
        let mut signature = Vec::with_capacity(64);
        for i in 0..64 {
            let byte = ((timestamp >> (i % 16)) & 0xFF) as u8;
            signature.push(byte);
        }
        
        Ok(signature)
    }
}
