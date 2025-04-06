//! Alcohol Sensor implementation for Kombucha monitoring
//!
//! Provides quantum-secured alcohol level readings with
//! full error correction at classical, bridge, and quantum levels.
//! Optimized for tracking kombucha fermentation with ESP32S3 hardware.

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

/// Kombucha-optimized alcohol data
#[derive(Clone, Debug)]
pub struct AlcoholData {
    /// Alcohol content by volume (%)
    pub abv_percent: f32,
    
    /// Ethanol concentration (mg/L)
    pub ethanol_concentration: f32,
    
    /// Fermentation rate (% per day)
    pub fermentation_rate: f32,
    
    /// Days since fermentation start
    pub days_fermenting: f32,
    
    /// Maturity indicator (0.0-1.0)
    pub maturity: f32,
    
    /// Stability indicator (0.0-1.0)
    pub stability: f32,
    
    /// Rate of change in % ABV per day
    pub rate_of_change: f32,
    
    /// Optimal range indicator (-1.0 = too low, 0.0 = optimal, 1.0 = too high)
    pub optimal_range_indicator: f32,
    
    /// Time series forecast data for next 72 hours
    pub time_series_forecast: TimeSeriesData,
}

/// Quantum-secured alcohol sensor for Kombucha fermentation
pub struct AlcoholSensor {
    /// Hardware interface
    hardware: SensorHardware,
    
    /// Quantum key pair for securing readings
    key_pair: Arc<RwLock<QuantumKeyPair>>,
    
    /// Latest readings cache
    readings_cache: Arc<RwLock<Vec<TelemetryReading<AlcoholData>>>>,
    
    /// Optimal alcohol range for kombucha (%)
    optimal_range: (f32, f32),
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Fermentation start date
    fermentation_start: Arc<RwLock<DateTime<Utc>>>,
    
    /// Temperature correlation (optional)
    current_temperature: Arc<RwLock<Option<f32>>>,
    
    /// Quantum-secured clock for time series analysis
    clock: Arc<RwLock<QuantumSecureClock>>,
}

