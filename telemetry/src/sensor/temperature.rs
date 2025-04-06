//! Temperature Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured temperature readings with
//! full error correction at classical, bridge, and quantum levels.

use crate::sensor::core::{
    QuantumSecuredSensor, TelemetryReading, ErrorCorrectionData,
    SensorHardware, SensorConnectionInfo, TimeSeriesData,
};
use crate::hardware::security::HardwareSecurityManager;
use crate::crypto::{QuantumSecuredData, QuantumKeyPair, blake3_hash};
use crate::error::{ClassicalError, BridgeError, QuantumError, TelemetryResult};
use crate::time::QuantumSecureClock;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Temperature data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct TemperatureData {
    /// Temperature in Celsius
    pub temperature: f32,
    
    /// Fermentation rate multiplier (relative to optimal temp)
    pub fermentation_rate_factor: f32,
    
    /// SCOBY health impact (-1.0 to 1.0)
    pub scoby_health_impact: f32,
    
    /// Flavor development impact (-1.0 to 1.0)
    pub flavor_impact: f32,
}

/// Quantum-secured temperature sensor for kombucha fermentation
pub struct TemperatureSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<TemperatureData>>>>,
    
    /// Optimal temperature range for kombucha (°C)
    optimal_range: (f32, f32),
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Temperature calibration offset
    calibration_offset: f32,
}

impl TemperatureSensor {
    /// Create a new temperature sensor
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Default optimal temperature range for kombucha (22-27°C)
        let optimal_range = (22.0, 27.0);
        
