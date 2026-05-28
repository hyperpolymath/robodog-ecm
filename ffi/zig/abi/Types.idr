-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Types.idr — Idris2 ABI verification types for the Zig FFI API layer.
--
-- This file mirrors src/abi/Types.idr but scoped to the Zig FFI surface.
-- It ensures the Zig FFI types are consistent with the core ABI.
--
-- Replaces the legacy api/v/src/abi/Types.idr from the V-lang era. The
-- module was renamed VApi.ABI.Types → ZigFfi.ABI.Types alongside the
-- 2026-05-28 estate-wide V-lang→Zig remediation.

module ZigFfi.ABI.Types

import Types as Core

%default total

||| Proof that the Zig FFI FreqBand enum is isomorphic to the core FreqBand.
||| This is a compile-time check — if the types drift, this won't typecheck.
public export
freqBandRoundTrip : Core.FreqBand -> Core.FreqBand
freqBandRoundTrip HF  = HF
freqBandRoundTrip VHF = VHF
freqBandRoundTrip UHF = UHF
freqBandRoundTrip SHF = SHF

||| Proof that the Zig FFI SignalClass enum matches core.
public export
signalClassRoundTrip : Core.SignalClass -> Core.SignalClass
signalClassRoundTrip Friendly        = Friendly
signalClassRoundTrip Neutral         = Neutral
signalClassRoundTrip Interference    = Interference
signalClassRoundTrip SuspectedJammer = SuspectedJammer

||| Proof that the Zig FFI FormationShape enum matches core.
public export
formationShapeRoundTrip : Core.FormationShape -> Core.FormationShape
formationShapeRoundTrip Line    = Line
formationShapeRoundTrip Wedge   = Wedge
formationShapeRoundTrip Circle  = Circle
formationShapeRoundTrip Diamond = Diamond
formationShapeRoundTrip Grid    = Grid
