// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Signal type definitions for ECM analysis.
//!
//! DEFENSIVE USE ONLY — signal models for spectrum monitoring and
//! interference classification. No offensive jamming capability.

use serde::{Deserialize, Serialize};

/// Modulation scheme detected or simulated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Modulation {
    /// Continuous wave (unmodulated carrier).
    Cw,
    /// Amplitude modulation.
    Am,
    /// Frequency modulation.
    Fm,
    /// Phase-shift keying (digital).
    Psk,
    /// Frequency-shift keying (digital).
    Fsk,
    /// Orthogonal frequency-division multiplexing.
    Ofdm,
    /// Frequency hopping spread spectrum.
    Fhss,
    /// Direct-sequence spread spectrum.
    Dsss,
    /// Unknown or unclassified modulation.
    Unknown,
}

/// Classification of a detected signal's intent.
///
/// This is a defensive classification — we detect and categorise
/// signals to protect our own communications, not to attack others.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalClassification {
    /// Friendly / known cooperative signal.
    Friendly,
    /// Neutral / unidentified but non-threatening.
    Neutral,
    /// Potential interference source (unintentional).
    Interference,
    /// Suspected hostile jamming (intentional disruption).
    SuspectedJamming,
}

/// A detected signal in the monitored spectrum.
///
/// Represents the parameters extracted from spectral analysis
/// of a single emitter or signal source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedSignal {
    /// Centre frequency in Hz.
    pub frequency_hz: f64,
    /// Estimated bandwidth in Hz.
    pub bandwidth_hz: f64,
    /// Signal-to-noise ratio in dB.
    pub snr_db: f64,
    /// Detected modulation scheme.
    pub modulation: Modulation,
    /// Defensive classification.
    pub classification: SignalClassification,
    /// Bearing estimate in degrees (0–360), if available.
    pub bearing_deg: Option<f64>,
    /// Timestamp of detection (Unix epoch seconds).
    pub timestamp_s: f64,
}

/// Spectrum occupancy snapshot.
///
/// Captures the state of monitored spectrum at a point in time.
/// Used for situational awareness and interference avoidance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectrumSnapshot {
    /// Centre frequency of the monitored band in Hz.
    pub centre_freq_hz: f64,
    /// Total bandwidth being monitored in Hz.
    pub bandwidth_hz: f64,
    /// Noise floor estimate in dBm.
    pub noise_floor_dbm: f64,
    /// All signals detected in this snapshot.
    pub signals: Vec<DetectedSignal>,
    /// Timestamp of the snapshot (Unix epoch seconds).
    pub timestamp_s: f64,
}

impl SpectrumSnapshot {
    /// Count signals matching a given classification.
    #[must_use]
    pub fn count_by_class(&self, class: SignalClassification) -> usize {
        self.signals
            .iter()
            .filter(|s| s.classification == class)
            .count()
    }

    /// Return true if any suspected jamming is present.
    #[must_use]
    pub fn has_jamming(&self) -> bool {
        self.count_by_class(SignalClassification::SuspectedJamming) > 0
    }

    /// Strongest signal in the snapshot by SNR.
    #[must_use]
    pub fn strongest_signal(&self) -> Option<&DetectedSignal> {
        self.signals
            .iter()
            .max_by(|a, b| a.snr_db.partial_cmp(&b.snr_db).unwrap_or(std::cmp::Ordering::Equal))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spectrum_snapshot_jamming_detection() {
        let snapshot = SpectrumSnapshot {
            centre_freq_hz: 2_400_000_000.0,
            bandwidth_hz: 20_000_000.0,
            noise_floor_dbm: -90.0,
            timestamp_s: 1710000000.0,
            signals: vec![
                DetectedSignal {
                    frequency_hz: 2_412_000_000.0,
                    bandwidth_hz: 22_000_000.0,
                    snr_db: 40.0,
                    modulation: Modulation::Ofdm,
                    classification: SignalClassification::Friendly,
                    bearing_deg: None,
                    timestamp_s: 1710000000.0,
                },
                DetectedSignal {
                    frequency_hz: 2_410_000_000.0,
                    bandwidth_hz: 50_000_000.0,
                    snr_db: 60.0,
                    modulation: Modulation::Cw,
                    classification: SignalClassification::SuspectedJamming,
                    bearing_deg: Some(135.0),
                    timestamp_s: 1710000000.0,
                },
            ],
        };

        assert!(snapshot.has_jamming());
        assert_eq!(snapshot.count_by_class(SignalClassification::SuspectedJamming), 1);
        assert_eq!(snapshot.strongest_signal().expect("TODO: handle error").snr_db, 60.0);
    }
}
