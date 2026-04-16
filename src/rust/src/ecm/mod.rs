// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! ECM Signal Analysis — spectrum monitoring and interference detection.
//!
//! DEFENSIVE USE ONLY — Wassenaar Category 11 (defensive countermeasures).
//! All operations use synthetic signal models. No real RF captures.

pub mod detection;
pub mod signals;

use std::f64::consts::PI;

use num_complex::Complex64;
use rustfft::{FftPlanner, num_complex::Complex};

/// Supported frequency bands for simulation.
///
/// Each band corresponds to an allowed simulation range
/// defined in the Trustfile's `SPECTRUM_SECURITY` section.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FrequencyBand {
    /// 0–30 MHz: propagation modelling for SAR coordination.
    Hf,
    /// 30–300 MHz: formation communication simulation.
    Vhf,
    /// 300 MHz–3 GHz: general-purpose ECM simulation.
    Uhf,
    /// 3–30 GHz: radar countermeasure simulation.
    Shf,
}

impl FrequencyBand {
    /// Lower bound of the band in Hz.
    #[must_use]
    pub const fn lower_hz(&self) -> u64 {
        match self {
            Self::Hf => 0,
            Self::Vhf => 30_000_000,
            Self::Uhf => 300_000_000,
            Self::Shf => 3_000_000_000,
        }
    }

    /// Upper bound of the band in Hz.
    #[must_use]
    pub const fn upper_hz(&self) -> u64 {
        match self {
            Self::Hf => 30_000_000,
            Self::Vhf => 300_000_000,
            Self::Uhf => 3_000_000_000,
            Self::Shf => 30_000_000_000,
        }
    }

    /// Whether a given frequency falls within this band.
    #[must_use]
    pub const fn contains(&self, freq_hz: u64) -> bool {
        freq_hz >= self.lower_hz() && freq_hz < self.upper_hz()
    }
}

/// A synthetic signal sample for simulation purposes.
///
/// Represents a single-frequency tone with optional noise floor.
/// Real RF captures are never used — see EXPORT-CONTROL.md.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyntheticSignal {
    /// Centre frequency in Hz.
    pub frequency_hz: f64,
    /// Signal amplitude (linear scale, 0.0–1.0).
    pub amplitude: f64,
    /// Phase offset in radians.
    pub phase_rad: f64,
    /// Sample rate in Hz.
    pub sample_rate_hz: f64,
    /// Number of samples.
    pub num_samples: usize,
}

impl SyntheticSignal {
    /// Generate time-domain IQ samples for this signal.
    ///
    /// Returns a vector of complex samples (I + jQ) representing the
    /// signal at the configured frequency, amplitude, and phase.
    #[must_use]
    pub fn generate_iq(&self) -> Vec<Complex64> {
        (0..self.num_samples)
            .map(|n| {
                let t = n as f64 / self.sample_rate_hz;
                let angle = 2.0 * PI * self.frequency_hz * t + self.phase_rad;
                Complex64::new(
                    self.amplitude * angle.cos(),
                    self.amplitude * angle.sin(),
                )
            })
            .collect()
    }
}

/// Compute the power spectral density of a signal via FFT.
///
/// Returns a vector of power values (magnitude squared) for each
/// frequency bin. The frequency resolution is `sample_rate / num_samples`.
#[must_use]
pub fn power_spectrum(samples: &[Complex64], sample_rate_hz: f64) -> Vec<(f64, f64)> {
    let n = samples.len();
    if n == 0 {
        return Vec::new();
    }

    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(n);

    let mut buffer: Vec<Complex<f64>> = samples
        .iter()
        .map(|c| Complex::new(c.re, c.im))
        .collect();

    fft.process(&mut buffer);

    let freq_resolution = sample_rate_hz / n as f64;

    buffer
        .iter()
        .enumerate()
        .take(n / 2)
        .map(|(i, c)| {
            let freq = i as f64 * freq_resolution;
            let power = (c.re * c.re + c.im * c.im) / (n as f64).powi(2);
            (freq, power)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequency_band_containment() {
        assert!(FrequencyBand::Vhf.contains(150_000_000));
        assert!(!FrequencyBand::Vhf.contains(500_000_000));
        assert!(FrequencyBand::Uhf.contains(1_000_000_000));
    }

    #[test]
    fn synthetic_signal_generates_correct_length() {
        let sig = SyntheticSignal {
            frequency_hz: 1000.0,
            amplitude: 0.5,
            phase_rad: 0.0,
            sample_rate_hz: 8000.0,
            num_samples: 1024,
        };
        let iq = sig.generate_iq();
        assert_eq!(iq.len(), 1024);
    }

    #[test]
    fn power_spectrum_peak_at_signal_frequency() {
        let sig = SyntheticSignal {
            frequency_hz: 1000.0,
            amplitude: 1.0,
            phase_rad: 0.0,
            sample_rate_hz: 8000.0,
            num_samples: 8192,
        };
        let iq = sig.generate_iq();
        let spectrum = power_spectrum(&iq, sig.sample_rate_hz);

        // Find the bin with maximum power.
        let (peak_freq, _peak_power) = spectrum
            .iter()
            .copied()
            .max_by(|a, b| a.1.partial_cmp(&b.1).expect("TODO: handle error"))
            .expect("TODO: handle error");

        // Peak should be within one frequency bin of 1000 Hz.
        let bin_width = sig.sample_rate_hz / sig.num_samples as f64;
        assert!((peak_freq - 1000.0).abs() < bin_width * 1.5);
    }
}
