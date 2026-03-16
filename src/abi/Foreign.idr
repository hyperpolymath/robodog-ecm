-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Foreign.idr — FFI foreign function declarations (Zig bridge).
--
-- Declares the C-ABI-compatible functions implemented in ffi/zig/
-- that bridge between the Idris2 type universe and the Rust core.

module Foreign

import Types
import Crypto

%default total

||| FFI result type — success or error code.
public export
data FFIResult : Type where
  FFIOk  : FFIResult
  FFIErr : (code : Nat) -> FFIResult

-- Foreign function declarations for the Zig FFI bridge.
-- These are the C-ABI functions that the Zig layer exports.

||| Generate a Kyber1024 keypair via the Zig FFI.
||| Writes the public key and secret key to the provided buffers.
|||
||| @pk_buf  Output buffer for public key (must be >= Kyber1024_PK_Size).
||| @sk_buf  Output buffer for secret key (must be >= Kyber1024_SK_Size).
||| @return  0 on success, non-zero error code on failure.
export
%foreign "C:robodog_kyber1024_keygen,librobodog_ffi"
robodog_kyber1024_keygen : (pk_buf : AnyPtr) -> (sk_buf : AnyPtr) -> PrimIO Int

||| Encapsulate a shared secret using a Kyber1024 public key.
|||
||| @pk      Public key buffer (Kyber1024_PK_Size bytes).
||| @ct_buf  Output buffer for ciphertext (Kyber1024_CT_Size bytes).
||| @ss_buf  Output buffer for shared secret (Kyber1024_SS_Size bytes).
||| @return  0 on success.
export
%foreign "C:robodog_kyber1024_encapsulate,librobodog_ffi"
robodog_kyber1024_encapsulate : (pk : AnyPtr) -> (ct_buf : AnyPtr) -> (ss_buf : AnyPtr) -> PrimIO Int

||| Classify a signal based on its characteristics.
|||
||| @freq_hz      Centre frequency in Hz.
||| @bandwidth_hz Bandwidth in Hz.
||| @snr_db       Signal-to-noise ratio in dB (x100 integer).
||| @modulation   Modulation type enum value.
||| @return       SignalClass enum value (0=Friendly, 1=Neutral, 2=Interference, 3=SuspectedJammer).
export
%foreign "C:robodog_classify_signal,librobodog_ffi"
robodog_classify_signal : (freq_hz : Bits64) -> (bandwidth_hz : Bits64) -> (snr_db : Int) -> (modulation : Int) -> PrimIO Int

||| Compute formation positions for a set of agents.
|||
||| @shape       Formation shape enum (0=Line, 1=Wedge, 2=Circle, 3=Diamond, 4=Grid).
||| @num_agents  Number of agents.
||| @spacing_mm  Spacing in millimetres.
||| @out_buf     Output buffer for positions (num_agents * 3 * sizeof(int64)).
||| @return      0 on success.
export
%foreign "C:robodog_compute_formation,librobodog_ffi"
robodog_compute_formation : (shape : Int) -> (num_agents : Int) -> (spacing_mm : Bits64) -> (out_buf : AnyPtr) -> PrimIO Int
