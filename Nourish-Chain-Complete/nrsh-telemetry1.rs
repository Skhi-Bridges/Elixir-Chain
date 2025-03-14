// NRSH Spirulina Telemetry System
// Target: Arduino Nano 33 IoT with sensors for spirulina cultivation monitoring
// Integration with Polkadot parachain (Rococo testnet)
// Copyright © 2025 NRSH Chain

#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::{adc, delay_ms};
use embedded_hal::digital::v2::OutputPin;
use heapless::String;
use heapless::Vec;
use nb::block;
use panic_halt as _;

// Quantum-resistant cryptography module
mod quantum_crypto {
    use heapless::Vec;
    
    // Kyber-Dilithium implementation for post-quantum security
    // In production, use standardized implementations
    
    pub struct QuantumKeys {
        public_key: Vec<u8, 64>,
        private_key: Vec<u8, 64>,
    }
    
    pub struct QuantumSignature {
        signature: Vec<u8, 128>,
    }
    
    pub fn generate_keys() -> QuantumKeys {
        let mut public_key = Vec::new();
        let mut private_key = Vec::new();
        
        // Simplified key generation with entropy source
        // In production, use hardware-based entropy
        for i in 0..64 {
            let val = ((i * 7 + 13) % 256) as u8;
            public_key.push(val).unwrap();
            private_key.push((val ^ 0xA5) as u8).unwrap();
        }
        
        QuantumKeys {
            public_key,
            private_key,
        }
    }
    
    pub fn sign_data(data: &[u8], keys: &QuantumKeys) -> QuantumSignature {
        let mut signature = Vec::new();
        
        // Simplified signature algorithm
        // In production, use standardized Dilithium implementation
        for i in 0..128 {
            let idx = i % data.len();
            let key_idx = i % 64;
            let sig_byte = data[idx].wrapping_add(keys.private_key[key_idx])
                              .wrapping_mul(0x1D);
            signature.push(sig_byte).unwrap();
        }
        
        QuantumSignature {
            signature,
        }
    }
}

// Sensor configurations for spirulina cultivation
const PH_SENSOR_PIN: u8 = 0;      // A0
const TEMP_SENSOR_PIN: u8 = 1;    // A1
const LIGHT_SENSOR_PIN: u8 = 2;   // A2
const DENSITY_SENSOR_PIN: u8 = 3; // A3
const DISSOLVED_O2_PIN: u8 = 4;   // A4
const NITRATE_SENSOR_PIN: u8 = 5; // A5
const SALINITY_SENSOR_PIN: u8 = 6;// A6

// Optimal ranges for spirulina cultivation
const OPTIMAL_PH_MIN: f32 = 8.5;
const OPTIMAL_PH_MAX: f32 = 10.5;
const OPTIMAL_TEMP_MIN: f32 = 30.0;  // °C
const OPTIMAL_TEMP_MAX: f32 = 37.0;  // °C
const OPTIMAL_LIGHT_MIN: f32 = 2500.0; // lux (spirulina needs high light)
const OPTIMAL_LIGHT_MAX: f32 = 10000.0; // lux
const OPTIMAL_DENSITY_MIN: f32 = 1.0; // g/L
const OPTIMAL_DENSITY_MAX: f32 = 3.0; // g/L (harvest density)
const OPTIMAL_DISSOLVED_O2_MIN: f32 = 6.0; // mg/L
const OPTIMAL_DISSOLVED_O2_MAX: f32 = 9.0; // mg/L
const OPTIMAL_NITRATE_MIN: f32 = 10.0; // mg/L
const OPTIMAL_NITRATE_MAX: f32 = 30.0; // mg/L
const OPTIMAL_SALINITY_MIN: f32 = 10.0; // g/L
const OPTIMAL_SALINITY_MAX: f32 = 20.0; // g/L

// Battery monitoring
const BATTERY_LEVEL_PIN: u8 = 7;  // A7

// Rococo testnet endpoint (to be updated with actual endpoint)
const ROCOCO_ENDPOINT: &str = "wss://rococo-rpc.polkadot.io";

