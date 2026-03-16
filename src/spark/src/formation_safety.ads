-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Formation Safety — SPARK-proven collision avoidance invariants.
--
-- DEFENSIVE USE ONLY — ensures minimum safe separation distance
-- between agents in a formation. This is the mathematical proof
-- that the system cannot cause a collision.

pragma SPARK_Mode (On);

package Formation_Safety
  with SPARK_Mode
is
   -- Position coordinate (metres), bounded to a 100km x 100km area.
   subtype Coordinate is Float range -50_000.0 .. 50_000.0;

   -- Altitude (metres above ground level).
   subtype Altitude is Float range 0.0 .. 10_000.0;

   -- Distance (metres), always non-negative.
   subtype Distance is Float range 0.0 .. Float'Last;

   -- Minimum safe separation for ground agents (metres).
   Min_Separation_Ground : constant Distance := 2.0;

   -- Minimum safe separation for aerial agents (metres).
   Min_Separation_Aerial : constant Distance := 10.0;

   -- 3D position.
   type Position_3D is record
      X   : Coordinate;
      Y   : Coordinate;
      Z   : Altitude;
   end record;

   -- Squared Euclidean distance between two positions.
   -- We use squared distance to avoid the sqrt, which is not
   -- provable in SPARK without custom axioms.
   function Distance_Squared
     (A, B : Position_3D) return Float
     with Post =>
       Distance_Squared'Result >= 0.0;

   -- Check whether two positions satisfy the ground separation constraint.
   function Ground_Safe
     (A, B : Position_3D) return Boolean
     with Post =>
       Ground_Safe'Result =
         (Distance_Squared (A, B) >= Min_Separation_Ground ** 2);

   -- Check whether two positions satisfy the aerial separation constraint.
   function Aerial_Safe
     (A, B : Position_3D) return Boolean
     with Post =>
       Aerial_Safe'Result =
         (Distance_Squared (A, B) >= Min_Separation_Aerial ** 2);

end Formation_Safety;
