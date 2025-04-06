//! Dummy telemetry data generator for Elixir Chain (ELXR)
//! Provides simulated sensor readings for kombucha fermentation

use std::time::{SystemTime, UNIX_EPOCH};
use rand::{Rng, thread_rng, distributions::Distribution};
use rand::distributions::{Normal, Uniform};
use std::collections::HashMap;
use blake3;

/// Configuration for the dummy data generator
#[derive(Debug, Clone)]
pub struct DummyGeneratorConfig {
    /// How often to generate readings (in milliseconds)
    pub interval_ms: u64,
    /// Whether to add random noise to readings
    pub add_noise: bool,
    /// The noise level (0.0 to 1.0)
    pub noise_level: f64,
    /// Whether to simulate sensor failures occasionally
    pub simulate_failures: bool,
    /// Batch ID to use for generated data
    pub batch_id: String,
    /// Facility ID to use for generated data
    pub facility_id: String,
    /// Recipe ID for the kombucha
    pub recipe_id: String,
}

impl Default for DummyGeneratorConfig {
    fn default() -> Self {
        Self {
            interval_ms: 5000, // 5 seconds between readings
            add_noise: true,
            noise_level: 0.05, // 5% noise
            simulate_failures: false,
            batch_id: "dummy-batch-001".to_string(),
            facility_id: "brewery-alpha-001".to_string(),
            recipe_id: "classic-green-001".to_string(),
        }
    }
}

/// Dummy data generator for kombucha fermentation telemetry
pub struct DummyTelemetryGenerator {
    /// Configuration for the generator
    config: DummyGeneratorConfig,
    /// Sensor devices used for generating data
    sensors: HashMap<String, crate::SensorDevice>,
    /// Random number generator
    rng: rand::rngs::ThreadRng,
    /// Simulated fermentation day (0-30)
    fermentation_day: u32,
    /// Current fermentation stage
    fermentation_stage: crate::FermentationStage,
    /// Whether the current batch is healthy
    is_healthy_batch: bool,
    /// Base values for different sensor types
    sensor_base_values: HashMap<crate::SensorType, f64>,
}

impl DummyTelemetryGenerator {
    /// Create a new dummy telemetry generator with the given configuration
    pub fn new(config: DummyGeneratorConfig) -> Self {
        let mut sensor_base_values = HashMap::new();
        
        // Typical base values for kombucha fermentation
        sensor_base_values.insert(crate::SensorType::PH, 4.5); // Initial pH (will decrease)
        sensor_base_values.insert(crate::SensorType::Temperature, 24.0); // Optimal temp 24°C
        sensor_base_values.insert(crate::SensorType::Sugar, 8.0); // Initial sugar %
        sensor_base_values.insert(crate::SensorType::Alcohol, 0.1); // Initial alcohol %
        sensor_base_values.insert(crate::SensorType::SCOBYHealth, 95.0); // SCOBY health score
        sensor_base_values.insert(crate::SensorType::Acidity, 0.2); // Initial acidity %
        sensor_base_values.insert(crate::SensorType::Pressure, 1.0); // Pressure (for 2F)
        
        let mut generator = Self {
            config,
            sensors: HashMap::new(),
            rng: thread_rng(),
            fermentation_day: 0,
            fermentation_stage: crate::FermentationStage::Primary,
            is_healthy_batch: true,
            sensor_base_values,
        };
        
        // Create dummy sensors
        generator.initialize_sensors();
        
        generator
    }
    
    /// Initialize dummy sensors for generating data
    fn initialize_sensors(&mut self) {
        // Generate sensors for each type
        for sensor_type in [
            crate::SensorType::PH,
            crate::SensorType::Temperature,
            crate::SensorType::Sugar,
            crate::SensorType::Alcohol,
            crate::SensorType::SCOBYHealth,
            crate::SensorType::Acidity,
            crate::SensorType::Pressure,
        ].iter() {
            // Generate quantum-resistant key for each sensor
            let mut hasher = blake3::Hasher::new();
            hasher.update(format!("sensor-{:?}-{}", sensor_type, self.rng.gen::<u64>()).as_bytes());
            let result = hasher.finalize();
            let public_key = result.as_bytes().to_vec();
            
            // Create the sensor
            let sensor = crate::SensorDevice {
                id: format!("{:?}-{}", sensor_type, self.rng.gen::<u16>()),
                sensor_type: sensor_type.clone(),
                last_calibration: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                status: crate::DeviceStatus::Active,
                public_key,
            };
            
            self.sensors.insert(sensor.id.clone(), sensor);
        }
    }
    
