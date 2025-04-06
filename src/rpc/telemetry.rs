//! RPC endpoints for ELXR telemetry data
//!
//! Provides access to kombucha fermentation telemetry data via RPC.

use jsonrpsee::{
    core::{async_trait, Error as JsonRpseeError, RpcResult},
    proc_macros::rpc,
    types::error::{CallError, ErrorCode, ErrorObject},
};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;

use crate::telemetry;
use crate::telemetry::api::{TelemetrySummary, DetailedTelemetry};

#[rpc(server)]
pub trait TelemetryRpc {
    /// Get all batch IDs
    #[method(name = "elxr_getAllBatchIds")]
    fn get_all_batch_ids(&self) -> RpcResult<Vec<String>>;
    
    /// Get a summary of all telemetry sessions
    #[method(name = "elxr_getAllSummaries")]
    fn get_all_summaries(&self) -> RpcResult<Vec<TelemetrySummary>>;
    
    /// Get detailed telemetry for a specific batch
    #[method(name = "elxr_getDetailedTelemetry")]
    fn get_detailed_telemetry(&self, batch_id: String) -> RpcResult<Option<DetailedTelemetry>>;
    
    /// Export telemetry data to blockchain
    #[method(name = "elxr_exportToBlockchain")]
    fn export_to_blockchain(&self, batch_id: String) -> RpcResult<String>;
}

/// Telemetry RPC API implementation.
pub struct TelemetryRpcImpl {
    /// Reference to the telemetry API
    telemetry_api: Option<Arc<telemetry::api::TelemetryApi>>,
}

impl TelemetryRpcImpl {
    /// Create a new Telemetry RPC API implementation.
    pub fn new() -> Self {
        Self {
            telemetry_api: telemetry::get_api(),
        }
    }
}

#[async_trait]
impl TelemetryRpcServer for TelemetryRpcImpl {
    fn get_all_batch_ids(&self) -> RpcResult<Vec<String>> {
        match &self.telemetry_api {
            Some(api) => Ok(api.get_all_batch_ids()),
            None => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::borrowed(
                ErrorCode::InternalError.code(),
                "Telemetry API not initialized",
                None,
            )))),
        }
    }
    
    fn get_all_summaries(&self) -> RpcResult<Vec<TelemetrySummary>> {
        match &self.telemetry_api {
            Some(api) => Ok(api.get_all_summaries()),
            None => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::borrowed(
                ErrorCode::InternalError.code(),
                "Telemetry API not initialized",
                None,
            )))),
        }
    }
    
    fn get_detailed_telemetry(&self, batch_id: String) -> RpcResult<Option<DetailedTelemetry>> {
        match &self.telemetry_api {
            Some(api) => Ok(api.get_detailed_telemetry(&batch_id)),
            None => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::borrowed(
                ErrorCode::InternalError.code(),
                "Telemetry API not initialized",
                None,
            )))),
        }
    }
    
    fn export_to_blockchain(&self, batch_id: String) -> RpcResult<String> {
        match &self.telemetry_api {
            Some(api) => {
                match api.export_to_blockchain(&batch_id) {
                    Ok(hash) => {
                        // Convert hash bytes to hex string and ensure quantum resistance using blake3
                        let hex_hash = hash.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect::<String>();
                        Ok(hex_hash)
                    },
                    Err(e) => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::borrowed(
                        ErrorCode::InvalidRequest.code(),
                        e,
                        None,
                    )))),
                }
            },
            None => Err(JsonRpseeError::Call(CallError::Custom(ErrorObject::borrowed(
                ErrorCode::InternalError.code(),
                "Telemetry API not initialized",
                None,
            )))),
        }
    }
}
