//! Elixir Chain Node
//! 
//! A Substrate-based blockchain for kombucha fermentation verification
//! using quantum-resistant security technologies.

use std::net::SocketAddr;
use structopt::StructOpt;

use sc_cli::{RunCmd, Subcommand};
use sc_service::config::PrometheusConfig;

#[derive(Debug, StructOpt)]
pub struct Cli {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    #[structopt(flatten)]
    pub run: RunCmd,

    /// Enable Prometheus metrics
    #[structopt(long)]
    pub prometheus: bool,

    /// Prometheus metrics endpoint
    #[structopt(long, default_value = "127.0.0.1:9615")]
    pub prometheus_endpoint: String,

    /// Telemetry URL
    #[structopt(long)]
    pub telemetry_url: Option<String>,
}

fn main() -> sc_cli::Result<()> {
    let cli = Cli::from_args();

    let prometheus_config = if cli.prometheus {
        let prometheus_endpoint: SocketAddr = cli.prometheus_endpoint.parse()
            .expect("Prometheus endpoint must be a valid socket address");
        Some(PrometheusConfig::new_with_default_registry(prometheus_endpoint))
    } else {
        None
    };

    match cli.subcommand {
        Some(ref subcommand) => {
            let runner = cli.create_runner(subcommand)?;
            runner.run_subcommand(subcommand, |config| {
                let (builder, _, _) = service::new_full_parts::<
                    elixir_runtime::RuntimeApi,
                    elixir_runtime::Executor,
                >(&config)?;
                Ok(builder.into_chain_ops())
            })
        }
        None => {
            let runner = cli.create_runner(&cli.run)?;
            runner.run_node_until_exit(|config| async move {
                service::new_full(config, prometheus_config, cli.telemetry_url).map_err(sc_cli::Error::Service)
            })
        }
    }
}

// Include module definitions
mod service;
mod chain_spec;
mod commands;