    /// Generate a reading for a specific sensor type
    pub fn generate_reading(&mut self, sensor_type: &crate::SensorType) -> crate::TelemetryReading {
        // Find a sensor of the specified type
        let sensor = self.sensors.values()
            .find(|s| s.sensor_type == *sensor_type)
            .expect("Sensor not found");
            
        // Get the base value for this sensor type
        let base_value = *self.sensor_base_values.get(sensor_type).unwrap_or(&0.0);
        
        // Apply fermentation progression based on day and stage
        let progression_factor = match *sensor_type {
            crate::SensorType::PH => {
                // pH decreases over time (starts ~4.5, ends ~3.0)
                match self.fermentation_stage {
                    crate::FermentationStage::Primary => {
                        base_value - (f64::from(self.fermentation_day) * 0.12).min(1.5)
                    },
                    crate::FermentationStage::Secondary => {
                        // pH stabilizes in secondary fermentation
                        3.0 + (0.2 * self.rng.gen::<f64>())
                    },
                    crate::FermentationStage::Bottled => {
                        // pH remains stable in bottled state
                        3.0 + (0.1 * self.rng.gen::<f64>())
                    },
                    _ => base_value,
                }
            },
            crate::SensorType::Sugar => {
                // Sugar decreases over time as it's consumed
                match self.fermentation_stage {
                    crate::FermentationStage::Primary => {
                        // Rapid sugar consumption in primary (80% reduction)
                        base_value * (1.0 - (f64::from(self.fermentation_day) * 0.08).min(0.8))
                    },
                    crate::FermentationStage::Secondary => {
                        // Additional sugar added for secondary
                        2.0 - (f64::from(self.fermentation_day - 10) * 0.15).min(1.8)
                    },
                    crate::FermentationStage::Bottled => {
                        // Small amount left
                        0.2 + (0.1 * self.rng.gen::<f64>())
                    },
                    _ => base_value,
                }
            },
            crate::SensorType::Alcohol => {
                // Alcohol increases as sugar is consumed
                match self.fermentation_stage {
                    crate::FermentationStage::Primary => {
                        // Gradual increase during primary (up to ~1%)
                        base_value + (f64::from(self.fermentation_day) * 0.09).min(0.9)
                    },
                    crate::FermentationStage::Secondary => {
                        // Further increase during secondary
                        1.0 + (f64::from(self.fermentation_day - 10) * 0.12).min(1.0)
                    },
                    crate::FermentationStage::Bottled => {
                        // Stabilizes
                        2.0 + (0.2 * self.rng.gen::<f64>())
                    },
                    _ => base_value,
                }
            },
            crate::SensorType::Acidity => {
                // Acidity increases over time
                match self.fermentation_stage {
                    crate::FermentationStage::Primary => {
                        // Steady increase in acidity
                        base_value + (f64::from(self.fermentation_day) * 0.08).min(0.8)
                    },
                    crate::FermentationStage::Secondary => {
                        // Continues to increase slightly
                        1.0 + (f64::from(self.fermentation_day - 10) * 0.05).min(0.5)
                    },
                    crate::FermentationStage::Bottled => {
                        // Stabilizes
                        1.5 + (0.1 * self.rng.gen::<f64>())
                    },
                    _ => base_value,
                }
            },
            crate::SensorType::Pressure => {
                // Pressure changes, especially in secondary and bottled
                match self.fermentation_stage {
                    crate::FermentationStage::Primary => {
                        // Minimal pressure change in open primary fermentation
                        base_value + (0.1 * self.rng.gen::<f64>())
                    },
                    crate::FermentationStage::Secondary => {
                        // Pressure builds in secondary
                        1.0 + (f64::from(self.fermentation_day - 10) * 0.2).min(2.0)
                    },
                    crate::FermentationStage::Bottled => {
                        // Highest pressure in bottled (carbonation)
                        3.0 + (f64::from(self.fermentation_day - 20) * 0.1).min(1.0)
                    },
                    _ => base_value,
                }
            },
            crate::SensorType::SCOBYHealth => {
                // SCOBY health fluctuates but generally improves with a healthy batch
                if self.is_healthy_batch {
                    (base_value + f64::from(self.fermentation_day) * 0.2).min(99.0)
                } else {
                    (base_value - f64::from(self.fermentation_day) * 0.5).max(50.0)
                }
            },
            _ => base_value,
        };
        
        // Add daily cyclical patterns (e.g., temperature varies throughout the day)
        let time_of_day = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() % 86400) as f64 / 86400.0; // 0.0 to 1.0 representing time of day
            
