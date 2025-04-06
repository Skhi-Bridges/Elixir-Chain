//! Microbial Balance Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured monitoring of yeast-bacteria balance with
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

/// Microbial Balance data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct MicrobialBalanceData {
    /// Bacteria-to-yeast ratio (higher = more bacteria)
    pub bacteria_yeast_ratio: f32,
    
    /// Proportion of bacteria in culture (0.0-1.0)
    pub bacteria_proportion: f32,
    
    /// Proportion of yeast in culture (0.0-1.0)
    pub yeast_proportion: f32,
    
    /// Culture health score (0.0-1.0)
    pub culture_health: f32,
    
    /// Fermentation stage indicator (0.0-1.0)
    /// Lower values indicate early stage, higher values indicate later stage
    pub fermentation_stage: f32,
}

/// Quantum-secured microbial balance sensor for kombucha fermentation
pub struct MicrobialBalanceSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<MicrobialBalanceData>>>>,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Reference to pH sensor
    ph_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::ph::PhData> + Send + Sync>>,
    
    /// Reference to temperature sensor
    temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
    
    /// Reference to brix sensor
    brix_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::brix::BrixData> + Send + Sync>>,
}

impl MicrobialBalanceSensor {
    /// Create a new microbial balance sensor
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize microbial balance sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            security_manager,
            clock,
            ph_sensor: None,
            temperature_sensor: None,
            brix_sensor: None,
        })
    }
    
    /// Connect related sensors for improved readings
    pub fn connect_sensors(
        &mut self,
        ph_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::ph::PhData> + Send + Sync>>,
        temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
        brix_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::brix::BrixData> + Send + Sync>>,
    ) {
        self.ph_sensor = ph_sensor;
        self.temperature_sensor = temperature_sensor;
        self.brix_sensor = brix_sensor;
    }
    
    /// Measure bacteria-yeast ratio in kombucha
    async fn measure_microbial_balance(&self) -> TelemetryResult<(f32, f32, f32)> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot measure microbial balance: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would use microscopy, flow cytometry,
        // or other methods to determine relative microbial populations
        
        // For this sketch, simulate based on fermentation variables and time
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(30.0), // Approximate days
        };
        
        // Get pH value if available (affects microbial balance)
        let ph = if let Some(ph_sensor) = &self.ph_sensor {
            if let Some(reading) = ph_sensor.latest_reading().await? {
                reading.data.get_data().ph
            } else {
                4.0 // Default if no reading available
            }
        } else {
            4.0 // Default if no sensor connected
        };
        
        // Get temperature if available (affects growth rates)
        let temperature = if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(reading) = temp_sensor.latest_reading().await? {
                reading.data.get_data().temperature
            } else {
                25.0 // Default if no reading available
            }
        } else {
            25.0 // Default if no sensor connected
        };
        
        // Get sugar content if available (affects growth)
        let brix = if let Some(brix_sensor) = &self.brix_sensor {
            if let Some(reading) = brix_sensor.latest_reading().await? {
                reading.data.get_data().brix
            } else {
                6.0 // Default if no reading available
            }
        } else {
            6.0 // Default if no sensor connected
        };
        
        // Typical kombucha progression:
        // - Day 0: Mostly yeast active, bacteria-yeast ratio ~0.5-1
        // - Days 1-7: Increasing bacteria activity, ratio ~1-2
        // - Days 7-14: Strong bacteria growth, ratio ~2-3
        // - Days 14+: Bacteria dominant, ratio ~3-4+
        
        // Base ratio progression over time
        let base_ratio = if days_factor < 3.0 {
            0.7 + days_factor * 0.2 // 0.7-1.3 range
        } else if days_factor < 10.0 {
            1.3 + (days_factor - 3.0) * 0.1 // 1.3-2.0 range
        } else if days_factor < 20.0 {
            2.0 + (days_factor - 10.0) * 0.15 // 2.0-3.5 range
        } else {
            3.5 + (days_factor - 20.0) * 0.05 // 3.5+ range
        };
        
        // pH adjustment factor
        // Lower pH favors bacteria as yeast growth slows more at lower pH
        let ph_factor = if ph < 3.2 {
            1.3 // Strong bacterial advantage at very low pH
        } else if ph < 3.8 {
            1.1 // Moderate bacterial advantage
        } else if ph < 4.5 {
            1.0 // Neutral
        } else {
            0.9 // Slight yeast advantage at higher pH
        };
        
        // Temperature adjustment factor
        // Bacteria prefer warmer temperatures within kombucha range
        let temp_factor = if temperature < 20.0 {
            0.9 // Cooler favors yeast somewhat
        } else if temperature < 24.0 {
            1.0 // Neutral
        } else if temperature < 28.0 {
            1.1 // Warmer favors bacteria somewhat
        } else {
            1.2 // Very warm favors bacteria more
        };
        
        // Sugar content adjustment
        // High sugar initially favors yeast
        let sugar_factor = if brix > 8.0 {
            0.85 // High sugar favors yeast
        } else if brix > 5.0 {
            0.95 // Moderate sugar slightly favors yeast
        } else if brix > 2.0 {
            1.1 // Lower sugar shifts advantage to bacteria
        } else {
            1.2 // Very low sugar strongly favors bacteria
        };
        
        // Apply all factors and add random variation
        let variation = 0.15;
        let final_ratio = base_ratio * ph_factor * temp_factor * sugar_factor 
            * (1.0 + (rng.gen::<f32>() * 2.0 - 1.0) * variation);
        
        // Calculate proportions
        // Ratio = bacteria / yeast, so bacteria_proportion = ratio / (ratio + 1)
        let bacteria_proportion = final_ratio / (final_ratio + 1.0);
        let yeast_proportion = 1.0 - bacteria_proportion;
        
        Ok((final_ratio, bacteria_proportion, yeast_proportion))
    }
    
    /// Calculate culture health based on balance and environmental conditions
    async fn calculate_culture_health(
        &self,
        bacteria_yeast_ratio: f32,
        bacteria_proportion: f32,
        yeast_proportion: f32,
    ) -> TelemetryResult<f32> {
        // Get pH value if available (affects health optimality)
        let ph = if let Some(ph_sensor) = &self.ph_sensor {
            if let Some(reading) = ph_sensor.latest_reading().await? {
                reading.data.get_data().ph
            } else {
                4.0 // Default if no reading available
            }
        } else {
            4.0 // Default if no sensor connected
        };
        
        // Get temperature if available (affects health optimality)
        let temperature = if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(reading) = temp_sensor.latest_reading().await? {
                reading.data.get_data().temperature
            } else {
                25.0 // Default if no reading available
            }
        } else {
            25.0 // Default if no sensor connected
        };
        
        // Balance score - both bacteria and yeast need to be present
        // Optimal bacteria:yeast ratio is typically 2:1 to 3:1 in mature kombucha
        let balance_score = if bacteria_yeast_ratio < 0.5 {
            // Too much yeast, not enough bacteria
            0.5 + (bacteria_yeast_ratio / 0.5) * 0.3
        } else if bacteria_yeast_ratio <= 3.5 {
            // Good balance range
            0.8 + (bacteria_yeast_ratio - 0.5) / 3.0 * 0.2
        } else if bacteria_yeast_ratio <= 5.0 {
            // Starting to be too much bacteria
            1.0 - (bacteria_yeast_ratio - 3.5) / 1.5 * 0.2
        } else {
            // Far too much bacteria, not enough yeast
            0.8 - (bacteria_yeast_ratio - 5.0) / 5.0 * 0.3
        };
        
        // Environment score based on pH and temperature
        let ph_score = if ph < 2.8 {
            // Too acidic
            0.5 + (ph - 2.0) / 0.8 * 0.3
        } else if ph <= 3.5 {
            // Ideal range
            0.8 + (3.5 - ph.abs_diff(3.15)) / 0.35 * 0.2
        } else if ph <= 4.5 {
            // Acceptable but not ideal
            0.8 - (ph - 3.5) / 1.0 * 0.3
        } else {
            // Too basic
            0.5 - (ph - 4.5) / 2.0 * 0.3
        };
        
        let temp_score = if temperature < 18.0 {
            // Too cold
            0.6 + (temperature - 15.0) / 3.0 * 0.2
        } else if temperature <= 30.0 {
            // Good range (with peak at ~26°C)
            0.8 + (1.0 - (temperature - 26.0).abs() / 8.0) * 0.2
        } else if temperature <= 34.0 {
            // Too warm
            0.8 - (temperature - 30.0) / 4.0 * 0.4
        } else {
            // Much too hot
            0.4 - (temperature - 34.0) / 6.0 * 0.4
        };
        
        // Combined score with balance having the most weight
        let combined = (balance_score * 0.6) + (ph_score * 0.2) + (temp_score * 0.2);
        
        // Ensure valid range
        Ok(combined.max(0.0).min(1.0))
    }
    
    /// Calculate fermentation stage indicator
    fn calculate_fermentation_stage(
        &self,
        bacteria_yeast_ratio: f32,
        bacteria_proportion: f32,
    ) -> f32 {
        // Fermentation stage based primarily on bacteria-yeast ratio
        // Higher ratio indicates later fermentation stage
        
        if bacteria_yeast_ratio < 1.0 {
            // Very early stage (0.0-0.2)
            0.1 + (bacteria_yeast_ratio / 1.0) * 0.1
        } else if bacteria_yeast_ratio < 2.0 {
            // Early stage (0.2-0.4)
            0.2 + (bacteria_yeast_ratio - 1.0) / 1.0 * 0.2
        } else if bacteria_yeast_ratio < 3.0 {
            // Mid stage (0.4-0.7)
            0.4 + (bacteria_yeast_ratio - 2.0) / 1.0 * 0.3
        } else if bacteria_yeast_ratio < 4.0 {
            // Late stage (0.7-0.9)
            0.7 + (bacteria_yeast_ratio - 3.0) / 1.0 * 0.2
        } else {
            // Very late stage (0.9-1.0)
            0.9 + (bacteria_yeast_ratio - 4.0) / 6.0 * 0.1
        }
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for MicrobialBalanceSensor {
    type ReadingType = MicrobialBalanceData;
    
    /// Take a quantum-secured microbial balance reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take microbial balance reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take measurements
        let (bacteria_yeast_ratio, bacteria_proportion, yeast_proportion) = 
            self.measure_microbial_balance().await?;
        
        // Calculate derived metrics
        let culture_health = self.calculate_culture_health(
            bacteria_yeast_ratio, 
            bacteria_proportion, 
            yeast_proportion
        ).await?;
        
        let fermentation_stage = self.calculate_fermentation_stage(
            bacteria_yeast_ratio,
            bacteria_proportion
        );
        
        // Create the reading data
        let reading_data = MicrobialBalanceData {
            bacteria_yeast_ratio,
            bacteria_proportion,
            yeast_proportion,
            culture_health,
            fermentation_stage,
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
            sensor_type: "microbial_balance".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}:{}",
                reading_id,
                secure_time,
                bacteria_yeast_ratio,
                bacteria_proportion,
                fermentation_stage
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
            data.bacteria_yeast_ratio,
            data.bacteria_proportion,
            data.fermentation_stage
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
                "Microbial balance reading failed verification".to_string()
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