        // Default calibration offset
        let calibration_offset = 0.0;
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize temperature sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            optimal_range,
            security_manager,
            clock,
            calibration_offset,
        })
    }
    
    /// Set temperature calibration offset
    pub async fn set_calibration_offset(&mut self, offset: f32) -> TelemetryResult<()> {
        if offset.abs() > 5.0 {
            return Err(ClassicalError::InvalidParameter(
                "Calibration offset too large, maximum ±5.0°C allowed".to_string()
            ).into());
        }
        
        self.calibration_offset = offset;
        Ok(())
    }
    
    /// Read raw temperature from sensor
    async fn read_raw_temperature(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take temperature reading: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would read from a temperature sensor
        // such as DS18B20, BME280, or similar through appropriate drivers
        
        // For this sketch, we'll simulate temperature readings
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Base temperature for kombucha fermentation (24.5°C)
        let base_temp = 24.5;
        
        // Add random variation (±0.8°C)
        let variation = 0.8;
        let temp_with_noise = base_temp + (rng.gen::<f32>() * 2.0 - 1.0) * variation;
        
        // Apply calibration offset
        let calibrated_temp = temp_with_noise + self.calibration_offset;
        
        Ok(calibrated_temp)
    }
    
    /// Calculate fermentation rate factor based on temperature
    fn calculate_fermentation_rate_factor(&self, temperature: f32) -> f32 {
        let (min_optimal, max_optimal) = self.optimal_range;
        let mid_optimal = (min_optimal + max_optimal) / 2.0;
        
        // Q10 temperature coefficient (rate ~ doubles every 10°C)
        // Using Q10=2 as a common biological approximation
        let q10 = 2.0;
        
        if temperature < 10.0 {
            // Below 10°C: very slow fermentation, nearly dormant
            return 0.1 * (temperature / 10.0);
        } else if temperature < min_optimal {
            // Between 10°C and min_optimal: slow fermentation
            // Rate increases exponentially as temp approaches optimal
            let delta_t = (temperature - 10.0) / (min_optimal - 10.0);
            return 0.1 + 0.6 * delta_t.powf(1.5); // Non-linear increase
        } else if temperature <= max_optimal {
            // Optimal range: best fermentation rate
            // Peak at mid_optimal, slight decrease at edges of optimal range
            let distance_from_mid = (temperature - mid_optimal).abs() / (max_optimal - min_optimal) * 2.0;
            return 0.9 + 0.1 * (1.0 - distance_from_mid.powf(2.0));
        } else if temperature <= 35.0 {
            // Above optimal but below harmful: increased rate but less optimal
            let delta_t = (temperature - max_optimal) / (35.0 - max_optimal);
            return 0.7 * (1.0 - delta_t) + 0.3;
        } else {
            // Above 35°C: rapid decline in fermentation quality
            // SCOBY stressed, yeast may die off
            return 0.3 * ((40.0 - temperature) / 5.0).max(0.0);
        }
    }
    
    /// Calculate SCOBY health impact based on temperature
    fn calculate_scoby_health_impact(&self, temperature: f32) -> f32 {
        let (min_optimal, max_optimal) = self.optimal_range;
        
        if temperature < 10.0 {
            // Below 10°C: dormant but not harmed if temporary
            return -0.2 - 0.3 * (10.0 - temperature) / 10.0;
        } else if temperature < min_optimal {
            // Below optimal but above 10°C: slightly suboptimal
            return -0.2 * (min_optimal - temperature) / (min_optimal - 10.0);
        } else if temperature <= max_optimal {
            // Optimal range: healthy
            return 0.5 * ((temperature - min_optimal) / (max_optimal - min_optimal)).min(1.0);
        } else if temperature <= 32.0 {
            // Above optimal but below harmful: slightly negative
            return 0.3 - 0.4 * ((temperature - max_optimal) / (32.0 - max_optimal));
        } else if temperature <= 38.0 {
            // Potentially harmful: negative impact
            return -0.1 - 0.5 * ((temperature - 32.0) / 6.0);
        } else {
            // Severely harmful: strong negative impact
            return -0.6 - 0.4 * ((temperature - 38.0) / 5.0).min(1.0);
        }
    }
    
    /// Calculate flavor impact based on temperature
    fn calculate_flavor_impact(&self, temperature: f32) -> f32 {
        let (min_optimal, max_optimal) = self.optimal_range;
        
        if temperature < 15.0 {
            // Very cold: minimal flavor development, potentially flat
            return -0.5 + 0.3 * (temperature / 15.0);
        } else if temperature < min_optimal {
            // Cool fermentation: slower flavor development
            // Can produce smoother, less acidic flavors
            let t_factor = (temperature - 15.0) / (min_optimal - 15.0);
            return -0.2 + 0.5 * t_factor;
        } else if temperature <= 25.0 {
            // Lower optimal range: balanced flavor development
            let t_factor = (temperature - min_optimal) / (25.0 - min_optimal);
            return 0.3 + 0.4 * t_factor;
        } else if temperature <= max_optimal {
            // Upper optimal range: more pronounced acidity
            let t_factor = (temperature - 25.0) / (max_optimal - 25.0);
            return 0.7 + 0.2 * t_factor;
        } else if temperature <= 30.0 {
            // Above optimal: stronger acidity, potential off-flavors
            return 0.9 - 0.5 * ((temperature - max_optimal) / (30.0 - max_optimal));
        } else if temperature <= 35.0 {
            // Too warm: increased risk of off-flavors
            return 0.4 - 0.6 * ((temperature - 30.0) / 5.0);
        } else {
            // Hot: high risk of spoilage and bad flavors
            return -0.2 - 0.7 * ((temperature - 35.0) / 5.0).min(1.0);
        }
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for TemperatureSensor {
    type ReadingType = TemperatureData;
    
    /// Take a quantum-secured temperature reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take temperature reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take the raw temperature measurement
        let temperature = self.read_raw_temperature().await?;
        
        // Calculate derived metrics
        let fermentation_rate_factor = self.calculate_fermentation_rate_factor(temperature);
        let scoby_health_impact = self.calculate_scoby_health_impact(temperature);
        let flavor_impact = self.calculate_flavor_impact(temperature);
        
        // Create the reading data
        let reading_data = TemperatureData {
            temperature,
            fermentation_rate_factor,
            scoby_health_impact,
            flavor_impact,
        };
        
        // Generate a unique ID for this reading
        let reading_id = Uuid::new_v4().to_string();
        
        // Create quantum-secured data package
        let secure_data = QuantumSecuredData::new(
            reading_data.clone(),
            self.key_pair.read().await.get_public_key()?,
        )?;
        
        // Apply quantum error correction to the data
        let error_correction = ErrorCorrectionData {
            classical_ecc: reed_solomon_generate(&secure_data.to_bytes()?),
            bridge_ecc: secure_data.generate_bridge_ecc()?,
            quantum_ecc: secure_data.generate_quantum_ecc()?,
        };
        
        // Create the telemetry reading
        let reading = TelemetryReading {
            id: reading_id,
            sensor_id: self.hardware.id.clone(),
            sensor_type: "temperature".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}",
                reading_id,
                secure_time,
                temperature,
                fermentation_rate_factor
            ))?,
        };
        
        // Add to readings cache
        self.readings_cache.write().await.push(reading.clone());
        
        // Trim cache to last 100 readings if needed
        let mut cache = self.readings_cache.write().await;
        if cache.len() > 100 {
            *cache = cache.split_off(cache.len() - 100);
        }
        
        Ok(reading)
    }
    
    async fn verify_reading(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<bool> {
        // Verify reading integrity using quantum-secure methods
        
        // First, verify the timestamp is valid
        let timestamp_valid = self.clock.read().await.verify_timestamp(&reading.timestamp).await?;
        if !timestamp_valid {
            return Ok(false);
        }
        
        // Verify data integrity using quantum-resistant signature
        let data_valid = reading.data.verify()?;
        if !data_valid {
            return Ok(false);
        }
        
        // Verify hash integrity
        let calculated_hash = blake3_hash(&format!(
            "{}:{}:{}:{}",
            reading.id,
            reading.timestamp,
            reading.data.get_data().temperature,
            reading.data.get_data().fermentation_rate_factor
        ))?;
        
        if calculated_hash != reading.verification_hash {
            return Ok(false);
        }
        
        // Verify Reed-Solomon error correction codes
        let rs_valid = reed_solomon_verify(
            &reading.data.to_bytes()?, 
            &reading.error_correction.classical_ecc
        );
        
        if !rs_valid {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    async fn commit_reading(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<String> {
        // Verify reading before committing
        let verified = self.verify_reading(reading).await?;
        if !verified {
            return Err(ClassicalError::VerificationFailed(
                "Temperature reading failed verification".to_string()
            ).into());
        }
        
        // In a real implementation, this would commit the reading to storage
        // or a blockchain using ActorX integration
        
        // For now, just return a simulated transaction hash
        let tx_hash = blake3_hash(&format!(
            "commit:{}:{}:{}",
            reading.id,
            reading.timestamp,
            Utc::now()
        ))?;
        
        Ok(tx_hash)
    }
    
    async fn apply_error_correction(
        &self, 
        corrupted_reading: &TelemetryReading<Self::ReadingType>
    ) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Apply multi-level error correction
        
        // First, attempt classical error correction (Reed-Solomon)
        let corrected_data = reed_solomon_correct(
            &corrupted_reading.data.to_bytes()?,
            &corrupted_reading.error_correction.classical_ecc,
        )?;
        
        // Reconstruct reading with corrected data
        let mut corrected_reading = corrupted_reading.clone();
        corrected_reading.data = QuantumSecuredData::from_bytes(&corrected_data)?;
        
        // Apply bridge-level error correction if needed
        if !self.verify_reading(&corrected_reading).await? {
            corrected_reading.data.apply_bridge_ecc(&corrupted_reading.error_correction.bridge_ecc)?;
        }
        
        // Apply quantum-level error correction if needed
        if !self.verify_reading(&corrected_reading).await? {
            corrected_reading.data.apply_quantum_ecc(&corrupted_reading.error_correction.quantum_ecc)?;
        }
        
        Ok(corrected_reading)
    }
    
    async fn latest_reading(&self) -> TelemetryResult<Option<TelemetryReading<Self::ReadingType>>> {
        let readings = self.readings_cache.read().await;
        if readings.is_empty() {
            Ok(None)
        } else {
            Ok(Some(readings.last().unwrap().clone()))
        }
    }
}

/// Generate Reed-Solomon error correction codes (simulation)
fn reed_solomon_generate(data: &[u8]) -> Vec<u8> {
    // In a real implementation, this would use a Reed-Solomon library
    // For this sketch, we'll create a simple parity-based ECC
    let mut result = Vec::with_capacity(data.len() / 4);
    
    for chunk in data.chunks(4) {
        let mut parity: u8 = 0;
        for &byte in chunk {
            parity ^= byte;
        }
        result.push(parity);
    }
    
    result
}

/// Verify Reed-Solomon error correction codes (simulation)
fn reed_solomon_verify(data: &[u8], rs_data: &[u8]) -> bool {
    // In a real implementation, this would use a Reed-Solomon library
    let generated = reed_solomon_generate(data);
    
    if generated.len() != rs_data.len() {
        return false;
    }
    
    for (i, &byte) in generated.iter().enumerate() {
        if byte != rs_data[i] {
            return false;
        }
    }
    
    true
}

/// Correct data using Reed-Solomon error correction codes (simulation)
fn reed_solomon_correct(data: &[u8], rs_data: &[u8]) -> TelemetryResult<Vec<u8>> {
    // In a real implementation, this would use a Reed-Solomon library
    // For this sketch, we'll just return the original data if verification passes
    if reed_solomon_verify(data, rs_data) {
        Ok(data.to_vec())
    } else {
        Err(ClassicalError::DataCorruption(
            "Reed-Solomon error correction failed".to_string()
        ).into())
    }
}