        let cyclical_factor = match *sensor_type {
            crate::SensorType::Temperature => {
                // Temperature varies throughout the day (±1.5°C)
                1.5 * (2.0 * std::f64::consts::PI * time_of_day).sin()
            },
            _ => 0.0,
        };
        
        // Calculate the value with progression and cyclical factors
        let mut value = progression_factor + cyclical_factor;
        
        // Add noise if configured
        if self.config.add_noise {
            let noise_distribution = Normal::new(0.0, self.config.noise_level * base_value.max(1.0));
            value += noise_distribution.sample(&mut self.rng);
        }
        
        // Ensure non-negative values
        value = value.max(0.0);
        
        // Occasional sensor failures if configured
        let status = if self.config.simulate_failures && self.rng.gen::<f64>() < 0.01 {
            crate::ReadingStatus::Error
        } else {
            crate::ReadingStatus::Valid
        };
        
        // Create a signature for the reading using Blake3 (simulating quantum-resistant signature)
        let mut hasher = blake3::Hasher::new();
        hasher.update(format!("{}-{}-{}", sensor.id, SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(), value).as_bytes());
        let signature = hasher.finalize().as_bytes().to_vec();
        
        // Create the telemetry reading
        crate::TelemetryReading {
            sensor_id: sensor.id.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            value,
            status,
            signature,
        }
    }
    
    /// Generate a complete set of readings for all sensors
    pub fn generate_all_readings(&mut self) -> Vec<crate::TelemetryReading> {
        let sensor_types = [
            crate::SensorType::PH,
            crate::SensorType::Temperature,
            crate::SensorType::Sugar,
            crate::SensorType::Alcohol,
            crate::SensorType::SCOBYHealth,
            crate::SensorType::Acidity,
            crate::SensorType::Pressure,
        ];
        
        sensor_types.iter()
            .map(|sensor_type| self.generate_reading(sensor_type))
            .collect()
    }
    
    /// Advance the simulation by one day
    pub fn advance_day(&mut self) {
        self.fermentation_day += 1;
        
        // Update fermentation stage based on days
        self.fermentation_stage = match self.fermentation_day {
            0..=9 => crate::FermentationStage::Primary,
            10..=19 => crate::FermentationStage::Secondary,
            _ => crate::FermentationStage::Bottled,
        };
        
        // Occasionally switch between healthy and unhealthy conditions
        if self.rng.gen::<f64>() < 0.1 {
            self.is_healthy_batch = !self.is_healthy_batch;
        }
        
        // Update sensor statuses - some might need calibration
        for sensor in self.sensors.values_mut() {
            if self.rng.gen::<f64>() < 0.05 {
                sensor.status = crate::DeviceStatus::NeedsCalibration;
            } else {
                sensor.status = crate::DeviceStatus::Active;
            }
        }
    }
    
    /// Generate a complete fermentation cycle simulation
    /// Returns a map of day => readings for that day
    pub fn simulate_complete_fermentation_cycle(&mut self, days: u32) -> HashMap<u32, Vec<crate::TelemetryReading>> {
        let mut simulation_data = HashMap::new();
        
        // Reset the simulation
        self.fermentation_day = 0;
        self.fermentation_stage = crate::FermentationStage::Primary;
        
        // For each day in the cycle
        for _ in 0..days {
            // Generate multiple readings per day (e.g., 4 readings per day)
            let daily_readings = (0..4).flat_map(|_| self.generate_all_readings()).collect();
            
            // Store the readings for this day
            simulation_data.insert(self.fermentation_day, daily_readings);
            
            // Advance to the next day
            self.advance_day();
        }
        
        simulation_data
    }
    
    /// Get the batch ID for this generator
    pub fn batch_id(&self) -> &str {
        &self.config.batch_id
    }
    
    /// Get the facility ID for this generator
    pub fn facility_id(&self) -> &str {
        &self.config.facility_id
    }
    
    /// Get the recipe ID for this generator
    pub fn recipe_id(&self) -> &str {
        &self.config.recipe_id
    }
    
    /// Get the current fermentation stage
    pub fn fermentation_stage(&self) -> &crate::FermentationStage {
        &self.fermentation_stage
    }
}

