//! Ethyl Acetate Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured ethyl acetate monitoring with
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

/// Ethyl Acetate data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct EthylAcetateData {
    /// Ethyl acetate concentration (ppm)
    pub concentration: f32,
    
    /// Production rate (ppm/day)
    pub production_rate: f32,
    
    /// Aroma intensity score (0.0-1.0)
    pub aroma_intensity: f32,
    
    /// Flavor profile impact (-1.0 to 1.0)
    pub flavor_impact: f32,
    
    /// Fermentation stage indicator (0.0-1.0)
    /// Higher values indicate later fermentation stages
    pub stage_indicator: f32,
}

/// Quantum-secured ethyl acetate sensor for kombucha fermentation
pub struct EthylAcetateSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<EthylAcetateData>>>>,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Reference to alcohol sensor for correlation
    alcohol_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::alcohol::AlcoholData> + Send + Sync>>,
    
    /// Aroma threshold settings (ppm)
    aroma_threshold_low: f32,
    
    /// Aroma threshold high (ppm)
    aroma_threshold_high: f32,
}

impl EthylAcetateSensor {
    /// Create a new ethyl acetate sensor
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Default thresholds for ethyl acetate in kombucha
        // Detection threshold ~5ppm, pleasant fruity aroma 5-80pppm,
        // solvent-like above ~100ppm
        let aroma_threshold_low = 5.0;  // ppm
        let aroma_threshold_high = 100.0; // ppm
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize ethyl acetate sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            security_manager,
            clock,
            alcohol_sensor: None,
            aroma_threshold_low,
            aroma_threshold_high,
        })
    }
    
    /// Connect related sensors for correlated measurements
    pub fn connect_alcohol_sensor(
        &mut self,
        alcohol_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::alcohol::AlcoholData> + Send + Sync>>,
    ) {
        self.alcohol_sensor = alcohol_sensor;
    }
    
    /// Set aroma threshold values
    pub fn set_aroma_thresholds(&mut self, low: f32, high: f32) -> TelemetryResult<()> {
        if low <= 0.0 || low >= high {
            return Err(ClassicalError::InvalidParameter(
                "Low threshold must be positive and less than high threshold".to_string()
            ).into());
        }
        
        self.aroma_threshold_low = low;
        self.aroma_threshold_high = high;
        Ok(())
    }
    
    /// Measure ethyl acetate concentration
    async fn measure_concentration(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot measure ethyl acetate: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would use a gas chromatograph
        // or specialized volatile compounds sensor
        
        // For this sketch, we'll simulate ethyl acetate development
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(30.0), // Approximate days
        };
        
        // Ethyl acetate typically increases over fermentation time
        // Following a sigmoidal curve with lag phase, exponential, and plateau
        
        // Parameters for sigmoidal curve
        let max_concentration = 120.0 + rng.gen::<f32>() * 40.0; // 120-160 ppm max
        let growth_rate = 0.5; // Controls steepness
        let midpoint = 10.0; // Day at which concentration is at half max
        
        // Sigmoidal function: max_conc/(1+e^(-growth_rate*(days-midpoint)))
        let expected_concentration = max_concentration / 
            (1.0 + (-growth_rate * (days_factor - midpoint)).exp());
        
        // Add random variation (±15%)
        let variation = 0.15;
        let concentration = expected_concentration * (1.0 + (rng.gen::<f32>() * 2.0 - 1.0) * variation);
        
        // If we have an alcohol sensor, correlate with alcohol content
        // Higher alcohol typically means more ethyl acetate precursors
        if let Some(alcohol_sensor) = &self.alcohol_sensor {
            if let Some(reading) = alcohol_sensor.latest_reading().await? {
                let alcohol_content = reading.data.get_data().alcohol_content;
                
                // Adjust concentration based on alcohol content
                // More alcohol means more potential for ethyl acetate
                let alcohol_factor = 1.0 + (alcohol_content - 0.5) * 0.4;
                
                return Ok((concentration * alcohol_factor).max(0.0));
            }
        }
        
        Ok(concentration.max(0.0))
    }
    
    /// Calculate ethyl acetate production rate
    async fn calculate_production_rate(&self, current_concentration: f32) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        if readings.is_empty() {
            // If no history, use typical rate based on fermentation stage
            return Ok(5.0); // Typical production rate of 5 ppm/day
        }
        
        // Get most recent reading
        let latest = &readings[readings.len() - 1];
        let previous_concentration = latest.data.get_data().concentration;
        
        // Calculate time difference in days
        let time_diff_days = (Utc::now() - latest.timestamp).num_seconds() as f32 / (24.0 * 3600.0);
        
        if time_diff_days < 0.01 {
            // Readings too close in time, use previous rate
            return Ok(latest.data.get_data().production_rate);
        }
        
        // Calculate daily production rate
        let concentration_change = current_concentration - previous_concentration;
        let daily_rate = concentration_change / time_diff_days;
        
        // Apply sanity limits
        let capped_rate = daily_rate.max(0.0).min(20.0); // 0-20 ppm/day is reasonable
        
        Ok(capped_rate)
    }
    
    /// Calculate aroma intensity based on concentration
    fn calculate_aroma_intensity(&self, concentration: f32) -> f32 {
        // Below threshold: not detectable
        if concentration < self.aroma_threshold_low {
            return 0.0;
        }
        
        // Between low and high thresholds: increasing intensity
        if concentration <= self.aroma_threshold_high {
            return (concentration - self.aroma_threshold_low) / 
                (self.aroma_threshold_high - self.aroma_threshold_low);
        }
        
        // Above high threshold: intensity plateaus
        return 1.0;
    }
    
    /// Calculate flavor impact of ethyl acetate
    fn calculate_flavor_impact(&self, concentration: f32) -> f32 {
        // Ethyl acetate has different flavor impacts at different concentrations:
        // - Below ~5ppm: not detectable (neutral impact)
        // - 5-30ppm: pleasant fruity notes (positive impact)
        // - 30-100ppm: increasingly strong, but still acceptable (mixed impact)
        // - Above 100ppm: solvent-like, nail polish (negative impact)
        
        if concentration < 5.0 {
            // Below detection threshold: no impact
            return 0.0;
        } else if concentration <= 30.0 {
            // Pleasant fruity range: positive impact
            return 0.3 + 0.5 * ((concentration - 5.0) / 25.0);
        } else if concentration <= 100.0 {
            // Strong but acceptable range: declining positive impact
            return 0.8 - 0.6 * ((concentration - 30.0) / 70.0);
        } else {
            // Solvent-like range: negative impact
            return 0.2 - 1.0 * ((concentration - 100.0) / 100.0).min(1.0);
        }
    }
    
    /// Calculate fermentation stage indicator
    fn calculate_stage_indicator(&self, concentration: f32, production_rate: f32) -> f32 {
        // Ethyl acetate concentration and production rate can indicate fermentation stage
        // Early: low concentration, increasing production rate
        // Middle: medium concentration, high production rate
        // Late: high concentration, decreasing production rate
        
        // Normalize concentration (0-200 ppm scale)
        let concentration_normalized = (concentration / 200.0).min(1.0);
        
        // For production rate, peak is around 10-15 ppm/day
        // So we want to capture both the magnitude and whether it's rising or falling
        let production_stage = if production_rate < 0.1 {
            // Very low production rate
            if concentration_normalized < 0.1 {
                // Low concentration + low production = very early stage
                0.05
            } else {
                // High concentration + low production = very late stage
                0.95
            }
        } else if production_rate < 5.0 {
            // Production ramping up (early) or down (late)
            if concentration_normalized < 0.4 {
                // Low-medium concentration + low-medium production = early stage
                0.25
            } else {
                // High concentration + medium-low production = late stage
                0.8
            }
        } else if production_rate < 15.0 {
            // Production in active range
            if concentration_normalized < 0.3 {
                // Low concentration + high production = early-mid stage
                0.35
            } else if concentration_normalized < 0.6 {
                // Medium concentration + high production = mid stage
                0.5
            } else {
                // High concentration + high production = mid-late stage
                0.7
            }
        } else {
            // Very high production rate (peak activity)
            if concentration_normalized < 0.4 {
                // Low-medium concentration + very high production = mid stage
                0.4
            } else {
                // High concentration + very high production = mid-late stage
                0.6
            }
        };
        
        production_stage
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for EthylAcetateSensor {
    type ReadingType = EthylAcetateData;
    
    /// Take a quantum-secured ethyl acetate reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take ethyl acetate reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take measurements
        let concentration = self.measure_concentration().await?;
        let production_rate = self.calculate_production_rate(concentration).await?;
        
        // Calculate derived metrics
        let aroma_intensity = self.calculate_aroma_intensity(concentration);
        let flavor_impact = self.calculate_flavor_impact(concentration);
        let stage_indicator = self.calculate_stage_indicator(concentration, production_rate);
        
        // Create the reading data
        let reading_data = EthylAcetateData {
            concentration,
            production_rate,
            aroma_intensity,
            flavor_impact,
            stage_indicator,
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
            sensor_type: "ethyl_acetate".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}:{}",
                reading_id,
                secure_time,
                concentration,
                production_rate,
                aroma_intensity
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
            data.concentration,
            data.production_rate,
            data.aroma_intensity
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
                "Ethyl acetate reading failed verification".to_string()
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