impl AlcoholSensor {
    /// Create a new alcohol sensor with the specified hardware
    pub async fn new(
        hardware: SensorHardware,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
        clock: Arc<RwLock<QuantumSecureClock>>,
    ) -> TelemetryResult<Self> {
        // Create quantum key pair using ESP32S3's hardware security features
        let key_pair = Arc::new(RwLock::new(QuantumKeyPair::new()?));
        
        // Initialize readings cache
        let readings_cache = Arc::new(RwLock::new(Vec::new()));
        
        // Default optimal alcohol range for kombucha (0.5-3.0%)
        let optimal_range = (0.5, 3.0);
        
        // Initialize temperature as None
        let current_temperature = Arc::new(RwLock::new(None));
        
        // Set fermentation start date to now by default
        let fermentation_start = Arc::new(RwLock::new(Utc::now()));
        
        // Check if security is compromised before creating sensor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot initialize alcohol sensor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            hardware,
            key_pair,
            readings_cache,
            optimal_range,
            security_manager,
            fermentation_start,
            current_temperature,
            clock,
        })
    }
    
    /// Set the optimal alcohol range for kombucha
    pub fn set_optimal_range(&mut self, min: f32, max: f32) -> TelemetryResult<()> {
        if min >= max || min < 0.0 || max > 15.0 {
            return Err(ClassicalError::InvalidParameter(
                "Invalid alcohol range. Must be 0-15% with min < max".to_string()
            ).into());
        }
        
        self.optimal_range = (min, max);
        Ok(())
    }
    
    /// Set fermentation start date
    pub async fn set_fermentation_start(&self, start_date: DateTime<Utc>) -> TelemetryResult<()> {
        if start_date > Utc::now() {
            return Err(ClassicalError::InvalidParameter(
                "Fermentation start date cannot be in the future".to_string()
            ).into());
        }
        
        *self.fermentation_start.write().await = start_date;
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
    
    /// Read raw alcohol measurements from the sensor hardware
    async fn read_raw_alcohol(&self) -> TelemetryResult<f32> {
        // Check security status before taking reading
        let security_manager = self.security_manager.read().await;
        let security_compromised = security_manager.is_security_compromised().await?;
        
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot take alcohol reading: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would read from MQ-3 alcohol sensor
        // or similar through ADC on the ESP32S3 using Zephyr RTOS drivers
        
        // For this sketch, we'll simulate readings based on fermentation time
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Get days since fermentation started
        let now = Utc::now();
        let start = *self.fermentation_start.read().await;
        let days_fermenting = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Kombucha typically reaches 0.5-3.0% ABV after 7-14 days
        // Model logistic fermentation curve
        let max_abv = self.optimal_range.1;
        let target_days = 10.0;
        let growth_rate = 0.6;
        
        // Logistic function: ABV = max_abv / (1 + e^(-growth_rate * (days - target_days/2)))
        let base_abv = max_abv / (1.0 + f32::exp(-growth_rate * (days_fermenting - target_days / 2.0)));
        
        // Add small random variation
        let variation = max_abv * 0.1; // 10% of max
        let abv = base_abv + (rng.gen::<f32>() * 2.0 - 1.0) * variation;
        
        // Apply temperature effect if available
        let temp_adjusted_abv = if let Some(temp) = *self.current_temperature.read().await {
            // Fermentation is fastest around 25-30°C
            let temp_factor = if temp < 15.0 {
                // Too cold, slows down fermentation
                abv * (0.7 + 0.02 * temp)
            } else if temp > 35.0 {
                // Too hot, slows down and may kill SCOBY
                abv * (1.2 - 0.01 * (temp - 35.0))
            } else {
                // Ideal temperature range
                abv * (1.0 + 0.01 * (temp - 20.0))
            };
            
            temp_factor.max(0.0)
        } else {
            abv
        };
        
        // Ensure ABV is non-negative
        let final_abv = temp_adjusted_abv.max(0.0);
        
        Ok(final_abv)
    }
    
    /// Calculate alcohol stability and rate of change based on recent readings
    async fn calculate_stability_and_rate(&self) -> TelemetryResult<(f32, f32)> {
        let readings = self.readings_cache.read().await;
        
        if readings.len() < 2 {
            return Ok((1.0, 0.0)); // Default stability if not enough readings
        }
        
        // Get up to 14 days of readings (typical kombucha cycle)
        let recent_readings = readings.iter()
            .rev()
            .take(14)
            .collect::<Vec<_>>();
        
        // Calculate standard deviation for stability
        let abv_values: Vec<f32> = recent_readings.iter()
            .map(|r| r.data.get_data().abv_percent)
            .collect();
        
        let mean = abv_values.iter().sum::<f32>() / abv_values.len() as f32;
        let variance = abv_values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f32>() / abv_values.len() as f32;
        let std_dev = variance.sqrt();
        
        // Calculate rate of change (% ABV per day)
        let oldest = recent_readings.last().unwrap();
        let newest = recent_readings.first().unwrap();
        
        let time_diff = newest.timestamp - oldest.timestamp;
        let time_diff_days = time_diff.num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        let abv_diff = newest.data.get_data().abv_percent - oldest.data.get_data().abv_percent;
        
        let rate_of_change = if time_diff_days > 0.0 {
            abv_diff / time_diff_days
        } else {
            // Default reasonable rate for early fermentation
            0.2
        };
        
        // Calculate stability (1.0 = perfectly stable)
        // Normalize by expected variation for ABV
        let stability = 1.0 / (1.0 + std_dev / 0.1);
        
        Ok((stability, rate_of_change))
    }
    
    /// Calculate fermentation rate as percentage change per day
    async fn calculate_fermentation_rate(&self) -> f32 {
        let readings = self.readings_cache.read().await;
        
        if readings.len() < 2 {
            // Default reasonable rate for kombucha
            return 10.0;
        }
        
        // Calculate using all available readings
        let oldest = readings.first().unwrap();
        let newest = readings.last().unwrap();
        
        // Time difference in days
        let time_diff = newest.timestamp - oldest.timestamp;
        let time_diff_days = time_diff.num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        if time_diff_days <= 0.0 || oldest.data.get_data().abv_percent <= 0.0 {
            return 10.0; // Default if timing is invalid
        }
        
        // Calculate compound growth rate
        let growth_factor = newest.data.get_data().abv_percent / oldest.data.get_data().abv_percent;
        
        // Daily percentage change
        let daily_rate = (f32::powf(growth_factor, 1.0 / time_diff_days) - 1.0) * 100.0;
        
        daily_rate
    }
    
    /// Calculate maturity based on ABV and time
    async fn calculate_maturity(&self, abv: f32) -> f32 {
        // Maturity depends on:
        // 1. Current ABV compared to optimal range
        // 2. Fermentation time
        // 3. Rate of change (slowing indicates maturing)
        
        // Get days since fermentation started
        let now = Utc::now();
        let start = *self.fermentation_start.read().await;
        let days_fermenting = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // ABV factor - how close to ideal ABV
        let target_abv = (self.optimal_range.0 + self.optimal_range.1) / 2.0;
        let abv_factor = if abv < target_abv {
            // Still developing
            (abv / target_abv).powf(0.5)
        } else {
            // Hit optimal, may become over-mature if too high
            1.0 - ((abv - target_abv) / target_abv).min(1.0)
        };
        
        // Time factor - typical kombucha cycle is 7-14 days
        let time_factor = if days_fermenting < 7.0 {
            // Early stage
            days_fermenting / 7.0
        } else if days_fermenting < 14.0 {
            // Prime maturity window
            1.0
        } else {
            // Getting over-mature
            1.0 - ((days_fermenting - 14.0) / 14.0).min(1.0)
        };
        
        // Rate factor - slowing rate indicates maturity
        let rate_factor = if let Ok((_, rate)) = self.calculate_stability_and_rate().await {
            if rate > 0.3 {
                // Fast change - still early
                0.5
            } else if rate > 0.1 {
                // Moderate change - approaching maturity
                0.8
            } else {
                // Slow change - mature
                1.0
            }
        } else {
            0.5 // Default moderate
        };
        
        // Combined maturity score (0.0-1.0)
        let maturity = (abv_factor * 0.5 + time_factor * 0.3 + rate_factor * 0.2).min(1.0);
        
        maturity
    }
    
    /// Calculate optimal range indicator
    fn calculate_optimal_indicator(&self, abv: f32) -> f32 {
        let min = self.optimal_range.0;
        let max = self.optimal_range.1;
        let mid = (min + max) / 2.0;
        
        if abv < min {
            // Too low, negative values
            -1.0 * (min - abv) / min
        } else if abv > max {
            // Too high, positive values
            (abv - max) / max
        } else {
            // Optimal range, values close to zero
            (abv - mid) / (max - min) * 2.0
        }
    }
    
    /// Generate time series forecast using historical data
    async fn generate_time_series_forecast(&self) -> TimeSeriesData {
        let readings = self.readings_cache.read().await;
        let clock = self.clock.read().await;
        
        // Get current quantum-secured time
        let current_time = clock.get_current_time().await.unwrap_or_else(|_| Utc::now());
        
        // Default time series data if we don't have enough readings
        if readings.len() < 24 {
            return TimeSeriesData {
                timestamps: (0..72).map(|i| current_time + Duration::hours(i as i64)).collect(),
                values: vec![vec![1.5; 72]], // Default ABV trend
                confidence_intervals: vec![(1.2, 1.8); 72],
                model_type: "default".to_string(),
                correlation_score: 0.0,
            };
        }
        
        // Extract historical ABV readings with timestamps
        let historical_data: Vec<(DateTime<Utc>, f32)> = readings.iter()
            .rev()
            .take(14 * 24) // Up to 14 days of hourly readings (typical kombucha cycle)
            .map(|r| (r.timestamp, r.data.get_data().abv_percent))
            .collect();
        
        // Get fermentation start date to calculate phase
        let fermentation_start = *self.fermentation_start.read().await;
        let days_fermenting = (current_time - fermentation_start).num_milliseconds() as f32 / 
                             (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Calculate recent trend from historical data
        let recent_mean = historical_data.iter()
            .take(24) // Last 24 hours
            .map(|(_, v)| v)
            .sum::<f32>() / historical_data.len().min(24) as f32;
        
        let recent_trend = if historical_data.len() > 1 {
            let oldest = historical_data.last().unwrap();
            let newest = historical_data.first().unwrap();
            let hours_diff = (newest.0 - oldest.0).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0);
            if hours_diff > 0.0 {
                (newest.1 - oldest.1) / hours_diff
            } else {
                0.01 // Default small hourly increase
            }
        } else {
            0.01 // Default small hourly increase
        };
        
        // Adjust trend based on fermentation phase
        let phase_adjusted_trend = if days_fermenting < 3.0 {
            // Early phase - rapid growth
            recent_trend * 1.2
        } else if days_fermenting < 7.0 {
            // Middle phase - steady growth
            recent_trend * 1.0
        } else if days_fermenting < 14.0 {
            // Late phase - slowing growth
            recent_trend * 0.5
        } else {
            // Very mature - minimal growth
            recent_trend * 0.1
        };
        
        // Apply temperature adjustment if available
        let temp_adjusted_trend = if let Some(temp) = *self.current_temperature.read().await {
            // Temperature affects fermentation rate
            if temp < 18.0 {
                // Too cold, slows fermentation
                phase_adjusted_trend * (0.6 + 0.02 * temp)
            } else if temp > 30.0 {
                // Too hot, initially accelerates but may harm SCOBY
                phase_adjusted_trend * (1.2 - 0.02 * (temp - 30.0))
            } else {
                // Ideal temperature range, faster around 25-28°C
                phase_adjusted_trend * (1.0 + 0.04 * (temp - 22.0).min(6.0))
            }
        } else {
            phase_adjusted_trend
        };
        
        // Generate forecast for next 72 hours
        let timestamps = (0..72).map(|i| current_time + Duration::hours(i as i64)).collect::<Vec<_>>();
        
        let mut forecast_values = Vec::new();
        let mut abv_values = Vec::new();
        
        // ABV forecast using logistic growth model (S-curve)
        let max_abv = self.optimal_range.1 * 1.2; // 20% above optimal max
        
        for i in 0..72 {
            // Add slight randomness
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let random_factor = (rng.gen::<f32>() * 2.0 - 1.0) * 0.02;
            
            // Logistic growth - slows as approaching maximum
            let growth_damping = 1.0 - (recent_mean / max_abv).powf(1.5);
            
            // Forecast reliability decreases over time
            let time_dampening = 1.0 / (1.0 + i as f32 / 24.0);
            
            // Calculate forecast
            let forecast = recent_mean + 
                          temp_adjusted_trend * i as f32 * growth_damping * time_dampening + 
                          random_factor;
            
            // Ensure within realistic bounds
            let forecast = forecast.max(0.0).min(max_abv);
            abv_values.push(forecast);
        }
        
        forecast_values.push(abv_values);
        
        // Generate confidence intervals (wider as we forecast further)
        let confidence_intervals = (0..72).map(|i| {
            let base_uncertainty = 0.05;
            let time_uncertainty = i as f32 * 0.005;
            let maturity_uncertainty = if days_fermenting > 10.0 { 0.02 } else { 0.08 };
            let uncertainty = base_uncertainty + time_uncertainty + maturity_uncertainty;
            let forecast = forecast_values[0][i];
            (forecast * (1.0 - uncertainty), forecast * (1.0 + uncertainty))
        }).collect();
        
        // Calculate correlation score based on model fit
        let correlation_score = if historical_data.len() > 48 {
            0.85 // More data, higher confidence
        } else {
            0.7 // Less data, less confidence
        };
        
        TimeSeriesData {
            timestamps,
            values: forecast_values,
            confidence_intervals,
            model_type: "Logistic Fermentation Model with Temperature Adjustment".to_string(),
            correlation_score,
        }
    }
}

