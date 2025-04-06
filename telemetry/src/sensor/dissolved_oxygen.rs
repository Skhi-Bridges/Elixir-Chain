//! Dissolved Oxygen Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured dissolved oxygen monitoring with
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

/// Dissolved Oxygen data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct DissolvedOxygenData {
    /// Dissolved oxygen level (mg/L or ppm)
    pub oxygen_level: f32,
    
    /// Oxygen saturation percentage (0-100%)
    pub saturation_percentage: f32,
    
    /// Microbial respiration rate (0.0-1.0 normalized)
    pub respiration_rate: f32,
    
    /// Fermentation aerobicity score (0.0-1.0)
    /// Higher values indicate more aerobic conditions
    pub aerobicity: f32,
    
    /// Oxygen depletion rate (mg/L/hour)
    pub depletion_rate: f32,
}

/// Quantum-secured dissolved oxygen sensor for kombucha fermentation
pub struct DissolvedOxygenSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<DissolvedOxygenData>>>>,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Temperature compensation reference
    temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
    
    /// Calibration settings
    calibration_zero_point: f32,
    calibration_slope: f32,
}

impl DissolvedOxygenSensor {
    /// Create a new dissolved oxygen sensor
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Default calibration values for optical DO sensor
        let calibration_zero_point = 0.0;  // mg/L
        let calibration_slope = 1.0;       // linear calibration factor
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize dissolved oxygen sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            security_manager,
            clock,
            temperature_sensor: None,
            calibration_zero_point,
            calibration_slope,
        })
    }
    
    /// Connect temperature sensor for temperature compensation
    pub fn connect_temperature_sensor(
        &mut self,
        temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
    ) {
        self.temperature_sensor = temperature_sensor;
    }
    
    /// Calibrate the dissolved oxygen sensor
    pub fn calibrate(&mut self, zero_point: f32, slope: f32) -> TelemetryResult<()> {
        if slope <= 0.0 {
            return Err(ClassicalError::InvalidParameter(
                "Calibration slope must be positive".to_string()
            ).into());
        }
        
        if zero_point < 0.0 {
            return Err(ClassicalError::InvalidParameter(
                "Calibration zero point cannot be negative".to_string()
            ).into());
        }
        
        self.calibration_zero_point = zero_point;
        self.calibration_slope = slope;
        Ok(())
    }
    
    /// Measure dissolved oxygen level with temperature compensation
    async fn measure_oxygen_level(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot measure dissolved oxygen: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would interface with an optical
        // or galvanic dissolved oxygen probe
        
        // For this sketch, we'll simulate dissolved oxygen levels in kombucha
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(30.0), // Approximate days
        };
        
        // Dissolved oxygen in kombucha typically starts high (near saturation)
        // and gradually decreases as microbes consume it
        // The decline follows a roughly exponential decay pattern
        
        // Starting O2 near saturation (8 mg/L at room temperature)
        let initial_o2 = 8.0;
        
        // Decay rate (slower at first, faster as fermentation progresses)
        let decay_rate = if days_factor < 3.0 {
            0.05  // Initial slow decline
        } else if days_factor < 7.0 {
            0.15  // Mid-fermentation faster decline
        } else {
            0.35  // Late fermentation, approaching equilibrium
        };
        
        // Exponential decay: O2(t) = O2(0) * e^(-decay_rate * t)
        let expected_o2 = initial_o2 * (-decay_rate * days_factor).exp();
        
        // Add random variation (±10%)
        let variation = 0.1;
        let raw_o2 = expected_o2 * (1.0 + (rng.gen::<f32>() * 2.0 - 1.0) * variation);
        
        // Apply temperature compensation if available
        let mut compensated_o2 = raw_o2;
        
        if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(temp_reading) = temp_sensor.latest_reading().await? {
                let temp = temp_reading.data.get_data().temperature;
                
                // Apply temperature compensation - dissolved oxygen decreases
                // with increasing temperature (roughly -2% per °C above 20°C)
                if temp > 20.0 {
                    let temp_factor = 1.0 - 0.02 * (temp - 20.0);
                    compensated_o2 *= temp_factor;
                }
            }
        }
        
        // Apply calibration
        let calibrated_o2 = (compensated_o2 - self.calibration_zero_point) * self.calibration_slope;
        
        // Ensure positive value (with noise floor)
        Ok(calibrated_o2.max(0.1))
    }
    
    /// Calculate oxygen saturation percentage
    async fn calculate_saturation_percentage(&self, oxygen_level: f32) -> TelemetryResult<f32> {
        // Get temperature for calculating saturation value
        let temperature = if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(temp_reading) = temp_sensor.latest_reading().await? {
                temp_reading.data.get_data().temperature
            } else {
                25.0 // Default temperature if no reading available
            }
        } else {
            25.0 // Default temperature if no sensor connected
        };
        
        // Calculate saturation DO level based on temperature
        // This is an approximation of oxygen solubility in water at 1 atm
        // DO (mg/L) = 14.652 - 0.41022*T + 0.007991*T^2 - 0.000077774*T^3
        let temp_squared = temperature * temperature;
        let temp_cubed = temp_squared * temperature;
        
        let saturation_level = 14.652 - 0.41022 * temperature
            + 0.007991 * temp_squared - 0.000077774 * temp_cubed;
        
        // Calculate saturation percentage
        let saturation_percentage = (oxygen_level / saturation_level) * 100.0;
        
        // Cap at 100% and ensure non-negative
        Ok(saturation_percentage.max(0.0).min(100.0))
    }
    
    /// Calculate microbial respiration rate
    async fn calculate_respiration_rate(&self, oxygen_level: f32) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        if readings.is_empty() {
            // If no history, estimate based on typical values
            
            // In early fermentation, respiration rate is typically 0.3-0.5
            // In mid-fermentation, respiration rate is typically 0.5-0.8
            // In late fermentation, respiration rate decreases to 0.2-0.4
            
            // For a default value without history, use mid-range
            return Ok(0.4);
        }
        
        // Get most recent reading
        let latest = &readings[readings.len() - 1];
        let previous_o2 = latest.data.get_data().oxygen_level;
        
        // Calculate time difference in hours
        let time_diff_hours = (Utc::now() - latest.timestamp).num_seconds() as f32 / 3600.0;
        
        if time_diff_hours < 0.1 {
            // Readings too close in time, use previous rate
            return Ok(latest.data.get_data().respiration_rate);
        }
        
        // Calculate oxygen consumption rate
        let o2_consumption = previous_o2 - oxygen_level;
        
        // Negative consumption (if any) likely indicates measurement error or
        // recent kombucha agitation introducing oxygen - cap at zero
        let o2_consumption_capped = o2_consumption.max(0.0);
        
        // Scale to normalized respiration rate (0-1 scale)
        // Maximum respiration would be around 2 mg/L/hour
        let max_respiration = 2.0;
        let hourly_rate = o2_consumption_capped / time_diff_hours;
        let normalized_rate = (hourly_rate / max_respiration).min(1.0);
        
        Ok(normalized_rate)
    }
    
    /// Calculate oxygen depletion rate (mg/L/hour)
    async fn calculate_depletion_rate(&self, oxygen_level: f32) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        if readings.is_empty() {
            // If no history, use a typical value
            return Ok(0.1); // 0.1 mg/L/hour is a typical initial rate
        }
        
        // Get most recent reading
        let latest = &readings[readings.len() - 1];
        let previous_o2 = latest.data.get_data().oxygen_level;
        
        // Calculate time difference in hours
        let time_diff_hours = (Utc::now() - latest.timestamp).num_seconds() as f32 / 3600.0;
        
        if time_diff_hours < 0.1 {
            // Readings too close in time, use previous rate
            return Ok(latest.data.get_data().depletion_rate);
        }
        
        // Calculate depletion rate per hour
        let o2_change = previous_o2 - oxygen_level;
        let hourly_rate = o2_change / time_diff_hours;
        
        // Cap at reasonable values
        // Negative values would indicate oxygen increase (agitation, etc.)
        let capped_rate = hourly_rate.max(-0.5).min(3.0);
        
        Ok(capped_rate)
    }
    
    /// Calculate aerobicity score based on oxygen levels
    fn calculate_aerobicity(&self, oxygen_level: f32, saturation_percentage: f32) -> f32 {
        // Kombucha transitions from aerobic to increasingly anaerobic conditions
        // as fermentation progresses
        
        // Aerobicity scale:
        // 0.0-0.2: Strongly anaerobic conditions
        // 0.2-0.4: Moderately anaerobic
        // 0.4-0.6: Microaerobic (limited oxygen)
        // 0.6-0.8: Moderately aerobic
        // 0.8-1.0: Fully aerobic
        
        // Base score on oxygen saturation percentage
        let base_score = saturation_percentage / 100.0;
        
        // Adjust by absolute oxygen level (low absolute levels matter)
        // Below 0.5 mg/L is considered functionally anaerobic
        let absolute_factor = if oxygen_level < 0.5 {
            oxygen_level / 0.5
        } else if oxygen_level < 2.0 {
            0.75 + (oxygen_level - 0.5) / 6.0
        } else {
            0.9 + (oxygen_level / 20.0).min(0.1) // Cap contribution
        };
        
        // Combined score with saturation percentage having more weight
        let combined = (base_score * 0.7) + (absolute_factor * 0.3);
        
        // Ensure valid range
        combined.max(0.0).min(1.0)
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for DissolvedOxygenSensor {
    type ReadingType = DissolvedOxygenData;
    
    /// Take a quantum-secured dissolved oxygen reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take dissolved oxygen reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take measurements
        let oxygen_level = self.measure_oxygen_level().await?;
        let saturation_percentage = self.calculate_saturation_percentage(oxygen_level).await?;
        let respiration_rate = self.calculate_respiration_rate(oxygen_level).await?;
        let depletion_rate = self.calculate_depletion_rate(oxygen_level).await?;
        
        // Calculate derived metrics
        let aerobicity = self.calculate_aerobicity(oxygen_level, saturation_percentage);
        
        // Create the reading data
        let reading_data = DissolvedOxygenData {
            oxygen_level,
            saturation_percentage,
            respiration_rate,
            aerobicity,
            depletion_rate,
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
            sensor_type: "dissolved_oxygen".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}:{}",
                reading_id,
                secure_time,
                oxygen_level,
                saturation_percentage,
                aerobicity
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
        let data = reading.data.get_data();
        let calculated_hash = blake3_hash(&format!(
            "{}:{}:{}:{}:{}",
            reading.id,
            reading.timestamp,
            data.oxygen_level,
            data.saturation_percentage,
            data.aerobicity
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
                "Dissolved oxygen reading failed verification".to_string()
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
