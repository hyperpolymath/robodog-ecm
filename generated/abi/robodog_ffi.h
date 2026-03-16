/* SPDX-License-Identifier: PMPL-1.0-or-later */
/* Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> */
/*
 * robodog_ffi.h — C header for the Robodog ECM FFI bridge.
 *
 * DEFENSIVE USE ONLY.
 *
 * Generated from: src/abi/ (Idris2) → ffi/zig/ (implementation).
 * Do not edit manually; regenerate with: just gen-headers
 */

#ifndef ROBODOG_FFI_H
#define ROBODOG_FFI_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ── Type definitions ─────────────────────────────────────────────────── */

typedef struct {
    int64_t x;  /* East displacement in mm */
    int64_t y;  /* North displacement in mm */
    int64_t z;  /* Altitude in mm AGL */
} robodog_position_3d;

/* Frequency band: 0=HF, 1=VHF, 2=UHF, 3=SHF */
typedef int32_t robodog_freq_band;

/* Modulation: 0=CW, 1=AM, 2=FM, 3=PSK, 4=FSK, 5=OFDM, 6=FHSS, 7=DSSS */
typedef int32_t robodog_modulation;

/* Signal class: 0=Friendly, 1=Neutral, 2=Interference, 3=SuspectedJammer */
typedef int32_t robodog_signal_class;

/* Formation shape: 0=Line, 1=Wedge, 2=Circle, 3=Diamond, 4=Grid */
typedef int32_t robodog_formation_shape;

/* ── Signal classification ────────────────────────────────────────────── */

/**
 * Classify a detected signal.
 *
 * @param freq_hz       Centre frequency in Hz.
 * @param bandwidth_hz  Bandwidth in Hz.
 * @param snr_db        SNR in dB * 100 (fixed-point integer).
 * @param modulation    Modulation enum value.
 * @return              Signal classification enum value.
 */
robodog_signal_class robodog_classify_signal(
    uint64_t freq_hz,
    uint64_t bandwidth_hz,
    int32_t snr_db,
    int32_t modulation
);

/**
 * Check whether a frequency falls within a given band.
 *
 * @param freq_hz  Frequency in Hz.
 * @param band     Frequency band enum value.
 * @return         1 if in band, 0 if not.
 */
int32_t robodog_freq_in_band(uint64_t freq_hz, int32_t band);

/* ── Distance and separation ──────────────────────────────────────────── */

/**
 * Squared distance between two 3D positions (mm^2).
 * Integer arithmetic — fully deterministic, no floating-point.
 */
int64_t robodog_distance_squared_mm(
    int64_t ax, int64_t ay, int64_t az,
    int64_t bx, int64_t by, int64_t bz
);

/**
 * Ground separation safety check (2.0m minimum).
 * @return 1 if safe, 0 if violation.
 */
int32_t robodog_ground_safe(
    int64_t ax, int64_t ay, int64_t az,
    int64_t bx, int64_t by, int64_t bz
);

/**
 * Aerial separation safety check (10.0m minimum).
 * @return 1 if safe, 0 if violation.
 */
int32_t robodog_aerial_safe(
    int64_t ax, int64_t ay, int64_t az,
    int64_t bx, int64_t by, int64_t bz
);

/* ── Formation control ────────────────────────────────────────────────── */

/**
 * Compute formation positions for a given shape.
 *
 * @param shape       Formation shape enum value.
 * @param num_agents  Number of agents (1..256).
 * @param spacing_mm  Spacing between agents in mm.
 * @param out_buf     Output: array of num_agents Position3D structs.
 * @return            0 on success, -1 on invalid parameters.
 */
int32_t robodog_compute_formation(
    int32_t shape,
    int32_t num_agents,
    uint64_t spacing_mm,
    robodog_position_3d *out_buf
);

#ifdef __cplusplus
}
#endif

#endif /* ROBODOG_FFI_H */
