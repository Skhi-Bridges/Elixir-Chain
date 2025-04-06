//! Service implementation for Eigenlayer integration
use crate::eigenlayer::{
    client::EigenlayerClient,
    config::EigenConfig,
    operator::{OperatorManager, OperatorInfo},
    types::{RestakeInfo, QuorumInfo, StakeAmount},
};
use anyhow::{Result, Context};
use eigensdk::eigen_services::types::EigenMetrics;
use log::{info, error, warn, debug};
use std::{sync::{Arc, Mutex}, time::Duration};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{self, Receiver, Sender},
    task::JoinHandle,
    time,
};

/// Message types for the Eigenlayer service
#[derive(Debug)]
enum ServiceMessage {
    /// Request information about an operator
    GetOperator(Vec<u8>, Sender<Result<OperatorInfo>>),
    
    /// Request all operator information
    GetAllOperators(Sender<Result<Vec<OperatorInfo>>>),
    
    /// Request information about a quorum
    GetQuorum(u8, Sender<Result<QuorumInfo>>),
    
    /// Request all quorum information
    GetAllQuorums(Sender<Result<Vec<QuorumInfo>>>),
    
    /// Register a new operator
    RegisterOperator(Vec<u8>, Vec<u8>, Vec<u8>, Sender<Result<()>>),
    
    /// Record a successful validation by an operator
    RecordValidation(Vec<u8>, Sender<Result<()>>),
    
    /// Record a slash event for an operator
    RecordSlash(Vec<u8>, u8, Sender<Result<()>>),
    
    /// Stop the service
    Stop,
}

/// Eigenlayer integration service
pub struct EigenlayerService {
    /// Sender for the service message channel
    tx: Mutex<Option<Sender<ServiceMessage>>>,
    
    /// Handle for the service task
    task_handle: Mutex<Option<JoinHandle<()>>>,
    
    /// Runtime for async operations
    runtime: Arc<Runtime>,
}

impl EigenlayerService {
    /// Create a new Eigenlayer service
    pub fn new(config: EigenConfig) -> Result<Self> {
        // Create runtime for async operations
        let runtime = Arc::new(Runtime::new()?);
        
        // Create Eigenlayer client and operator manager inside the runtime
        let (client, operator_manager) = runtime.block_on(async {
            let client = EigenlayerClient::new(config)
                .context("Failed to create Eigenlayer client")?;
            
            let client_arc = Arc::new(client);
            
            let operator_manager = OperatorManager::new(client_arc)
                .context("Failed to create operator manager")?;
                
            Result::<_, anyhow::Error>::Ok((client_arc, operator_manager))
        })?;
        
        // Create channel for communicating with the service
        let (tx, rx) = mpsc::channel::<ServiceMessage>(100);
        
        // Create and start the service task
        let task_handle = runtime.spawn(Self::run_service(rx, operator_manager));
        
        Ok(Self {
            tx: Mutex::new(Some(tx)),
            task_handle: Mutex::new(Some(task_handle)),
            runtime,
        })
    }
    
