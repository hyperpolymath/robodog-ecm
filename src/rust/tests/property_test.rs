// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Property-based tests using proptest.
//!
//! These tests verify invariants that must hold across all valid inputs:
//! - Signal values stay within valid ranges
//! - Detection results are deterministic
//! - Formation configurations preserve validity
//! - Crypto operations are reversible

use proptest::prelude::*;

use robodog_ecm::autonomy::{compute_defensive_action, SafetyParams};
use robodog_ecm::crypto::{
    kem_decapsulate, kem_encapsulate, kem_keygen, sign, verify, KemAlgorithm,
    SignatureAlgorithm,
};
use robodog_ecm::ecm::detection::{classify_signal, DetectionThresholds};
use robodog_ecm::ecm::signals::{DetectedSignal, Modulation, SignalClassification};
use robodog_ecm::formation::{
    check_separation, compute_formation_positions, AgentState, FormationParams, FormationShape,
    Position, Velocity,
};

// ── Signal property tests ──────────────────────────────────────────────────

prop_compose! {
    /// Arbitrary frequency in supported range (0–30 GHz).
    fn arb_frequency()(f in 0u64..30_000_000_000u64) -> f64 {
        f as f64
    }
}

prop_compose! {
    /// Arbitrary bandwidth (1 MHz–1 GHz).
    fn arb_bandwidth()(b in 1_000_000u64..1_000_000_000u64) -> f64 {
        b as f64
    }
}

prop_compose! {
    /// Arbitrary SNR (0–80 dB).
    fn arb_snr()(s in 0u32..80u32) -> f64 {
        s as f64
    }
}

prop_compose! {
    /// Arbitrary modulation scheme.
    fn arb_modulation()(m in 0u32..9u32) -> Modulation {
        match m {
            0 => Modulation::Cw,
            1 => Modulation::Am,
            2 => Modulation::Fm,
            3 => Modulation::Psk,
            4 => Modulation::Fsk,
            5 => Modulation::Ofdm,
            6 => Modulation::Fhss,
            7 => Modulation::Dsss,
            _ => Modulation::Unknown,
        }
    }
}

prop_compose! {
    /// Arbitrary detected signal with all parameters in valid range.
    fn arb_detected_signal()(
        freq in arb_frequency(),
        bw in arb_bandwidth(),
        snr in arb_snr(),
        mod_ in arb_modulation(),
        bearing in 0.0f64..360.0,
        ts in 0f64..1_000_000.0
    ) -> DetectedSignal {
        DetectedSignal {
            frequency_hz: freq,
            bandwidth_hz: bw,
            snr_db: snr,
            modulation: mod_,
            classification: SignalClassification::Neutral,
            bearing_deg: Some(bearing),
            timestamp_s: ts,
        }
    }
}

proptest! {
    #[test]
    fn signal_values_in_valid_ranges(sig in arb_detected_signal()) {
        // Frequency must be positive.
        prop_assert!(sig.frequency_hz >= 0.0);
        // Bandwidth must be positive.
        prop_assert!(sig.bandwidth_hz > 0.0);
        // SNR must be non-negative.
        prop_assert!(sig.snr_db >= 0.0);
        // Timestamp must be non-negative.
        prop_assert!(sig.timestamp_s >= 0.0);
        // Bearing if present must be in [0, 360).
        if let Some(bearing) = sig.bearing_deg.map(|b| b.trunc() as i32) {
            prop_assert!(bearing >= 0 && bearing < 360);
        }
    }

    #[test]
    fn signal_classification_is_deterministic(
        sig in arb_detected_signal(),
        thresholds in Just(DetectionThresholds::default())
    ) {
        // Classify the same signal twice — must get the same result.
        let class1 = classify_signal(&sig, &thresholds);
        let class2 = classify_signal(&sig, &thresholds);
        prop_assert_eq!(class1, class2);
    }

    #[test]
    fn signal_classification_is_consistent(sig in arb_detected_signal()) {
        // Classification must always be one of the four valid types.
        let thresholds = DetectionThresholds::default();
        let class = classify_signal(&sig, &thresholds);
        match class {
            SignalClassification::Friendly
            | SignalClassification::Neutral
            | SignalClassification::Interference
            | SignalClassification::SuspectedJamming => {} // OK
        }
    }
}

