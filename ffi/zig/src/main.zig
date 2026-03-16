// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// main.zig — Zig FFI bridge for Robodog ECM.
//
// DEFENSIVE USE ONLY — C-ABI bridge between Idris2 types and Rust core.
// All exported functions have the `robodog_` prefix for namespace safety.

const std = @import("std");
const math = std.math;

// ============================================================================
// Type definitions matching the Idris2 ABI (src/abi/Types.idr)
// ============================================================================

/// Frequency band enumeration (matches Idris2 FreqBand).
pub const FreqBand = enum(i32) {
    hf = 0,
    vhf = 1,
    uhf = 2,
    shf = 3,
};

/// Signal modulation scheme (matches Idris2 Modulation).
pub const Modulation = enum(i32) {
    cw = 0,
    am = 1,
    fm = 2,
    psk = 3,
    fsk = 4,
    ofdm = 5,
    fhss = 6,
    dsss = 7,
};

/// Signal classification (matches Idris2 SignalClass).
pub const SignalClass = enum(i32) {
    friendly = 0,
    neutral = 1,
    interference = 2,
    suspected_jammer = 3,
};

/// Formation shape (matches Idris2 FormationShape).
pub const FormationShape = enum(i32) {
    line = 0,
    wedge = 1,
    circle = 2,
    diamond = 3,
    grid = 4,
};

/// 3D position in millimetres (matches Idris2 Position3D).
pub const Position3D = extern struct {
    x: i64,
    y: i64,
    z: i64,
};

// ============================================================================
// Detection thresholds (matching Rust DetectionThresholds)
// ============================================================================

const INTERFERENCE_SNR_DB: i32 = 2000; // 20.0 dB * 100
const WIDEBAND_THRESHOLD_HZ: u64 = 10_000_000;
const MAX_LEGITIMATE_BW_HZ: u64 = 40_000_000;

// ============================================================================
// Exported C-ABI functions
// ============================================================================

/// Classify a signal based on its characteristics.
///
/// Parameters:
///   freq_hz:      Centre frequency in Hz.
///   bandwidth_hz: Bandwidth in Hz.
///   snr_db:       SNR in dB * 100 (integer fixed-point).
///   modulation:   Modulation enum value.
///
/// Returns: SignalClass enum value.
export fn robodog_classify_signal(
    freq_hz: u64,
    bandwidth_hz: u64,
    snr_db: i32,
    modulation: i32,
) callconv(.c) i32 {
    _ = freq_hz; // Frequency not used in v0.1 classification rules.

    const mod_enum: Modulation = @enumFromInt(modulation);

    // Rule 1: CW with high SNR and wide bandwidth = barrage jamming.
    if (mod_enum == .cw and snr_db > INTERFERENCE_SNR_DB and bandwidth_hz > WIDEBAND_THRESHOLD_HZ) {
        return @intFromEnum(SignalClass.suspected_jammer);
    }

    // Rule 2: Exceeding maximum legitimate bandwidth.
    if (bandwidth_hz > MAX_LEGITIMATE_BW_HZ) {
        return @intFromEnum(SignalClass.suspected_jammer);
    }

    // Rule 3: High-power unknown modulation.
    if (modulation > 7 and snr_db > INTERFERENCE_SNR_DB) {
        return @intFromEnum(SignalClass.interference);
    }

    return @intFromEnum(SignalClass.neutral);
}

/// Check whether a frequency falls within a given band.
///
/// Returns: 1 if in band, 0 if not.
export fn robodog_freq_in_band(freq_hz: u64, band: i32) callconv(.c) i32 {
    const b: FreqBand = @enumFromInt(band);
    const lower: u64 = switch (b) {
        .hf => 0,
        .vhf => 30_000_000,
        .uhf => 300_000_000,
        .shf => 3_000_000_000,
    };
    const upper: u64 = switch (b) {
        .hf => 30_000_000,
        .vhf => 300_000_000,
        .uhf => 3_000_000_000,
        .shf => 30_000_000_000,
    };

    return if (freq_hz >= lower and freq_hz < upper) 1 else 0;
}

/// Compute squared distance between two 3D positions (in mm^2).
///
/// Uses integer arithmetic — no floating point, fully deterministic.
export fn robodog_distance_squared_mm(
    ax: i64,
    ay: i64,
    az: i64,
    bx: i64,
    by: i64,
    bz: i64,
) callconv(.c) i64 {
    const dx = ax - bx;
    const dy = ay - by;
    const dz = az - bz;
    return dx * dx + dy * dy + dz * dz;
}