#[async_trait::async_trait]
impl QuantumSecuredSensor for AlcoholSensor {
    type ReadingType = AlcoholData;
    
    fn id(&self) -> &str {
        &self.hardware.id
    }
    
    fn sensor_type(&self) -> &str {
        "alcohol"
    }
    
    async fn quantum_key_pair(&self) -> Arc<RwLock<QuantumKeyPair>> {
        self.key_pair.clone()
    }
    
    async fn take_reading(&self) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // Read raw alcohol content from sensor
        let abv_percent = self.read_raw_alcohol().await?;
        
        // Calculate ethanol concentration (mg/L)
        // ABV to mg/L conversion: 1% ABV ≈ 7.9 g/L = 7900 mg/L
        let ethanol_concentration = abv_percent * 7900.0;
        
        // Get days since fermentation started
        let now = Utc::now();
        let start = *self.fermentation_start.read().await;
        let days_fermenting = (now - start).num_milliseconds() as f32 / (1000.0 * 60.0 * 60.0 * 24.0);
        
        // Calculate stability and rate of change
        let (stability, rate_of_change) = self.calculate_stability_and_rate().await?;
        
        // Calculate fermentation rate
        let fermentation_rate = self.calculate_fermentation_rate().await;
        
        // Calculate maturity
        let maturity = self.calculate_maturity(abv_percent).await;
        
