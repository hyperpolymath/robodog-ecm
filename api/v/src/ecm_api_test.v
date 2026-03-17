// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

// ecm_api_test.v — Tests for the V-lang ECM API.

module ecm_api

// ── Point-to-point tests ────────────────────────────────────────────────

fn test_classify_cw_jammer() {
	sig := DetectedSignal{
		frequency_hz: 2_400_000_000
		bandwidth_hz: 50_000_000
		snr_db: 30.0
		modulation: .cw
		classification: .neutral
		bearing_deg: none
		timestamp_s: 0.0
	}
	assert classify_signal(sig) == .suspected_jammer
}

fn test_classify_normal_ofdm() {
	sig := DetectedSignal{
		frequency_hz: 2_412_000_000
		bandwidth_hz: 20_000_000
		snr_db: 25.0
		modulation: .ofdm
		classification: .neutral
		bearing_deg: none
		timestamp_s: 0.0
	}
	assert classify_signal(sig) == .neutral
}

fn test_classify_wideband_jammer() {
	sig := DetectedSignal{
		frequency_hz: 1_000_000_000
		bandwidth_hz: 50_000_000
		snr_db: 15.0
		modulation: .fm
		classification: .neutral
		bearing_deg: none
		timestamp_s: 0.0
	}
	assert classify_signal(sig) == .suspected_jammer
}

// ── Frequency band tests ────────────────────────────────────────────────

fn test_freq_in_band_vhf() {
	assert freq_in_band(150_000_000, .vhf) == true
	assert freq_in_band(500_000_000, .vhf) == false
}

fn test_freq_in_band_uhf() {
	assert freq_in_band(1_000_000_000, .uhf) == true
	assert freq_in_band(30_000_000, .uhf) == false
}

fn test_freq_bands_contiguous() {
	// HF ends where VHF starts
	assert freq_in_band(29_999_999, .hf) == true
	assert freq_in_band(30_000_000, .hf) == false
	assert freq_in_band(30_000_000, .vhf) == true
}

// ── Distance and separation tests ───────────────────────────────────────

fn test_distance_squared_basic() {
	a := Position3D{ x: 0, y: 0, z: 0 }
	b := Position3D{ x: 3000, y: 4000, z: 0 }
	// 3^2 + 4^2 = 25 (in metres), so 3000^2 + 4000^2 = 25,000,000 mm^2
	assert distance_squared_mm(a, b) == 25_000_000
}

fn test_distance_symmetric() {
	a := Position3D{ x: 1000, y: 2000, z: 3000 }
	b := Position3D{ x: -500, y: 1500, z: 0 }
	assert distance_squared_mm(a, b) == distance_squared_mm(b, a)
}

fn test_ground_safe() {
	a := Position3D{ x: 0, y: 0, z: 0 }
	far := Position3D{ x: 3000, y: 0, z: 0 }
	close := Position3D{ x: 1000, y: 0, z: 0 }
	assert ground_safe(a, far) == true
	assert ground_safe(a, close) == false
}

fn test_aerial_safe() {
	a := Position3D{ x: 0, y: 0, z: 0 }
	far := Position3D{ x: 15000, y: 0, z: 0 }
	close := Position3D{ x: 5000, y: 0, z: 0 }
	assert aerial_safe(a, far) == true
	assert aerial_safe(a, close) == false
}

// ── Snapshot and recommendation tests ───────────────────────────────────

fn test_has_jamming_false() {
	snap := SpectrumSnapshot{
		centre_freq_hz: 2_400_000_000.0
		bandwidth_hz: 20_000_000.0
		noise_floor_dbm: -90.0
		signals: []
		timestamp_s: 0.0
	}
	assert has_jamming(snap) == false
}

fn test_has_jamming_true() {
	snap := SpectrumSnapshot{
		centre_freq_hz: 2_400_000_000.0
		bandwidth_hz: 20_000_000.0
		noise_floor_dbm: -90.0
		signals: [
			DetectedSignal{
				frequency_hz: 2_400_000_000
				bandwidth_hz: 50_000_000
				snr_db: 40.0
				modulation: .cw
				classification: .suspected_jammer
				bearing_deg: none
				timestamp_s: 0.0
			},
		]
		timestamp_s: 0.0
	}
	assert has_jamming(snap) == true
}

fn test_recommend_no_action() {
	snap := SpectrumSnapshot{
		centre_freq_hz: 150_000_000.0
		bandwidth_hz: 10_000_000.0
		noise_floor_dbm: -100.0
		signals: []
		timestamp_s: 0.0
	}
	assert recommend_response(snap) == .no_action
}

fn test_recommend_alert_on_jammer() {
	snap := SpectrumSnapshot{
		centre_freq_hz: 2_400_000_000.0
		bandwidth_hz: 20_000_000.0
		noise_floor_dbm: -90.0
		signals: [
			DetectedSignal{
				frequency_hz: 2_400_000_000
				bandwidth_hz: 50_000_000
				snr_db: 40.0
				modulation: .cw
				classification: .suspected_jammer
				bearing_deg: none
				timestamp_s: 0.0
			},
		]
		timestamp_s: 0.0
	}
	assert recommend_response(snap) == .alert_operator
}
