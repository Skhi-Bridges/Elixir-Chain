//! ActorX Integration for Elixir (ELXR) Chain Sensors
//!
//! Provides integration between the telemetry sensors and ActorX framework
//! to enable secure submission of sensor data to the blockchain.

use crate::sensor::{
    alcohol::AlcoholSensor,
    turbidity::TurbiditySensor,
    core::{QuantumSecuredSensor, TelemetryReading}
};
use crate::crypto::QuantumCrypto;
use crate::error::TelemetryResult;
use crate::hardware::security::HardwareSecurityManager;
use crate::time::QuantumSecureClock;

use actorx_frameworks::core::{
    actor::{Actor, ActorId, ActorRegistration},
    actor_token::{ActorToken, TokenAmount, TokenOperation},
    chain::{ChainContext, ChainSubmission, SubmissionResult},
    error::ActorError,
    msg::{Message, MessageStatus, SignedMessage}
};

use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// ActorX integration for the AlcoholSensor
pub struct AlcoholSensorActor {
    /// The alcohol sensor to integrate
    sensor: Arc<AlcoholSensor>,
    
    /// Actor registration with the ActorX framework
    actor_registration: ActorRegistration,
    
    /// Actor token for blockchain operations
    actor_token: Arc<RwLock<ActorToken>>,
    
    /// Quantum security components for message signing
    quantum_crypto: Arc<QuantumCrypto>,
    
    /// Chain context for the ELXR chain
    chain_context: ChainContext,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
}

/// ActorX integration for the TurbiditySensor
pub struct TurbiditySensorActor {
    /// The turbidity sensor to integrate
    sensor: Arc<TurbiditySensor>,
    
    /// Actor registration with the ActorX framework
    actor_registration: ActorRegistration,
    
    /// Actor token for blockchain operations
    actor_token: Arc<RwLock<ActorToken>>,
    
    /// Quantum security components for message signing
    quantum_crypto: Arc<QuantumCrypto>,
    
    /// Chain context for the ELXR chain
    chain_context: ChainContext,
    
    /// Security manager
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
}

/// Alcohol reading data for blockchain submission
#[derive(Serialize, Deserialize)]
pub struct AlcoholReadingSubmission {
    /// ID of the reading
    pub id: String,
    
    /// Timestamp of the reading
    pub timestamp: DateTime<Utc>,
    
    /// ABV percentage
    pub abv_percent: f32,
    
    /// Ethanol concentration
    pub ethanol_concentration: f32,
    
    /// Fermentation stage
    pub days_fermenting: f32,
    
    /// Maturity indicator
    pub maturity: f32,
    
    /// Stability indicator
    pub stability: f32,
    
    /// Forecast for next 24 hours
    pub forecast_24h: f32,
    
    /// Quantum security verification hash
    pub verification_hash: String,
}

/// Turbidity reading data for blockchain submission
#[derive(Serialize, Deserialize)]
pub struct TurbidityReadingSubmission {
    /// ID of the reading
    pub id: String,
    
    /// Timestamp of the reading
    pub timestamp: DateTime<Utc>,
    
    /// Turbidity value (NTU)
    pub ntu_value: f32,
    
    /// Clarity score
    pub clarity_score: f32,
    
    /// Particle concentration
    pub particle_concentration: f32,
    
    /// SCOBY particulate estimation
    pub scoby_particulate: f32,
    
    /// Fermentation phase
    pub fermentation_phase: f32,
    
    /// Forecast clarity for next 24 hours
    pub forecast_24h: f32,
    
    /// Quantum security verification hash
    pub verification_hash: String,
}

