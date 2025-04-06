//! Kombucha Fermentation Monitoring System
//!
//! Integrates quantum-secured sensors with the ActorX framework 
//! for comprehensive kombucha monitoring. Provides time-series 
//! analysis, alerting, and secure blockchain submission.

use crate::actorx_integration::{
    AlcoholSensorActor, TurbiditySensorActor, ElxrSensorActorFactory,
    AlcoholReadingSubmission, TurbidityReadingSubmission
};
use crate::sensor::{
    alcohol::{AlcoholSensor, AlcoholData},
    turbidity::{TurbiditySensor, TurbidityData},
    temperature::{TemperatureSensor, TemperatureData},
    brix::{BrixSensor, BrixData},
    scoby_health::{ScobyHealthSensor, ScobyHealthData},
    yeast_activity::{YeastActivitySensor, YeastActivityData},
    gluconic_acid::{GluconicAcidSensor, GluconicAcidData},
    ethyl_acetate::{EthylAcetateSensor, EthylAcetateData},
    dissolved_oxygen::{DissolvedOxygenSensor, DissolvedOxygenData},
    carbon_dioxide::{CarbonDioxideSensor, CarbonDioxideData},
    microbial_balance::{MicrobialBalanceSensor, MicrobialBalanceData},
    core::{QuantumSecuredSensor, TelemetryReading, TimeSeriesData}
};
use crate::crypto::QuantumCrypto;
use crate::error::{TelemetryResult, ClassicalError, BridgeError, QuantumError};
use crate::hardware::security::HardwareSecurityManager;
use crate::time::QuantumSecureClock;

use actorx_frameworks::core::{
    actor::{Actor, ActorId, ActorRegistration},
    actor_token::{ActorToken, TokenAmount, TokenOperation},
    chain::{ChainContext, ChainSubmission, SubmissionResult, SubmissionStatus},
    error::ActorError,
    msg::{Message, MessageStatus, SignedMessage}
};

use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc, Duration};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tokio::time;

/// Threshold configuration for monitoring alerts
#[derive(Clone, Debug)]
pub struct MonitoringThresholds {
    /// Minimum acceptable alcohol content (ABV %)
    pub min_abv: f32,
    
    /// Maximum acceptable alcohol content (ABV %)
    pub max_abv: f32,
    
    /// Minimum acceptable turbidity (NTU)
    pub min_turbidity: f32,
    
    /// Maximum acceptable turbidity (NTU)
    pub max_turbidity: f32,
    
    /// Maximum rate of change for ABV (% per day)
    pub max_abv_change_rate: f32,
    
    /// Maximum rate of change for turbidity (NTU per day)
    pub max_turbidity_change_rate: f32,
    
    /// Minimum fermentation progress (0.0-1.0)
    pub min_fermentation_progress: f32,
    
    /// Minimum acceptable temperature (°C)
    pub min_temperature: f32,
    
    /// Maximum acceptable temperature (°C)
    pub max_temperature: f32,
    
    /// Minimum acceptable brix/sugar content
    pub min_brix: f32,
    
    /// Maximum acceptable brix/sugar content
    pub max_brix: f32,
    
    /// Minimum SCOBY health score (0.0-1.0)
    pub min_scoby_health: f32,
    
    /// Minimum yeast activity level (0.0-1.0)
    pub min_yeast_activity: f32,
    
    /// Maximum yeast activity level (0.0-1.0)
    pub max_yeast_activity: f32,
    
    /// Maximum gluconic acid concentration (g/L)
    pub max_gluconic_acid: f32,
    
    /// Maximum ethyl acetate concentration (ppm)
    pub max_ethyl_acetate: f32,
    
    /// Minimum dissolved oxygen (mg/L)
    pub min_dissolved_oxygen: f32,
    
    /// Maximum dissolved oxygen (mg/L)
    pub max_dissolved_oxygen: f32,
    
    /// Maximum carbon dioxide (ppm)
    pub max_carbon_dioxide: f32,
    
    /// Minimum bacteria-yeast ratio
    pub min_bacteria_yeast_ratio: f32,
    
    /// Maximum bacteria-yeast ratio
    pub max_bacteria_yeast_ratio: f32,
    
    /// Minimum culture health score (0.0-1.0)
    pub min_culture_health: f32,
}

