# Elixir Chain (ELXR)

A quantum-resistant blockchain for kombucha production verification, global quality assurance, and decentralized exchange.

## Overview

Elixir Chain (ELXR) provides a robust solution for kombucha producers to verify and track their processes, connect with consumers, and participate in a decentralized marketplace. Built on Substrate with quantum-resistant cryptography and three-level error correction.

## Components

- **Core Pallet**: Manages kombucha verification, brewer registration, and batch tracking
- **Oracle**: Daemonless oracle providing price feeds and verified fermentation data
- **Eigenlayer Integration**: Enables restaking of ELXR tokens with quantum-resistant verification
- **Liquidity Pools**: Shared liquidity system with NRSH chain
- **Frontend**: Leptos UI for fermentation monitoring and Next.js for DEX and social features

## Quantum Resistance & Error Correction

- Utilizes CRYSTALS Kyber-512 for key exchange
- CRYSTALS Dilithium-1024 for signatures
- Comprehensive error correction at three levels:
  - Classical: Reed-Solomon codes
  - Bridge: Custom redundancy protocols
  - Quantum: Surface codes for quantum error correction (QEC)

## Repository Structure

- /src: Core implementation
  - /pallet: Substrate pallets including oracle and data verification
  - /eigenlayer: Integration with Eigenlayer staking
- /runtime: Parachain and runtime components
- /node: Substrate node implementation
- /telemetry: Kombucha telemetry systems
- /frontend: UI implementations
  - /leptos: Rust-based UI
  - /nextjs: Web interface for DEX and social features
- /docs: Documentation and whitepapers

## Integration with NRSH

ELXR and NRSH (Nourish Chain) form the backbone of the Matrix-Magiq ecosystem, connected through:
- Shared liquidity pools for ELXR/NRSH trading
- Cross-chain verification of supply chain data
- Compatible validation mechanisms
- Eigenlayer integration for shared security

## Getting Started

See individual READMEs in each directory for setup instructions.
