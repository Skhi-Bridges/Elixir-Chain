//! API module for ELXR telemetry
//! Provides endpoints for accessing kombucha fermentation telemetry data

use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::telemetry::{SensorDevice, SensorType, TelemetryReading, FermentationTelemetry, FermentationStage};
use crate::telemetry::sensor::dummy_generator::DummyTelemetryGenerator;

/// DTO for telemetry summary response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TelemetrySummary {
    /// Batch ID
    pub batch_id: String,
    /// Facility ID
    pub facility_id: String,
    /// Recipe ID
    pub recipe_id: String,
    /// Start timestamp
    pub start_time: u64,
    /// End timestamp (if session is complete)
    pub end_time: Option<u64>,
    /// Current fermentation stage
    pub fermentation_stage: String,
    /// Current day in fermentation cycle
    pub fermentation_day: u32,
    /// Number of sensor readings
    pub reading_count: usize,
    /// Latest quality score (0-100)
    pub quality_score: u8,
    /// SCOBY health status
    pub scoby_health: String,
    /// Active sensors
    pub active_sensors: Vec<String>,
}

/// DTO for detailed telemetry response
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DetailedTelemetry {
    /// Basic summary
    pub summary: TelemetrySummary,
    /// Latest readings for each sensor
    pub latest_readings: HashMap<String, SensorReading>,
    /// Historical readings for each sensor (last 24 hours)
    pub historical_readings: HashMap<String, Vec<SensorReading>>,
    /// Stage history
    pub stage_history: Vec<StageChange>,
}

/// DTO for sensor reading
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SensorReading {
    /// Sensor ID
    pub sensor_id: String,
    /// Sensor type
    pub sensor_type: String,
    /// Reading timestamp
    pub timestamp: u64,
    /// Reading value
    pub value: f64,
    /// Reading status
    pub status: String,
}

/// DTO for fermentation stage change
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StageChange {
    /// Stage name
    pub stage: String,
    /// Timestamp when the stage started
    pub timestamp: u64,
    /// Duration in this stage (seconds)
    pub duration: Option<u64>,
}

/// Singleton manager for telemetry API
#[derive(Default)]
pub struct TelemetryApi {
    /// Active telemetry sessions
    sessions: Arc<RwLock<HashMap<String, FermentationTelemetry>>>,
    /// Dummy generators for demonstration
    dummy_generators: Arc<RwLock<HashMap<String, DummyTelemetryGenerator>>>,
    /// Stage history for each batch
    stage_history: Arc<RwLock<HashMap<String, Vec<StageChange>>>>,
}

impl TelemetryApi {
    /// Create a new telemetry API instance
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            dummy_generators: Arc::new(RwLock::new(HashMap::new())),
            stage_history: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Initialize with dummy data for demonstration
    pub fn initialize_with_dummy_data(&self) -> Result<(), &'static str> {
        use crate::telemetry::sensor::dummy_generator::{DummyGeneratorConfig, DummyTelemetryGenerator};
        
        // Create multiple dummy batches
        let batch_configs = vec![
            DummyGeneratorConfig {
                batch_id: "kombucha-batch-001".to_string(),
                facility_id: "elxr-brewery-001".to_string(),
                recipe_id: "classic-green-001".to_string(),
                ..Default::default()
            },
            DummyGeneratorConfig {
                batch_id: "kombucha-batch-002".to_string(),
                facility_id: "elxr-brewery-002".to_string(),
                recipe_id: "hibiscus-ginger-001".to_string(),
                ..Default::default()
            },
            DummyGeneratorConfig {
                batch_id: "kombucha-batch-003".to_string(),
                facility_id: "elxr-brewery-001".to_string(),
                recipe_id: "jasmine-mint-001".to_string(),
                ..Default::default()
            },
        ];
        