    /// Main service loop
    async fn run_service(mut rx: Receiver<ServiceMessage>, manager: OperatorManager) {
        info!("Eigenlayer service started");
        
        // Periodic refresh task
        let refresh_handle = tokio::spawn(async move {
            let refresh_interval = Duration::from_secs(60); // Refresh every minute
            
            loop {
                time::sleep(refresh_interval).await;
                
                if let Err(e) = manager.refresh_operators() {
                    error!("Failed to refresh operators: {:?}", e);
                }
                
                if let Err(e) = manager.refresh_quorums() {
                    error!("Failed to refresh quorums: {:?}", e);
                }
            }
        });
        
        // Main message processing loop
        while let Some(msg) = rx.recv().await {
            match msg {
                ServiceMessage::GetOperator(address, reply) => {
                    let result = manager.get_operator(&address);
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::GetAllOperators(reply) => {
                    let result = manager.get_all_operators();
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::GetQuorum(id, reply) => {
                    let result = manager.get_quorum(id);
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::GetAllQuorums(reply) => {
                    let result = manager.get_all_quorums();
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::RegisterOperator(address, pubkey, sig, reply) => {
                    let result = manager.register_operator(&address, &pubkey, &sig);
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::RecordValidation(address, reply) => {
                    let result = manager.record_successful_validation(&address);
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::RecordSlash(address, severity, reply) => {
                    let result = manager.record_slash(&address, severity);
                    let _ = reply.send(result).await;
                }
                
                ServiceMessage::Stop => {
                    info!("Eigenlayer service stopping");
                    refresh_handle.abort();
                    break;
                }
            }
        }
        
        info!("Eigenlayer service stopped");
    }
    
    /// Get information about a specific operator
    pub fn get_operator(&self, operator_address: &[u8]) -> Result<OperatorInfo> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::GetOperator(
                operator_address.to_vec(), 
                reply_tx
            )).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to get operator info")),
            }
        })
    }
    
    /// Get all operators
    pub fn get_all_operators(&self) -> Result<Vec<OperatorInfo>> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::GetAllOperators(reply_tx)).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to get all operators")),
            }
        })
    }
    
    /// Get information about a specific quorum
    pub fn get_quorum(&self, quorum_id: u8) -> Result<QuorumInfo> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::GetQuorum(quorum_id, reply_tx)).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to get quorum info")),
            }
        })
    }
    
    /// Get all quorums
    pub fn get_all_quorums(&self) -> Result<Vec<QuorumInfo>> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::GetAllQuorums(reply_tx)).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to get all quorums")),
            }
        })
    }
    
    /// Register a new operator
    pub fn register_operator(&self, 
                            operator_address: &[u8], 
                            bls_public_key: &[u8], 
                            bls_signature: &[u8]) -> Result<()> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::RegisterOperator(
                operator_address.to_vec(),
                bls_public_key.to_vec(),
                bls_signature.to_vec(),
                reply_tx
            )).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to register operator")),
            }
        })
    }
    
    /// Record a successful validation by an operator
    pub fn record_successful_validation(&self, operator_address: &[u8]) -> Result<()> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::RecordValidation(
                operator_address.to_vec(),
                reply_tx
            )).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to record validation")),
            }
        })
    }
    
    /// Record a slash event for an operator
    pub fn record_slash(&self, operator_address: &[u8], severity: u8) -> Result<()> {
        let tx = self.tx.lock().unwrap();
        let tx = tx.as_ref().ok_or_else(|| anyhow::anyhow!("Service not running"))?;
        
        // Create a channel for the reply
        let (reply_tx, reply_rx) = mpsc::channel(1);
        
        // Send the request
        self.runtime.block_on(async {
            tx.send(ServiceMessage::RecordSlash(
                operator_address.to_vec(),
                severity,
                reply_tx
            )).await
        })?;
        
        // Wait for the reply
        self.runtime.block_on(async {
            match reply_rx.recv().await {
                Some(result) => result,
                None => Err(anyhow::anyhow!("Failed to record slash")),
            }
        })
    }
    
    /// Stop the service
    pub fn stop(&self) -> Result<()> {
        let mut tx_guard = self.tx.lock().unwrap();
        
        if let Some(tx) = tx_guard.take() {
            // Send stop message
            self.runtime.block_on(async {
                let _ = tx.send(ServiceMessage::Stop).await;
            });
            
            // Wait for the task to finish
            let mut handle_guard = self.task_handle.lock().unwrap();
            if let Some(handle) = handle_guard.take() {
                self.runtime.block_on(async {
                    let _ = handle.await;
                });
            }
            
            info!("Eigenlayer service stopped");
        }
        
        Ok(())
    }
}

impl Drop for EigenlayerService {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
