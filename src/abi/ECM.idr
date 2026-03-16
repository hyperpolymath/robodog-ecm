-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- ECM.idr — ECM signal types with spectral validity proofs.
--
-- DEFENSIVE USE ONLY — signal analysis types for spectrum monitoring.

module ECM

import Types

%default total

||| Signal power in milliwatts, bounded to simulation limits.
||| Maximum 100 mW for normal operation, 1000 mW for calibration.
public export
data SimPower : Type where
  MkSimPower : (mw : Nat) -> {auto prf : LTE mw 1000} -> SimPower

||| Operational power limit (100 mW).
public export
data OpPower : Type where
  MkOpPower : (mw : Nat) -> {auto prf : LTE mw 100} -> OpPower

||| A detected signal with all required fields.
public export
record DetectedSignal where
  constructor MkDetected
  frequency   : Frequency
  modulation  : Modulation
  signalClass : SignalClass
  power       : SimPower

||| Proof that a detected signal is within operational power limits.
||| This type witness can only be constructed for signals <= 100 mW.
public export
data WithinOpLimits : DetectedSignal -> Type where
  IsOpSafe : {sig : DetectedSignal} ->
             (prf : LTE (case sig.power of MkSimPower mw => mw) 100) ->
             WithinOpLimits sig

||| Spectrum snapshot — a collection of detected signals.
public export
record SpectrumSnapshot where
  constructor MkSnapshot
  signals : List DetectedSignal

||| Check if any suspected jamming is present in a snapshot.
public export
hasJamming : SpectrumSnapshot -> Bool
hasJamming snap = any isJammer snap.signals
  where
    isJammer : DetectedSignal -> Bool
    isJammer sig = case sig.signalClass of
      SuspectedJammer => True
      _               => False
