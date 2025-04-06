//! Acetic Acid Sensor implementation for Kombucha/Vinegar monitoring
//!
//! Provides quantum-secured acetic acid level readings with
//! full error correction at classical, bridge, and quantum levels.
//! Optimized for tracking kombucha-to-vinegar conversion with ESP32S3 hardware.

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

/// Kombucha-to-vinegar monitoring data
#[derive(Clone, Debug)]
pub struct AceticAcidData {
    /// Acetic acid concentration (g/L)
    pub acetic_acid_concentration: f32,
    
    /// Vinegar conversion ratio (0.0-1.0)
    /// where 0.0 is pure kombucha and 1.0 is fully converted vinegar
    pub vinegar_conversion_ratio: f32,
    
    /// Sourness score (0-100 scale)
    pub sourness_score: f32,
    
    /// Ethanol to acetic acid conversion rate (g/L per day)
    pub conversion_rate: f32,
    
    /// Estimated days to full vinegar conversion
    pub days_to_full_conversion: f32,
    
    /// Vinegar maturity (0.0-1.0)
    pub vinegar_maturity: f32,
    
    /// Optimal range indicator (-1.0 = too low, 0.0 = optimal, 1.0 = too high)
    pub optimal_range_indicator: f32,
    
    /// Time series forecast data for next 72 hours
    pub time_series_forecast: TimeSeriesData,
}

/// Quantum-secured acetic acid sensor for kombucha-to-vinegar monitoring
pub struct AceticAcidSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<AceticAcidData>>>>,
    
    /// Optimal acetic acid range (g/L)
    /// For kombucha: 1-3 g/L
    /// For vinegar: 40-50 g/L
    optimal_range: (f32, f32),
    
    /// Current target product (kombucha or vinegar)
    target_product: TargetProduct,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Process start date
    process_start: Arc<RwLock<DateTime<Utc>>>,
    
    /// Temperature correlation (optional)
    current_temperature: Arc<RwLock<Option<f32>>>,
    
    /// Current alcohol level (optional, for conversion modeling)
    current_alcohol: Arc<RwLock<Option<f32>>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
}

/// Target product enum to differentiate between kombucha and vinegar targets
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TargetProduct {
    /// Targeting kombucha production (lower acetic acid)
    Kombucha,
    
    /// Targeting vinegar production (higher acetic acid)
    Vinegar,
}

impl AceticAcidSensor {
    /// Create a new acetic acid sensor with the specified hardware
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
        target_product: TargetProduct,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair using ESP32S3's hardware security features
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Set optimal range based on target product
        let optimal_range = match target_product {
            TargetProduct::Kombucha => (1.0, 3.0),   // g/L for kombucha
            TargetProduct::Vinegar => (40.0, 50.0),  // g/L for vinegar
        };
        
        // Initialize temperature as None
        let current_temperature = Arc::new(RwLock::new(None));
        
        // Initialize alcohol as None
        let current_alcohol = Arc::new(RwLock::new(None));
        
