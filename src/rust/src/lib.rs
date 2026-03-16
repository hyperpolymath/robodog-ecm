// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! # Robodog ECM — Electronic Countermeasures & Defensive Technologies
//!
//! DEFENSIVE USE ONLY — Electronic countermeasures for protective applications.
//! Export control classification: Wassenaar Category 5A2 (Crypto), Category 11 (ECM).
//! See EXPORT-CONTROL.md for compliance requirements.
//!
//! This crate provides the core Rust implementation for:
//!
//! - **ECM signal analysis** — spectrum monitoring, interference detection, jamming
//!   pattern recognition. Simulation-only, no real RF hardware interface.
//! - **Post-quantum cryptographic protocols** — Kyber1024 key encapsulation,
//!   Dilithium5 signatures, SPHINCS+ fallback. Hybrid classical+PQ mode.
//! - **Formation control** — distributed coordination algorithms for multi-robot
//!   systems. Defensive formations for SAR and disaster response.
//! - **Defensive autonomy** — collision avoidance, threat response, safe-state
//!   transitions. All safety-critical logic has SPARK proof counterparts.

pub mod autonomy;
pub mod crypto;
pub mod ecm;
pub mod formation;

/// Crate-level version, synchronised with Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Export control classification for this crate.
pub const EXPORT_CLASSIFICATION: &str = "Wassenaar Cat 5A2 / Cat 11 — Defensive Use Only";
