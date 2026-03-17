// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
//
// integration_test.zig — Integration tests for the Robodog ECM FFI bridge.
//
// These tests verify that the FFI functions produce results consistent
// with the SPARK safety proofs and Idris2 ABI type constraints.

const std = @import("std");
const main = @import("main");

test "classification agrees across all frequency bands" {
    // A CW jammer should be classified the same regardless of band.
    const bands = [_]u64{ 15_000_000, 150_000_000, 1_500_000_000, 15_000_000_000 };
    for (bands) |freq| {
        const result = main.robodog_classify_signal(freq, 50_000_000, 3000, 0);
        try std.testing.expectEqual(@as(i32, 3), result);
    }
}

test "all frequency bands are contiguous" {
    // Verify there are no gaps between bands.
    // HF upper == VHF lower, VHF upper == UHF lower, etc.
    try std.testing.expectEqual(@as(i32, 1), main.robodog_freq_in_band(0, 0)); // HF starts at 0
    try std.testing.expectEqual(@as(i32, 0), main.robodog_freq_in_band(30_000_000, 0)); // HF ends before 30M
    try std.testing.expectEqual(@as(i32, 1), main.robodog_freq_in_band(30_000_000, 1)); // VHF starts at 30M
}

test "separation constraints are consistent between ground and aerial" {
    // Aerial minimum (10m) should be stricter than ground (2m).
    // At 5m separation: ground safe, aerial unsafe.
    try std.testing.expectEqual(@as(i32, 1), main.robodog_ground_safe(0, 0, 0, 5000, 0, 0));
    try std.testing.expectEqual(@as(i32, 0), main.robodog_aerial_safe(0, 0, 0, 5000, 0, 0));
}

test "formation with invalid agent count returns error" {
    var buf: [1]main.Position3D = undefined;
    try std.testing.expectEqual(@as(i32, -1), main.robodog_compute_formation(0, 0, 10000, &buf));
    try std.testing.expectEqual(@as(i32, -1), main.robodog_compute_formation(0, -1, 10000, &buf));
}