        // Get current time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // Create a generator for each batch and generate telemetry data
        for config in batch_configs {
            let batch_id = config.batch_id.clone();
            let recipe_id = config.recipe_id.clone();
            let mut generator = DummyTelemetryGenerator::new(config);
            
            // Generate data for the first 20 days
            let simulation_data = generator.simulate_complete_fermentation_cycle(20);
            
            // Extract readings for storage in the API
            let mut telemetry = FermentationTelemetry {
                batch_id: batch_id.clone(),
                facility_id: generator.facility_id().to_string(),
                recipe_id: recipe_id.clone(),
                start_time: current_time - 20 * 86400, // 20 days ago
                end_time: None,
                sensors: generator.sensors.clone(),
                readings: Vec::new(),
                current_stage: match generator.fermentation_day {
                    0..=9 => FermentationStage::Primary,
                    10..=19 => FermentationStage::Secondary,
                    _ => FermentationStage::Bottled,
                },
                metadata: HashMap::new(),
            };
            
            // Flatten the readings from all days
            for (day, daily_readings) in simulation_data {
                telemetry.readings.extend(daily_readings);
            }
            
            // Add metadata based on recipe
            match recipe_id.as_str() {
                "classic-green-001" => {
                    telemetry.metadata.insert("tea_type".to_string(), "Green Tea".to_string());
                    telemetry.metadata.insert("sugar_type".to_string(), "Organic Cane Sugar".to_string());
                    telemetry.metadata.insert("additives".to_string(), "None".to_string());
                },
                "hibiscus-ginger-001" => {
                    telemetry.metadata.insert("tea_type".to_string(), "Black Tea".to_string());
                    telemetry.metadata.insert("sugar_type".to_string(), "Raw Honey".to_string());
                    telemetry.metadata.insert("additives".to_string(), "Hibiscus, Ginger".to_string());
                },
                "jasmine-mint-001" => {
                    telemetry.metadata.insert("tea_type".to_string(), "Jasmine Green Tea".to_string());
                    telemetry.metadata.insert("sugar_type".to_string(), "Coconut Sugar".to_string());
                    telemetry.metadata.insert("additives".to_string(), "Mint".to_string());
                },
                _ => {
                    telemetry.metadata.insert("tea_type".to_string(), "Black Tea".to_string());
                    telemetry.metadata.insert("sugar_type".to_string(), "Organic Cane Sugar".to_string());
                    telemetry.metadata.insert("additives".to_string(), "None".to_string());
                }
            }
            
            // Store the telemetry session
            self.sessions.write().unwrap().insert(batch_id.clone(), telemetry);
            
            // Create and store stage history
            let mut stage_history = Vec::new();
            
            // Primary fermentation started at the beginning
            stage_history.push(StageChange {
                stage: "Primary".to_string(),
                timestamp: current_time - 20 * 86400, // 20 days ago
                duration: Some(10 * 86400), // 10 days
            });
            
            // Secondary fermentation started 10 days into the process
            stage_history.push(StageChange {
                stage: "Secondary".to_string(),
                timestamp: current_time - 10 * 86400, // 10 days ago
                duration: Some(10 * 86400), // 10 days
            });
            
            // For batches that have reached bottling
            if generator.fermentation_day >= 20 {
                stage_history.push(StageChange {
                    stage: "Bottled".to_string(),
                    timestamp: current_time, // Just now
                    duration: None, // Currently in this stage
                });
            }
            
            // Store the stage history
            self.stage_history.write().unwrap().insert(batch_id.clone(), stage_history);
            
            // Store the generator for future updates
            self.dummy_generators.write().unwrap().insert(batch_id, generator);
        }
        
