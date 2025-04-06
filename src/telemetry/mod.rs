//! Telemetry module for ELXR chain
//! 
//! This module provides integration between the kombucha fermentation telemetry
//! sensors and the blockchain, allowing for transparent reporting
//! and verification of brewing data.

pub mod api;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use std::thread;

static TELEMETRY_INITIALIZED: AtomicBool = AtomicBool::new(false);
static mut TELEMETRY_API: Option<Arc<api::TelemetryApi>> = None;

/// Initialize the telemetry subsystem
pub fn initialize() -> Result<(), &'static str> {
    // Only initialize once
    if TELEMETRY_INITIALIZED.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).is_err() {
        return Ok(());
    }
    
    // Create and initialize the API
    let api = api::initialize();
    
    // Store the API singleton
    unsafe {
        TELEMETRY_API = Some(api.clone());
    }
    
    // Start the background update thread
    thread::spawn(move || {
        // Wait for initial startup to complete
        thread::sleep(Duration::from_secs(5));
        
        loop {
            // Update dummy data every 30 seconds
            if let Err(e) = api.update_dummy_data() {
                log::error!("Failed to update dummy telemetry data: {}", e);
            }
            
            // Wait for next update
            thread::sleep(Duration::from_secs(30));
        }
    });
    
    Ok(())
}

/// Get a reference to the telemetry API
pub fn get_api() -> Option<Arc<api::TelemetryApi>> {
    unsafe {
        TELEMETRY_API.clone()
    }
}

/// Reexport from telemetry module
pub use crate::telemetry::{
    SensorDevice, SensorType, TelemetryReading, FermentationTelemetry,
    FermentationStage, DeviceStatus, ReadingStatus
};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_telemetry_initialization() {
        initialize().unwrap();
        
        let api = get_api();
        assert!(api.is_some());
        
        let api = api.unwrap();
        let batches = api.get_all_batch_ids();
        assert!(!batches.is_empty());
    }
}
