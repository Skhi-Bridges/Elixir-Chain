//! pH Sensor implementation for Kombucha fermentation
//!
//! Provides quantum-secured pH readings with full error correction at classical, bridge, and quantum levels.
//! Optimized for tracking kombucha fermentation and vinegar production.

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
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

/// Kombucha-optimized pH data
#[derive(Clone, Debug)]
pub struct PhData {
    /// Raw pH value (typically 2.5-4.5 for kombucha)
    pub ph_value: f32,
    
    /// Acidity level in g/L acetic acid equivalent
    pub acidity: f32,
    
    /// Fermentation stage indicator (0.0-1.0)
    pub fermentation_stage: f32,
    
    /// Scoby health indicator (0-100)
    pub scoby_health: f32,
    
    /// Time series forecast data for next 72 hours
    pub time_series_forecast: TimeSeriesData,
    
    /// Rate of change (pH units per day)
    pub rate_of_change: f32,
    
    /// Potential hydrogen ion concentration [H+] in mol/L
    pub h_plus_concentration: f32,
    
    /// Buffer capacity indicator (0.0-1.0)
    pub buffer_capacity: f32,
}

/// Quantum-secured pH sensor for kombucha fermentation
pub struct PhSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<PhData>>>>,
    
    /// Optimal pH range for kombucha fermentation
    optimal_range: (f32, f32),
    
    /// Optimal pH range for vinegar production
    vinegar_range: (f32, f32),
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Fermentation start date
    fermentation_start: Arc<RwLock<DateTime<Utc>>>,
    
    /// Current temperature (optional, affects pH readings)
    current_temperature: Arc<RwLock<Option<f32>>>,
    
    /// Current mode (kombucha or vinegar)
    current_mode: Arc<RwLock<FermentationMode>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
}

/// Fermentation mode for the pH sensor
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FermentationMode {
    /// Kombucha fermentation mode (optimal pH 3.5-4.2)
    Kombucha,
    
    /// Vinegar production mode (optimal pH 2.5-3.3)
    Vinegar,
}

impl PhSensor {
    /// Create a new pH sensor with the specified hardware
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Default optimal pH ranges
        let optimal_range = (3.5, 4.2); // Kombucha
        let vinegar_range = (2.5, 3.3); // Vinegar
        
        // Initialize optional parameters
        let current_temperature = Arc::new(RwLock::new(None));
        
        // Set fermentation start date to now by default
        let fermentation_start = Arc::new(RwLock::new(Utc::now()));
        