/// Check ground separation safety (minimum 2.0m = 2000mm).
///
/// Returns: 1 if safe, 0 if violation.
export fn robodog_ground_safe(
    ax: i64,
    ay: i64,
    az: i64,
    bx: i64,
    by: i64,
    bz: i64,
) callconv(.c) i32 {
    const dist_sq = robodog_distance_squared_mm(ax, ay, az, bx, by, bz);
    // 2000mm squared = 4,000,000
    return if (dist_sq >= 4_000_000) 1 else 0;
}

/// Check aerial separation safety (minimum 10.0m = 10000mm).
///
/// Returns: 1 if safe, 0 if violation.
export fn robodog_aerial_safe(
    ax: i64,
    ay: i64,
    az: i64,
    bx: i64,
    by: i64,
    bz: i64,
) callconv(.c) i32 {
    const dist_sq = robodog_distance_squared_mm(ax, ay, az, bx, by, bz);
    // 10000mm squared = 100,000,000
    return if (dist_sq >= 100_000_000) 1 else 0;
}

/// Compute formation positions for a given shape.
///
/// Writes positions to out_buf as an array of Position3D structs.
/// Returns: 0 on success, -1 on invalid parameters.
export fn robodog_compute_formation(
    shape: i32,
    num_agents: i32,
    spacing_mm: u64,
    out_buf: [*]Position3D,
) callconv(.c) i32 {
    if (num_agents <= 0 or num_agents > 256) return -1;

    const n: usize = @intCast(num_agents);
    const sp: f64 = @floatFromInt(spacing_mm);
    const shape_enum: FormationShape = @enumFromInt(shape);

    switch (shape_enum) {
        .line => {
            const start = -(@as(f64, @floatFromInt(n)) - 1.0) / 2.0 * sp;
            for (0..n) |i| {
                const fi: f64 = @floatFromInt(i);
                out_buf[i] = .{
                    .x = @intFromFloat(start + fi * sp),
                    .y = 0,
                    .z = 0,
                };
            }
        },
        .circle => {
            if (n <= 1) {
                out_buf[0] = .{ .x = 0, .y = 0, .z = 0 };
                return 0;
            }
            const angle_step = 2.0 * math.pi / @as(f64, @floatFromInt(n));
            const radius = sp / (2.0 * @sin(angle_step / 2.0));
            for (0..n) |i| {
                const angle = @as(f64, @floatFromInt(i)) * angle_step;
                out_buf[i] = .{
                    .x = @intFromFloat(radius * @cos(angle)),
                    .y = @intFromFloat(radius * @sin(angle)),
                    .z = 0,
                };
            }
        },
        else => {
            // Wedge, Diamond, Grid — v0.2.
            // For now, fall back to line formation.
            const start = -(@as(f64, @floatFromInt(n)) - 1.0) / 2.0 * sp;
            for (0..n) |i| {
                const fi: f64 = @floatFromInt(i);
                out_buf[i] = .{
                    .x = @intFromFloat(start + fi * sp),
                    .y = 0,
                    .z = 0,
                };
            }
        },
    }

    return 0;
}

// ============================================================================
// Tests
// ============================================================================

test "classify CW barrage jamming" {
    const result = robodog_classify_signal(2_400_000_000, 50_000_000, 3000, 0);
    try std.testing.expectEqual(@as(i32, 3), result); // suspected_jammer
}

test "classify normal OFDM as neutral" {
    const result = robodog_classify_signal(2_400_000_000, 20_000_000, 2500, 5);
    try std.testing.expectEqual(@as(i32, 1), result); // neutral
}

test "VHF band check" {
    try std.testing.expectEqual(@as(i32, 1), robodog_freq_in_band(150_000_000, 1));
    try std.testing.expectEqual(@as(i32, 0), robodog_freq_in_band(500_000_000, 1));
}

test "ground separation safe" {
    try std.testing.expectEqual(@as(i32, 1), robodog_ground_safe(0, 0, 0, 3000, 0, 0));
    try std.testing.expectEqual(@as(i32, 0), robodog_ground_safe(0, 0, 0, 1000, 0, 0));
}

test "aerial separation safe" {
    try std.testing.expectEqual(@as(i32, 1), robodog_aerial_safe(0, 0, 0, 15000, 0, 0));
    try std.testing.expectEqual(@as(i32, 0), robodog_aerial_safe(0, 0, 0, 5000, 0, 0));
}

test "line formation positions" {
    var buf: [4]Position3D = undefined;
    const result = robodog_compute_formation(0, 4, 10000, &buf);
    try std.testing.expectEqual(@as(i32, 0), result);
    // 4 agents at 10m spacing: positions should be -15m, -5m, 5m, 15m.
    try std.testing.expectEqual(@as(i64, -15000), buf[0].x);
    try std.testing.expectEqual(@as(i64, -5000), buf[1].x);
    try std.testing.expectEqual(@as(i64, 5000), buf[2].x);
    try std.testing.expectEqual(@as(i64, 15000), buf[3].x);
}