impl Default for MonitoringThresholds {
    fn default() -> Self {
        Self {
            min_abv: 0.5,
            max_abv: 3.0,
            min_turbidity: 5.0,
            max_turbidity: 25.0,
            max_abv_change_rate: 0.5,
            max_turbidity_change_rate: 8.0,
            min_fermentation_progress: 0.1,
            min_temperature: 15.0,
            max_temperature: 25.0,
            min_brix: 1.0,
            max_brix: 20.0,
            min_scoby_health: 0.5,
            min_yeast_activity: 0.2,
            max_yeast_activity: 0.8,
            max_gluconic_acid: 1.5,
            max_ethyl_acetate: 50.0,
            min_dissolved_oxygen: 2.0,
            max_dissolved_oxygen: 8.0,
            max_carbon_dioxide: 1000.0,
            min_bacteria_yeast_ratio: 1.0,
            max_bacteria_yeast_ratio: 5.0,
            min_culture_health: 0.5,
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    
    /// Warning alert
    Warning,
    
    /// Critical alert
    Critical,
}

/// Monitoring alert for abnormal fermentation conditions
#[derive(Debug, Clone)]
pub struct FermentationAlert {
    /// Alert ID
    pub id: String,
    
    /// Alert timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Alert severity
    pub severity: AlertSeverity,
    
    /// Alert message
    pub message: String,
    
    /// Sensor reading that triggered the alert
    pub sensor_reading_id: String,
    
    /// Sensor type that triggered the alert
    pub sensor_type: String,
    
    /// Measured value that triggered the alert
    pub measured_value: f32,
    
    /// Threshold value that was exceeded
    pub threshold_value: f32,
    
    /// Recommended action
    pub recommended_action: String,
}

/// Combined reading data from all sensors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombinedFermentationData {
    /// Reading ID (combined)
    pub id: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Alcohol content (ABV %)
    pub abv_percent: f32,
    
    /// Turbidity (NTU)
    pub turbidity_ntu: f32,
    
    /// Temperature (°C)
    pub temperature: f32,
    
    /// Sugar content (Brix)
    pub brix: f32,
    
    /// SCOBY health score (0.0-1.0)
    pub scoby_health: f32,
    
    /// Yeast activity level (0.0-1.0)
    pub yeast_activity: f32,
    
    /// Gluconic acid concentration (g/L)
    pub gluconic_acid: f32,
    
    /// Ethyl acetate concentration (ppm)
    pub ethyl_acetate: f32,
    
    /// Dissolved oxygen (mg/L)
    pub dissolved_oxygen: f32,
    
    /// Carbon dioxide (ppm)
    pub carbon_dioxide: f32,
    
    /// Bacteria-to-yeast ratio
    pub bacteria_yeast_ratio: f32,
    
    /// Culture health score (0.0-1.0)
    pub culture_health: f32,
    
    /// Clarity score (0-100)
    pub clarity_score: f32,
    
    /// Fermentation progress (0.0-1.0)
    pub fermentation_progress: f32,
    
    /// Days fermenting
    pub days_fermenting: f32,
    
    /// Maturity score (0.0-1.0)
    pub maturity: f32,
    
    /// Stability indicator (0.0-1.0)
    pub stability: f32,
    
    /// Estimated completion time
    pub estimated_completion_time: DateTime<Utc>,
    
    /// Quantum security verification hash (combined)
    pub verification_hash: String,
}

/// Integrated kombucha fermentation monitor
pub struct KombuchaMonitor {
    /// Alcohol sensor actor
    alcohol_actor: Arc<AlcoholSensorActor>,
    
    /// Turbidity sensor actor
    turbidity_actor: Arc<TurbiditySensorActor>,
    
    /// Alcohol sensor
    alcohol_sensor: Arc<AlcoholSensor>,
    
    /// Turbidity sensor
    turbidity_sensor: Arc<TurbiditySensor>,
    
    /// Temperature sensor
    temperature_sensor: Arc<TemperatureSensor>,
    
    /// Brix/sugar content sensor
    brix_sensor: Arc<BrixSensor>,
    
    /// SCOBY health sensor
    scoby_health_sensor: Arc<ScobyHealthSensor>,
    
    /// Yeast activity sensor
    yeast_activity_sensor: Arc<YeastActivitySensor>,
    
    /// Gluconic acid sensor
    gluconic_acid_sensor: Arc<GluconicAcidSensor>,
    
    /// Ethyl acetate sensor
    ethyl_acetate_sensor: Arc<EthylAcetateSensor>,
    
    /// Dissolved oxygen sensor
    dissolved_oxygen_sensor: Arc<DissolvedOxygenSensor>,
    
    /// Carbon dioxide sensor
    carbon_dioxide_sensor: Arc<CarbonDioxideSensor>,
    
    /// Microbial balance sensor
    microbial_balance_sensor: Arc<MicrobialBalanceSensor>,
    
    /// Monitoring thresholds for alerts
    thresholds: MonitoringThresholds,
    
    /// Hardware security manager
    security_manager: Arc<HardwareSecurityManager>,
    
    /// Quantum cryptography module
    quantum_crypto: Arc<QuantumCrypto>,
    
    /// Secure clock
    secure_clock: Arc<QuantumSecureClock>,
    
    /// Historical readings (last 30 days)
    history: Arc<RwLock<Vec<CombinedFermentationData>>>,
    
    /// Active alerts
    alerts: Arc<RwLock<HashMap<String, FermentationAlert>>>,
    
    /// Batch identifier for this fermentation process
    batch_id: String,
    
