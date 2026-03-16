-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Formation.idr — Formation geometry types with distance invariants.
--
-- DEFENSIVE USE ONLY — formation algorithms for SAR and disaster response.

module Formation

import Types

%default total

||| Coordinate in metres, represented as an integer millimetre count
||| to avoid floating-point in proofs. Bounded to +-50 km.
public export
data Coord : Type where
  MkCoord : (mm : Integer) ->
            {auto lo : So (mm >= -50000000)} ->
            {auto hi : So (mm <= 50000000)} ->
            Coord

||| 3D position with integer-millimetre precision.
public export
record Position3D where
  constructor MkPos
  x : Coord
  y : Coord
  z : Nat  -- Altitude in mm above ground (non-negative by construction).

||| Squared distance between two positions (in mm^2).
||| Avoids sqrt for provability. Compare against squared thresholds.
public export
distanceSquaredMM : Position3D -> Position3D -> Integer
distanceSquaredMM a b =
  let dx = coordVal a.x - coordVal b.x
      dy = coordVal a.y - coordVal b.y
      dz = cast {to=Integer} a.z - cast {to=Integer} b.z
  in dx * dx + dy * dy + dz * dz
  where
    coordVal : Coord -> Integer
    coordVal (MkCoord mm) = mm

||| Minimum safe separation for ground agents: 2.0 m = 2000 mm.
||| Squared: 4,000,000 mm^2.
public export
MinSepGroundSqMM : Integer
MinSepGroundSqMM = 4000000

||| Minimum safe separation for aerial agents: 10.0 m = 10000 mm.
||| Squared: 100,000,000 mm^2.
public export
MinSepAerialSqMM : Integer
MinSepAerialSqMM = 100000000

||| Proof witness that two positions are safely separated (ground).
public export
data GroundSafe : Position3D -> Position3D -> Type where
  IsGroundSafe : {a, b : Position3D} ->
                 So (distanceSquaredMM a b >= MinSepGroundSqMM) ->
                 GroundSafe a b

||| Proof witness that two positions are safely separated (aerial).
public export
data AerialSafe : Position3D -> Position3D -> Type where
  IsAerialSafe : {a, b : Position3D} ->
                 So (distanceSquaredMM a b >= MinSepAerialSqMM) ->
                 AerialSafe a b
