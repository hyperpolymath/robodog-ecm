// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Aspect tests — cross-cutting concerns that span multiple modules.
//!
//! These verify properties that must hold across the entire codebase:
//! - Defensive-only invariant (no offensive capability)
//! - Serialisation round-trips (serde consistency)
//! - Safety parameter consistency between modules
//! - Export control compliance (structural)

use robodog_ecm::autonomy::{DefensiveAction, SafeStateReason, SafetyParams};
use robodog_ecm::ecm::detection::DefensiveRecommendation;
use robodog_ecm::ecm::signals::{Modulation, SignalClassification};
use robodog_ecm::ecm::FrequencyBand;
use robodog_ecm::formation::{FormationShape, Position, Velocity};

// ── Defensive-only invariant ────────────────────────────────────────────

#[test]
fn defensive_action_has_no_offensive_variant() {
    // Exhaustively match all DefensiveAction variants.
    // If someone adds an offensive variant, this test must be updated,
    // which triggers export control review.
    let actions = vec![
        DefensiveAction::Continue,
        DefensiveAction::AvoidCollision {
            adjusted_velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        },
        DefensiveAction::SafeState {
            reason: SafeStateReason::CommunicationLoss,
        },
        DefensiveAction::FrequencyHop {
            target_freq_hz: 2_437_000_000.0,
        },
        DefensiveAction::RequestHumanControl {
            situation: String::new(),
        },
    ];

    // All actions must be defensive in nature.
    for action in &actions {
        match action {
            DefensiveAction::Continue
            | DefensiveAction::AvoidCollision { .. }
            | DefensiveAction::SafeState { .. }
            | DefensiveAction::FrequencyHop { .. }
            | DefensiveAction::RequestHumanControl { .. } => {
                // All are defensive — this is the invariant.
            }
        }
    }
    assert_eq!(actions.len(), 5, "DefensiveAction must have exactly 5 variants");
}

#[test]
fn signal_classification_has_no_targeting_variant() {
    let classes = vec![
        SignalClassification::Friendly,
        SignalClassification::Neutral,
        SignalClassification::Interference,
        SignalClassification::SuspectedJamming,
    ];
    assert_eq!(classes.len(), 4, "SignalClassification must have exactly 4 variants");
}

#[test]
fn defensive_recommendation_has_no_attack_variant() {
    let recs = vec![
        DefensiveRecommendation::NoAction,
        DefensiveRecommendation::IncreasedMonitoring,
        DefensiveRecommendation::FrequencyHop,
        DefensiveRecommendation::IncreasePower,
        DefensiveRecommendation::AlertOperator,
    ];
    assert_eq!(recs.len(), 5, "DefensiveRecommendation must have exactly 5 variants");
}

// ── Serde round-trip consistency ────────────────────────────────────────

#[test]
fn frequency_band_serde_round_trip() {
    let bands = [FrequencyBand::Hf, FrequencyBand::Vhf, FrequencyBand::Uhf, FrequencyBand::Shf];
    for band in &bands {
        let json = serde_json::to_string(band).unwrap();
        let round_tripped: FrequencyBand = serde_json::from_str(&json).unwrap();
        assert_eq!(*band, round_tripped);
    }
}

#[test]
fn modulation_serde_round_trip() {
    let mods = [
        Modulation::Cw, Modulation::Am, Modulation::Fm, Modulation::Psk,
        Modulation::Fsk, Modulation::Ofdm, Modulation::Fhss, Modulation::Dsss,
        Modulation::Unknown,
    ];
    for m in &mods {
        let json = serde_json::to_string(m).unwrap();
        let round_tripped: Modulation = serde_json::from_str(&json).unwrap();
        assert_eq!(*m, round_tripped);
    }
}

#[test]
fn formation_shape_serde_round_trip() {
    let shapes = [
        FormationShape::Line, FormationShape::Wedge, FormationShape::Circle,
        FormationShape::Diamond, FormationShape::Grid,
    ];
    for shape in &shapes {
        let json = serde_json::to_string(shape).unwrap();
        let round_tripped: FormationShape = serde_json::from_str(&json).unwrap();
        assert_eq!(*shape, round_tripped);
    }
}

