// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! End-to-end tests — full scenario simulations.
//!
//! Each test exercises the complete pipeline: signal generation →
//! spectrum analysis → threat detection → formation response →
//! autonomous action, with cryptographic channel establishment.

use robodog_ecm::autonomy::{
    compute_defensive_action, DefensiveAction, SafeStateReason, SafetyParams,
};
use robodog_ecm::crypto::{
    kem_decapsulate, kem_encapsulate, kem_keygen, sig_keygen, sign, verify,
    KemAlgorithm, SignatureAlgorithm,
};
use robodog_ecm::ecm::detection::{
    classify_signal, recommend_response, DefensiveRecommendation, DetectionThresholds,
};
use robodog_ecm::ecm::signals::{DetectedSignal, Modulation, SignalClassification, SpectrumSnapshot};
use robodog_ecm::ecm::{power_spectrum, SyntheticSignal};
use robodog_ecm::formation::{
    check_separation, compute_formation_positions, AgentState, FormationParams, FormationShape,
    Position, Velocity,
};

/// Full scenario: SAR mission with clear spectrum.
///
/// 1. Establish secure channel (Kyber1024 + Dilithium5).
/// 2. Deploy 4-agent wedge formation for area search.
/// 3. Monitor spectrum — no threats detected.
/// 4. All agents continue their trajectory.
#[test]
fn e2e_sar_mission_clear_spectrum() {
    // Step 1: Secure channel establishment.
    let kem_kp = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
    let encap = kem_encapsulate(&kem_kp.public_key, KemAlgorithm::Kyber1024).unwrap();
    let shared_secret = kem_decapsulate(
        &encap.ciphertext,
        &kem_kp.secret_key,
        KemAlgorithm::Kyber1024,
    )
    .unwrap();
    assert_eq!(encap.shared_secret, shared_secret);

    // Sign formation orders.
    let sig_kp = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let order = b"WEDGE formation, spacing 15m, heading 045";
    let signed_order = sign(order, &sig_kp.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
    let verified_order = verify(&signed_order, &sig_kp.public_key, SignatureAlgorithm::Dilithium5).unwrap();
    assert_eq!(verified_order, order);

    // Step 2: Deploy formation.
    let agent_ids = vec![1, 2, 3, 4];
    let params = FormationParams {
        shape: FormationShape::Wedge,
        spacing_m: 15.0,
        heading_deg: 45.0,
        centre: Position { x: 100.0, y: 200.0, z: 0.0 },
    };
    let positions = compute_formation_positions(&agent_ids, &params);
    assert_eq!(positions.len(), 4);

    let agents: Vec<AgentState> = positions
        .iter()
        .map(|(id, pos)| AgentState {
            id: *id,
            position: *pos,
            velocity: Velocity { vx: 1.0, vy: 1.0, vz: 0.0 },
            operational: true,
        })
        .collect();

    // Verify separation.
    let violations = check_separation(&agents, 2.0);
    assert!(violations.is_empty());

    // Step 3: Spectrum monitoring — clear.
    let snapshot = SpectrumSnapshot {
        centre_freq_hz: 150_000_000.0,
        bandwidth_hz: 10_000_000.0,
        noise_floor_dbm: -100.0,
        signals: vec![],
        timestamp_s: 1000.0,
    };
    assert!(!snapshot.has_jamming());

    // Step 4: All agents continue.
    let safety = SafetyParams::default();
    for agent in &agents {
        let action = compute_defensive_action(
            agent,
            &agents,
            1000.0,
            1000.0,
            DefensiveRecommendation::NoAction,
            &safety,
            false,
        );
        assert_eq!(action, DefensiveAction::Continue);
    }
}

/// Full scenario: formation under ECM attack.
///
/// 1. 4-agent circle formation established.
/// 2. Barrage jammer detected on UHF band.
/// 3. Classification → SuspectedJamming.
/// 4. Recommendation → AlertOperator.
/// 5. Autonomy → RequestHumanControl for all agents.
#[test]
fn e2e_formation_under_ecm_attack() {
    // Step 1: Formation.
    let agent_ids = vec![1, 2, 3, 4];
    let params = FormationParams {
        shape: FormationShape::Circle,
        spacing_m: 20.0,
        heading_deg: 0.0,
        centre: Position { x: 0.0, y: 0.0, z: 15.0 },
    };
    let positions = compute_formation_positions(&agent_ids, &params);
    let agents: Vec<AgentState> = positions
        .iter()
        .map(|(id, pos)| AgentState {
            id: *id,
            position: *pos,
            velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
            operational: true,
        })
        .collect();

    // Step 2: Jammer detected.
    let jammer = DetectedSignal {
        frequency_hz: 1_500_000_000.0,
        bandwidth_hz: 80_000_000.0,
        snr_db: 45.0,
        modulation: Modulation::Cw,
        classification: SignalClassification::Neutral,
        bearing_deg: Some(270.0),
        timestamp_s: 2000.0,
    };

    // Step 3: Classify.
    let thresholds = DetectionThresholds::default();
    let class = classify_signal(&jammer, &thresholds);
    assert_eq!(class, SignalClassification::SuspectedJamming);

    // Step 4: Recommend.
    let recommendation = recommend_response(&[jammer], &thresholds);
    assert_eq!(recommendation, DefensiveRecommendation::AlertOperator);

    // Step 5: All agents escalate to human control.
    let safety = SafetyParams::default();
    for agent in &agents {
        let action = compute_defensive_action(
            agent,
            &agents,
            2000.0,
            2000.0,
            recommendation,
            &safety,
            true,
        );
        assert!(matches!(action, DefensiveAction::RequestHumanControl { .. }));
    }
}

/// Full scenario: communication loss triggers safe-state.
///
/// 1. Single agent in flight.
/// 2. Comms lost (3+ seconds elapsed).
/// 3. Autonomy immediately transitions to safe state.
/// 4. Safe state chosen based on agent type (aerial → hover).
#[test]
fn e2e_comms_loss_safe_state() {
    let agent = AgentState {
        id: 1,
        position: Position { x: 50.0, y: 50.0, z: 30.0 },
        velocity: Velocity { vx: 5.0, vy: 5.0, vz: 0.0 },
        operational: true,
    };

    let safety = SafetyParams::default();

    // Comms last heard at t=100, current time t=104 (4s elapsed > 3s timeout).
    let action = compute_defensive_action(
        &agent,
        &[],
        100.0,
        104.0,
        DefensiveRecommendation::NoAction,
        &safety,
        true,
    );

    assert_eq!(
        action,
        DefensiveAction::SafeState {
            reason: SafeStateReason::CommunicationLoss,
        }
    );
}

/// Full scenario: signal analysis pipeline.
///
/// 1. Generate synthetic signal at known frequency.
/// 2. Compute power spectrum.
/// 3. Verify peak at expected frequency.
#[test]
fn e2e_signal_analysis_pipeline() {
    let signal = SyntheticSignal {
        frequency_hz: 2000.0,
        amplitude: 1.0,
        phase_rad: 0.0,
        sample_rate_hz: 16000.0,
        num_samples: 16384,
    };

    let iq = signal.generate_iq();
    assert_eq!(iq.len(), 16384);

    let spectrum = power_spectrum(&iq, signal.sample_rate_hz);
    assert!(!spectrum.is_empty());

    // Find peak.
    let (peak_freq, peak_power) = spectrum
        .iter()
        .copied()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
        .unwrap();

    // Peak should be at 2000 Hz ± one bin.
    let bin_width = signal.sample_rate_hz / signal.num_samples as f64;
    assert!(
        (peak_freq - 2000.0).abs() < bin_width * 1.5,
        "Peak at {peak_freq} Hz, expected ~2000 Hz"
    );
    assert!(peak_power > 0.0, "Peak power must be positive");
}

/// Full scenario: crypto channel with signed formation orders.
///
/// 1. Generate KEM keypair.
/// 2. Encapsulate shared secret.
/// 3. Sign formation order with Dilithium5.
/// 4. Verify order integrity.
/// 5. Fallback: re-sign with SPHINCS+ and verify.
#[test]
fn e2e_crypto_channel_with_fallback() {
    // Primary: Kyber1024 + Dilithium5.
    let kem_kp = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
    let encap = kem_encapsulate(&kem_kp.public_key, KemAlgorithm::Kyber1024).unwrap();
    let ss = kem_decapsulate(&encap.ciphertext, &kem_kp.secret_key, KemAlgorithm::Kyber1024).unwrap();
    assert_eq!(encap.shared_secret, ss);

    let dil_kp = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let order = b"GRID formation, 6 agents, 20m spacing";
    let signed = sign(order, &dil_kp.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
    let opened = verify(&signed, &dil_kp.public_key, SignatureAlgorithm::Dilithium5).unwrap();
    assert_eq!(opened, order);

    // Fallback: SPHINCS+ (hash-based, quantum-resistant).
    let sph_kp = sig_keygen(SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    let fallback_signed = sign(order, &sph_kp.secret_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    let fallback_opened = verify(&fallback_signed, &sph_kp.public_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
    assert_eq!(fallback_opened, order);
}