impl AlcoholSensorActor {
    /// Create a new AlcoholSensorActor
    pub async fn new(
        sensor: Arc<AlcoholSensor>,
        actor_token: Arc<RwLock<ActorToken>>,
        quantum_crypto: Arc<QuantumCrypto>,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
    ) -> TelemetryResult<Self> {
        // Check security before creating actor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(crate::error::ClassicalError::SecurityCompromised(
                "Cannot create alcohol sensor actor: hardware security compromised".to_string()
            ).into());
        }
        
        // Create actor registration
        let actor_id = ActorId::new("elxr-alcohol-sensor-1");
        let actor_registration = ActorRegistration::new(actor_id, "ElixirChain", "AlcoholSensor");
        
        // Create chain context for ELXR chain
        let chain_context = ChainContext::new(
            "elxr-main",
            "wss://elxr-rpc.matrix-magiq.io",
            "Elixir Chain"
        );
        
        Ok(Self {
            sensor,
            actor_registration,
            actor_token,
            quantum_crypto,
            chain_context,
            security_manager,
        })
    }
    
    /// Take a reading and submit it to the blockchain
    pub async fn take_reading_and_submit(&self) -> TelemetryResult<SubmissionResult> {
        // Check security
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(crate::error::ClassicalError::SecurityCompromised(
                "Cannot take alcohol reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Take reading from sensor
        let reading = self.sensor.take_reading().await?;
        
        // Verify the reading
        let verified = self.sensor.verify_reading(&reading).await?;
        if !verified {
            return Err(crate::error::ClassicalError::VerificationFailed(
                "Alcohol reading verification failed".to_string()
            ).into());
        }
        
        // Create blockchain submission from reading
        let reading_data = reading.data.get_data();
        
        // Get 24-hour forecast from time series data
        let forecast_24h = if !reading_data.time_series_forecast.values.is_empty() 
                          && reading_data.time_series_forecast.values[0].len() >= 24 {
            reading_data.time_series_forecast.values[0][23]
        } else {
            reading_data.abv_percent // Fallback to current if forecast unavailable
        };
        
        let submission = AlcoholReadingSubmission {
            id: reading.id.clone(),
            timestamp: reading.timestamp,
            abv_percent: reading_data.abv_percent,
            ethanol_concentration: reading_data.ethanol_concentration,
            days_fermenting: reading_data.days_fermenting,
            maturity: reading_data.maturity,
            stability: reading_data.stability,
            forecast_24h,
            verification_hash: reading.hash.to_string(),
        };
        
        // Serialize the submission
        let submission_data = serde_json::to_string(&submission)
            .map_err(|e| crate::error::ClassicalError::SerializationFailed(e.to_string()))?;
        
        // Create signed message with quantum crypto
        let message = Message::new(
            self.actor_registration.actor_id.clone(),
            "alcohol_reading".to_string(),
            submission_data.into_bytes(),
        );
        
        // Sign the message with quantum-resistant signature
        let signed_message = self.quantum_crypto.sign_message(message)
            .map_err(|e| crate::error::ClassicalError::CryptoError(e.to_string()))?;
        
        // Create chain submission
        let chain_submission = ChainSubmission::new(
            signed_message,
            self.chain_context.clone(),
            "submit_alcohol_reading".to_string(),
        );
        
        // Submit to blockchain and get result
        let token = self.actor_token.read().await;
        let result = token.submit_to_chain(chain_submission)
            .await
            .map_err(|e| crate::error::ClassicalError::SubmissionFailed(e.to_string()))?;
        
        // Update reading with committed status
        let mut reading_mut = reading.clone();
        self.sensor.commit_reading(&mut reading_mut).await?;
        
        Ok(result)
    }
}

impl TurbiditySensorActor {
    /// Create a new TurbiditySensorActor
    pub async fn new(
        sensor: Arc<TurbiditySensor>,
        actor_token: Arc<RwLock<ActorToken>>,
        quantum_crypto: Arc<QuantumCrypto>,
        security_manager: Arc<RwLock<HardwareSecurityManager>>,
    ) -> TelemetryResult<Self> {
        // Check security before creating actor
        let security_compromised = security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(crate::error::ClassicalError::SecurityCompromised(
                "Cannot create turbidity sensor actor: hardware security compromised".to_string()
            ).into());
        }
        
        // Create actor registration
        let actor_id = ActorId::new("elxr-turbidity-sensor-1");
        let actor_registration = ActorRegistration::new(actor_id, "ElixirChain", "TurbiditySensor");
        
        // Create chain context for ELXR chain
        let chain_context = ChainContext::new(
            "elxr-main",
            "wss://elxr-rpc.matrix-magiq.io",
            "Elixir Chain"
        );
        
        Ok(Self {
            sensor,
            actor_registration,
            actor_token,
            quantum_crypto,
            chain_context,
            security_manager,
        })
    }
    
    /// Take a reading and submit it to the blockchain
    pub async fn take_reading_and_submit(&self) -> TelemetryResult<SubmissionResult> {
        // Check security
        let security_compromised = self.security_manager.read().await.is_security_compromised().await?;
        if security_compromised {
            return Err(crate::error::ClassicalError::SecurityCompromised(
                "Cannot take turbidity reading: hardware security compromised".to_string()
            ).into());
        }
        
        // Take reading from sensor
        let reading = self.sensor.take_reading().await?;
        
        // Verify the reading
        let verified = self.sensor.verify_reading(&reading).await?;
        if !verified {
            return Err(crate::error::ClassicalError::VerificationFailed(
                "Turbidity reading verification failed".to_string()
            ).into());
        }
        
        // Create blockchain submission from reading
        let reading_data = reading.data.get_data();
        
        // Get 24-hour forecast from time series data
        let forecast_24h = if !reading_data.time_series_forecast.values.is_empty() 
                          && reading_data.time_series_forecast.values[0].len() >= 24 {
            reading_data.time_series_forecast.values[0][23]
        } else {
            reading_data.ntu_value // Fallback to current if forecast unavailable
        };
        
        let submission = TurbidityReadingSubmission {
            id: reading.id.clone(),
            timestamp: reading.timestamp,
            ntu_value: reading_data.ntu_value,
            clarity_score: reading_data.clarity_score,
            particle_concentration: reading_data.particle_concentration,
            scoby_particulate: reading_data.scoby_particulate,
            fermentation_phase: reading_data.fermentation_phase,
            forecast_24h,
            verification_hash: reading.hash.to_string(),
        };
        
        // Serialize the submission
        let submission_data = serde_json::to_string(&submission)
            .map_err(|e| crate::error::ClassicalError::SerializationFailed(e.to_string()))?;
        
        // Create signed message with quantum crypto
        let message = Message::new(
            self.actor_registration.actor_id.clone(),
            "turbidity_reading".to_string(),
            submission_data.into_bytes(),
        );
        
        // Sign the message with quantum-resistant signature
        let signed_message = self.quantum_crypto.sign_message(message)
            .map_err(|e| crate::error::ClassicalError::CryptoError(e.to_string()))?;
        
        // Create chain submission
        let chain_submission = ChainSubmission::new(
            signed_message,
            self.chain_context.clone(),
            "submit_turbidity_reading".to_string(),
        );
        
        // Submit to blockchain and get result
        let token = self.actor_token.read().await;
        let result = token.submit_to_chain(chain_submission)
            .await
            .map_err(|e| crate::error::ClassicalError::SubmissionFailed(e.to_string()))?;
        
        // Update reading with committed status
        let mut reading_mut = reading.clone();
        self.sensor.commit_reading(&mut reading_mut).await?;
        
        Ok(result)
    }
}