        Ok(())
    }
    
    /// Update dummy data to simulate new readings
    pub fn update_dummy_data(&self) -> Result<(), &'static str> {
        // Get current time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let mut generators = self.dummy_generators.write().unwrap();
        let mut sessions = self.sessions.write().unwrap();
        let mut stage_histories = self.stage_history.write().unwrap();
        
        for (batch_id, generator) in generators.iter_mut() {
            // Advance by 1 day
            generator.fermentation_day += 1;
            
            // Determine current stage based on day
            let current_stage = match generator.fermentation_day {
                0..=9 => FermentationStage::Primary,
                10..=19 => FermentationStage::Secondary,
                _ => FermentationStage::Bottled,
            };
            
            // Generate new readings
            let new_readings = generator.generate_all_readings();
            
            // Add to the telemetry session
            if let Some(session) = sessions.get_mut(batch_id) {
                // Check if stage has changed
                if current_stage != session.current_stage {
                    // Update the stage in the session
                    session.current_stage = current_stage.clone();
                    
                    // Update stage history
                    if let Some(history) = stage_histories.get_mut(batch_id) {
                        // Complete the previous stage
                        if let Some(last_stage) = history.last_mut() {
                            last_stage.duration = Some(current_time - last_stage.timestamp);
                        }
                        
                        // Add the new stage
                        history.push(StageChange {
                            stage: format!("{:?}", current_stage),
                            timestamp: current_time,
                            duration: None,
                        });
                    }
                }
                
                // Add new readings
                session.readings.extend(new_readings);
            }
        }
        
        Ok(())
    }
    
    /// Get all batch IDs
    pub fn get_all_batch_ids(&self) -> Vec<String> {
        self.sessions.read().unwrap().keys().cloned().collect()
    }
    
    /// Get a summary of all telemetry sessions
    pub fn get_all_summaries(&self) -> Vec<TelemetrySummary> {
        let sessions = self.sessions.read().unwrap();
        
        sessions.values()
            .map(|session| self.create_summary_for_session(session))
            .collect()
    }
    
    /// Get detailed telemetry for a specific batch
    pub fn get_detailed_telemetry(&self, batch_id: &str) -> Option<DetailedTelemetry> {
        let sessions = self.sessions.read().unwrap();
        let stage_histories = self.stage_history.read().unwrap();
        
        if let Some(session) = sessions.get(batch_id) {
            // Create summary
            let summary = self.create_summary_for_session(session);
            
            // Get latest readings for each sensor
            let mut latest_readings = HashMap::new();
            let mut historical_readings = HashMap::new();
            
            // Group readings by sensor ID
            let mut readings_by_sensor: HashMap<String, Vec<&TelemetryReading>> = HashMap::new();
            
            for reading in &session.readings {
                readings_by_sensor
                    .entry(reading.sensor_id.clone())
                    .or_default()
                    .push(reading);
            }
            
            // Get current timestamp
            let current_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
                
            // 24 hours ago timestamp
            let day_ago = current_time.saturating_sub(86400);
            
            // For each sensor, get the latest reading and historical readings
            for (sensor_id, readings) in readings_by_sensor {
                // Sort readings by timestamp (newest first)
                let mut sorted_readings = readings.clone();
                sorted_readings.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                
                // Get the latest reading
                if let Some(latest) = sorted_readings.first() {
                    let sensor_type = session.sensors.get(&latest.sensor_id)
                        .map(|s| format!("{:?}", s.sensor_type))
                        .unwrap_or_else(|| "Unknown".to_string());
                        
                    latest_readings.insert(sensor_id.clone(), SensorReading {
                        sensor_id: latest.sensor_id.clone(),
                        sensor_type,
                        timestamp: latest.timestamp,
                        value: latest.value,
                        status: format!("{:?}", latest.status),
                    });
                }
                
                // Get historical readings (last 24 hours)
                let recent_readings: Vec<SensorReading> = sorted_readings.iter()
                    .filter(|r| r.timestamp >= day_ago)
                    .map(|r| {
                        let sensor_type = session.sensors.get(&r.sensor_id)
                            .map(|s| format!("{:?}", s.sensor_type))
                            .unwrap_or_else(|| "Unknown".to_string());
                            
                        SensorReading {
                            sensor_id: r.sensor_id.clone(),
                            sensor_type,
                            timestamp: r.timestamp,
                            value: r.value,
                            status: format!("{:?}", r.status),
                        }
                    })
                    .collect();
                    
                historical_readings.insert(sensor_id, recent_readings);
            }
            
            // Get stage history
            let stage_history = stage_histories.get(batch_id)
                .cloned()
                .unwrap_or_default();
            
            Some(DetailedTelemetry {
                summary,
                latest_readings,
                historical_readings,
                stage_history,
            })
        } else {
            None
        }
    }
    
    /// Create a summary for a telemetry session
    fn create_summary_for_session(&self, session: &FermentationTelemetry) -> TelemetrySummary {
        // Get the number of readings
        let reading_count = session.readings.len();
        
        // Determine the fermentation day based on start time
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
            
        let fermentation_day = if session.start_time <= current_time {
            ((current_time - session.start_time) / 86400) as u32 // Convert seconds to days
        } else {
            0
        };
        
        // Get active sensors
        let active_sensors = session.sensors.values()
            .filter(|s| matches!(s.status, super::DeviceStatus::Active))
            .map(|s| format!("{:?}", s.sensor_type))
            .collect();
            
        // Calculate quality score
        let quality_score = if !session.readings.is_empty() {
            // Use utility function if available, otherwise provide a dummy value
            super::utils::generate_quality_score(&session.readings)
        } else {
            0
        };
        
        // Get SCOBY health
        let scoby_health = {
            // Find the most recent SCOBY health reading
            let scoby_readings: Vec<&TelemetryReading> = session.readings.iter()
                .filter(|r| {
                    session.sensors.get(&r.sensor_id)
                        .map(|s| s.sensor_type == SensorType::SCOBYHealth)
                        .unwrap_or(false)
                })
                .collect();
                
            if let Some(latest) = scoby_readings.iter().max_by_key(|r| r.timestamp) {
                match latest.value as u8 {
                    0..=50 => "Poor",
                    51..=70 => "Fair",
                    71..=85 => "Good",
                    86..=95 => "Very Good",
                    _ => "Excellent",
                }.to_string()
            } else {
                "Unknown".to_string()
            }
        };
        
        TelemetrySummary {
            batch_id: session.batch_id.clone(),
            facility_id: session.facility_id.clone(),
            recipe_id: session.recipe_id.clone(),
            start_time: session.start_time,
            end_time: session.end_time,
            fermentation_stage: format!("{:?}", session.current_stage),
            fermentation_day,
            reading_count,
            quality_score,
            scoby_health,
            active_sensors,
        }
    }
    
    /// Export telemetry data to blockchain (simulation)
    pub fn export_to_blockchain(&self, batch_id: &str) -> Result<Vec<u8>, &'static str> {
        let sessions = self.sessions.read().unwrap();
        let stage_histories = self.stage_history.read().unwrap();
        
        if let Some(session) = sessions.get(batch_id) {
            // Simulate exporting to blockchain by creating a hash of the data
            let mut hasher = blake3::Hasher::new();
            
            // Hash batch ID, facility ID and recipe ID
            hasher.update(batch_id.as_bytes());
            hasher.update(session.facility_id.as_bytes());
            hasher.update(session.recipe_id.as_bytes());
            
            // Hash timestamps
            hasher.update(&session.start_time.to_le_bytes());
            if let Some(end_time) = session.end_time {
                hasher.update(&end_time.to_le_bytes());
            }
            
            // Hash current stage
            hasher.update(format!("{:?}", session.current_stage).as_bytes());
            
            // Hash metadata
            for (key, value) in &session.metadata {
                hasher.update(key.as_bytes());
                hasher.update(value.as_bytes());
            }
            
            // Hash each reading
            for reading in &session.readings {
                hasher.update(reading.sensor_id.as_bytes());
                hasher.update(&reading.timestamp.to_le_bytes());
                hasher.update(&reading.value.to_le_bytes());
            }
            
            // Hash stage history
            if let Some(history) = stage_histories.get(batch_id) {
                for stage in history {
                    hasher.update(stage.stage.as_bytes());
                    hasher.update(&stage.timestamp.to_le_bytes());
                    if let Some(duration) = stage.duration {
                        hasher.update(&duration.to_le_bytes());
                    }
                }
            }
            
            // Generate hash
            let hash = hasher.finalize();
            
            Ok(hash.as_bytes().to_vec())
        } else {
            Err("Batch not found")
        }
    }
}

/// Initialize the telemetry API singleton
/// This is called during node startup
pub fn initialize() -> Arc<TelemetryApi> {
    let api = Arc::new(TelemetryApi::new());
    
    // Initialize with dummy data
    if let Err(e) = api.initialize_with_dummy_data() {
        log::error!("Failed to initialize telemetry API with dummy data: {}", e);
    }
    
    api
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_api_initialization() {
        let api = TelemetryApi::new();
        api.initialize_with_dummy_data().unwrap();
        
        // Verify we have batch data
        let summaries = api.get_all_summaries();
        assert!(!summaries.is_empty());
        
        // Verify we can get detailed telemetry
        let batch_id = &summaries[0].batch_id;
        let detailed = api.get_detailed_telemetry(batch_id);
        assert!(detailed.is_some());
        
        // Verify we can update dummy data
        api.update_dummy_data().unwrap();
    }
}