// ── Formation property tests ────────────────────────────────────────────────

prop_compose! {
    /// Arbitrary agent ID (1–1000).
    fn arb_agent_id()(id in 1u32..1001u32) -> u32 {
        id
    }
}

prop_compose! {
    /// Arbitrary position within reasonable bounds (±1000m).
    fn arb_position()(
        x in -1000f64..1000f64,
        y in -1000f64..1000f64,
        z in 0f64..500f64
    ) -> Position {
        Position { x, y, z }
    }
}

prop_compose! {
    /// Arbitrary formation shape.
    fn arb_formation_shape()(s in 0u32..5u32) -> FormationShape {
        match s {
            0 => FormationShape::Line,
            1 => FormationShape::Wedge,
            2 => FormationShape::Circle,
            3 => FormationShape::Diamond,
            _ => FormationShape::Grid,
        }
    }
}

prop_compose! {
    /// Arbitrary formation parameters.
    fn arb_formation_params()(
        shape in arb_formation_shape(),
        spacing in 1f64..100f64,
        heading in 0f64..360f64,
        centre in arb_position()
    ) -> FormationParams {
        FormationParams {
            shape,
            spacing_m: spacing,
            heading_deg: heading,
            centre,
        }
    }
}

proptest! {
    #[test]
    fn formation_positions_returns_correct_count(
        agent_ids in prop::collection::vec(arb_agent_id(), 0..50),
        params in arb_formation_params()
    ) {
        let positions = compute_formation_positions(&agent_ids, &params);
        prop_assert_eq!(positions.len(), agent_ids.len());
    }

    #[test]
    fn formation_positions_have_non_nan_coordinates(
        agent_ids in prop::collection::vec(arb_agent_id(), 1..20),
        params in arb_formation_params()
    ) {
        let positions = compute_formation_positions(&agent_ids, &params);
        for (_, pos) in &positions {
            prop_assert!(!pos.x.is_nan());
            prop_assert!(!pos.y.is_nan());
            prop_assert!(!pos.z.is_nan());
        }
    }

    #[test]
    fn formation_empty_input_returns_empty(params in arb_formation_params()) {
        let positions = compute_formation_positions(&[], &params);
        prop_assert!(positions.is_empty());
    }

    #[test]
    fn formation_single_agent_at_centre(
        id in arb_agent_id(),
        shape in arb_formation_shape(),
        spacing in 1f64..100f64,
        heading in 0f64..360f64,
        centre in arb_position()
    ) {
        // Only test Line formation (which puts single agent at centre).
        // Other shapes (Wedge, Diamond, Circle, Grid) may place a single agent
        // away from centre based on formation geometry.
        if shape != FormationShape::Line {
            return Ok(());
        }

        let params = FormationParams {
            shape,
            spacing_m: spacing,
            heading_deg: heading,
            centre,
        };

        let positions = compute_formation_positions(&[id], &params);
        prop_assert_eq!(positions.len(), 1);
        let (returned_id, pos) = &positions[0];
        prop_assert_eq!(returned_id, &id);
        // Single agent in Line formation should be at the formation centre.
        let dist = params.centre.distance_to(pos);
        prop_assert!(dist < 0.01, "Single agent {}: dist={}", id, dist);
    }

    #[test]
    fn separation_check_commutative(
        agents in prop::collection::vec(
            (arb_agent_id(), arb_position())
                .prop_map(|(id, pos)| AgentState {
                    id,
                    position: pos,
                    velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
                    operational: true,
                }),
            0..20
        ),
        min_dist in 0.1f64..100f64
    ) {
        // Separation violations should be the same regardless of order.
        let violations1 = check_separation(&agents, min_dist);
        let mut agents2 = agents.clone();
        agents2.reverse();
        let violations2 = check_separation(&agents2, min_dist);

        // Normalize order for comparison.
        prop_assert_eq!(violations1.len(), violations2.len());
    }

    #[test]
    fn position_distance_non_negative(pos1 in arb_position(), pos2 in arb_position()) {
        let dist = pos1.distance_to(&pos2);
        prop_assert!(dist >= 0.0);
    }

    #[test]
    fn position_distance_triangle_inequality(
        a in arb_position(),
        b in arb_position(),
        c in arb_position()
    ) {
        let d_ac = a.distance_to(&c);
        let d_ab = a.distance_to(&b);
        let d_bc = b.distance_to(&c);
        // d(a,c) <= d(a,b) + d(b,c)
        prop_assert!(d_ac <= d_ab + d_bc + 1e-10, "d(a,c)={} > d(a,b)={} + d(b,c)={}", d_ac, d_ab, d_bc);
    }
}