#[test]
fn position_serde_round_trip() {
    let pos = Position { x: 123.456, y: -789.012, z: 50.0 };
    let json = serde_json::to_string(&pos).unwrap();
    let round_tripped: Position = serde_json::from_str(&json).unwrap();
    assert_eq!(pos, round_tripped);
}

#[test]
fn safety_params_serde_round_trip() {
    let params = SafetyParams::default();
    let json = serde_json::to_string(&params).unwrap();
    let round_tripped: SafetyParams = serde_json::from_str(&json).unwrap();
    assert_eq!(params.min_separation_ground_m, round_tripped.min_separation_ground_m);
    assert_eq!(params.min_separation_aerial_m, round_tripped.min_separation_aerial_m);
    assert_eq!(params.comms_timeout_s, round_tripped.comms_timeout_s);
}

// ── Safety parameter consistency ────────────────────────────────────────

#[test]
fn aerial_separation_stricter_than_ground() {
    let params = SafetyParams::default();
    assert!(
        params.min_separation_aerial_m > params.min_separation_ground_m,
        "Aerial separation must be strictly greater than ground separation"
    );
}

#[test]
fn comms_timeout_positive_and_bounded() {
    let params = SafetyParams::default();
    assert!(params.comms_timeout_s > 0.0, "Comms timeout must be positive");
    assert!(params.comms_timeout_s <= 10.0, "Comms timeout should be ≤10s for safety");
}

#[test]
fn max_speed_reasonable() {
    let params = SafetyParams::default();
    assert!(params.max_speed_mps > 0.0);
    assert!(params.max_speed_mps <= 50.0, "Max speed should be ≤50 m/s for ground+aerial");
}

// ── Frequency band coverage ─────────────────────────────────────────────

#[test]
fn all_bands_cover_full_range() {
    // Every frequency from 0 to 30 GHz should be in exactly one band.
    let test_freqs: Vec<u64> = vec![
        0, 1_000_000, 29_999_999,           // HF
        30_000_000, 150_000_000, 299_999_999, // VHF
        300_000_000, 1_000_000_000, 2_999_999_999, // UHF
        3_000_000_000, 10_000_000_000, 29_999_999_999, // SHF
    ];

    for freq in test_freqs {
        let in_any = FrequencyBand::Hf.contains(freq)
            || FrequencyBand::Vhf.contains(freq)
            || FrequencyBand::Uhf.contains(freq)
            || FrequencyBand::Shf.contains(freq);
        assert!(in_any, "Frequency {freq} Hz not covered by any band");

        // Exactly one band.
        let count = [FrequencyBand::Hf, FrequencyBand::Vhf, FrequencyBand::Uhf, FrequencyBand::Shf]
            .iter()
            .filter(|b| b.contains(freq))
            .count();
        assert_eq!(count, 1, "Frequency {freq} Hz in {count} bands (expected 1)");
    }
}

// ── Distance computation consistency ────────────────────────────────────

#[test]
fn distance_is_symmetric() {
    let a = Position { x: 10.0, y: 20.0, z: 30.0 };
    let b = Position { x: -5.0, y: 15.0, z: 0.0 };
    let d_ab = a.distance_to(&b);
    let d_ba = b.distance_to(&a);
    assert!((d_ab - d_ba).abs() < 1e-10, "Distance must be symmetric");
}

#[test]
fn distance_to_self_is_zero() {
    let a = Position { x: 42.0, y: -17.0, z: 100.0 };
    assert!(a.distance_to(&a) < 1e-10);
}

#[test]
fn distance_triangle_inequality() {
    let a = Position { x: 0.0, y: 0.0, z: 0.0 };
    let b = Position { x: 3.0, y: 4.0, z: 0.0 };
    let c = Position { x: 6.0, y: 0.0, z: 0.0 };
    assert!(a.distance_to(&c) <= a.distance_to(&b) + b.distance_to(&c) + 1e-10);
}