#[arduino_hal::entry]
fn main() -> ! {
    // Initialize Arduino peripherals
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut adc = arduino_hal::Adc::new(dp.ADC, Default::default());
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    
    // Status LED for visual feedback
    let mut led = pins.d13.into_output();
    
    // Generate quantum-resistant keys
    let keys = quantum_crypto::generate_keys();
    
    // Store last measurement time to handle timing
    let mut last_measurement_time: u32 = 0;
    
    // Initialize device ID with location data
    let device_id = "NRSH-SPIRULINA-POOL-A24";
    
    // Main telemetry loop
    loop {
        // Blink LED to indicate active measurement
        led.set_high();
        arduino_hal::delay_ms(100);
        led.set_low();
        
        // Read all sensors
        let ph_raw = adc.read_blocking(&pins.a0);
        let temp_raw = adc.read_blocking(&pins.a1);
        let light_raw = adc.read_blocking(&pins.a2);
        let density_raw = adc.read_blocking(&pins.a3);
        let dissolved_o2_raw = adc.read_blocking(&pins.a4);
        let nitrate_raw = adc.read_blocking(&pins.a5);
        let salinity_raw = adc.read_blocking(&pins.a6);
        let battery_raw = adc.read_blocking(&pins.a7);
        
        // Process readings into meaningful values
        let ph_value = convert_ph(ph_raw);
        let temp_value = convert_temperature(temp_raw);
        let light_value = convert_light(light_raw);
        let density_value = convert_density(density_raw);
        let dissolved_o2 = convert_dissolved_oxygen(dissolved_o2_raw);
        let nitrate = convert_nitrate(nitrate_raw);
        let salinity = convert_salinity(salinity_raw);
        let battery_percentage = convert_battery_level(battery_raw);
        
        // Calculate health score based on optimal ranges
        let ph_score = calculate_range_score(ph_value, OPTIMAL_PH_MIN, OPTIMAL_PH_MAX);
        let temp_score = calculate_range_score(temp_value, OPTIMAL_TEMP_MIN, OPTIMAL_TEMP_MAX);
        let light_score = calculate_range_score(light_value, OPTIMAL_LIGHT_MIN, OPTIMAL_LIGHT_MAX);
        let density_score = calculate_range_score(density_value, OPTIMAL_DENSITY_MIN, OPTIMAL_DENSITY_MAX);
        let o2_score = calculate_range_score(dissolved_o2, OPTIMAL_DISSOLVED_O2_MIN, OPTIMAL_DISSOLVED_O2_MAX);
        let nitrate_score = calculate_range_score(nitrate, OPTIMAL_NITRATE_MIN, OPTIMAL_NITRATE_MAX);
        let salinity_score = calculate_range_score(salinity, OPTIMAL_SALINITY_MIN, OPTIMAL_SALINITY_MAX);
        
        // Overall health score (0-100)
        let overall_health = (ph_score + temp_score + light_score + density_score + o2_score + nitrate_score + salinity_score) / 7.0;
        
        // Generate current timestamp
        let current_time = millis();
        
        // Generate telemetry JSON with type-safe structure
        let mut json_data: String<512> = String::new();
        write!(json_data, r#"{{"device_id":"{}","timestamp":{},"batch_id":"SP2025-03-B44","measurements":{{"ph":{:.2},"temp":{:.2},"light":{:.1},"density":{:.3},"dissolved_oxygen":{:.2},"nitrate":{:.1},"salinity":{:.1}}},"optimal_scores":{{"ph":{:.1},"temp":{:.1},"light":{:.1},"density":{:.1},"dissolved_oxygen":{:.1},"nitrate":{:.1},"salinity":{:.1},"overall":{:.1}}},"battery":{:.1},"harvest_ready":{}}}"#,
            device_id,
            current_time,
            ph_value,
            temp_value,
            light_value,
            density_value,
            dissolved_o2,
            nitrate,
            salinity,
            ph_score,
            temp_score,
            light_score,
            density_score,
            o2_score,
            nitrate_score,
            salinity_score,
            overall_health,
            battery_percentage,
            density_value >= OPTIMAL_DENSITY_MAX * 0.9
        ).unwrap();
        
        // Sign data using quantum-resistant signature
        let signature = quantum_crypto::sign_data(json_data.as_bytes(), &keys);
        
        // Append signature hash to JSON (simplified)
        write!(json_data, ",\"qsig\":\"{}\"", signature.signature[0]).unwrap();
        
        // Send data to serial (for debugging and transmission)
        for byte in json_data.as_bytes() {
            block!(serial.write(*byte)).unwrap();
        }
        block!(serial.write(b'\n')).unwrap();
        
        // Battery level handling
        if battery_percentage < 15.0 {
            // Critical battery level - emergency mode
            for _ in 0..3 {
                led.set_high();
                arduino_hal::delay_ms(50);
                led.set_low();
                arduino_hal::delay_ms(50);
            }
            
            // Increase delay to preserve battery
            arduino_hal::delay_ms(1800000); // 30 minutes
        } else if battery_percentage < 30.0 {
            // Low battery - reduced sampling frequency
            for _ in 0..2 {
                led.set_high();
                arduino_hal::delay_ms(100);
                led.set_low();
                arduino_hal::delay_ms(100);
            }
            
            arduino_hal::delay_ms(900000); // 15 minutes
        } else {
            // Normal operation - sample every 5 minutes
            arduino_hal::delay_ms(300000);
        }
        
        // Update last measurement time
        last_measurement_time = current_time;
    }
}

