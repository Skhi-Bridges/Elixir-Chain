//! Carbon Dioxide Sensor implementation for kombucha fermentation
//!
//! Provides quantum-secured CO2 monitoring with
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

/// Carbon Dioxide data for kombucha fermentation
#[derive(Clone, Debug)]
pub struct CarbonDioxideData {
    /// CO2 concentration (ppm)
    pub concentration: f32,
    
    /// CO2 production rate (ppm/hour)
    pub production_rate: f32,
    
    /// Fermentation activity score (0.0-1.0)
    pub fermentation_activity: f32,
    
    /// Carbonation level (g/L)
    pub carbonation_level: f32,
    
    /// Gas pressure (if in sealed container) in kPa
    pub pressure: Option<f32>,
}

/// Quantum-secured carbon dioxide sensor for kombucha fermentation
pub struct CarbonDioxideSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<CarbonDioxideData>>>>,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
    
    /// Temperature sensor reference for compensation
    temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
    
    /// Pressure sensor configured
    pressure_sensor_enabled: bool,
    
    /// Container volume in liters (used for pressure calculations)
    container_volume: Option<f32>,
    
    /// Container headspace percentage (used for pressure calculations)
    container_headspace_percent: Option<f32>,
}

impl CarbonDioxideSensor {
    /// Create a new carbon dioxide sensor
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
                "Cannot initialize carbon dioxide sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            security_manager,
            clock,
            temperature_sensor: None,
            pressure_sensor_enabled: false,
            container_volume: None,
            container_headspace_percent: None,
        })
    }
    
    /// Connect temperature sensor for readings compensation
    pub fn connect_temperature_sensor(
        &mut self,
        temperature_sensor: Option<Arc<dyn QuantumSecuredSensor<ReadingType = crate::sensor::temperature::TemperatureData> + Send + Sync>>,
    ) {
        self.temperature_sensor = temperature_sensor;
    }
    
    /// Configure pressure sensor capability and container parameters
    pub fn configure_pressure_monitoring(
        &mut self, 
        enabled: bool,
        container_volume_liters: Option<f32>,
        headspace_percent: Option<f32>,
    ) -> TelemetryResult<()> {
        // Validate parameters if enabling pressure monitoring
        if enabled {
            // Container volume is required for pressure calculations
            if container_volume_liters.is_none() || container_volume_liters.unwrap() <= 0.0 {
                return Err(ClassicalError::InvalidParameter(
                    "Container volume must be positive when pressure monitoring is enabled".to_string()
                ).into());
            }
            
            // Headspace percentage must be valid (0-100%)
            if let Some(headspace) = headspace_percent {
                if headspace <= 0.0 || headspace >= 100.0 {
                    return Err(ClassicalError::InvalidParameter(
                        "Headspace percentage must be between 0 and 100".to_string()
                    ).into());
                }
            } else {
                return Err(ClassicalError::InvalidParameter(
                    "Headspace percentage is required when pressure monitoring is enabled".to_string()
                ).into());
            }
        }
        
        self.pressure_sensor_enabled = enabled;
        self.container_volume = container_volume_liters;
        self.container_headspace_percent = headspace_percent;
        
        Ok(())
    }
    
    /// Measure carbon dioxide concentration
    async fn measure_co2_concentration(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        if security_manager.is_security_compromised().await? {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot measure CO2: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would interface with an NDIR CO2 sensor
        // or other gas sensor appropriate for the concentration range
        
        // For this sketch, we'll simulate CO2 development in kombucha fermentation
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get current fermentation stage from historical data
        let readings = self.readings_cache.read().await;
        let days_factor = match readings.len() {
            0 => 0.0,
            n => (n as f32 / 24.0).min(30.0), // Approximate days
        };
        
        // CO2 in kombucha typically increases during active fermentation
        // then plateaus or decreases as fermentation slows
        
        // Parameters for CO2 progression
        // Atmospheric CO2 is ~400 ppm
        // During fermentation can rise to 1000-5000+ ppm depending on vessel
        
        let base_co2 = 400.0; // Baseline atmospheric CO2
        
        // Logistic growth followed by decay
        if days_factor < 14.0 {
            // Growth phase: logistic growth model
            let max_co2 = 5000.0 + rng.gen::<f32>() * 1000.0; // 5000-6000 ppm max
            let growth_rate = 0.6; // Controls steepness
            let midpoint = 7.0; // Day at which concentration is at half max
            
            // Logistic function: base + (max-base)/(1+e^(-growth_rate*(days-midpoint)))
            let expected_co2 = base_co2 + (max_co2 - base_co2) / 
                (1.0 + (-growth_rate * (days_factor - midpoint)).exp());
                
            // Add random variation (±15%)
            let variation = 0.15;
            let concentration = expected_co2 * (1.0 + (rng.gen::<f32>() * 2.0 - 1.0) * variation);
            
            Ok(concentration)
        } else {
            // Decay phase: exponential decay
            // Start from peak
            let peak_co2 = 5500.0;
            let decay_rate = 0.1; // Rate of decline
            
            // Decay function: base + (peak-base) * e^(-decay_rate * (days-peak_day))
            let decay_days = days_factor - 14.0;
            let expected_co2 = base_co2 + (peak_co2 - base_co2) * (-decay_rate * decay_days).exp();
            
            // Add random variation (±15%)
            let variation = 0.15;
            let concentration = expected_co2 * (1.0 + (rng.gen::<f32>() * 2.0 - 1.0) * variation);
            
            Ok(concentration)
        }
    }
    
    /// Calculate CO2 production rate
    async fn calculate_production_rate(&self, current_concentration: f32) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        if readings.is_empty() {
            // If no history, use typical rate based on fermentation stage
            return Ok(50.0); // 50 ppm/hour is a reasonable initial rate
        }
        
        // Get most recent reading
        let latest = &readings[readings.len() - 1];
        let previous_concentration = latest.data.get_data().concentration;
        
        // Calculate time difference in hours
        let time_diff_hours = (Utc::now() - latest.timestamp).num_seconds() as f32 / 3600.0;
        
        if time_diff_hours < 0.01 {
            // Readings too close in time, use previous rate
            return Ok(latest.data.get_data().production_rate);
        }
        
        // Calculate hourly production rate
        let concentration_change = current_concentration - previous_concentration;
        let hourly_rate = concentration_change / time_diff_hours;
        
        // Apply sanity limits - cap negative rate (which might happen during
        // degassing or measurement errors)
        let capped_rate = hourly_rate.max(-100.0).min(200.0);
        
        Ok(capped_rate)
    }
    
    /// Calculate fermentation activity score based on CO2 metrics
    fn calculate_fermentation_activity(
        &self, 
        concentration: f32,
        production_rate: f32
    ) -> f32 {
        // Normalize concentration (scale 400-6000 ppm to 0-1)
        let atmospheric_co2 = 400.0;
        let max_expected_co2 = 6000.0;
        
        let concentration_normalized = ((concentration - atmospheric_co2) / 
            (max_expected_co2 - atmospheric_co2)).max(0.0).min(1.0);
        
        // Normalize production rate (scale 0-200 ppm/hour to 0-1)
        let max_production_rate = 200.0;
        let production_normalized = if production_rate < 0.0 {
            // Negative rate (CO2 decreasing) indicates low activity
            0.2 * (1.0 + production_rate / 100.0) // Scale 0.0-0.2
        } else {
            // Positive rate indicates active fermentation
            0.2 + 0.8 * (production_rate / max_production_rate).min(1.0)
        };
        
        // Combined score with production rate having more weight
        // as it's more indicative of current activity
        let combined = (concentration_normalized * 0.3) + (production_normalized * 0.7);
        
        // Ensure valid range
        combined.max(0.0).min(1.0)
    }
    
    /// Calculate carbonation level in g/L
    async fn calculate_carbonation(&self, co2_concentration: f32) -> TelemetryResult<f32> {
        // Get temperature for solubility calculation
        let temperature = if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(temp_reading) = temp_sensor.latest_reading().await? {
                temp_reading.data.get_data().temperature
            } else {
                25.0 // Default temperature if no reading available
            }
        } else {
            25.0 // Default temperature if no sensor connected
        };
        
        // Henry's Law constant for CO2 in water depends on temperature
        // This is a simplified approximation
        // K_H = 1.25 * 10^-3 * e^(1900 * (1/T - 1/298.15)) mol/(L*atm)
        let k_h = 1.25e-3 * ((1900.0 * (1.0 / (temperature + 273.15) - 1.0 / 298.15)).exp());
        
        // Convert CO2 concentration from ppm to partial pressure in atm
        // This is a simplification; in reality, need to account for total pressure
        let co2_partial_pressure = co2_concentration / 1_000_000.0; // ppm to fraction
        
        // Apply Henry's Law to calculate dissolved CO2
        // [CO2] = K_H * P_CO2
        let dissolved_co2_mol_per_l = k_h * co2_partial_pressure;
        
        // Convert from mol/L to g/L
        // Molar mass of CO2 is 44.01 g/mol
        let dissolved_co2_g_per_l = dissolved_co2_mol_per_l * 44.01;
        
        // Typical kombucha carbonation ranges:
        // Flat: 0-1 g/L
        // Lightly carbonated: 1-3 g/L
        // Medium carbonation: 3-5 g/L
        // Highly carbonated: 5+ g/L
        
        // Adjust for observed carbonic acid formation in kombucha
        // (simplified approximation)
        let final_carbonation = dissolved_co2_g_per_l * 1.2;
        
        Ok(final_carbonation.max(0.0))
    }
    
    /// Calculate gas pressure in sealed container
    async fn calculate_pressure(&self, co2_production_rate: f32) -> TelemetryResult<Option<f32>> {
        // Only calculate if pressure monitoring is configured
        if !self.pressure_sensor_enabled || 
           self.container_volume.is_none() || 
           self.container_headspace_percent.is_none() {
            return Ok(None);
        }
        
        // Get container parameters
        let volume_l = self.container_volume.unwrap();
        let headspace_percent = self.container_headspace_percent.unwrap();
        let headspace_l = volume_l * (headspace_percent / 100.0);
        
        // Get temperature for ideal gas law
        let temperature_c = if let Some(temp_sensor) = &self.temperature_sensor {
            if let Some(temp_reading) = temp_sensor.latest_reading().await? {
                temp_reading.data.get_data().temperature
            } else {
                25.0 // Default temperature if no reading available
            }
        } else {
            25.0 // Default temperature if no sensor connected
        };
        
        // Convert to Kelvin
        let temperature_k = temperature_c + 273.15;
        
        // Get accumulated CO2 from historical data
        let readings = self.readings_cache.read().await;
        let historical_co2_accumulated: f32 = if readings.is_empty() {
            // If no history, assume just starting
            0.0
        } else {
            // Sum accumulated CO2 from production rates
            // This is a simplification that assumes all readings equally spaced
            let mut accumulated = 0.0;
            
            for reading in readings.iter() {
                let reading_data = reading.data.get_data();
                accumulated += reading_data.production_rate / 24.0; // Adjust to daily rate
            }
            
            accumulated
        };
        
        // Convert accumulated CO2 (ppm) to moles
        // This is a rough calculation
        // 1 ppm = 0.0409 μmol/L at 25°C and 1 atm
        let moles_per_ppm = 0.0409e-6; // mol/L per ppm
        let co2_moles = historical_co2_accumulated * moles_per_ppm * headspace_l;
        
        // Apply ideal gas law: P = nRT/V
        // R = 8.314 J/(mol·K)
        let gas_constant_r = 8.314; // J/(mol·K)
        
        // Calculate pressure in pascals
        let pressure_pa = (co2_moles * gas_constant_r * temperature_k) / headspace_l;
        
        // Convert to kPa for more manageable numbers
        let pressure_kpa = pressure_pa / 1000.0;
        
        // Add atmospheric pressure (101.325 kPa)
        let total_pressure_kpa = 101.325 + pressure_kpa;
        
        // Apply sanity check - cap at reasonable values for kombucha vessels
        let safe_pressure = total_pressure_kpa.max(101.0).min(200.0);
        
        Ok(Some(safe_pressure))
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for CarbonDioxideSensor {
    type ReadingType = CarbonDioxideData;
    
    /// Take a quantum-secured carbon dioxide reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take carbon dioxide reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take measurements
        let concentration = self.measure_co2_concentration().await?;
        let production_rate = self.calculate_production_rate(concentration).await?;
        let carbonation_level = self.calculate_carbonation(concentration).await?;
        let pressure = self.calculate_pressure(production_rate).await?;
        
        // Calculate derived metrics
        let fermentation_activity = self.calculate_fermentation_activity(concentration, production_rate);
        
        // Create the reading data
        let reading_data = CarbonDioxideData {
            concentration,
            production_rate,
            fermentation_activity,
            carbonation_level,
            pressure,
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
            sensor_type: "carbon_dioxide".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}:{}",
                reading_id,
                secure_time,
                concentration,
                production_rate,
                fermentation_activity
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
            data.fermentation_activity
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
                "Carbon dioxide reading failed verification".to_string()
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