/// Factory for creating sensor actors for the ELXR chain
pub struct ElxrSensorActorFactory {
    /// Security manager for all actors
    security_manager: Arc<RwLock<HardwareSecurityManager>>,
    
    /// Quantum crypto system
    quantum_crypto: Arc<QuantumCrypto>,
    
    /// Actor token for blockchain operations
    actor_token: Arc<RwLock<ActorToken>>,
    
    /// Quantum-secured clock for time-series data
    quantum_clock: Arc<RwLock<QuantumSecureClock>>,
}

impl ElxrSensorActorFactory {
    /// Create a new factory
    pub async fn new() -> TelemetryResult<Self> {
        // Initialize security manager
        let security_manager = Arc::new(RwLock::new(HardwareSecurityManager::new()?));
        
        // Initialize quantum crypto
        let quantum_crypto = Arc::new(QuantumCrypto::new()?);
        
        // Create actor token for blockchain operations
        let actor_token = Arc::new(RwLock::new(ActorToken::new(
            "elxr-telemetry",
            quantum_crypto.clone(),
        )?));
        
        // Initialize quantum clock
        let quantum_clock = Arc::new(RwLock::new(QuantumSecureClock::new(quantum_crypto.clone())?));
        
        Ok(Self {
            security_manager,
            quantum_crypto,
            actor_token,
            quantum_clock,
        })
    }
    
    /// Create an alcohol sensor actor
    pub async fn create_alcohol_sensor_actor(&self) -> TelemetryResult<AlcoholSensorActor> {
        // Create sensor connection info
        let connection_info = crate::sensor::core::SensorConnectionInfo {
            address: "0xA0:20:D2:1C:FF:23".to_string(),
            port: 8033,
            protocol: "esp32s3-secure".to_string(),
            sensor_type: "alcohol".to_string(),
        };
        
        // Create sensor hardware
        let hardware = crate::sensor::core::SensorHardware::new(
            "ELXR-ALC-001".to_string(),
            "ESP32S3-ALCOHOL-SENSOR".to_string(),
            connection_info,
        );
        
        // Create and initialize the alcohol sensor
        let sensor = Arc::new(AlcoholSensor::new(
            hardware,
            self.security_manager.clone(),
            self.quantum_clock.clone(),
        ).await?);
        
        // Create the actor for the sensor
        let actor = AlcoholSensorActor::new(
            sensor,
            self.actor_token.clone(),
            self.quantum_crypto.clone(),
            self.security_manager.clone(),
        ).await?;
        
        Ok(actor)
    }
    
    /// Create a turbidity sensor actor
    pub async fn create_turbidity_sensor_actor(&self) -> TelemetryResult<TurbiditySensorActor> {
        // Create sensor connection info
        let connection_info = crate::sensor::core::SensorConnectionInfo {
            address: "0xA0:20:D2:1C:FF:24".to_string(),
            port: 8034,
            protocol: "esp32s3-secure".to_string(),
            sensor_type: "turbidity".to_string(),
        };
        
        // Create sensor hardware
        let hardware = crate::sensor::core::SensorHardware::new(
            "ELXR-TRB-001".to_string(),
            "ESP32S3-TURBIDITY-SENSOR".to_string(),
            connection_info,
        );
        
        // Create and initialize the turbidity sensor
        let sensor = Arc::new(TurbiditySensor::new(
            hardware,
            self.security_manager.clone(),
            self.quantum_clock.clone(),
        ).await?);
        
        // Create the actor for the sensor
        let actor = TurbiditySensorActor::new(
            sensor,
            self.actor_token.clone(),
            self.quantum_crypto.clone(),
            self.security_manager.clone(),
        ).await?;
        
        Ok(actor)
    }
}
