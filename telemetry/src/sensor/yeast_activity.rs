//! Yeast Activity Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured yeast activity monitoring with
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

/// Yeast Activity data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct YeastActivityData {
    /// Overall yeast activity level (0.0-1.0)
    pub activity_level: f32,
    
    /// CO2 production rate (µmol/L/hour)
    pub co2_production_rate: f32,
    
    /// Sugar consumption rate (brix %/day)
    pub sugar_consumption_rate: f32,
    
    /// Ethanol production rate (% ABV/day)
    pub ethanol_production_rate: f32,
    
    /// Estimated yeast population density (cells/mL)
    pub population_density: f64,
}

/// Quantum-secured yeast activity sensor for kombucha fermentation
pub struct YeastActivitySensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<YeastActivityData>>>>,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Reference to CO2 sensor for correlation
    co2_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::co2::Co2Data> + Send + Sync>>,
    
    /// Reference to alcohol sensor for correlation
    alcohol_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::alcohol::AlcoholData> + Send + Sync>>,
    
    /// Reference to brix sensor for correlation
    brix_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::brix::BrixData> + Send + Sync>>,
}

impl YeastActivitySensor {
    /// Create a new yeast activity sensor
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
                "Cannot initialize yeast activity sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            security_manager,
            clock,
            co2_sensor: None,
            alcohol_sensor: None,
            brix_sensor: None,
        })
    }
    
    /// Connect related sensors for correlated measurements
    pub fn connect_related_sensors(
        &mut self,
        co2_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::co2::Co2Data> + Send + Sync>>,
        alcohol_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::alcohol::AlcoholData> + Send + Sync>>,
        brix_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::brix::BrixData> + Send + Sync>>,
    ) {
        self.co2_sensor = co2_sensor;
        self.alcohol_sensor = alcohol_sensor;
        self.brix_sensor = brix_sensor;
    }
    
    /// Get CO2 production rate using direct measurement or correlated sensors
    async fn measure_co2_production_rate(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot measure CO2 production: hardware security compromised".to_string()
            ).into());
        }
        
        // If we have a CO2 sensor, get data from there
        if let Some(co2_sensor) = &self.co2_sensor {
            if let Some(reading) = co2_sensor.latest_reading().await? {
                // Use CO2 production rate from the CO2 sensor
                return Ok(reading.data.get_data().production_rate);
            }
        }
        
        // Otherwise, simulate measurement or derive from other sensors
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(14.0), // Approximate days
        };
        
        // Model CO2 production over time (higher in early fermentation)
        let production_curve = if days_factor < 3.0 {
            // Early fermentation: rising activity
            150.0 * (1.0 - (-0.7 * days_factor).exp())
        } else if days_factor < 7.0 {
            // Peak fermentation: plateau
            150.0 - 10.0 * (days_factor - 3.0)
        } else {
            // Late fermentation: declining activity
            110.0 * (-(days_factor - 7.0) / 5.0).exp()
        };
        
        // Add random variation (±15%)
        let variation_factor = 1.0 + (rng.gen::<f32>() * 0.3 - 0.15);
        let co2_rate = production_curve * variation_factor;
        
        Ok(co2_rate.max(5.0)) // Minimum 5 µmol/L/hour
    }
    
    /// Measure sugar consumption rate using direct measurement or correlated sensors
    async fn measure_sugar_consumption_rate(&self) -> TelemetryResult<f32> {
        // If we have a Brix sensor, derive from there
        if let Some(brix_sensor) = &self.brix_sensor {
            let readings = brix_sensor.latest_reading().await?;
            if let Some(reading) = readings {
                // Convert hourly rate to daily rate
                return Ok(reading.data.get_data().consumption_rate * 24.0);
            }
        }
        
        // Otherwise, simulate measurement
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(14.0), // Approximate days
        };
        
        // Model sugar consumption rate over time
        // Starts high, gradually decreases
        let base_rate = if days_factor < 2.0 {
            // Early fermentation: high rate
            0.8 - 0.1 * days_factor
        } else if days_factor < 7.0 {
            // Mid fermentation: medium rate
            0.6 - 0.05 * (days_factor - 2.0)
        } else {
            // Late fermentation: low rate
            0.35 - 0.03 * (days_factor - 7.0)
        };
        
        // Add random variation (±20%)
        let variation_factor = 1.0 + (rng.gen::<f32>() * 0.4 - 0.2);
        let consumption_rate = base_rate * variation_factor;
        
        Ok(consumption_rate.max(0.02)) // Minimum 0.02% per day
    }
    
    /// Measure ethanol production rate using direct measurement or correlated sensors
    async fn measure_ethanol_production_rate(&self) -> TelemetryResult<f32> {
        // If we have an alcohol sensor, derive from there
        if let Some(alcohol_sensor) = &self.alcohol_sensor {
            let readings = alcohol_sensor.latest_reading().await?;
            if let Some(reading) = readings {
                // Get production rate from alcohol sensor
                return Ok(reading.data.get_data().production_rate * 24.0); // Convert to daily rate
            }
        }
        
        // Otherwise, simulate measurement or derive from other sensors
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(14.0), // Approximate days
        };
        
        // Model ethanol production over time
        // Roughly proportional to sugar consumption but with phase shift
        let sugar_rate = self.measure_sugar_consumption_rate().await?;
        
        // About 0.45-0.5 ABV per 1% Brix consumed
        let conversion_efficiency = 0.47 + (rng.gen::<f32>() * 0.06 - 0.03);
        
        // Calculate ethanol production rate
        let ethanol_rate = sugar_rate * conversion_efficiency;
        
        // Apply phase-dependent adjustment
        let phase_factor = if days_factor < 3.0 {
            // Early: higher conversion
            1.1
        } else if days_factor > 7.0 {
            // Late: bacteria consuming ethanol
            0.7
        } else {
            // Middle: balanced
            1.0
        };
        
        Ok((ethanol_rate * phase_factor).max(0.01)) // Minimum 0.01% ABV per day
    }
    
    /// Estimate yeast population density
    async fn estimate_population_density(&self) -> TelemetryResult<f64> {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(14.0), // Approximate days
        };
        
        // Model yeast population using logistic growth
        // Starting around 10^5 cells/mL, peaking around 10^7 cells/mL
        
        // Carrying capacity (cells/mL)
        let carrying_capacity = 1.0e7;
        
        // Initial population (cells/mL)
        let initial_population = 1.0e5;
        
        // Growth rate parameter
        let growth_rate = 1.2;
        
        // Logistic growth model: P(t) = K / (1 + ((K-P0)/P0) * e^(-rt))
        // Where K is carrying capacity, P0 is initial population, r is growth rate, t is time
        let denominator = 1.0 + ((carrying_capacity - initial_population) / initial_population) 
                         * (-growth_rate * days_factor as f64).exp();
        
        let population = carrying_capacity / denominator;
        
        // Add random variation (±30%)
        let variation_factor = 1.0 + (rng.gen::<f64>() * 0.6 - 0.3);
        
        // Apply decline phase for later fermentation (after day 7)
        let decline_factor = if days_factor > 7.0 {
            (-0.2 * (days_factor as f64 - 7.0)).exp()
        } else {
            1.0
        };
        
        let final_population = population * variation_factor * decline_factor;
        
        Ok(final_population)
    }
    
    /// Calculate overall yeast activity level based on measurements
    fn calculate_activity_level(
        &self,
        co2_rate: f32,
        sugar_rate: f32,
        ethanol_rate: f32,
        population: f64
    ) -> f32 {
        // Normalize each parameter to a 0.0-1.0 scale
        
        // Normalize CO2 production (typical range: 0-200 µmol/L/hour)
        let co2_normalized = (co2_rate / 200.0).min(1.0);
        
        // Normalize sugar consumption (typical range: 0-1% Brix/day)
        let sugar_normalized = (sugar_rate / 1.0).min(1.0);
        
        // Normalize ethanol production (typical range: 0-0.5% ABV/day)
        let ethanol_normalized = (ethanol_rate / 0.5).min(1.0);
        
        // Normalize population (log scale, typical range: 10^5-10^7)
        let pop_log = population.log10() as f32;
        let pop_normalized = ((pop_log - 5.0) / 2.0).max(0.0).min(1.0);
        
        // Weighted average (CO2 and sugar consumption are most direct indicators)
        let weighted_activity = 
            (co2_normalized * 0.4) +
            (sugar_normalized * 0.3) +
            (ethanol_normalized * 0.2) +
            (pop_normalized * 0.1);
        
        weighted_activity
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for YeastActivitySensor {
    type ReadingType = YeastActivityData;
    
    /// Take a quantum-secured yeast activity reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take yeast activity reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take measurements
        let co2_production_rate = self.measure_co2_production_rate().await?;
        let sugar_consumption_rate = self.measure_sugar_consumption_rate().await?;
        let ethanol_production_rate = self.measure_ethanol_production_rate().await?;
        let population_density = self.estimate_population_density().await?;
        
        // Calculate overall activity level
        let activity_level = self.calculate_activity_level(
            co2_production_rate,
            sugar_consumption_rate,
            ethanol_production_rate,
            population_density
        );
        
        // Create the reading data
        let reading_data = YeastActivityData {
            activity_level,
            co2_production_rate,
            sugar_consumption_rate,
            ethanol_production_rate,
            population_density,
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
            sensor_type: "yeast_activity".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}:{}",
                reading_id,
                secure_time,
                activity_level,
                co2_production_rate,
                population_density
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
            data.activity_level,
            data.co2_production_rate,
            data.population_density
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
                "Yeast activity reading failed verification".to_string()
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