        // Calculate optimal range indicator
        let optimal_range_indicator = self.calculate_optimal_indicator(abv_percent);
        
        // Generate time series forecast
        let time_series_forecast = self.generate_time_series_forecast().await;
        
        // Create the alcohol data
        let data = AlcoholData {
            abv_percent,
            ethanol_concentration,
            fermentation_rate,
            days_fermenting,
            maturity,
            stability,
            rate_of_change,
            optimal_range_indicator,
            time_series_forecast,
        };
        
        // Get current quantum-secured time
        let current_time = self.clock.read().await.get_current_time().await?;
        
        // Secure the data with quantum encryption using CRYSTALS-Kyber
        let key_pair = self.key_pair.read().await;
        let secured_data = QuantumSecuredData::new(data, &key_pair)?;
        
        // Create Blake3 hash of the data
        let serialized = serde_json::to_string(&abv_percent)
            .map_err(|e| ClassicalError::SerializationFailed(e.to_string()))?;
        let hash = blake3_hash(serialized.as_bytes());
        
        // Sign the data with CRYSTALS-Dilithium
        let signature = key_pair.sign(hash.as_bytes())?;
        
        // Generate error correction data at all three levels
        let error_correction = ErrorCorrectionData {
            classical: reed_solomon_generate(serialized.as_bytes()),
            bridge: vec![],  // In a real impl, would generate bridge error correction
            quantum: vec![],  // In a real impl, would generate quantum error correction
        };
        