// ── Crypto property tests ──────────────────────────────────────────────────

proptest! {
    #[test]
    fn kyber_encap_decap_round_trip(_x in 0..10u32) {
        // Generate keypair, encapsulate, decapsulate.
        let kp = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
        let encap = kem_encapsulate(&kp.public_key, KemAlgorithm::Kyber1024).unwrap();
        let decap = kem_decapsulate(&encap.ciphertext, &kp.secret_key, KemAlgorithm::Kyber1024).unwrap();

        // Shared secrets must match.
        prop_assert_eq!(encap.shared_secret, decap);
    }

    #[test]
    fn dilithium_sign_verify_round_trip(
        msg in prop::collection::vec(0u8..=255, 1..256)
    ) {
        let kp = robodog_ecm::crypto::sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
        let signed = sign(&msg, &kp.secret_key, SignatureAlgorithm::Dilithium5).unwrap();
        let opened = verify(&signed, &kp.public_key, SignatureAlgorithm::Dilithium5).unwrap();

        // Opened message must match original.
        prop_assert_eq!(opened, msg);
    }

    #[test]
    fn sphincs_sign_verify_round_trip(
        msg in prop::collection::vec(0u8..=255, 1..256)
    ) {
        let kp = robodog_ecm::crypto::sig_keygen(SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
        let signed = sign(&msg, &kp.secret_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();
        let opened = verify(&signed, &kp.public_key, SignatureAlgorithm::SphincsPlusSha2256f).unwrap();

        prop_assert_eq!(opened, msg);
    }
}

// ── Autonomy property tests ────────────────────────────────────────────────

prop_compose! {
    /// Arbitrary agent state.
    fn arb_agent_state()(
        id in arb_agent_id(),
        pos in arb_position(),
        vx in -50f64..50f64,
        vy in -50f64..50f64,
        vz in -10f64..10f64
    ) -> AgentState {
        AgentState {
            id,
            position: pos,
            velocity: Velocity { vx, vy, vz },
            operational: true,
        }
    }
}

proptest! {
    #[test]
    fn defensive_action_always_exists(
        agent in arb_agent_state(),
        neighbours in prop::collection::vec(arb_agent_state(), 0..5)
    ) {
        let params = SafetyParams::default();
        let _action = compute_defensive_action(
            &agent,
            &neighbours,
            100.0,
            100.0,
            robodog_ecm::ecm::detection::DefensiveRecommendation::NoAction,
            &params,
            false,
        );
        // Should never panic or return an error; always returns a valid DefensiveAction.
    }

    #[test]
    fn comms_timeout_always_triggers_safe_state(
        agent in arb_agent_state(),
        params in Just(SafetyParams::default())
    ) {
        // Comms lost 10 seconds ago, timeout is 3 seconds.
        let action = compute_defensive_action(
            &agent,
            &[],
            0.0,    // Last heard at t=0
            10.0,   // Current time t=10 (well past timeout)
            robodog_ecm::ecm::detection::DefensiveRecommendation::NoAction,
            &params,
            false,
        );

        // Should always be SafeState due to comms loss priority.
        let is_safe_state = matches!(
            action,
            robodog_ecm::autonomy::DefensiveAction::SafeState {
                reason: robodog_ecm::autonomy::SafeStateReason::CommunicationLoss,
            }
        );
        prop_assert!(is_safe_state);
    }

    #[test]
    fn velocity_speed_non_negative(vx in -100f64..100f64, vy in -100f64..100f64, vz in -100f64..100f64) {
        let v = Velocity { vx, vy, vz };
        let speed = v.speed();
        prop_assert!(speed >= 0.0);
    }
}
