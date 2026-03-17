-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Types.idr — Core type universe for Robodog ECM ABI.
--
-- DEFENSIVE USE ONLY — Wassenaar Cat 5A2 / Cat 11.
-- All types are total. No partial functions, no escape hatches.
-- Invalid states are unrepresentable by construction.

module Types

import Data.Nat

%default total

||| Frequency in Hertz. Positive and bounded to the SHF upper limit.
public export
data Frequency : Type where
  MkFreq : (hz : Nat) -> {auto prf : LTE 1 hz} -> {auto upper : LTE hz 30000000000} -> Frequency

||| Extract the raw Hz value from a Frequency.
public export
freqHz : Frequency -> Nat
freqHz (MkFreq hz) = hz

||| Supported frequency bands for ECM simulation.
public export
data FreqBand : Type where
  HF  : FreqBand  -- 0–30 MHz
  VHF : FreqBand  -- 30–300 MHz
  UHF : FreqBand  -- 300 MHz–3 GHz
  SHF : FreqBand  -- 3–30 GHz

||| Signal modulation scheme.
public export
data Modulation : Type where
  CW   : Modulation  -- Continuous wave
  AM   : Modulation  -- Amplitude modulation
  FM   : Modulation  -- Frequency modulation
  PSK  : Modulation  -- Phase-shift keying
  FSK  : Modulation  -- Frequency-shift keying
  OFDM : Modulation  -- Orthogonal frequency-division multiplexing
  FHSS : Modulation  -- Frequency hopping spread spectrum
  DSSS : Modulation  -- Direct-sequence spread spectrum

||| Defensive signal classification.
||| Note: there is NO constructor for offensive targeting.
public export
data SignalClass : Type where
  Friendly        : SignalClass
  Neutral         : SignalClass
  Interference    : SignalClass
  SuspectedJammer : SignalClass

||| Post-quantum key encapsulation mechanism.
public export
data KEMAlgorithm : Type where
  Kyber1024 : KEMAlgorithm

||| Post-quantum digital signature algorithm.
public export
data SigAlgorithm : Type where
  Dilithium5    : SigAlgorithm
  SPHINCSPlus   : SigAlgorithm

||| Defensive action — NO offensive variant exists.
||| The type structurally excludes lethal or offensive actions.
public export
data DefensiveAction : Type where
  Continue           : DefensiveAction
  AvoidCollision     : DefensiveAction
  EnterSafeState     : DefensiveAction
  FrequencyHop       : DefensiveAction
  RequestHumanControl : DefensiveAction

||| Formation shape templates.
public export
data FormationShape : Type where
  Line    : FormationShape
  Wedge   : FormationShape
  Circle  : FormationShape
  Diamond : FormationShape
  Grid    : FormationShape

||| Agent identifier — positive natural number.
public export
data AgentId : Type where
  MkAgentId : (n : Nat) -> {auto prf : LTE 1 n} -> AgentId
