//! Service and ServiceFactory implementation for Elixir Chain.
//! Specialized wrapper over substrate service.

use std::{sync::Arc, time::Duration};

use elixir_runtime::{self, opaque::Block, RuntimeApi};
use sc_consensus_aura::{ImportQueueParams, SlotProportion, StartAuraParams};
use sc_executor::NativeElseWasmExecutor;
use sc_finality_grandpa::{
    GrandpaBlockImport, GrandpaFinalityProofProvider, SharedVoterState,
    FinalityProofProvider,
};
use sc_service::{
    error::Error as ServiceError, Configuration, TaskManager, TFullBackend,
    TFullClient, TLightBackend, TLightClient, TLightClientWithBackend,
    TLightClientBackend, Role, DatabaseSource,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_consensus::CanAuthorWithNativeVersion;
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_api::ConstructRuntimeApi;
use sc_basic_authorship::ProposerFactory;
use sc_consensus::DefaultImportQueue;
use futures::prelude::*;
use sc_client_api::{Backend, BlockBackend};
use sp_runtime::traits::Block as BlockT;

type FullClient = sc_service::TFullClient<
    Block,
    RuntimeApi,
    NativeElseWasmExecutor<elixir_runtime::Executor>,
>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type GrandpaBlockImportType = GrandpaBlockImport<
    FullBackend,
    Block,
    FullClient,
    FullSelectChain,
>;

/// Returns most parts of a service. Not enough to run a full chain,
/// but enough to perform chain operations like purge-chain
pub fn new_full_parts<RuntimeApi, Executor>(
    config: &Configuration,
) -> Result<
    (
        sc_service::TaskManager,
        Arc<FullClient>,
        sc_consensus::LongestChain<FullBackend, Block>,
    ),
    ServiceError,
>
where
    RuntimeApi: ConstructRuntimeApi<Block, FullClient> + Send + Sync + 'static,
    RuntimeApi::RuntimeApi:
        RuntimeApiCollection<StateBackend = sc_client_api::StateBackendFor<FullBackend, Block>>,
    Executor: sc_executor::NativeExecutionDispatch + 'static,
{
    let telemetry = config.telemetry_endpoints.clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, RuntimeApi, _>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    Ok((task_manager, client, select_chain))
}

/// Builds a new service for a full client.
pub fn new_full(
    config: Configuration,
    prometheus_config: Option<sc_service::config::PrometheusConfig>,
    telemetry_url: Option<String>,
) -> Result<TaskManager, ServiceError> {
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (block_import, grandpa_link, mut telemetry),
    } = new_partial(&config)?;

    // Configure Prometheus metrics if enabled
    if let Some(prometheus_config) = prometheus_config {
        let metrics_service = sc_service::spawn_prometheus_metrics(&prometheus_config)?;
        task_manager.spawn_handle().spawn("prometheus-metrics", None, metrics_service);
    }

    // Setup custom telemetry
    if let Some(url) = telemetry_url {
        if let Some(telemetry) = telemetry.as_mut() {
            telemetry.connect_to_telemetry_endpoints(vec![url])?;
        }
    }

    // Spawn network tasks
    sc_service::build_network(sc_service::BuildNetworkParams {
        config: &config,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        spawn_handle: task_manager.spawn_handle(),
        import_queue,
        on_demand: None,
        block_announce_validator_builder: None,
    })?;

    // Initialize RPC
    let rpc_extensions_builder = {
        let client = client.clone();
        let pool = transaction_pool.clone();

        Box::new(move |deny_unsafe, _| {
            let deps = crate::rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                deny_unsafe,
            };

            crate::rpc::create_full(deps)
        })
    };

    // Spawn authorship tasks
    if matches!(role, Role::Authority) {
        let proposer_factory = ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool,
            prometheus_registry.as_ref(),
        );

        let can_author_with = CanAuthorWithNativeVersion::new(client.executor().clone());

        let aura = sc_consensus_aura::start_aura::<AuraPair, _, _, _, _, _, _, _, _, _>(
            StartAuraParams {
                slot_duration: sc_consensus_aura::slot_duration(&*client)?,
                client: client.clone(),
                select_chain,
                block_import,
                proposer_factory,
                create_inherent_data_providers: move |_, ()| async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                    let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                        *timestamp,
                        slot_duration,
                    );
                    Ok((timestamp, slot))
                },
                force_authoring,
                backoff_authoring_blocks,
                keystore: keystore_container.sync_keystore(),
                can_author_with,
                sync_oracle: network.clone(),
                block_proposal_slot_portion: SlotProportion::new(2f32 / 3f32),
                telemetry: telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            },
        )?;

        task_manager.spawn_essential_handle().spawn_blocking(
            "aura",
            None,
            aura,
        );
    }

    // Spawn GRANDPA tasks
    sc_finality_grandpa::spawn_grandpa_node(
        config,
        task_manager.spawn_essential_handle(),
        client.clone(),
        network,
        grandpa_link,
        keystore_container.sync_keystore(),
        telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
    )?;

    Ok(task_manager)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<TaskManager, ServiceError> {
    let telemetry = config.telemetry_endpoints.clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<elixir_runtime::Executor>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
    );

    let (client, backend, keystore_container, mut task_manager, on_demand) =
        sc_service::new_light_parts::<Block, RuntimeApi, _>(
            &config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let import_queue = sc_consensus_aura::import_queue::<AuraPair, _, _, _, _, _>(
        ImportQueueParams {
            block_import: client.clone(),
            justification_import: None,
            client: client.clone(),
            create_inherent_data_providers: move |_, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
                let slot = sp_consensus_aura::inherents::InherentDataProvider::from_timestamp_and_duration(
                    *timestamp,
                    Duration::from_millis(SLOT_DURATION),
                );
                Ok((timestamp, slot))
            },
            spawner: &task_manager.spawn_essential_handle(),
            can_author_with: sp_consensus::NeverCanAuthor,
            registry: None,
            check_for_equivocation: Default::default(),
            telemetry: telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
        },
    )?;

    let (network, system_rpc_tx, network_starter) = sc_service::build_network(sc_service::BuildNetworkParams {
        config: &config,
        client: client.clone(),
        transaction_pool: transaction_pool.clone(),
        spawn_handle: task_manager.spawn_handle(),
        import_queue,
        on_demand: Some(on_demand.clone()),
        block_announce_validator_builder: None,
    })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config, task_manager.spawn_handle(), client.clone(), network.clone(),
        );
    }

    sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        remote_blockchain: Some(backend.remote_blockchain()),
        transaction_pool,
        task_manager: &mut task_manager,
        on_demand: Some(on_demand),
        rpc_extensions_builder: Box::new(|_, _| Ok(())),
        config,
        client,
        keystore: keystore_container.sync_keystore(),
        backend,
        network,
        system_rpc_tx,
        telemetry: telemetry.map(|(worker, telemetry)| {
            task_manager.spawn_handle().spawn("telemetry", None, worker);
            telemetry
        }),
    })?;

    network_starter.start_network();

    Ok(task_manager)
}

// Helper trait to define runtime API implementations required by the node
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::Metadata<Block>
    + sp_session::SessionKeys<Block>
    + sp_api::ApiExt<Block>
    + sp_block_builder::BlockBuilder<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_consensus_aura::AuraApi<Block, AuraPair>
    + sp_finality_grandpa::GrandpaApi<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, elixir_runtime::AccountId, elixir_runtime::Index>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, elixir_runtime::Balance>
    + substrate_frame_rpc_system::AccountNonceApi<Block, elixir_runtime::AccountId, elixir_runtime::Index>
{}