/// Runs a dummy telemetry session for demonstration
/// Returns the TelemetryManager with the simulated data
pub fn run_dummy_session() -> Result<crate::TelemetryManager, &'static str> {
    use std::thread;
    use std::time::Duration;
    
    // Create a new telemetry manager
    let mut manager = crate::TelemetryManager::new();
    
    // Create a dummy generator
    let mut generator = DummyTelemetryGenerator::new(DummyGeneratorConfig::default());
    
    // Start a telemetry session
    manager.start_telemetry_session(
        generator.batch_id().to_string(),
        generator.facility_id().to_string(),
        generator.recipe_id().to_string(),
    )?;
    
    // Register all the sensors
    for sensor in generator.sensors.values() {
        manager.register_sensor(generator.batch_id(), sensor.clone())?;
    }
    
    // Simulate 30 days of fermentation with 4 readings per day
    for day in 0..30 {
        generator.fermentation_day = day;
        
        // Update fermentation stage based on days
        let stage = match day {
            0..=9 => crate::FermentationStage::Primary,
            10..=19 => crate::FermentationStage::Secondary,
            _ => crate::FermentationStage::Bottled,
        };
        
        // Update the fermentation stage if it changed
        if stage != generator.fermentation_stage {
            generator.fermentation_stage = stage.clone();
            manager.update_fermentation_stage(generator.batch_id(), stage)?;
        }
        
        // Generate 4 readings for this day
        for _ in 0..4 {
            // Generate readings for all sensors
            let readings = generator.generate_all_readings();
            
            // Add the readings to the telemetry manager
            for reading in readings {
                manager.add_reading(generator.batch_id(), reading)?;
            }
            
            // Wait a bit between readings (only in real-time mode)
            // thread::sleep(Duration::from_millis(500));
        }
        
        // Advance the simulation
        generator.advance_day();
    }
    
    // End the telemetry session
    manager.end_telemetry_session(generator.batch_id())?;
    
    Ok(manager)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dummy_generator() {
        let mut generator = DummyTelemetryGenerator::new(DummyGeneratorConfig::default());
        let readings = generator.generate_all_readings();
        
        // Verify we have the right number of readings
        assert_eq!(readings.len(), 7);
        
        // Verify all readings have valid values
        for reading in readings {
            assert!(reading.value >= 0.0);
            assert!(matches!(reading.status, crate::ReadingStatus::Valid | crate::ReadingStatus::Error));
        }
    }
    
    #[test]
    fn test_complete_simulation() {
        let mut generator = DummyTelemetryGenerator::new(DummyGeneratorConfig::default());
        let simulation = generator.simulate_complete_fermentation_cycle(30);
        
        // Verify we have data for 30 days
        assert_eq!(simulation.len(), 30);
        
        // Verify sugar decreases and alcohol increases over time
        let day1_sugar = simulation.get(&0).unwrap().iter()
            .find(|r| generator.sensors.get(&r.sensor_id).unwrap().sensor_type == crate::SensorType::Sugar)
            .unwrap().value;
            
        let day30_sugar = simulation.get(&29).unwrap().iter()
            .find(|r| generator.sensors.get(&r.sensor_id).unwrap().sensor_type == crate::SensorType::Sugar)
            .unwrap().value;
            
        let day1_alcohol = simulation.get(&0).unwrap().iter()
            .find(|r| generator.sensors.get(&r.sensor_id).unwrap().sensor_type == crate::SensorType::Alcohol)
            .unwrap().value;
            
        let day30_alcohol = simulation.get(&29).unwrap().iter()
            .find(|r| generator.sensors.get(&r.sensor_id).unwrap().sensor_type == crate::SensorType::Alcohol)
            .unwrap().value;
            
        assert!(day30_sugar < day1_sugar);
        assert!(day30_alcohol > day1_alcohol);
    }
}