        // Default to kombucha mode
        let current_mode = Arc::new(RwLock::new(FermentationMode::Kombucha));
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize pH sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            optimal_range,
            vinegar_range,
            security_manager,
            fermentation_start,
            current_temperature,
            current_mode,
            clock,
        })
    }
    
    /// Set the fermentation mode
    pub async fn set_mode(&self, mode: FermentationMode) -> TelemetryResult<()> {
        *self.current_mode.write().await = mode;
        Ok(())
    }
    
    /// Get the current optimal pH range based on mode
    async fn get_optimal_range(&self) -> (f32, f32) {
        match *self.current_mode.read().await {
            FermentationMode::Kombucha => self.optimal_range,
            FermentationMode::Vinegar => self.vinegar_range,
        }
    }
    
    /// Set custom optimal pH range for kombucha
    pub fn set_kombucha_range(&mut self, min: f32, max: f32) -> TelemetryResult<()> {
        if min >= max || min < 2.0 || max > 5.0 {
            return Err(ClassicalError::InvalidParameter(
                "Invalid pH range for kombucha. Must be 2.0-5.0 with min < max".to_string()
            ).into());
        }
        
        self.optimal_range = (min, max);
        Ok(())
    }
    
    /// Set custom optimal pH range for vinegar
    pub fn set_vinegar_range(&mut self, min: f32, max: f32) -> TelemetryResult<()> {
        if min >= max || min < 2.0 || max > 4.0 {
            return Err(ClassicalError::InvalidParameter(
                "Invalid pH range for vinegar. Must be 2.0-4.0 with min < max".to_string()
            ).into());
        }
        
        self.vinegar_range = (min, max);
        Ok(())
    }
    
    /// Update current temperature
    pub async fn update_temperature(&self, temperature: f32) -> TelemetryResult<()> {
        if temperature < 10.0 || temperature > 40.0 {
            return Err(ClassicalError::InvalidParameter(
                "Temperature out of reasonable range (10-40°C)".to_string()
            ).into());
        }
        
        *self.current_temperature.write().await = Some(temperature);
        Ok(())
    }
    
    /// Read raw pH measurement from the sensor hardware
    async fn read_raw_ph(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        let security_compromised = security_manager.is_security_compromised().await?;
        
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take pH reading: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would read from pH sensor
        // through ADC on hardware using appropriate drivers
        
        // For this sketch, we'll simulate readings based on fermentation stages
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get days since fermentation started
        let now = Utc::now();
        let start = *self.fermentation_start.read().await;
        let days_fermenting = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Calculate pH based on current mode
        let mode = *self.current_mode.read().await;
        
        match mode {
            FermentationMode::Kombucha => {
                // Kombucha typically starts around pH 4.5 and drops to 3.5 or lower
                
                // Base pH curve model: starts high and decreases over time
                // pH = 4.5 - 0.3 * log(days + 1) gives a good approximation
                let base_ph = 4.5 - (0.3 * (days_fermenting + 1.0).ln());
                
                // Add random variation (±0.1 pH units)
                let ph_with_noise = base_ph + (rng.gen::<f32>() * 0.2 - 0.1);
                
                // Apply temperature effect if available (higher temp = faster fermentation = lower pH)
                if let Some(temp) = *self.current_temperature.read().await {
                    let optimal_temp = 25.0; // °C
                    let temp_effect = (temp - optimal_temp) * 0.01; // 0.01 pH units per degree
                    ph_with_noise - temp_effect
                } else {
                    ph_with_noise
                }
            },
            FermentationMode::Vinegar => {
                // Vinegar production starts from kombucha (pH ~3.5) and drops to ~2.8
                
                // Base pH curve model for vinegar
                let base_ph = 3.5 - (0.2 * (days_fermenting + 1.0).ln().min(3.0));
                
                // Add random variation (±0.1 pH units)
                let ph_with_noise = base_ph + (rng.gen::<f32>() * 0.2 - 0.1);
                
                // Apply temperature effect if available
                if let Some(temp) = *self.current_temperature.read().await {
                    let optimal_temp = 30.0; // °C (vinegar mothers prefer warmer temps)
                    let temp_effect = (temp - optimal_temp) * 0.008; // smaller effect for vinegar
                    ph_with_noise - temp_effect
                } else {
                    ph_with_noise
                }
            }
        }
        .max(2.0)  // Ensure pH doesn't go below 2.0
        .min(5.0)  // Ensure pH doesn't go above 5.0
    }
    
    /// Convert pH to H+ concentration
    fn ph_to_h_concentration(&self, ph: f32) -> f32 {
        // [H+] = 10^(-pH) mol/L
        10_f32.powf(-ph)
    }
    
    /// Convert pH to acidity in g/L acetic acid equivalent
    fn ph_to_acidity(&self, ph: f32) -> f32 {
        // This is an approximation. In reality, the relationship is complex
        // and depends on the buffering capacity of the solution
        
        // Formula approximation: acidity (g/L) = 10^(4.75 - pH) * 0.6
        // This gives ~30 g/L at pH 2.5 (strong vinegar)
        10_f32.powf(4.75 - ph) * 0.6
    }
    
    /// Calculate fermentation stage indicator
    async fn calculate_fermentation_stage(&self, ph: f32) -> f32 {
        let mode = *self.current_mode.read().await;
        
        match mode {
            FermentationMode::Kombucha => {
                // For kombucha, fermentation stage goes from 0.0 (fresh tea) to 1.0 (fully fermented)
                // Fresh tea is ~pH 4.5, fully fermented is ~pH 3.2
                let min_ph = 3.2;
                let max_ph = 4.5;
                let ph_range = max_ph - min_ph;
                
                // Normalize to 0.0-1.0 range (inverted since pH decreases as fermentation progresses)
                (max_ph - ph) / ph_range
            },
            FermentationMode::Vinegar => {
                // For vinegar, stage goes from 0.0 (kombucha) to 1.0 (vinegar)
                // Starting kombucha is ~pH 3.5, finished vinegar is ~pH 2.7
                let min_ph = 2.7;
                let max_ph = 3.5;
                let ph_range = max_ph - min_ph;
                
                // Normalize to 0.0-1.0 range (inverted since pH decreases as production progresses)
                (max_ph - ph) / ph_range
            }
        }
        .max(0.0)
        .min(1.0)
    }
    
    /// Calculate Scoby health indicator based on pH and other factors
    async fn calculate_scoby_health(&self, ph: f32) -> f32 {
        let mode = *self.current_mode.read().await;
        let optimal_range = self.get_optimal_range().await;
        
        // Base health on pH within optimal range
        let ph_score = if ph < optimal_range.0 {
            // Below optimal range - reduced health
            70.0 * (ph / optimal_range.0)
        } else if ph > optimal_range.1 {
            // Above optimal range - reduced health
            70.0 * (1.0 - ((ph - optimal_range.1) / 2.0))
        } else {
            // Within optimal range - maximum health
            70.0 + 30.0 * (1.0 - ((ph - (optimal_range.0 + optimal_range.1) / 2.0).abs() / ((optimal_range.1 - optimal_range.0) / 2.0)))
        };
        
        // Adjust for temperature if available
        let temp_adjusted = if let Some(temp) = *self.current_temperature.read().await {
            match mode {
                FermentationMode::Kombucha => {
                    if temp >= 20.0 && temp <= 28.0 {
                        // Optimal temperature range for kombucha
                        ph_score
                    } else if temp < 15.0 || temp > 32.0 {
                        // Extremely suboptimal temperature
                        ph_score * 0.7
                    } else {
                        // Moderately suboptimal temperature
                        ph_score * 0.85
                    }
                },
                FermentationMode::Vinegar => {
                    if temp >= 25.0 && temp <= 35.0 {
                        // Optimal temperature range for vinegar
                        ph_score
                    } else if temp < 20.0 || temp > 40.0 {
                        // Extremely suboptimal temperature
                        ph_score * 0.7
                    } else {
                        // Moderately suboptimal temperature
                        ph_score * 0.85
                    }
                }
            }
        } else {
            ph_score
        };
        
        // Additional factors could be included here
        
        temp_adjusted.max(0.0).min(100.0)
    }
    
    /// Calculate buffer capacity indicator
    fn calculate_buffer_capacity(&self, ph: f32) -> f32 {
        // Buffer capacity is highest in the middle of fermentation
        // and decreases as fermentation completes
        
        // For kombucha/vinegar, buffer capacity peaks around pH 3.8
        let peak_capacity_ph = 3.8;
        
        // Capacity decreases as pH moves away from this value
        let distance_from_peak = (ph - peak_capacity_ph).abs();
        
        // Normalize to 0.0-1.0 range
        (1.0 - (distance_from_peak / 1.5)).max(0.0).min(1.0)
    }
    
    /// Calculate rate of change from historical readings
    async fn calculate_rate_of_change(&self) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        // Need at least two readings to calculate rate
        if readings.len() < 2 {
            return Ok(0.0); // Default value if not enough data
        }
        
        // Get the two most recent readings
        let recent = &readings[readings.len() - 1];
        let previous = &readings[readings.len() - 2];
        
        // Calculate time difference in days
        let time_diff = (recent.timestamp - previous.timestamp).num_seconds() as f32 / 86400.0;
        
        if time_diff <= 0.0 {
            return Err(ClassicalError::InvalidData(
                "Invalid time difference between readings".to_string()
            ).into());
        }
        
        // Calculate rate in pH units per day
        let ph_diff = recent.data.get_data().ph_value - previous.data.get_data().ph_value;
        
        Ok(ph_diff / time_diff)
    }
    
    /// Generate time series forecast for pH levels
    async fn generate_time_series_forecast(&self) -> TimeSeriesData {
        let readings = self.readings_cache.read().await;
        
        // Prepare output structure
        let mut forecast = TimeSeriesData {
            timestamps: Vec::new(),
            values: vec![Vec::new(), Vec::new(), Vec::new()], // [mean, lower_bound, upper_bound]
            labels: vec![
                "pH".to_string(),
                "Lower Bound".to_string(),
                "Upper Bound".to_string(),
            ],
            confidence_level: 0.95,
        };
        
        // Current time
        let now = Utc::now();
        
        // Generate hourly forecast for 72 hours
        for hour in 1..=72 {
            let forecast_time = now + Duration::hours(hour);
            forecast.timestamps.push(forecast_time);
            
            // Calculate current pH level and rate of change
            let current_ph = if !readings.is_empty() {
                readings.last().unwrap().data.get_data().ph_value
            } else {
                match *self.current_mode.read().await {
                    FermentationMode::Kombucha => 4.5, // Default starting pH for kombucha
                    FermentationMode::Vinegar => 3.5,  // Default starting pH for vinegar production
                }
            };
            
            let rate_of_change = self.calculate_rate_of_change().await.unwrap_or(0.0);
            
            // Simple logarithmic model for forecast (pH change typically slows over time)
            let hours_diff = hour as f32;
            
            // Base forecast with decreasing rate of change
            let forecast_ph = current_ph + (rate_of_change * hours_diff / 24.0) * (1.0 / (1.0 + 0.1 * hours_diff / 24.0));
            
            // Calculate error bounds (wider as we forecast further)
            let uncertainty = 0.05 + (hours_diff * 0.001); // 0.05 + 0.001 per hour
            let lower_bound = forecast_ph - uncertainty;
            let upper_bound = forecast_ph + uncertainty;
            
            // Add to forecast
            forecast.values[0].push(forecast_ph);
            forecast.values[1].push(lower_bound);
            forecast.values[2].push(upper_bound);
        }
        
        forecast
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for PhSensor {
    type ReadingType = PhData;
    
    /// Take a quantum-secured pH reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take pH reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take the raw pH measurement
        let ph_value = self.read_raw_ph().await?;
        
        // Calculate derived metrics
        let h_plus_concentration = self.ph_to_h_concentration(ph_value);
        let acidity = self.ph_to_acidity(ph_value);
        let fermentation_stage = self.calculate_fermentation_stage(ph_value).await;
        let scoby_health = self.calculate_scoby_health(ph_value).await;
        let buffer_capacity = self.calculate_buffer_capacity(ph_value);
        let rate_of_change = self.calculate_rate_of_change().await?;
        
        // Generate time series forecast
        let time_series_forecast = self.generate_time_series_forecast().await;
        
        // Create the reading data
        let reading_data = PhData {
            ph_value,
            acidity,
            fermentation_stage,
            scoby_health,
            time_series_forecast,
            rate_of_change,
            h_plus_concentration,
            buffer_capacity,
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
            sensor_type: "ph".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}",
                reading_id,
                secure_time,
                ph_value
            ))?,
        };
        
        // Add to readings cache
        self.readings_cache.write().await.push(reading.clone());
        
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
            "{}:{}:{}",
            reading.id,
            reading.timestamp,
            reading.data.get_data().ph_value
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
                "pH reading failed verification".to_string()
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
