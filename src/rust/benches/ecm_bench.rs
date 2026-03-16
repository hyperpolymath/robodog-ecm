// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Performance benchmarks for Robodog ECM.
//!
//! Measures throughput of critical paths:
//! - Signal generation and FFT spectrum analysis
//! - PQ crypto operations (Kyber1024, Dilithium5)
//! - Formation position computation
//! - Autonomous decision loop

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use robodog_ecm::autonomy::{compute_defensive_action, SafetyParams};
use robodog_ecm::crypto::{
    kem_decapsulate, kem_encapsulate, kem_keygen, sig_keygen, sign,
    KemAlgorithm, SignatureAlgorithm,
};
use robodog_ecm::ecm::detection::{classify_signal, DefensiveRecommendation, DetectionThresholds};
use robodog_ecm::ecm::signals::{DetectedSignal, Modulation, SignalClassification};
use robodog_ecm::ecm::{power_spectrum, SyntheticSignal};
use robodog_ecm::formation::{
    compute_formation_positions, AgentState, FormationParams, FormationShape, Position, Velocity,
};

fn bench_signal_generation(c: &mut Criterion) {
    let signal = SyntheticSignal {
        frequency_hz: 2000.0,
        amplitude: 1.0,
        phase_rad: 0.0,
        sample_rate_hz: 44100.0,
        num_samples: 4096,
    };
    c.bench_function("signal_generate_iq_4096", |b| {
        b.iter(|| black_box(signal.generate_iq()))
    });
}

fn bench_power_spectrum(c: &mut Criterion) {
    let signal = SyntheticSignal {
        frequency_hz: 2000.0,
        amplitude: 1.0,
        phase_rad: 0.0,
        sample_rate_hz: 44100.0,
        num_samples: 4096,
    };
    let iq = signal.generate_iq();
    c.bench_function("power_spectrum_4096", |b| {
        b.iter(|| black_box(power_spectrum(&iq, 44100.0)))
    });
}

fn bench_signal_classification(c: &mut Criterion) {
    let signal = DetectedSignal {
        frequency_hz: 2_400_000_000.0,
        bandwidth_hz: 50_000_000.0,
        snr_db: 30.0,
        modulation: Modulation::Cw,
        classification: SignalClassification::Neutral,
        bearing_deg: Some(180.0),
        timestamp_s: 0.0,
    };
    let thresholds = DetectionThresholds::default();
    c.bench_function("classify_signal", |b| {
        b.iter(|| black_box(classify_signal(&signal, &thresholds)))
    });
}

fn bench_kyber1024_keygen(c: &mut Criterion) {
    c.bench_function("kyber1024_keygen", |b| {
        b.iter(|| black_box(kem_keygen(KemAlgorithm::Kyber1024).unwrap()))
    });
}

fn bench_kyber1024_encap_decap(c: &mut Criterion) {
    let kp = kem_keygen(KemAlgorithm::Kyber1024).unwrap();
    c.bench_function("kyber1024_encap", |b| {
        b.iter(|| black_box(kem_encapsulate(&kp.public_key, KemAlgorithm::Kyber1024).unwrap()))
    });

    let encap = kem_encapsulate(&kp.public_key, KemAlgorithm::Kyber1024).unwrap();
    c.bench_function("kyber1024_decap", |b| {
        b.iter(|| {
            black_box(
                kem_decapsulate(&encap.ciphertext, &kp.secret_key, KemAlgorithm::Kyber1024)
                    .unwrap(),
            )
        })
    });
}

fn bench_dilithium5_sign(c: &mut Criterion) {
    let kp = sig_keygen(SignatureAlgorithm::Dilithium5).unwrap();
    let msg = b"formation waypoint update with coordinates and heading";
    c.bench_function("dilithium5_sign", |b| {
        b.iter(|| black_box(sign(msg, &kp.secret_key, SignatureAlgorithm::Dilithium5).unwrap()))
    });
}

fn bench_formation_circle_16(c: &mut Criterion) {
    let ids: Vec<u32> = (1..=16).collect();
    let params = FormationParams {
        shape: FormationShape::Circle,
        spacing_m: 15.0,
        heading_deg: 0.0,
        centre: Position { x: 0.0, y: 0.0, z: 0.0 },
    };
    c.bench_function("formation_circle_16_agents", |b| {
        b.iter(|| black_box(compute_formation_positions(&ids, &params)))
    });
}

fn bench_autonomy_decision(c: &mut Criterion) {
    let agent = AgentState {
        id: 1,
        position: Position { x: 0.0, y: 0.0, z: 10.0 },
        velocity: Velocity { vx: 5.0, vy: 0.0, vz: 0.0 },
        operational: true,
    };
    let neighbours: Vec<AgentState> = (2..=8)
        .map(|id| AgentState {
            id,
            position: Position {
                x: (id as f64) * 15.0,
                y: 0.0,
                z: 10.0,
            },
            velocity: Velocity { vx: 5.0, vy: 0.0, vz: 0.0 },
            operational: true,
        })
        .collect();
    let safety = SafetyParams::default();

    c.bench_function("autonomy_decision_8_neighbours", |b| {
        b.iter(|| {
            black_box(compute_defensive_action(
                &agent,
                &neighbours,
                100.0,
                100.0,
                DefensiveRecommendation::NoAction,
                &safety,
                true,
            ))
        })
    });
}

criterion_group!(
    benches,
    bench_signal_generation,
    bench_power_spectrum,
    bench_signal_classification,
    bench_kyber1024_keygen,
    bench_kyber1024_encap_decap,
    bench_dilithium5_sign,
    bench_formation_circle_16,
    bench_autonomy_decision,
);
criterion_main!(benches);
