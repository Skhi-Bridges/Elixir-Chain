//! ELXR Pallets Module
//!
//! This module exposes the various pallets implemented for the ELXR chain,
//! including DeFi and Eigenlayer functionality for kombucha production.

pub mod defi;
pub mod eigenlayer;

pub use defi::{Config as DefiConfig, Module as DefiModule, Call as DefiCall, Event as DefiEvent};
pub use eigenlayer::{Config as EigenlayerConfig, Module as EigenlayerModule, Call as EigenlayerCall, Event as EigenlayerEvent};