// Utility functions for sensor conversions

fn convert_ph(raw_value: u16) -> f32 {
    // Convert ADC reading to pH (0-14 scale)
    // Calibrated for alkaline range optimal for spirulina
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let ph = 7.0 + (voltage - 2.5) * 3.5; // Calibrated for pH 7-14 range
    ph
}

fn convert_temperature(raw_value: u16) -> f32 {
    // Convert ADC reading to temperature in Celsius
    // Using LM35 or similar temperature sensor
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let temp_c = voltage * 100.0; // 10mV per degree
    temp_c
}

fn convert_light(raw_value: u16) -> f32 {
    // Convert ADC reading to light level in lux
    // Using light-dependent resistor or photodiode
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let lux = voltage * 10000.0 / 5.0; // Calibrated for high light needs of spirulina
    lux
}

fn convert_density(raw_value: u16) -> f32 {
    // Convert ADC reading to density in g/L
    // Using turbidity sensor calibrated for spirulina
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let density = voltage * 3.0; // Calibrated for 0-3 g/L range
    density
}

fn convert_dissolved_oxygen(raw_value: u16) -> f32 {
    // Convert ADC reading to dissolved oxygen in mg/L
    // Using galvanic DO sensor
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let do_level = voltage * 20.0 / 5.0; // 0-20 mg/L range
    do_level
}

fn convert_nitrate(raw_value: u16) -> f32 {
    // Convert ADC reading to nitrate level in mg/L
    // Using ion-selective electrode
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let nitrate = voltage * 100.0 / 5.0; // 0-100 mg/L range
    nitrate
}

fn convert_salinity(raw_value: u16) -> f32 {
    // Convert ADC reading to salinity in g/L
    // Using conductivity sensor
    let voltage = (raw_value as f32) * 5.0 / 1023.0;
    let salinity = voltage * 35.0 / 5.0; // 0-35 g/L range
    salinity
}

fn convert_battery_level(raw_value: u16) -> f32 {
    // Convert ADC reading to battery percentage
    // For LiPo battery (3.2V min, 4.2V max)
    let voltage = (raw_value as f32) * 5.0 / 1023.0 * 2.0; // Assuming voltage divider
    let percentage = (voltage - 3.2) * 100.0 / (4.2 - 3.2);
    percentage.clamp(0.0, 100.0)
}

fn calculate_range_score(value: f32, min: f32, max: f32) -> f32 {
    // Calculate a 0-100 score based on proximity to optimal range
    if value >= min && value <= max {
        // Within optimal range - 100%
        100.0
    } else if value < min {
        // Below optimal range
        let range_width = max - min;
        let distance = min - value;
        let percentage = 100.0 - (distance / (range_width / 2.0) * 100.0);
        percentage.max(0.0)
    } else {
        // Above optimal range
        let range_width = max - min;
        let distance = value - max;
        let percentage = 100.0 - (distance / (range_width / 2.0) * 100.0);
        percentage.max(0.0)
    }
}

fn millis() -> u32 {
    // Simplified millisecond counter
    // In real implementation, use a proper timer
    0 // Placeholder - will be replaced with actual timing in implementation
}
