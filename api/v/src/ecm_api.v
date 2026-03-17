// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

// ecm_api.v — V-lang API for Robodog ECM consumers.
//
// DEFENSIVE USE ONLY — Wassenaar Cat 5A2 / Cat 11.
// Public API surface for the Robodog ECM platform.
// Wraps the Zig FFI bridge with V-native types and error handling.

module ecm_api

// ── Type definitions (matching Idris2 ABI) ──────────────────────────────

// FreqBand represents supported frequency bands for ECM simulation.
pub enum FreqBand {
	hf  // 0–30 MHz: SAR coordination
	vhf // 30–300 MHz: formation comms
	uhf // 300 MHz–3 GHz: general ECM
	shf // 3–30 GHz: radar countermeasures
}

// Modulation represents detected or simulated signal modulation.
pub enum Modulation {
	cw   // Continuous wave
	am   // Amplitude modulation
	fm   // Frequency modulation
	psk  // Phase-shift keying
	fsk  // Frequency-shift keying
	ofdm // Orthogonal frequency-division multiplexing
	fhss // Frequency hopping spread spectrum
	dsss // Direct-sequence spread spectrum
}

// SignalClass is the defensive classification of a detected signal.
// There is intentionally NO offensive/targeting variant.
pub enum SignalClass {
	friendly
	neutral
	interference
	suspected_jammer
}

// FormationShape defines formation geometry templates.
pub enum FormationShape {
	line
	wedge
	circle
	diamond
	grid
}

// Position3D represents a 3D position in millimetres.
pub struct Position3D {
pub mut:
	x i64 // East displacement (mm)
	y i64 // North displacement (mm)
	z i64 // Altitude (mm AGL)
}

// DetectedSignal represents a signal found during spectrum monitoring.
pub struct DetectedSignal {
pub:
	frequency_hz   u64
	bandwidth_hz   u64
	snr_db         f64
	modulation     Modulation
	classification SignalClass
	bearing_deg    ?f64
	timestamp_s    f64
}

// SpectrumSnapshot captures the state of monitored spectrum.
pub struct SpectrumSnapshot {
pub:
	centre_freq_hz f64
	bandwidth_hz   f64
	noise_floor_dbm f64
	signals        []DetectedSignal
	timestamp_s    f64
}

// DefensiveRecommendation is the advised response to detected threats.
pub enum DefensiveRecommendation {
	no_action
	increased_monitoring
	frequency_hop
	increase_power
	alert_operator
}

// ── API functions ───────────────────────────────────────────────────────

// classify_signal determines the defensive classification of a signal.
// Uses the same rule-based classifier as the Rust core and Zig FFI.
pub fn classify_signal(sig DetectedSignal) SignalClass {
	// Rule 1: CW with high SNR and wide bandwidth = barrage jamming.
	if sig.modulation == .cw && sig.snr_db > 20.0 && sig.bandwidth_hz > 10_000_000 {
		return .suspected_jammer
	}
	// Rule 2: Exceeding maximum legitimate bandwidth.
	if sig.bandwidth_hz > 40_000_000 {
		return .suspected_jammer
	}
	// Rule 3: Unknown high-power modulation — not representable in
	// the enum, so this path is structurally unreachable in V.
	// Kept as documentation of the Rust/Zig equivalent.
	return .neutral
}

// has_jamming checks whether any signals in a snapshot are suspected jammers.
pub fn has_jamming(snap SpectrumSnapshot) bool {
	for sig in snap.signals {
		if sig.classification == .suspected_jammer {
			return true
		}
	}
	return false
}

// recommend_response analyses a snapshot and returns a defensive action.
pub fn recommend_response(snap SpectrumSnapshot) DefensiveRecommendation {
	mut jamming_count := 0
	mut interference_count := 0
	for sig in snap.signals {
		match sig.classification {
			.suspected_jammer { jamming_count++ }
			.interference { interference_count++ }
			else {}
		}
	}
	if jamming_count > 0 {
		return .alert_operator
	}
	if interference_count > 1 {
		return .frequency_hop
	}
	if interference_count == 1 {
		return .increased_monitoring
	}
	return .no_action
}

// freq_in_band checks whether a frequency falls within a given band.
pub fn freq_in_band(freq_hz u64, band FreqBand) bool {
	lower := match band {
		.hf { u64(0) }
		.vhf { u64(30_000_000) }
		.uhf { u64(300_000_000) }
		.shf { u64(3_000_000_000) }
	}
	upper := match band {
		.hf { u64(30_000_000) }
		.vhf { u64(300_000_000) }
		.uhf { u64(3_000_000_000) }
		.shf { u64(30_000_000_000) }
	}
	return freq_hz >= lower && freq_hz < upper
}

// distance_squared_mm computes squared distance between two positions.
// Integer arithmetic for determinism — no floating-point.
pub fn distance_squared_mm(a Position3D, b Position3D) i64 {
	dx := a.x - b.x
	dy := a.y - b.y
	dz := a.z - b.z
	return dx * dx + dy * dy + dz * dz
}

// ground_safe checks ground separation (2.0m minimum).
pub fn ground_safe(a Position3D, b Position3D) bool {
	return distance_squared_mm(a, b) >= 4_000_000 // 2000mm^2
}

// aerial_safe checks aerial separation (10.0m minimum).
pub fn aerial_safe(a Position3D, b Position3D) bool {
	return distance_squared_mm(a, b) >= 100_000_000 // 10000mm^2
}
