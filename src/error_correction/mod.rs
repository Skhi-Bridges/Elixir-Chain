//! Error Correction Module for the ELXR Chain
//!
//! Implements multiple levels of error correction:
//! 1. Classical Error Correction (Reed-Solomon)
//! 2. Bridge Error Correction (for classical-quantum interface)
//! 3. Quantum Error Correction (Surface codes)
//!
//! This module ensures data integrity across all operations in the ELXR chain,
//! particularly during fermentation tracking and verification processes.

mod classical;
mod bridge;
mod quantum;

pub use classical::ClassicalErrorCorrection;
pub use bridge::BridgeErrorCorrection;
pub use quantum::QuantumErrorCorrection;

use codec::{Decode, Encode};
use sp_std::prelude::*;

/// Error correction trait implemented by all correction strategies
pub trait ErrorCorrection {
    /// Encode data with error correction codes
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>, ErrorCorrectionError>;
    
    /// Decode and correct errors in data
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, ErrorCorrectionError>;
    
    /// Check if data has errors that need correction
    fn has_errors(&self, data: &[u8]) -> bool;
    
    /// Get the type of error correction
    fn correction_type(&self) -> ErrorCorrectionType;
}

/// Error correction types available in the system
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq)]
pub enum ErrorCorrectionType {
    /// Classical error correction (Reed-Solomon, LDPC, etc.)
    Classical,
    /// Bridge error correction (hybrid classical-quantum)
    Bridge,
    /// Quantum error correction (Surface codes, Steane codes, etc.)
    Quantum,
    /// Comprehensive error correction (all levels combined)
    Comprehensive,
}

/// Error types that can occur during error correction
#[derive(Debug)]
pub enum ErrorCorrectionError {
    /// Data is too corrupted to be corrected
    Uncorrectable,
    /// Invalid input data format
    InvalidData,
    /// Error in the error correction algorithm
    AlgorithmError,
    /// Unsupported error correction type
    UnsupportedType,
}

/// Comprehensive error correction applying all levels
pub struct ComprehensiveErrorCorrection {
    /// Classical error correction
    classical: ClassicalErrorCorrection,
    /// Bridge error correction
    bridge: BridgeErrorCorrection,
    /// Quantum error correction
    quantum: QuantumErrorCorrection,
}

impl ComprehensiveErrorCorrection {
    /// Create a new comprehensive error correction
    pub fn new() -> Self {
        Self {
            classical: ClassicalErrorCorrection::new(10), // 10% redundancy
            bridge: BridgeErrorCorrection::new(),
            quantum: QuantumErrorCorrection::new(),
        }
    }
}

impl ErrorCorrection for ComprehensiveErrorCorrection {
    fn encode(&self, data: &[u8]) -> Result<Vec<u8>, ErrorCorrectionError> {
        // Apply all three levels of error correction sequentially
        let classical_encoded = self.classical.encode(data)?;
        let bridge_encoded = self.bridge.encode(&classical_encoded)?;
        self.quantum.encode(&bridge_encoded)
    }
    
    fn decode(&self, data: &[u8]) -> Result<Vec<u8>, ErrorCorrectionError> {
        // Decode in reverse order
        let quantum_decoded = self.quantum.decode(data)?;
        let bridge_decoded = self.bridge.decode(&quantum_decoded)?;
        self.classical.decode(&bridge_decoded)
    }
    
    fn has_errors(&self, data: &[u8]) -> bool {
        // Check for errors at any level
        self.quantum.has_errors(data) || 
        self.bridge.has_errors(data) || 
        self.classical.has_errors(data)
    }
    
    fn correction_type(&self) -> ErrorCorrectionType {
        ErrorCorrectionType::Comprehensive
    }
}

/// Create an error correction instance based on the specified type
pub fn create_error_correction(correction_type: ErrorCorrectionType) -> Box<dyn ErrorCorrection> {
    match correction_type {
        ErrorCorrectionType::Classical => Box::new(ClassicalErrorCorrection::new(8)),
        ErrorCorrectionType::Bridge => Box::new(BridgeErrorCorrection::new()),
        ErrorCorrectionType::Quantum => Box::new(QuantumErrorCorrection::new()),
        ErrorCorrectionType::Comprehensive => Box::new(ComprehensiveErrorCorrection::new()),
    }
}
