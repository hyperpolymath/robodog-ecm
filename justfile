# SPDX-License-Identifier: PMPL-1.0-or-later
# justfile — Build recipes for Robodog ECM
# Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

# Default recipe — list all available commands.
default:
    @just --list

# ── Build ────────────────────────────────────────────────────────────────

# Build the Rust core library.
build-rust:
    cd src/rust && cargo build --release

# Build the Zig FFI shared library.
build-zig:
    cd ffi/zig && zig build

# Build all components.
build: build-rust build-zig

# ── Test ─────────────────────────────────────────────────────────────────

# Run Rust tests.
test-rust:
    cd src/rust && cargo test

# Run Zig FFI tests.
test-zig:
    cd ffi/zig && zig build test

# Run Trustfile operational checks.
test-trust:
    bash contractiles/trust/run-checks.sh

# Run all tests.
test: test-rust test-zig test-trust

# ── Lint / Format ────────────────────────────────────────────────────────

# Lint Rust with Clippy.
lint-rust:
    cd src/rust && cargo clippy -- -D warnings

# Format Rust source.
fmt-rust:
    cd src/rust && cargo fmt

# Format Zig source.
fmt-zig:
    cd ffi/zig && zig fmt src/ test/

# Lint all.
lint: lint-rust

# Format all.
fmt: fmt-rust fmt-zig

# ── Formal Verification ─────────────────────────────────────────────────

# Typecheck Idris2 ABI definitions.
check-abi:
    cd src/abi && idris2 --check Types.idr

# Run SPARK proofs.
prove-spark:
    cd src/spark && gnatprove -P robodog_ecm.gpr --level=2

# Run all formal verification.
verify: check-abi prove-spark

# ── Clean ────────────────────────────────────────────────────────────────

# Clean Rust build artifacts.
clean-rust:
    cd src/rust && cargo clean

# Clean Zig build artifacts.
clean-zig:
    cd ffi/zig && rm -rf .zig-cache zig-out

# Clean SPARK proof artifacts.
clean-spark:
    cd src/spark && rm -rf obj proof

# Clean all build artifacts.
clean: clean-rust clean-zig clean-spark

# ── Multi-arch ───────────────────────────────────────────────────────────

# Build for RISC-V target.
build-riscv:
    cd src/rust && cross build --target riscv64gc-unknown-linux-gnu --release

# ── Audit ────────────────────────────────────────────────────────────────

# Run cargo-audit for dependency vulnerabilities.
audit:
    cd src/rust && cargo audit

# ── Generated Files ──────────────────────────────────────────────────────

# Regenerate C header from Zig FFI exports.
gen-headers:
    @echo "C header is at generated/abi/robodog_ffi.h"
    @echo "Manual generation — update generated/abi/robodog_ffi.h when FFI changes."

# Run panic-attacker pre-commit scan
assail:
    @command -v panic-attack >/dev/null 2>&1 && panic-attack assail . || echo "panic-attack not found — install from https://github.com/hyperpolymath/panic-attacker"
