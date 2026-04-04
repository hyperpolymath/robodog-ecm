// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Reflexive (self-consistency) and contract tests.
//!
//! These tests verify pre- and post-conditions of public APIs,
//! and self-consistency properties (e.g., an operation applied
//! twice should give predictable results).

use robodog_ecm::autonomy::{compute_defensive_action, SafetyParams};
use robodog_ecm::crypto::{kem_keygen, KemAlgorithm};
use robodog_ecm::ecm::detection::{classify_signal, DetectionThresholds};
use robodog_ecm::ecm::signals::{DetectedSignal, Modulation, SignalClassification, SpectrumSnapshot};
use robodog_ecm::formation::{check_separation, compute_formation_positions, AgentState, FormationParams, FormationShape, Position, Velocity};

// ── Reflexive: Formation shape idempotence ─────────────────────────────────

#[test]
fn formation_positions_idempotent() {
    // Computing formation positions twice with identical inputs must yield identical results.
    let agent_ids = vec![1, 2, 3, 4, 5];
    let params = FormationParams {
        shape: FormationShape::Circle,
        spacing_m: 20.0,
        heading_deg: 45.0,
        centre: Position { x: 100.0, y: 200.0, z: 50.0 },
    };

    let positions1 = compute_formation_positions(&agent_ids, &params);
    let positions2 = compute_formation_positions(&agent_ids, &params);

    assert_eq!(positions1.len(), positions2.len());
    for i in 0..positions1.len() {
        let (id1, pos1) = &positions1[i];
        let (id2, pos2) = &positions2[i];
        assert_eq!(id1, id2);
        assert!((pos1.x - pos2.x).abs() < 1e-10);
        assert!((pos1.y - pos2.y).abs() < 1e-10);
        assert!((pos1.z - pos2.z).abs() < 1e-10);
    }
}

// ── Reflexive: Signal classification idempotence ────────────────────────────

#[test]
fn signal_classification_idempotent() {
    let signal = DetectedSignal {
        frequency_hz: 2_400_000_000.0,
        bandwidth_hz: 50_000_000.0,
        snr_db: 30.0,
        modulation: Modulation::Cw,
        classification: SignalClassification::Neutral,
        bearing_deg: Some(180.0),
        timestamp_s: 1000.0,
    };

    let thresholds = DetectionThresholds::default();

    let class1 = classify_signal(&signal, &thresholds);
    let class2 = classify_signal(&signal, &thresholds);
    let class3 = classify_signal(&signal, &thresholds);

    assert_eq!(class1, class2);
    assert_eq!(class2, class3);
}

// ── Reflexive: Separation check symmetry ────────────────────────────────────