    /// Start date of the fermentation
    start_date: DateTime<Utc>,
}

/// Fermentation status enum
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FermentationStatus {
    /// Initializing sensors and monitoring
    Initializing,
    
    /// Primary fermentation stage
    PrimaryFermentation,
    
    /// Secondary fermentation stage
    SecondaryFermentation,
    
    /// Fermentation is complete
    Complete,
    
    /// Error state
    Error,
}

impl KombuchaMonitor {
    /// Create a new kombucha fermentation monitor
    pub async fn new(
        alcohol_actor: Arc<AlcoholSensorActor>,
        turbidity_actor: Arc<TurbiditySensorActor>,
        alcohol_sensor: Arc<AlcoholSensor>,
        turbidity_sensor: Arc<TurbiditySensor>,
        temperature_sensor: Arc<TemperatureSensor>,
        brix_sensor: Arc<BrixSensor>,
        scoby_health_sensor: Arc<ScobyHealthSensor>,
        yeast_activity_sensor: Arc<YeastActivitySensor>,
        gluconic_acid_sensor: Arc<GluconicAcidSensor>,
        ethyl_acetate_sensor: Arc<EthylAcetateSensor>,
        dissolved_oxygen_sensor: Arc<DissolvedOxygenSensor>,
        carbon_dioxide_sensor: Arc<CarbonDioxideSensor>,
        microbial_balance_sensor: Arc<MicrobialBalanceSensor>,
        quantum_crypto: Arc<QuantumCrypto>,
        security_manager: Arc<HardwareSecurityManager>,
        secure_clock: Arc<QuantumSecureClock>,
    ) -> TelemetryResult<Self> {
        // Verify security status before creating monitor
        let security_compromised = security_manager.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot create kombucha monitor: hardware security compromised".to_string()
            ).into());
        }
        
        Ok(Self {
            alcohol_actor,
            turbidity_actor,
            alcohol_sensor,
            turbidity_sensor,
            temperature_sensor,
            brix_sensor,
            scoby_health_sensor,
            yeast_activity_sensor,
            gluconic_acid_sensor,
            ethyl_acetate_sensor,
            dissolved_oxygen_sensor,
            carbon_dioxide_sensor,
            microbial_balance_sensor,
            quantum_crypto,
            security_manager,
            secure_clock,
            thresholds: MonitoringThresholds::default(),
            history: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(HashMap::new())),
            batch_id: "default-batch".to_string(),
            start_date: Utc::now(),
        })
    }
    
    /// Factory function to create a fully configured kombucha monitor
    pub async fn create_kombucha_monitor(
        batch_id: Option<String>,
        thresholds: Option<MonitoringThresholds>,
    ) -> TelemetryResult<KombuchaMonitor> {
        // Create sensor actors
        let sensor_factory = ElxrSensorActorFactory::new().await?;
        let alcohol_actor = sensor_factory.create_alcohol_sensor_actor().await?;
        let turbidity_actor = sensor_factory.create_turbidity_sensor_actor().await?;
        
        // Create hardware security manager
        let security_manager = Arc::new(HardwareSecurityManager::new().await?);
        
        // Create quantum crypto system
        let quantum_crypto = Arc::new(QuantumCrypto::new().await?);
        
        // Create quantum-secure clock
        let secure_clock = Arc::new(QuantumSecureClock::new().await?);
        
        // Create sensors
        let alcohol_sensor = Arc::new(AlcoholSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let turbidity_sensor = Arc::new(TurbiditySensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let temperature_sensor = Arc::new(TemperatureSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let brix_sensor = Arc::new(BrixSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let scoby_health_sensor = Arc::new(ScobyHealthSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let yeast_activity_sensor = Arc::new(YeastActivitySensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let gluconic_acid_sensor = Arc::new(GluconicAcidSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let ethyl_acetate_sensor = Arc::new(EthylAcetateSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let dissolved_oxygen_sensor = Arc::new(DissolvedOxygenSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let carbon_dioxide_sensor = Arc::new(CarbonDioxideSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        let microbial_balance_sensor = Arc::new(MicrobialBalanceSensor::new(
            quantum_crypto.clone(),
            security_manager.clone(),
            secure_clock.clone(),
        ).await?);
        
        // Create monitor
        let mut monitor = KombuchaMonitor::new(
            alcohol_actor,
            turbidity_actor,
            alcohol_sensor,
            turbidity_sensor,
            temperature_sensor,
            brix_sensor,
            scoby_health_sensor,
            yeast_activity_sensor,
            gluconic_acid_sensor,
            ethyl_acetate_sensor,
            dissolved_oxygen_sensor,
            carbon_dioxide_sensor,
            microbial_balance_sensor,
            quantum_crypto,
            security_manager,
            secure_clock,
        ).await?;
        
        // Set custom batch ID if provided
        if let Some(id) = batch_id {
            monitor.batch_id = id;
        }
        
        // Set custom thresholds if provided
        if let Some(custom_thresholds) = thresholds {
            monitor.thresholds = custom_thresholds;
        }
        
        Ok(monitor)
    }
    
    /// Take readings from all sensors and submit to blockchain
    pub async fn monitor_fermentation(&self) -> TelemetryResult<CombinedFermentationData> {
        // Verify security status
        let security_compromised = self.security_manager.is_security_compromised().await?;
        if security_compromised {
            return Err(ClassicalError::SecurityCompromised(
                "Cannot monitor fermentation: hardware security compromised".to_string()
            ));
        }
        
        // Take alcohol reading
        let alcohol_reading = self.alcohol_sensor.take_reading().await?;
        let abv_percent = alcohol_reading.data.abv_percent;
        
        // Take turbidity reading
        let turbidity_reading = self.turbidity_sensor.take_reading().await?;
        let turbidity_ntu = turbidity_reading.data.turbidity_ntu;
        
        // Take temperature reading
        let temperature_reading = self.temperature_sensor.take_reading().await?;
        let temperature = temperature_reading.data.temperature_celsius;
        
        // Take brix/sugar content reading
        let brix_reading = self.brix_sensor.take_reading().await?;
        let brix = brix_reading.data.brix_value;
        
        // Take SCOBY health reading
        let scoby_health_reading = self.scoby_health_sensor.take_reading().await?;
        let scoby_health = scoby_health_reading.data.health_score;
        
        // Take yeast activity reading
        let yeast_activity_reading = self.yeast_activity_sensor.take_reading().await?;
        let yeast_activity = yeast_activity_reading.data.activity_level;
        
        // Take gluconic acid reading
        let gluconic_acid_reading = self.gluconic_acid_sensor.take_reading().await?;
        let gluconic_acid = gluconic_acid_reading.data.concentration;
        
        // Take ethyl acetate reading
        let ethyl_acetate_reading = self.ethyl_acetate_sensor.take_reading().await?;
        let ethyl_acetate = ethyl_acetate_reading.data.concentration;
        
        // Take dissolved oxygen reading
        let dissolved_oxygen_reading = self.dissolved_oxygen_sensor.take_reading().await?;
        let dissolved_oxygen = dissolved_oxygen_reading.data.do_concentration;
        
        // Take carbon dioxide reading
        let carbon_dioxide_reading = self.carbon_dioxide_sensor.take_reading().await?;
        let carbon_dioxide = carbon_dioxide_reading.data.co2_ppm;
        
        // Take microbial balance reading
        let microbial_balance_reading = self.microbial_balance_sensor.take_reading().await?;
        let bacteria_yeast_ratio = microbial_balance_reading.data.bacteria_yeast_ratio;
        let culture_health = microbial_balance_reading.data.culture_health;
        
        // Calculate derived metrics
        let fermentation_progress = self.calculate_fermentation_progress(
            abv_percent,
            bacteria_yeast_ratio,
            brix,
            temperature,
            carbon_dioxide
        );
        
        // Calculate clarity score (0-100)
        let clarity_score = 100.0 - (turbidity_ntu * 4.0).min(100.0).max(0.0);
        
        // Calculate days fermenting
        let now = Utc::now();
        let days_fermenting = (now - self.start_date).num_seconds() as f32 / 86400.0;
        
        // Calculate maturity score
        let maturity = self.calculate_maturity_score(
            days_fermenting,
            fermentation_progress,
            abv_percent,
            brix,
            bacteria_yeast_ratio
        );
        
        // Calculate stability
        let ph_level = self.estimate_ph_from_readings(
            gluconic_acid,
            abv_percent,
            days_fermenting
        );
        
        let stability = self.calculate_stability(
            temperature,
            ph_level,
            dissolved_oxygen,
            carbon_dioxide,
            culture_health
        );
        
        // Estimate completion time
        let estimated_days_remaining = if fermentation_progress > 0.1 {
            (1.0 - fermentation_progress) * 20.0 / fermentation_progress
        } else {
            14.0 // Default estimate if no progress detected
        };
        
        let estimated_completion_time = self.start_date + Duration::seconds((days_fermenting + estimated_days_remaining) as i64 * 86400);
        
        // Create combined verification hash
        let combined_hash = self.quantum_crypto.blake3_hash(&format!(
            "{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{:.2}{}",
            abv_percent,
            turbidity_ntu,
            temperature,
            brix,
            scoby_health,
            yeast_activity,
            gluconic_acid,
            ethyl_acetate,
            dissolved_oxygen,
            carbon_dioxide,
            bacteria_yeast_ratio,
            culture_health,
            now.timestamp()
        )).await?;
        
        // Create combined data structure
        let combined_data = CombinedFermentationData {
            id: format!("combined-{}-{}", self.batch_id, now.timestamp()),
            timestamp: now,
            abv_percent,
            turbidity_ntu,
            temperature,
            brix,
            scoby_health,
            yeast_activity,
            gluconic_acid,
            ethyl_acetate,
            dissolved_oxygen,
            carbon_dioxide,
            bacteria_yeast_ratio,
            culture_health,
            clarity_score,
            fermentation_progress,
            days_fermenting,
            maturity,
            stability,
            estimated_completion_time,
            verification_hash: combined_hash,
        };
        
        // Store combined data
        self.history.write().await.push(combined_data.clone());
        
        // Check thresholds and generate alerts
        self.check_thresholds(&combined_data).await?;
        
        // Submit readings to blockchain via ActorX
        let _ = self.submit_to_blockchain(
            &alcohol_reading,
            &turbidity_reading,
            &combined_data
        ).await?;
        
        Ok(combined_data)
    }
    
    /// Calculate fermentation progress based on multiple factors
    async fn calculate_fermentation_progress(
        &self,
        abv_percent: f32,
        bacteria_yeast_ratio: f32,
        brix: f32,
        temperature: f32,
        carbon_dioxide: f32
    ) -> f32 {
        // Base progress on alcohol content (primary indicator)
        let abv_progress = if abv_percent < self.thresholds.min_abv {
            0.0
        } else if abv_percent >= self.thresholds.max_abv {
            1.0
        } else {
            (abv_percent - self.thresholds.min_abv) / (self.thresholds.max_abv - self.thresholds.min_abv)
        };
        
        // Sugar consumption progress (inverse of brix, as sugar decreases during fermentation)
        let sugar_progress = if brix > self.thresholds.max_brix {
            0.0
        } else if brix <= self.thresholds.min_brix {
            1.0
        } else {
            (self.thresholds.max_brix - brix) / (self.thresholds.max_brix - self.thresholds.min_brix)
        };
        
        // Microbial balance progress (higher ratio = later stage)
        let microbial_progress = if bacteria_yeast_ratio < self.thresholds.min_bacteria_yeast_ratio {
            0.0
        } else if bacteria_yeast_ratio >= self.thresholds.max_bacteria_yeast_ratio {
            1.0
        } else {
            (bacteria_yeast_ratio - self.thresholds.min_bacteria_yeast_ratio) / 
            (self.thresholds.max_bacteria_yeast_ratio - self.thresholds.min_bacteria_yeast_ratio)
        };
        
        // CO2 production as indicator (higher during active fermentation, lower at end)
        let co2_factor = if carbon_dioxide > self.thresholds.max_carbon_dioxide * 0.8 {
            // Still very active fermentation
            0.5
        } else if carbon_dioxide > self.thresholds.max_carbon_dioxide * 0.5 {
            // Mid-stage fermentation
            0.7
        } else if carbon_dioxide > self.thresholds.max_carbon_dioxide * 0.2 {
            // Late-stage fermentation
            0.9
        } else {
            // Finished fermentation
            1.0
        };
        
        // Weighted combination of indicators
        (abv_progress * 0.4) + (sugar_progress * 0.3) + (microbial_progress * 0.2) + (co2_factor * 0.1)
    }
    
    /// Calculate maturity score based on fermentation indicators
    async fn calculate_maturity_score(
        &self,
        days_fermenting: f32,
        fermentation_progress: f32,
        abv_percent: f32,
        brix: f32,
        bacteria_yeast_ratio: f32
    ) -> f32 {
        // Time-based maturity (base factor)
        let time_maturity = if days_fermenting < 3.0 {
            // Very young kombucha
            0.1 + (days_fermenting / 3.0) * 0.1
        } else if days_fermenting < 7.0 {
            // Young kombucha
            0.2 + ((days_fermenting - 3.0) / 4.0) * 0.2
        } else if days_fermenting < 14.0 {
            // Developing kombucha
            0.4 + ((days_fermenting - 7.0) / 7.0) * 0.3
        } else if days_fermenting < 30.0 {
            // Mature kombucha
            0.7 + ((days_fermenting - 14.0) / 16.0) * 0.2
        } else {
            // Very mature/aged kombucha
            0.9 + ((days_fermenting - 30.0) / 30.0).min(0.1)
        };
        
        // Combine with fermentation progress
        let combined = (time_maturity * 0.5) + (fermentation_progress * 0.5);
        
        // Adjust based on flavor development indicators
        let acidity_factor = if brix < self.thresholds.min_brix * 1.5 && abv_percent > self.thresholds.min_abv * 1.5 {
            0.1 // Good acidity development
        } else {
            0.0
        };
        
        let microbial_factor = if bacteria_yeast_ratio > self.thresholds.min_bacteria_yeast_ratio * 1.5 {
            0.1 // Good bacterial profile
        } else {
            0.0
        };
        
        // Final score capped at 1.0
        (combined + acidity_factor + microbial_factor).min(1.0)
    }
    
    /// Calculate stability indicator
    async fn calculate_stability(
        &self,
        temperature: f32,
        ph_level: f32,
        dissolved_oxygen: f32,
        carbon_dioxide: f32,
        culture_health: f32
    ) -> f32 {
        // Temperature stability (ideal range 20-24°C)
        let temp_stability = if temperature < 18.0 || temperature > 26.0 {
            0.5 // Less stable
        } else if temperature < 20.0 || temperature > 24.0 {
            0.8 // Moderately stable
        } else {
            1.0 // Highly stable
        };
        
        // pH stability (ideal range 2.8-3.5)
        let ph_stability = if ph_level < 2.5 || ph_level > 4.0 {
            0.4 // Less stable
        } else if ph_level < 2.8 || ph_level > 3.5 {
            0.7 // Moderately stable
        } else {
            1.0 // Highly stable
        };
        
        // Oxygen stability (lower is generally more stable for finished kombucha)
        let oxygen_stability = if dissolved_oxygen > self.thresholds.max_dissolved_oxygen {
            0.5 // Less stable
        } else if dissolved_oxygen > self.thresholds.min_dissolved_oxygen {
            0.8 // Moderately stable
        } else {
            1.0 // Highly stable
        };
        
        // CO2 stability
        let co2_stability = if carbon_dioxide > self.thresholds.max_carbon_dioxide {
            0.6 // Less stable (over-carbonated)
        } else if carbon_dioxide > self.thresholds.max_carbon_dioxide * 0.75 {
            0.8 // Moderately stable
        } else {
            1.0 // Highly stable
        };
        
        // Weighted stability score
        (temp_stability * 0.3) + 
        (ph_stability * 0.3) + 
        (oxygen_stability * 0.2) + 
        (co2_stability * 0.1) + 
        (culture_health * 0.1)
    }
    
    /// Estimate pH level from available readings
    async fn estimate_ph_from_readings(
        &self,
        gluconic_acid: f32,
        abv_percent: f32,
        days_fermenting: f32
    ) -> f32 {
        // Starting pH is typically around 4.5
        // More gluconic acid and alcohol lead to lower pH
        // pH typically decreases over fermentation time
        
        let base_ph = 4.5;
        let acid_contribution = gluconic_acid * 0.5; // Higher acid = lower pH
        let alcohol_contribution = abv_percent * 0.1; // Alcohol also lowers pH
        let time_contribution = days_fermenting.min(14.0) * 0.02; // pH decreases over time
        
        let estimated_ph = base_ph - acid_contribution - alcohol_contribution - time_contribution;
        
        // Kombucha pH is typically between 2.5 and 3.5 when complete
        estimated_ph.max(2.5).min(4.5)
    }

    /// Check all thresholds and generate alerts if needed
    async fn check_thresholds(&self, data: &CombinedFermentationData) -> TelemetryResult<()> {
        let mut new_alerts = Vec::new();
        
        // Check alcohol content thresholds
        if data.abv_percent < self.thresholds.min_abv {
            new_alerts.push(FermentationAlert {
                id: format!("alert-abv-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Alcohol content too low: {:.2}% (min: {:.2}%)", 
                    data.abv_percent, self.thresholds.min_abv),
                sensor_reading_id: data.id.clone(),
                sensor_type: "alcohol".to_string(),
                measured_value: data.abv_percent,
                threshold_value: self.thresholds.min_abv,
                recommended_action: "Ensure fermentation temperature is in optimal range. Check yeast activity.".to_string(),
            });
        } else if data.abv_percent > self.thresholds.max_abv {
            new_alerts.push(FermentationAlert {
                id: format!("alert-abv-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Alcohol content too high: {:.2}% (max: {:.2}%)", 
                    data.abv_percent, self.thresholds.max_abv),
                sensor_reading_id: data.id.clone(),
                sensor_type: "alcohol".to_string(),
                measured_value: data.abv_percent,
                threshold_value: self.thresholds.max_abv,
                recommended_action: "Consider harvesting kombucha now to prevent over-fermentation.".to_string(),
            });
        }
        
        // Check turbidity thresholds
        if data.turbidity_ntu < self.thresholds.min_turbidity {
            new_alerts.push(FermentationAlert {
                id: format!("alert-turbidity-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Info,
                message: format!("Turbidity too low: {:.2} NTU (min: {:.2} NTU)", 
                    data.turbidity_ntu, self.thresholds.min_turbidity),
                sensor_reading_id: data.id.clone(),
                sensor_type: "turbidity".to_string(),
                measured_value: data.turbidity_ntu,
                threshold_value: self.thresholds.min_turbidity,
                recommended_action: "Kombucha may be over-clarified. Consider bottling now.".to_string(),
            });
        } else if data.turbidity_ntu > self.thresholds.max_turbidity {
            new_alerts.push(FermentationAlert {
                id: format!("alert-turbidity-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Turbidity too high: {:.2} NTU (max: {:.2} NTU)", 
                    data.turbidity_ntu, self.thresholds.max_turbidity),
                sensor_reading_id: data.id.clone(),
                sensor_type: "turbidity".to_string(),
                measured_value: data.turbidity_ntu,
                threshold_value: self.thresholds.max_turbidity,
                recommended_action: "Kombucha is still very cloudy. Continue fermentation or filter if desired.".to_string(),
            });
        }
        
        // Check temperature thresholds
        if data.temperature < self.thresholds.min_temperature {
            new_alerts.push(FermentationAlert {
                id: format!("alert-temperature-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Temperature too low: {:.1}°C (min: {:.1}°C)", 
                    data.temperature, self.thresholds.min_temperature),
                sensor_reading_id: data.id.clone(),
                sensor_type: "temperature".to_string(),
                measured_value: data.temperature,
                threshold_value: self.thresholds.min_temperature,
                recommended_action: "Increase ambient temperature. Fermentation may be stalled.".to_string(),
            });
        } else if data.temperature > self.thresholds.max_temperature {
            new_alerts.push(FermentationAlert {
                id: format!("alert-temperature-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Temperature too high: {:.1}°C (max: {:.1}°C)", 
                    data.temperature, self.thresholds.max_temperature),
                sensor_reading_id: data.id.clone(),
                sensor_type: "temperature".to_string(),
                measured_value: data.temperature,
                threshold_value: self.thresholds.max_temperature,
                recommended_action: "Decrease ambient temperature. High temperatures favor bacteria over yeast.".to_string(),
            });
        }
        
        // Check brix/sugar content thresholds
        if data.brix < self.thresholds.min_brix {
            new_alerts.push(FermentationAlert {
                id: format!("alert-brix-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Sugar content too low: {:.1} Brix (min: {:.1} Brix)", 
                    data.brix, self.thresholds.min_brix),
                sensor_reading_id: data.id.clone(),
                sensor_type: "brix".to_string(),
                measured_value: data.brix,
                threshold_value: self.thresholds.min_brix,
                recommended_action: "Fermentation may be complete. Consider bottling or adding sugar for secondary fermentation.".to_string(),
            });
        } else if data.brix > self.thresholds.max_brix {
            new_alerts.push(FermentationAlert {
                id: format!("alert-brix-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Info,
                message: format!("Sugar content too high: {:.1} Brix (max: {:.1} Brix)", 
                    data.brix, self.thresholds.max_brix),
                sensor_reading_id: data.id.clone(),
                sensor_type: "brix".to_string(),
                measured_value: data.brix,
                threshold_value: self.thresholds.max_brix,
                recommended_action: "Sugar content high. Fermentation likely just beginning.".to_string(),
            });
        }
        
        // Check SCOBY health threshold
        if data.scoby_health < self.thresholds.min_scoby_health {
            new_alerts.push(FermentationAlert {
                id: format!("alert-scoby-health-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Critical,
                message: format!("SCOBY health compromised: {:.2} (min: {:.2})", 
                    data.scoby_health, self.thresholds.min_scoby_health),
                sensor_reading_id: data.id.clone(),
                sensor_type: "scoby_health".to_string(),
                measured_value: data.scoby_health,
                threshold_value: self.thresholds.min_scoby_health,
                recommended_action: "Inspect SCOBY for mold or contamination. May need to restart with fresh culture.".to_string(),
            });
        }
        
        // Check yeast activity thresholds
        if data.yeast_activity < self.thresholds.min_yeast_activity {
            new_alerts.push(FermentationAlert {
                id: format!("alert-yeast-activity-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Yeast activity too low: {:.2} (min: {:.2})", 
                    data.yeast_activity, self.thresholds.min_yeast_activity),
                sensor_reading_id: data.id.clone(),
                sensor_type: "yeast_activity".to_string(),
                measured_value: data.yeast_activity,
                threshold_value: self.thresholds.min_yeast_activity,
                recommended_action: "Check temperature and sugar levels. May need to add yeast nutrients.".to_string(),
            });
        } else if data.yeast_activity > self.thresholds.max_yeast_activity {
            new_alerts.push(FermentationAlert {
                id: format!("alert-yeast-activity-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Info,
                message: format!("Yeast activity very high: {:.2} (max: {:.2})", 
                    data.yeast_activity, self.thresholds.max_yeast_activity),
                sensor_reading_id: data.id.clone(),
                sensor_type: "yeast_activity".to_string(),
                measured_value: data.yeast_activity,
                threshold_value: self.thresholds.max_yeast_activity,
                recommended_action: "Vigorous fermentation in progress. Monitor alcohol levels.".to_string(),
            });
        }
        
        // Check gluconic acid threshold
        if data.gluconic_acid > self.thresholds.max_gluconic_acid {
            new_alerts.push(FermentationAlert {
                id: format!("alert-gluconic-acid-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Gluconic acid too high: {:.2} g/L (max: {:.2} g/L)", 
                    data.gluconic_acid, self.thresholds.max_gluconic_acid),
                sensor_reading_id: data.id.clone(),
                sensor_type: "gluconic_acid".to_string(),
                measured_value: data.gluconic_acid,
                threshold_value: self.thresholds.max_gluconic_acid,
                recommended_action: "Acidity high. Consider harvesting or diluting with fresh sweet tea.".to_string(),
            });
        }
        
        // Check ethyl acetate threshold
        if data.ethyl_acetate > self.thresholds.max_ethyl_acetate {
            new_alerts.push(FermentationAlert {
                id: format!("alert-ethyl-acetate-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Ethyl acetate too high: {:.1} ppm (max: {:.1} ppm)", 
                    data.ethyl_acetate, self.thresholds.max_ethyl_acetate),
                sensor_reading_id: data.id.clone(),
                sensor_type: "ethyl_acetate".to_string(),
                measured_value: data.ethyl_acetate,
                threshold_value: self.thresholds.max_ethyl_acetate,
                recommended_action: "High ester concentration may indicate contamination or over-fermentation.".to_string(),
            });
        }
        
        // Check dissolved oxygen thresholds
        if data.dissolved_oxygen < self.thresholds.min_dissolved_oxygen {
            new_alerts.push(FermentationAlert {
                id: format!("alert-dissolved-oxygen-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Dissolved oxygen too low: {:.2} mg/L (min: {:.2} mg/L)", 
                    data.dissolved_oxygen, self.thresholds.min_dissolved_oxygen),
                sensor_reading_id: data.id.clone(),
                sensor_type: "dissolved_oxygen".to_string(),
                measured_value: data.dissolved_oxygen,
                threshold_value: self.thresholds.min_dissolved_oxygen,
                recommended_action: "Provide gentle agitation to increase oxygen levels for SCOBY health.".to_string(),
            });
        } else if data.dissolved_oxygen > self.thresholds.max_dissolved_oxygen {
            new_alerts.push(FermentationAlert {
                id: format!("alert-dissolved-oxygen-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Info,
                message: format!("Dissolved oxygen high: {:.2} mg/L (max: {:.2} mg/L)", 
                    data.dissolved_oxygen, self.thresholds.max_dissolved_oxygen),
                sensor_reading_id: data.id.clone(),
                sensor_type: "dissolved_oxygen".to_string(),
                measured_value: data.dissolved_oxygen,
                threshold_value: self.thresholds.max_dissolved_oxygen,
                recommended_action: "High oxygen may favor bacterial growth. Monitor closely.".to_string(),
            });
        }
        
        // Check carbon dioxide threshold
        if data.carbon_dioxide > self.thresholds.max_carbon_dioxide {
            new_alerts.push(FermentationAlert {
                id: format!("alert-carbon-dioxide-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Carbon dioxide too high: {:.1} ppm (max: {:.1} ppm)", 
                    data.carbon_dioxide, self.thresholds.max_carbon_dioxide),
                sensor_reading_id: data.id.clone(),
                sensor_type: "carbon_dioxide".to_string(),
                measured_value: data.carbon_dioxide,
                threshold_value: self.thresholds.max_carbon_dioxide,
                recommended_action: "Very active fermentation. Consider venting or moving to secondary fermentation.".to_string(),
            });
        }
        
        // Check bacteria-yeast ratio thresholds
        if data.bacteria_yeast_ratio < self.thresholds.min_bacteria_yeast_ratio {
            new_alerts.push(FermentationAlert {
                id: format!("alert-bacteria-yeast-ratio-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Bacteria-to-yeast ratio too low: {:.2} (min: {:.2})", 
                    data.bacteria_yeast_ratio, self.thresholds.min_bacteria_yeast_ratio),
                sensor_reading_id: data.id.clone(),
                sensor_type: "microbial_balance".to_string(),
                measured_value: data.bacteria_yeast_ratio,
                threshold_value: self.thresholds.min_bacteria_yeast_ratio,
                recommended_action: "Yeast dominant culture. Increase temperature slightly to favor bacteria.".to_string(),
            });
        } else if data.bacteria_yeast_ratio > self.thresholds.max_bacteria_yeast_ratio {
            new_alerts.push(FermentationAlert {
                id: format!("alert-bacteria-yeast-ratio-high-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Bacteria-to-yeast ratio too high: {:.2} (max: {:.2})", 
                    data.bacteria_yeast_ratio, self.thresholds.max_bacteria_yeast_ratio),
                sensor_reading_id: data.id.clone(),
                sensor_type: "microbial_balance".to_string(),
                measured_value: data.bacteria_yeast_ratio,
                threshold_value: self.thresholds.max_bacteria_yeast_ratio,
                recommended_action: "Bacteria dominant culture. Decrease temperature slightly to favor yeast.".to_string(),
            });
        }
        
        // Check culture health threshold
        if data.culture_health < self.thresholds.min_culture_health {
            new_alerts.push(FermentationAlert {
                id: format!("alert-culture-health-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Critical,
                message: format!("Culture health compromised: {:.2} (min: {:.2})", 
                    data.culture_health, self.thresholds.min_culture_health),
                sensor_reading_id: data.id.clone(),
                sensor_type: "microbial_balance".to_string(),
                measured_value: data.culture_health,
                threshold_value: self.thresholds.min_culture_health,
                recommended_action: "Check for contamination. May need to restart with fresh culture.".to_string(),
            });
        }
        
        // Check fermentation progress threshold
        if data.fermentation_progress < self.thresholds.min_fermentation_progress &&
           data.days_fermenting > 3.0 {
            new_alerts.push(FermentationAlert {
                id: format!("alert-fermentation-progress-low-{}", data.timestamp.timestamp()),
                timestamp: data.timestamp,
                severity: AlertSeverity::Warning,
                message: format!("Fermentation progress too slow: {:.2} (min: {:.2})", 
                    data.fermentation_progress, self.thresholds.min_fermentation_progress),
                sensor_reading_id: data.id.clone(),
                sensor_type: "combined".to_string(),
                measured_value: data.fermentation_progress,
                threshold_value: self.thresholds.min_fermentation_progress,
                recommended_action: "Check temperature, sugar levels, and culture health. Fermentation may be stalled.".to_string(),
            });
        }
        
        // Add new alerts to active alerts
        let mut alerts = self.alerts.write().await;
        for alert in new_alerts {
            alerts.insert(alert.id.clone(), alert);
        }
        
        Ok(())
    }

    /// Submit sensor readings to blockchain using ActorX framework
    async fn submit_to_blockchain(
        &self,
        alcohol_reading: &TelemetryReading<AlcoholData>,
        turbidity_reading: &TelemetryReading<TurbidityData>,
        combined_data: &CombinedFermentationData,
    ) -> TelemetryResult<()> {
        // Create alcohol submission
        let alcohol_submission = AlcoholReadingSubmission {
            reading_id: alcohol_reading.id.clone(),
            timestamp: alcohol_reading.timestamp,
            abv_percent: alcohol_reading.data.abv_percent,
            batch_id: self.batch_id.clone(),
            verification_hash: alcohol_reading.verification_hash.clone(),
        };
        
        // Create turbidity submission
        let turbidity_submission = TurbidityReadingSubmission {
            reading_id: turbidity_reading.id.clone(),
            timestamp: turbidity_reading.timestamp,
            turbidity_ntu: turbidity_reading.data.turbidity_ntu,
            batch_id: self.batch_id.clone(),
            verification_hash: turbidity_reading.verification_hash.clone(),
        };
        
        // Submit alcohol reading
        let alcohol_result = self.alcohol_actor.submit_reading(
            alcohol_submission
        ).await.map_err(|e| ClassicalError::ActorSubmissionFailed(format!("Alcohol submission failed: {}", e)))?;
        
        // Submit turbidity reading
        let turbidity_result = self.turbidity_actor.submit_reading(
            turbidity_submission
        ).await.map_err(|e| ClassicalError::ActorSubmissionFailed(format!("Turbidity submission failed: {}", e)))?;
        
        // In a production environment, we would submit the combined data
        // and all other sensor readings to blockchain as well
        
        Ok(())
    }
    
    /// Get current active alerts
    pub async fn get_alerts(&self) -> TelemetryResult<Vec<FermentationAlert>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.values().cloned().collect())
    }
    
    /// Get alerts filtered by severity
    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> TelemetryResult<Vec<FermentationAlert>> {
        let alerts = self.alerts.read().await;
        Ok(alerts.values()
            .filter(|alert| alert.severity == severity)
            .cloned()
            .collect())
    }
    
    /// Clear an alert by ID
    pub async fn clear_alert(&self, alert_id: &str) -> TelemetryResult<bool> {
        let mut alerts = self.alerts.write().await;
        Ok(alerts.remove(alert_id).is_some())
    }
    
    /// Clear all alerts
    pub async fn clear_all_alerts(&self) -> TelemetryResult<usize> {
        let mut alerts = self.alerts.write().await;
        let count = alerts.len();
        alerts.clear();
        Ok(count)
    }
    
    /// Get historical data for analysis
    pub async fn get_history(&self) -> TelemetryResult<Vec<CombinedFermentationData>> {
        let history = self.history.read().await;
        Ok(history.clone())
    }
    
    /// Get historical data within a time range
    pub async fn get_history_in_range(
        &self, 
        start_time: DateTime<Utc>, 
        end_time: DateTime<Utc>
    ) -> TelemetryResult<Vec<CombinedFermentationData>> {
        let history = self.history.read().await;
        Ok(history.iter()
            .filter(|data| data.timestamp >= start_time && data.timestamp <= end_time)
            .cloned()
            .collect())
    }
    
    /// Get fermentation status based on current readings
    pub async fn get_fermentation_status(&self) -> TelemetryResult<FermentationStatus> {
        // Get latest reading
        let history = self.history.read().await;
        
        if history.is_empty() {
            return Ok(FermentationStatus::Initializing);
        }
        
        let latest = history.last().unwrap();
        
        // Determine status based on fermentation progress
        let status = if latest.fermentation_progress >= 0.95 {
            FermentationStatus::Complete
        } else if latest.fermentation_progress >= 0.50 {
            FermentationStatus::SecondaryFermentation
        } else if latest.fermentation_progress >= 0.05 {
            FermentationStatus::PrimaryFermentation
        } else {
            FermentationStatus::Initializing
        };
        
        // Check for errors
        let alerts = self.alerts.read().await;
        let has_critical_alerts = alerts.values().any(|alert| alert.severity == AlertSeverity::Critical);
        
        if has_critical_alerts {
            Ok(FermentationStatus::Error)
        } else {
            Ok(status)
        }
    }
    
    /// Update monitoring thresholds
    pub fn update_thresholds(&mut self, thresholds: MonitoringThresholds) {
        self.thresholds = thresholds;
    }
}
