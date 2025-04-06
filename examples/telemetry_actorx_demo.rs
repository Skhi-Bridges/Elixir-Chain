//! Elixir Chain (ELXR) Telemetry ActorX Integration Demo
//!
//! This example demonstrates how to use the quantum-secured sensors
//! for kombucha monitoring with ActorX framework integration.
//! Shows end-to-end telemetry from reading to blockchain submission.

use elxr_telemetry::{
    actorx_integration::ElxrSensorActorFactory,
    error::TelemetryResult,
    crypto::QuantumCrypto,
};

use actorx_frameworks::core::{
    actor::Actor,
    chain::SubmissionStatus,
    error::ActorError,
};

use std::time::Duration;
use chrono::{Utc, DateTime};
use tokio::time;

/// Print sensor reading details in a formatted way
fn print_alcohol_reading(reading_id: &str, timestamp: DateTime<Utc>, abv: f32, forecast: f32) {
    println!("┌─────────────────────────────────────────────────┐");
    println!("│       QUANTUM-SECURED ALCOHOL READING           │");
    println!("├─────────────────────────────────────────────────┤");
    println!("│ ID:        {:<35} │", reading_id);
    println!("│ Timestamp: {:<35} │", timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("│ ABV:       {:<35} │", format!("{:.2}%", abv));
    println!("│ Forecast:  {:<35} │", format!("{:.2}% (24h prediction)", forecast));
    println!("└─────────────────────────────────────────────────┘");
}

/// Print turbidity reading details in a formatted way
fn print_turbidity_reading(reading_id: &str, timestamp: DateTime<Utc>, ntu: f32, clarity: f32, forecast: f32) {
    println!("┌─────────────────────────────────────────────────┐");
    println!("│       QUANTUM-SECURED TURBIDITY READING         │");
    println!("├─────────────────────────────────────────────────┤");
    println!("│ ID:        {:<35} │", reading_id);
    println!("│ Timestamp: {:<35} │", timestamp.format("%Y-%m-%d %H:%M:%S UTC"));
    println!("│ NTU:       {:<35} │", format!("{:.2}", ntu));
    println!("│ Clarity:   {:<35} │", format!("{:.1}/100", clarity));
    println!("│ Forecast:  {:<35} │", format!("{:.2} NTU (24h prediction)", forecast));
    println!("└─────────────────────────────────────────────────┘");
}

/// Print submission results
fn print_submission_result(status: SubmissionStatus, tx_hash: &str) {
    println!("┌─────────────────────────────────────────────────┐");
    println!("│        QUANTUM-SECURED CHAIN SUBMISSION         │");
    println!("├─────────────────────────────────────────────────┤");
    println!("│ Status:    {:<35} │", format!("{:?}", status));
    println!("│ TX Hash:   {:<35} │", tx_hash);
    println!("└─────────────────────────────────────────────────┘");
}

/// Main function for the demo
#[tokio::main]
async fn main() -> TelemetryResult<()> {
    // Welcome message
    println!("==================================================");
    println!("  ELIXIR CHAIN QUANTUM-SECURED TELEMETRY DEMO");
    println!("  Kombucha Environmental Monitoring System");
    println!("==================================================");
    println!();
    
    // Initialize the sensor actor factory
    println!("Initializing sensor actor factory with quantum security...");
    let factory = ElxrSensorActorFactory::new().await?;
    
    // Create alcohol sensor actor
    println!("Creating alcohol sensor actor...");
    let alcohol_actor = factory.create_alcohol_sensor_actor().await?;
    
    // Create turbidity sensor actor
    println!("Creating turbidity sensor actor...");
    let turbidity_actor = factory.create_turbidity_sensor_actor().await?;
    
    // Take an alcohol reading and submit to blockchain
    println!("\nTaking alcohol reading with quantum security...");
    let alcohol_result = alcohol_actor.take_reading_and_submit().await?;
    
    // Display alcohol reading results
    print_alcohol_reading(
        &alcohol_result.submission_id,
        alcohol_result.timestamp,
        alcohol_result.metadata["abv_percent"].parse::<f32>().unwrap_or(0.0),
        alcohol_result.metadata["forecast_24h"].parse::<f32>().unwrap_or(0.0),
    );
    
    // Display submission result
    print_submission_result(
        alcohol_result.status,
        &alcohol_result.transaction_hash,
    );
    
    // Small delay between readings
    time::sleep(Duration::from_secs(2)).await;
    
    // Take a turbidity reading and submit to blockchain
    println!("\nTaking turbidity reading with quantum security...");
    let turbidity_result = turbidity_actor.take_reading_and_submit().await?;
    
    // Display turbidity reading results
    print_turbidity_reading(
        &turbidity_result.submission_id,
        turbidity_result.timestamp,
        turbidity_result.metadata["ntu_value"].parse::<f32>().unwrap_or(0.0),
        turbidity_result.metadata["clarity_score"].parse::<f32>().unwrap_or(0.0),
        turbidity_result.metadata["forecast_24h"].parse::<f32>().unwrap_or(0.0),
    );
    
    // Display submission result
    print_submission_result(
        turbidity_result.status,
        &turbidity_result.transaction_hash,
    );
    
    // Demonstrate time-series capabilities
    println!("\nDemonstrating time-series forecasting capabilities:");
    println!("- Readings include 72-hour forecasts with confidence intervals");
    println!("- Forecasts are adjusted based on fermentation phase");
    println!("- Temperature correlation improves prediction accuracy");
    println!("- Quantum-secured timestamps ensure data integrity");
    println!("- Error correction operates at classical, bridge, and quantum levels");
    
    println!("\nSuccessfully completed telemetry demo with quantum security!");
    
    Ok(())
}
