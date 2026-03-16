// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Interference detection and signal classification.
//!
//! DEFENSIVE USE ONLY — detects jamming and interference to protect
//! friendly communications. No offensive jamming capability.

use super::signals::{DetectedSignal, Modulation, SignalClassification};

/// Thresholds for interference classification.
///
/// These parameters control when a detected signal is classified
/// as interference or suspected jamming versus normal traffic.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DetectionThresholds {
    /// SNR threshold above noise floor (dB) to flag as interference.
    /// Signals above this with unusual characteristics are suspicious.
    pub interference_snr_db: f64,

    /// Bandwidth threshold (Hz). Wideband signals exceeding this
    /// are flagged as potential jamming (legitimate signals are narrower).
    pub wideband_threshold_hz: f64,

    /// Maximum expected signal bandwidth (Hz) for the monitored band.
    /// Anything wider than this is structurally anomalous.
    pub max_legitimate_bandwidth_hz: f64,

    /// Minimum dwell time (seconds) for a signal to be considered
    /// persistent rather than transient.
    pub persistence_threshold_s: f64,
}

impl Default for DetectionThresholds {
    fn default() -> Self {
        Self {
            interference_snr_db: 20.0,
            wideband_threshold_hz: 10_000_000.0,
            max_legitimate_bandwidth_hz: 40_000_000.0,
            persistence_threshold_s: 1.0,
        }
    }
}

/// Classify a detected signal based on its characteristics.
///
/// This is a rule-based classifier for v0.1. Future versions will
/// incorporate ML-based classification trained on synthetic data.
///
/// # Classification rules
///
/// 1. **CW with high SNR and wide bandwidth** → suspected jamming
/// 2. **Bandwidth exceeding maximum legitimate** → suspected jamming
/// 3. **High SNR with unknown modulation** → interference
/// 4. **Everything else** → neutral (pending further analysis)
#[must_use]
pub fn classify_signal(
    signal: &DetectedSignal,
    thresholds: &DetectionThresholds,
) -> SignalClassification {
    // Rule 1: CW with abnormally wide bandwidth is classic barrage jamming
    if signal.modulation == Modulation::Cw
        && signal.snr_db > thresholds.interference_snr_db
        && signal.bandwidth_hz > thresholds.wideband_threshold_hz
    {
        return SignalClassification::SuspectedJamming;
    }

    // Rule 2: Anything exceeding maximum legitimate bandwidth
    if signal.bandwidth_hz > thresholds.max_legitimate_bandwidth_hz {
        return SignalClassification::SuspectedJamming;
    }

    // Rule 3: High-power unknown modulation is suspicious
    if signal.modulation == Modulation::Unknown
        && signal.snr_db > thresholds.interference_snr_db
    {
        return SignalClassification::Interference;
    }

    // Default: not enough evidence to classify as hostile
    SignalClassification::Neutral
}

/// Defensive response recommendation based on detected threats.
///
/// These are purely advisory — the actual response is decided by
/// the autonomy module with SPARK-proven safety constraints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DefensiveRecommendation {
    /// No action needed — spectrum is clear.
    NoAction,
    /// Monitor the signal more closely (increase dwell time).
    IncreasedMonitoring,
    /// Switch to a backup communication frequency.
    FrequencyHop,
    /// Increase transmission power to overcome interference.
    IncreasePower,
    /// Alert operator — human decision required.
    AlertOperator,
}

/// Generate a defensive recommendation based on a spectrum snapshot.
///
/// Analyses all detected signals and returns the most appropriate
/// defensive action. Errs on the side of caution — if in doubt,
/// recommend alerting the operator.
#[must_use]
pub fn recommend_response(
    signals: &[DetectedSignal],
    thresholds: &DetectionThresholds,
) -> DefensiveRecommendation {
    let jamming_count = signals
        .iter()
        .filter(|s| classify_signal(s, thresholds) == SignalClassification::SuspectedJamming)
        .count();

    let interference_count = signals
        .iter()
        .filter(|s| classify_signal(s, thresholds) == SignalClassification::Interference)
        .count();

    match (jamming_count, interference_count) {
        (0, 0) => DefensiveRecommendation::NoAction,
        (0, 1) => DefensiveRecommendation::IncreasedMonitoring,
        (0, _) => DefensiveRecommendation::FrequencyHop,
        (1, _) => DefensiveRecommendation::AlertOperator,
        _ => DefensiveRecommendation::AlertOperator,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_signal(modulation: Modulation, snr_db: f64, bandwidth_hz: f64) -> DetectedSignal {
        DetectedSignal {
            frequency_hz: 2_400_000_000.0,
            bandwidth_hz,
            snr_db,
            modulation,
            classification: SignalClassification::Neutral,
            bearing_deg: None,
            timestamp_s: 0.0,
        }
    }

    #[test]
    fn barrage_jamming_detected() {
        let sig = make_signal(Modulation::Cw, 30.0, 50_000_000.0);
        let thresholds = DetectionThresholds::default();
        assert_eq!(classify_signal(&sig, &thresholds), SignalClassification::SuspectedJamming);
    }

    #[test]
    fn normal_ofdm_is_neutral() {
        let sig = make_signal(Modulation::Ofdm, 25.0, 20_000_000.0);
        let thresholds = DetectionThresholds::default();
        assert_eq!(classify_signal(&sig, &thresholds), SignalClassification::Neutral);
    }

    #[test]
    fn unknown_high_power_is_interference() {
        let sig = make_signal(Modulation::Unknown, 35.0, 5_000_000.0);
        let thresholds = DetectionThresholds::default();
        assert_eq!(classify_signal(&sig, &thresholds), SignalClassification::Interference);
    }

    #[test]
    fn response_to_jamming_is_alert() {
        let signals = vec![make_signal(Modulation::Cw, 30.0, 50_000_000.0)];
        let thresholds = DetectionThresholds::default();
        assert_eq!(recommend_response(&signals, &thresholds), DefensiveRecommendation::AlertOperator);
    }

    #[test]
    fn response_to_clear_spectrum_is_no_action() {
        let signals = vec![make_signal(Modulation::Ofdm, 15.0, 20_000_000.0)];
        let thresholds = DetectionThresholds::default();
        assert_eq!(recommend_response(&signals, &thresholds), DefensiveRecommendation::NoAction);
    }
}