        // Set process start date to now by default
        let process_start = Arc::new(RwLock::new(Utc::now()));
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize acetic acid sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            optimal_range,
            target_product,
            security_manager,
            process_start,
            current_temperature,
            current_alcohol,
            clock,
        })
    }
    
    /// Set the target product (kombucha or vinegar)
    pub fn set_target_product(&mut self, target_product: TargetProduct) {
        self.target_product = target_product;
        
        // Update optimal range based on new target
        self.optimal_range = match target_product {
            TargetProduct::Kombucha => (1.0, 3.0),   // g/L for kombucha
            TargetProduct::Vinegar => (40.0, 50.0),  // g/L for vinegar
        };
    }
    
    /// Set process start date
    pub async fn set_process_start(&self, start_date: DateTime<Utc>) -> TelemetryResult<()> {
        if start_date > Utc::now() {
            return Err(ClassicalError::InvalidParameter(
                "Process start date cannot be in the future".to_string()
            ).into());
        }
        
        *self.process_start.write().await = start_date;
        Ok(())
    }
    
    /// Update current temperature for correlation analysis
    pub async fn update_temperature(&self, temperature: f32) -> TelemetryResult<()> {
        if temperature < 0.0 || temperature > 50.0 {
            return Err(ClassicalError::InvalidParameter(
                "Temperature out of reasonable range (0°C to 50°C)".to_string()
            ).into());
        }
        
        *self.current_temperature.write().await = Some(temperature);
        Ok(())
    }
    
    /// Update current alcohol level (for conversion modeling)
    pub async fn update_alcohol(&self, abv_percent: f32) -> TelemetryResult<()> {
        if abv_percent < 0.0 || abv_percent > 15.0 {
            return Err(ClassicalError::InvalidParameter(
                "Alcohol level out of reasonable range (0-15%)".to_string()
            ).into());
        }
        
        *self.current_alcohol.write().await = Some(abv_percent);
        Ok(())
    }
    
    /// Read raw acetic acid measurements from the sensor hardware
    async fn read_raw_acetic_acid(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        let security_compromised = security_manager.is_security_compromised().await?;
        
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take acetic acid reading: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would read from acetic acid sensor
        // or similar through ADC on the ESP32S3 using Zephyr RTOS drivers
        
        // For this sketch, we'll simulate readings based on fermentation time
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get days since process started
        let now = Utc::now();
        let start = *self.process_start.read().await;
        let days_processing = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Model logistic conversion curve
        let max_acetic_acid = match self.target_product {
            TargetProduct::Kombucha => 5.0,   // Low target for kombucha
            TargetProduct::Vinegar => 50.0,   // High target for vinegar
        };
        
        // Different conversion rates based on target product
        let target_days = match self.target_product {
            TargetProduct::Kombucha => 14.0,   // Slower for kombucha
            TargetProduct::Vinegar => 21.0,    // Faster for intentional vinegar
        };
        
        let growth_rate = 0.4;
        
        // Logistic function: acetic_acid = max_acetic_acid / (1 + e^(-growth_rate * (days - target_days/2)))
        let base_acetic_acid = max_acetic_acid / (1.0 + f32::exp(-growth_rate * (days_processing - target_days / 2.0)));
        
        // Add small random variation
        let variation = max_acetic_acid * 0.05; // 5% of max
        let acetic_acid = base_acetic_acid + (rng.gen::<f32>() * 2.0 - 1.0) * variation;
        
        // Apply temperature effect if available
        let temp_adjusted_acetic_acid = if let Some(temp) = *self.current_temperature.read().await {
            // Acetic acid bacteria work best at 25-30°C
            if temp < 20.0 {
                // Too cold, slows down conversion
                acetic_acid * (0.7 + 0.02 * temp)
            } else if temp > 35.0 {
                // Too hot, kills bacteria
                acetic_acid * (1.2 - 0.01 * (temp - 35.0))
            } else {
                // Optimal temperature range
                acetic_acid * (1.0 + 0.005 * (temp - 20.0))
            }
        } else {
            acetic_acid
        };
        
        // Apply alcohol effect if available
        let fully_adjusted_acetic_acid = if let Some(abv) = *self.current_alcohol.read().await {
            // Acetic acid bacteria convert alcohol to acetic acid
            // Higher alcohol = faster initial conversion, but too high is inhibitory
            if abv < 0.5 {
                // Too low alcohol for good conversion
                temp_adjusted_acetic_acid * 0.8
            } else if abv > 8.0 {
                // Too high alcohol inhibits bacteria
                temp_adjusted_acetic_acid * 0.7
            } else {
                // Optimal alcohol range enhances conversion
                temp_adjusted_acetic_acid * (1.0 + 0.05 * (abv - 0.5).min(4.0))
            }
        } else {
            temp_adjusted_acetic_acid
        };
        
        Ok(fully_adjusted_acetic_acid)
    }
    
    /// Calculate vinegar conversion ratio based on acetic acid concentration
    fn calculate_vinegar_conversion_ratio(&self, acetic_acid: f32) -> f32 {
        // Typical vinegar has about 40-50g/L acetic acid
        // Typical kombucha has about 1-3g/L acetic acid
        let conversion_ratio = (acetic_acid - 1.0) / (50.0 - 1.0);
        conversion_ratio.max(0.0).min(1.0)
    }
    
    /// Calculate sourness score based on acetic acid level
    fn calculate_sourness_score(&self, acetic_acid: f32) -> f32 {
        // Map acetic acid levels to a 0-100 sourness scale
        // Exponential curve to match human taste perception
        // (sourness perception is roughly logarithmic)
        let base_score = 20.0 * f32::ln(acetic_acid + 1.0);
        base_score.max(0.0).min(100.0)
    }
    
    /// Calculate vinegar maturity based on acetic acid level and time
    async fn calculate_vinegar_maturity(&self, acetic_acid: f32) -> f32 {
        // Get days since process started
        let now = Utc::now();
        let start = *self.process_start.read().await;
        let days_processing = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Vinegar maturity depends on both acetic acid level and aging time
        let acetic_factor = self.calculate_vinegar_conversion_ratio(acetic_acid);
        
        // Aging factor: vinegar needs time to mature even after acetic acid conversion
        let aging_factor = (days_processing / 30.0).min(1.0); // Full maturity after 30 days
        
        // Weighted combination
        (acetic_factor * 0.7) + (aging_factor * 0.3)
    }
    
    /// Calculate conversion rate from historical readings
    async fn calculate_conversion_rate(&self) -> TelemetryResult<f32> {
        let readings = self.readings_cache.read().await;
        
        // Need at least two readings to calculate rate
        if readings.len() < 2 {
            return Ok(0.05); // Default value if not enough data
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
        
        // Calculate rate in g/L per day
        let acetic_diff = recent.data.get_data().acetic_acid_concentration
                        - previous.data.get_data().acetic_acid_concentration;
        
        Ok(acetic_diff / time_diff)
    }
    
    /// Calculate days until full vinegar conversion
    async fn calculate_days_to_full_conversion(&self, acetic_acid: f32, conversion_rate: f32) -> f32 {
        // Only relevant for vinegar target
        if self.target_product != TargetProduct::Vinegar {
            return f32::INFINITY; // Not targeting vinegar
        }
        
        // If no meaningful conversion rate, return default estimate
        if conversion_rate <= 0.01 {
            return 30.0; // Default estimate
        }
        
        // Calculate how much more acetic acid needed to reach target
        let target_acetic_acid = 45.0; // Middle of optimal vinegar range
        let acetic_acid_needed = target_acetic_acid - acetic_acid;
        
        if acetic_acid_needed <= 0.0 {
            // Already at or above target
            return 0.0;
        }
        
        // Calculate days based on current conversion rate
        acetic_acid_needed / conversion_rate
    }
    
    /// Calculate optimal range indicator
    fn calculate_optimal_indicator(&self, acetic_acid: f32) -> f32 {
        // Calculate position relative to optimal range
        let (min_optimal, max_optimal) = self.optimal_range;
        
        if acetic_acid < min_optimal {
            // Below range, negative indicator
            -1.0 * (min_optimal - acetic_acid) / min_optimal
        } else if acetic_acid > max_optimal {
            // Above range, positive indicator
            (acetic_acid - max_optimal) / max_optimal
        } else {
            // Within optimal range
            0.0
        }
    }
    
    /// Generate time series forecast for acetic acid levels
    async fn generate_time_series_forecast(&self) -> TimeSeriesData {
        let readings = self.readings_cache.read().await;
        
        // Prepare output structure
        let mut forecast = TimeSeriesData {
            timestamps: Vec::new(),
            values: vec![Vec::new(), Vec::new(), Vec::new()], // [mean, lower_bound, upper_bound]
            labels: vec![
                "Acetic Acid (g/L)".to_string(),
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
            
            // Calculate days since process start at forecast time
            let start = *self.process_start.read().await;
            let days_processing = (forecast_time - start).num_seconds() as f32 / 86400.0;
            
            // Current acetic acid level (most recent reading or default)
            let current_acetic_acid = if !readings.is_empty() {
                readings.last().unwrap().data.get_data().acetic_acid_concentration
            } else {
                1.0 // Default value if no readings
            };
            
            // Current conversion rate (from historical data or default)
            let conversion_rate = self.calculate_conversion_rate().await.unwrap_or(0.05);
            
            // Calculate forecast for this time point
            // Using a logistic model for vinegar fermentation
            let max_acetic_acid = match self.target_product {
                TargetProduct::Kombucha => 5.0,
                TargetProduct::Vinegar => 50.0,
            };
            
            // Base forecast using current trend
            let time_diff_days = hour as f32 / 24.0;
            let linear_forecast = current_acetic_acid + (conversion_rate * time_diff_days);
            
            // Apply logistic constraints to prevent unrealistic forecasts
            let logistic_factor = 1.0 / (1.0 + f32::exp(-0.1 * (linear_forecast - max_acetic_acid / 2.0)));
            let mean_forecast = linear_forecast * (1.0 - logistic_factor) + (max_acetic_acid * logistic_factor);
            
            // Apply temperature effects if available
            let temperature_adjusted = if let Some(temp) = *self.current_temperature.read().await {
                if temp < 20.0 || temp > 35.0 {
                    // Suboptimal temperature, slower progression
                    let temp_factor = if temp < 20.0 {
                        0.8 + (temp / 20.0) * 0.2 // 0.8-1.0 for 0-20°C
                    } else {
                        1.0 - ((temp - 35.0) / 15.0) * 0.3 // 1.0-0.7 for 35-50°C
                    };
                    
                    current_acetic_acid + ((mean_forecast - current_acetic_acid) * temp_factor)
                } else {
                    // Optimal temperature range
                    mean_forecast
                }
            } else {
                mean_forecast
            };
            
            // Calculate error bounds (wider as we forecast further)
            let uncertainty = 0.05 + (time_diff_days * 0.02); // 5% + 2% per day
            let lower_bound = temperature_adjusted * (1.0 - uncertainty);
            let upper_bound = temperature_adjusted * (1.0 + uncertainty);
            
            // Add to forecast
            forecast.values[0].push(temperature_adjusted);
            forecast.values[1].push(lower_bound);
            forecast.values[2].push(upper_bound);
        }
        
        forecast
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for AceticAcidSensor {
    type ReadingType = AceticAcidData;
    
    /// Take a quantum-secured acetic acid reading
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Check security before proceeding
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take acetic acid reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Get a quantum-secured timestamp
        let secure_time = self.clock.read().await.get_secure_time().await?;
        
        // Take the raw acetic acid measurement
        let acetic_acid = self.read_raw_acetic_acid().await?;
        
        // Calculate derived metrics
        let vinegar_conversion_ratio = self.calculate_vinegar_conversion_ratio(acetic_acid);
        let sourness_score = self.calculate_sourness_score(acetic_acid);
        let vinegar_maturity = self.calculate_vinegar_maturity(acetic_acid).await;
        let conversion_rate = self.calculate_conversion_rate().await?;
        let days_to_full_conversion = self.calculate_days_to_full_conversion(acetic_acid, conversion_rate).await;
        let optimal_range_indicator = self.calculate_optimal_indicator(acetic_acid);
        
        // Generate time series forecast
        let time_series_forecast = self.generate_time_series_forecast().await;
        
        // Create the reading data
        let reading_data = AceticAcidData {
            acetic_acid_concentration: acetic_acid,
            vinegar_conversion_ratio,
            sourness_score,
            conversion_rate,
            days_to_full_conversion,
            vinegar_maturity,
            optimal_range_indicator,
            time_series_forecast,
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
            sensor_type: "acetic_acid".to_string(),
            timestamp: secure_time,
            data: secure_data,
            error_correction,
            verification_hash: blake3_hash(&format!(
                "{}:{}:{}:{}",
                reading_id,
                secure_time,
                acetic_acid,
                vinegar_conversion_ratio
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
            "{}:{}:{}:{}",
            reading.id,
            reading.timestamp,
            reading.data.get_data().acetic_acid_concentration,
            reading.data.get_data().vinegar_conversion_ratio
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
                "Acetic acid reading failed verification".to_string()
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
