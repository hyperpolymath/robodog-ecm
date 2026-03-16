-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Types.idr — Idris2 ABI verification types for the V-lang API layer.
--
-- This file mirrors src/abi/Types.idr but scoped to the V API surface.
-- It ensures the V API types are consistent with the core ABI.

module VApi.ABI.Types

import Types as Core

%default total

||| Proof that the V API FreqBand enum is isomorphic to the core FreqBand.
||| This is a compile-time check — if the types drift, this won't typecheck.
public export
freqBandRoundTrip : Core.FreqBand -> Core.FreqBand
freqBandRoundTrip HF  = HF
freqBandRoundTrip VHF = VHF
freqBandRoundTrip UHF = UHF
freqBandRoundTrip SHF = SHF

||| Proof that the V API SignalClass enum matches core.
public export
signalClassRoundTrip : Core.SignalClass -> Core.SignalClass
signalClassRoundTrip Friendly        = Friendly
signalClassRoundTrip Neutral         = Neutral
signalClassRoundTrip Interference    = Interference
signalClassRoundTrip SuspectedJammer = SuspectedJammer

||| Proof that the V API FormationShape enum matches core.
public export
formationShapeRoundTrip : Core.FormationShape -> Core.FormationShape
formationShapeRoundTrip Line    = Line
formationShapeRoundTrip Wedge   = Wedge
formationShapeRoundTrip Circle  = Circle
formationShapeRoundTrip Diamond = Diamond
formationShapeRoundTrip Grid    = Grid