#[test]
fn separation_check_symmetric() {
    let agent1 = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 0.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let agent2 = AgentState {
        id: 2,
        position: Position { x: 5.0, y: 0.0, z: 0.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let violations_fwd = check_separation(&[agent1.clone(), agent2.clone()], 10.0);
    let violations_rev = check_separation(&[agent2, agent1], 10.0);

    // Both should detect the same violation (order may differ).
    assert_eq!(violations_fwd.len(), violations_rev.len());
    for (v1, v2) in violations_fwd.iter().zip(violations_rev.iter()) {
        // Violation pair may be reversed but should identify the same agents.
        let pair_same = (v1.0 == v2.0 && v1.1 == v2.1) || (v1.0 == v2.1 && v1.1 == v2.0);
        assert!(pair_same);
    }
}

// ── Contract: Formation position output bounds ─────────────────────────────

#[test]
fn formation_positions_respect_parameters() {
    // Post-condition: all positions should be real numbers (not NaN/Inf).
    let agent_ids = (1..=10).collect::<Vec<_>>();
    let params = FormationParams {
        shape: FormationShape::Grid,
        spacing_m: 15.0,
        heading_deg: 30.0,
        centre: Position { x: 500.0, y: 600.0, z: 100.0 },
    };

    let positions = compute_formation_positions(&agent_ids, &params);

    for (_, pos) in &positions {
        assert!(!pos.x.is_nan(), "Position.x is NaN");
        assert!(!pos.y.is_nan(), "Position.y is NaN");
        assert!(!pos.z.is_nan(), "Position.z is NaN");
        assert!(!pos.x.is_infinite(), "Position.x is infinite");
        assert!(!pos.y.is_infinite(), "Position.y is infinite");
        assert!(!pos.z.is_infinite(), "Position.z is infinite");
    }
}

// ── Contract: Separation check always returns valid violation pairs ─────────

#[test]
fn separation_violations_always_distinct() {
    // Post-condition: no violation should pair an agent with itself.
    let agents = vec![
        AgentState {
            id: 1,
            position: Position { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
            operational: true,
        },
        AgentState {
            id: 2,
            position: Position { x: 1.0, y: 0.0, z: 0.0 },
            velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
            operational: true,
        },
    ];

    let violations = check_separation(&agents, 2.0);
    for (a, b) in &violations {
        assert_ne!(a, b, "Violation should not pair an agent with itself");
    }
}

// ── Contract: Defensive action precondition check ───────────────────────────

#[test]
fn defensive_action_always_returns_valid_variant() {
    // Pre-condition: agent must be provided.
    // Post-condition: always returns one of the five valid variants.
    let agent = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 0.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let params = SafetyParams::default();

    let action = compute_defensive_action(
        &agent,
        &[],
        100.0,
        100.0,
        robodog_ecm::ecm::detection::DefensiveRecommendation::NoAction,
        &params,
        false,
    );

    // Must match one of the five valid variants.
    match action {
        robodog_ecm::autonomy::DefensiveAction::Continue => {}
        robodog_ecm::autonomy::DefensiveAction::AvoidCollision { .. } => {}
        robodog_ecm::autonomy::DefensiveAction::SafeState { .. } => {}
        robodog_ecm::autonomy::DefensiveAction::FrequencyHop { .. } => {}
        robodog_ecm::autonomy::DefensiveAction::RequestHumanControl { .. } => {}
    }
}

// ── Contract: Spectrum snapshot consistency ────────────────────────────────

#[test]
fn spectrum_snapshot_has_consistent_signal_count() {
    // Post-condition: signal count methods must agree.
    let snapshot = SpectrumSnapshot {
        centre_freq_hz: 2_400_000_000.0,
        bandwidth_hz: 20_000_000.0,
        noise_floor_dbm: -90.0,
        signals: vec![
            DetectedSignal {
                frequency_hz: 2_412_000_000.0,
                bandwidth_hz: 22_000_000.0,
                snr_db: 40.0,
                modulation: Modulation::Ofdm,
                classification: SignalClassification::Friendly,
                bearing_deg: None,
                timestamp_s: 100.0,
            },
            DetectedSignal {
                frequency_hz: 2_410_000_000.0,
                bandwidth_hz: 50_000_000.0,
                snr_db: 60.0,
                modulation: Modulation::Cw,
                classification: SignalClassification::SuspectedJamming,
                bearing_deg: Some(135.0),
                timestamp_s: 100.0,
            },
        ],
        timestamp_s: 100.0,
    };

    let friendly_count = snapshot.count_by_class(SignalClassification::Friendly);
    let jamming_count = snapshot.count_by_class(SignalClassification::SuspectedJamming);
    let total = friendly_count + jamming_count;

    // Total signals counted should equal actual signal count (assuming no other classes in snapshot).
    assert_eq!(total, 2);

    // has_jamming() should return true if jamming_count > 0.
    assert_eq!(snapshot.has_jamming(), jamming_count > 0);
}

// ── Contract: Crypto key sizes ─────────────────────────────────────────────

#[test]
fn kyber_keypair_has_reasonable_sizes() {
    // Post-condition: Kyber1024 keys should be non-empty and have expected sizes.
    let kp = kem_keygen(KemAlgorithm::Kyber1024).expect("Kyber keygen failed");

    // Public key should be ~1568 bytes, secret key ~3168 bytes (Kyber1024).
    assert!(!kp.public_key.is_empty(), "Public key is empty");
    assert!(!kp.secret_key.is_empty(), "Secret key is empty");
    assert!(kp.public_key.len() > 1000, "Public key suspiciously small");
    assert!(kp.secret_key.len() > 1000, "Secret key suspiciously small");
}

// ── Contract: Safety parameters are self-consistent ─────────────────────────

#[test]
fn safety_params_internally_consistent() {
    let params = SafetyParams::default();

    // Post-condition: all thresholds must be positive.
    assert!(params.min_separation_ground_m > 0.0);
    assert!(params.min_separation_aerial_m > 0.0);
    assert!(params.comms_timeout_s > 0.0);
    assert!(params.max_speed_mps > 0.0);
    assert!(params.safe_hover_altitude_m > 0.0);

    // Aerial separation should be strictly stricter than ground.
    assert!(params.min_separation_aerial_m > params.min_separation_ground_m);

    // Hover altitude should be reasonable (>= 1m).
    assert!(params.safe_hover_altitude_m >= 1.0);
}

// ── Contract: Detection thresholds are reasonable ───────────────────────────

#[test]
fn detection_thresholds_internally_consistent() {
    let thresholds = DetectionThresholds::default();

    // Post-condition: all thresholds must be positive.
    assert!(thresholds.interference_snr_db > 0.0);
    assert!(thresholds.wideband_threshold_hz > 0.0);
    assert!(thresholds.max_legitimate_bandwidth_hz > 0.0);
    assert!(thresholds.persistence_threshold_s > 0.0);

    // Max legitimate bandwidth should be stricter than wideband threshold.
    // (anything exceeding max is definitely suspicious)
    assert!(thresholds.max_legitimate_bandwidth_hz >= thresholds.wideband_threshold_hz);
}

// ── Reflexive: Empty formation positions ────────────────────────────────────

#[test]
fn formation_empty_input_output_reflexive() {
    let params = FormationParams {
        shape: FormationShape::Line,
        spacing_m: 10.0,
        heading_deg: 0.0,
        centre: Position { x: 0.0, y: 0.0, z: 0.0 },
    };

    let result1 = compute_formation_positions(&[], &params);
    let result2 = compute_formation_positions(&[], &params);

    assert_eq!(result1, result2);
    assert!(result1.is_empty());
}

// ── Contract: Operational agents only in avoidance calculation ──────────────

#[test]
fn non_operational_agents_ignored_in_separation() {
    let operational = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 0.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let non_operational = AgentState {
        id: 2,
        position: Position { x: 0.5, y: 0.0, z: 0.0 }, // Very close, would violate if counted
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: false,
    };

    // Non-operational agents should not trigger violations.
    let violations = check_separation(&[operational, non_operational], 2.0);
    assert!(violations.is_empty(), "Non-operational agent should be ignored");
}
