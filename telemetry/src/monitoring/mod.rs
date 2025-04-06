//! Monitoring systems for ELXR chain telemetry
//!
//! Provides integrated monitoring systems that combine multiple sensors
//! with quantum-secured data collection, time-series analysis, and
//! blockchain submission capabilities.

pub mod kombucha_monitor;

pub use kombucha_monitor::{
    KombuchaMonitor,
    MonitoringThresholds,
    AlertSeverity,
    FermentationAlert, 
    CombinedFermentationData,
    create_default_monitor,
};
