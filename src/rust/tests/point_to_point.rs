// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Point-to-point (unit integration) tests.
//!
//! Each test verifies a single boundary between two modules:
//! ECM↔Autonomy, Crypto↔Crypto (round-trips), Formation↔Autonomy.

use robodog_ecm::autonomy::{
    compute_defensive_action, DefensiveAction, SafetyParams,
};
use robodog_ecm::crypto::{
    kem_decapsulate, kem_encapsulate, kem_keygen, sig_keygen, sign, verify,
    KemAlgorithm, SignatureAlgorithm,
};
use robodog_ecm::ecm::detection::{classify_signal, recommend_response, DetectionThresholds};
use robodog_ecm::ecm::signals::{DetectedSignal, Modulation, SignalClassification};
use robodog_ecm::ecm::FrequencyBand;
use robodog_ecm::formation::{
    check_separation, compute_formation_positions, AgentState, FormationParams, FormationShape,
    Position, Velocity,
};

// ── ECM → Autonomy boundary ────────────────────────────────────────────

#[test]
fn ecm_detection_feeds_autonomy_response() {
    // Detect a jammer signal, classify it, get recommendation, feed to autonomy.
    let jammer = DetectedSignal {
        frequency_hz: 2_400_000_000.0,
        bandwidth_hz: 50_000_000.0,
        snr_db: 30.0,
        modulation: Modulation::Cw,
        classification: SignalClassification::Neutral, // Pre-classification.
        bearing_deg: Some(180.0),
        timestamp_s: 100.0,
    };

    let thresholds = DetectionThresholds::default();
    let classification = classify_signal(&jammer, &thresholds);
    assert_eq!(classification, SignalClassification::SuspectedJamming);

    let recommendation = recommend_response(&[jammer], &thresholds);
    let agent = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 10.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let action = compute_defensive_action(
        &agent,
        &[],
        100.0,
        100.0,
        recommendation,
        &SafetyParams::default(),
        true,
    );

    // Jammer detection should escalate to human control.
    assert!(matches!(action, DefensiveAction::RequestHumanControl { .. }));
}

#[test]
fn ecm_clear_spectrum_allows_continue() {
    let friendly = DetectedSignal {
        frequency_hz: 2_412_000_000.0,
        bandwidth_hz: 20_000_000.0,
        snr_db: 15.0,
        modulation: Modulation::Ofdm,
        classification: SignalClassification::Friendly,
        bearing_deg: None,
        timestamp_s: 100.0,
    };

    let thresholds = DetectionThresholds::default();
    let recommendation = recommend_response(&[friendly], &thresholds);

    let agent = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 10.0 },
        velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };

    let action = compute_defensive_action(
        &agent,
        &[],
        100.0,
        100.0,
        recommendation,
        &SafetyParams::default(),
        true,
    );

    assert_eq!(action, DefensiveAction::Continue);
}

// ── Crypto round-trip boundaries ────────────────────────────────────────

#[test]
fn kyber_keygen_encap_decap_boundary() {
    let kp = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
    assert!(!kp.public_key.is_empty());
    assert!(!kp.secret_key.is_empty());

    let encap = kem_encapsulate(&kp.public_key, KemAlgorithm::Kyber1024).unwrap();
    assert!(!encap.ciphertext.is_empty());
    assert!(!encap.shared_secret.is_empty());

    let decap = kem_decapsulate(&encap.ciphertext, &kp.secret_key, KemAlgorithm::Kyber1024).unwrap();
    assert_eq!(encap.shared_secret, decap, "Shared secrets must match across encap/decap boundary");
}

#[test]
fn dilithium_sign_verify_boundary() {
    let kp = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let msg = b"formation waypoint update";
    let signed = sign(msg, &kp.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
    let opened = verify(&signed, &kp.public_key, SignatureAlgorithm::Dilithium5).unwrap();
    assert_eq!(opened, msg, "Verified message must match original across sign/verify boundary");
}

#[test]
fn sphincs_sign_verify_boundary() {
    let kp = sig_keygen(SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    let msg = b"ecm threat advisory";
    let signed = sign(msg, &kp.secret_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    let opened = verify(&signed, &kp.public_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    assert_eq!(opened, msg);
}

#[test]
fn wrong_key_fails_verification() {
    let kp1 = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let kp2 = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let msg = b"tampered message";
    let signed = sign(msg, &kp1.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
    let result = verify(&signed, &kp2.public_key, SignatureAlgorithm::Dilithium5);
    assert!(result.is_err(), "Verification with wrong key must fail");
}

// ── Formation → Autonomy boundary ───────────────────────────────────────

#[test]
fn formation_separation_violation_triggers_avoidance() {
    let agents = vec![
        AgentState {
            id: 1,
            position: Position { x: 0.0, y: 0.0, z: 0.0 },
            velocity: Velocity { vx: 1.0, vy: 0.0, vz: 0.0 },
            operational: true,
        },
        AgentState {
            id: 2,
            position: Position { x: 1.5, y: 0.0, z: 0.0 },
            velocity: Velocity { vx: -1.0, vy: 0.0, vz: 0.0 },
            operational: true,
        },
    ];

    // Separation check detects violation.
    let violations = check_separation(&agents, 2.0);
    assert_eq!(violations.len(), 1);

    // Autonomy module should trigger avoidance.
    let action = compute_defensive_action(
        &agents[0],
        &agents[1..],
        100.0,
        100.0,
        robodog_ecm::ecm::detection::DefensiveRecommendation::NoAction,
        &SafetyParams::default(),
        false,
    );

    assert!(matches!(action, DefensiveAction::AvoidCollision { .. }));
}

#[test]
fn formation_positions_maintain_separation() {
    let ids: Vec<u32> = (1..=6).collect();
    let params = FormationParams {
        shape: FormationShape::Circle,
        spacing_m: 15.0, // Well above 10m aerial minimum.
        heading_deg: 0.0,
        centre: Position { x: 0.0, y: 0.0, z: 20.0 },
    };

    let positions = compute_formation_positions(&ids, &params);
    let agents: Vec<AgentState> = positions
        .iter()
        .map(|(id, pos)| AgentState {
            id: *id,
            position: *pos,
            velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
            operational: true,
        })
        .collect();

    let violations = check_separation(&agents, 10.0);
    assert!(
        violations.is_empty(),
        "Circle formation with 15m spacing must satisfy 10m aerial separation"
    );
}

// ── Frequency band consistency ──────────────────────────────────────────

#[test]
fn frequency_bands_are_contiguous() {
    // No gaps between bands.
    assert!(FrequencyBand::Hf.contains(0));
    assert!(!FrequencyBand::Hf.contains(30_000_000));
    assert!(FrequencyBand::Vhf.contains(30_000_000));
    assert!(!FrequencyBand::Vhf.contains(300_000_000));
    assert!(FrequencyBand::Uhf.contains(300_000_000));
    assert!(!FrequencyBand::Uhf.contains(3_000_000_000));
    assert!(FrequencyBand::Shf.contains(3_000_000_000));
}