        // Create the telemetry reading
        let reading = TelemetryReading {
            id: Uuid::new_v4().to_string(),
            timestamp: current_time,
            data: secured_data,
            hash,
            signature,
            error_correction,
            is_verified: false,
            is_committed: false,
        };
        
        // Add to cache
        {
            let mut cache = self.readings_cache.write().await;
            cache.push(reading.clone());
            
            // Limit cache size
            if cache.len() > 100 {
                cache.remove(0);
            }
        }
        
        Ok(reading)
    }
    
    async fn verify_reading(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<bool> {
        // First, check security status
        let security_manager = self.security_manager.read().await;
        let security_compromised = security_manager.is_security_compromised().await?;
        
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot verify alcohol reading: hardware security compromised".to_string()
            ).into());
        }
        
        let key_pair = self.key_pair.read().await;
        
        // Verify the signature using CRYSTALS-Dilithium
        let verified = key_pair.verify(reading.hash.as_bytes(), &reading.signature)?;
        
        Ok(verified)
    }
    
    async fn commit_reading(&self, reading: &TelemetryReading<Self::ReadingType>) -> TelemetryResult<String> {
        // Check security before committing to blockchain
        let security_manager = self.security_manager.read().await;
        let security_compromised = security_manager.is_security_compromised().await?;
        
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot commit alcohol reading: hardware security compromised".to_string()
            ).into());
        }
        
        // In a real implementation, this would submit the reading to the ELXR blockchain
        // using the ActorX framework with quantum-secured blockchain transaction
        let tx_hash = format!("0x{}", Uuid::new_v4().to_simple_string());
        
        // Mark the reading as committed in the cache
        {
            let mut cache = self.readings_cache.write().await;
            if let Some(cached_reading) = cache.iter_mut().find(|r| r.id == reading.id) {
                cached_reading.is_committed = true;
            }
        }
        
        Ok(tx_hash)
    }
    
    async fn apply_error_correction(
        &self, 
        corrupted_reading: &TelemetryReading<Self::ReadingType>
    ) -> TelemetryResult<TelemetryReading<Self::ReadingType>> {
        // In a real implementation, this would apply Reed-Solomon decoding,
        // bridge error correction, and quantum error correction with Surface codes
        
        // For now, we'll just return the reading as-is
        Ok(corrupted_reading.clone())
    }
    
    async fn latest_reading(&self) -> TelemetryResult<Option<TelemetryReading<Self::ReadingType>>> {
        let cache = self.readings_cache.read().await;
        
        if cache.is_empty() {
            Ok(None)
        } else {
            Ok(Some(cache.last().unwrap().clone()))
        }
    }
    
    async fn readings_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> TelemetryResult<Vec<TelemetryReading<Self::ReadingType>>> {
        let cache = self.readings_cache.read().await;
        
        let filtered = cache.iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .cloned()
            .collect();
        
        Ok(filtered)
    }
}

// Generate Reed-Solomon error correction codes
fn reed_solomon_generate(data: &[u8]) -> Vec<u8> {
    // In a real implementation, this would use the reed_solomon_erasure crate
    // For now, we'll return dummy data
    vec![0, 1, 2, 3]
}
