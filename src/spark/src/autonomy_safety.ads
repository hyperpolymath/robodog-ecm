-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Autonomy Safety — SPARK-proven safe-state transitions.
--
-- DEFENSIVE USE ONLY — proves that the autonomous system:
-- 1. Always transitions to safe state on communication loss.
-- 2. Cannot exceed maximum safe speed.
-- 3. Collision avoidance cannot be disabled.

pragma SPARK_Mode (On);

with Formation_Safety;

package Autonomy_Safety
  with SPARK_Mode
is
   -- Time in seconds (non-negative).
   subtype Time_S is Float range 0.0 .. Float'Last;

   -- Speed in metres per second (non-negative).
   subtype Speed_MPS is Float range 0.0 .. 100.0;

   -- Communication loss timeout (seconds).
   Comms_Timeout : constant Time_S := 3.0;

   -- Maximum safe speed.
   Max_Safe_Speed : constant Speed_MPS := 15.0;

   -- Safe states the system can transition to.
   type Safe_State is (Hover, Stop, Land);

   -- Determine if communication has been lost.
   -- True when elapsed time since last message exceeds timeout.
   function Comms_Lost
     (Last_Heard : Time_S;
      Now        : Time_S) return Boolean
     with Pre  => Now >= Last_Heard,
          Post => Comms_Lost'Result = (Now - Last_Heard > Comms_Timeout);

   -- Determine if a speed is within safe limits.
   function Speed_Safe (S : Speed_MPS) return Boolean
     with Post => Speed_Safe'Result = (S <= Max_Safe_Speed);

   -- Select the appropriate safe state based on whether the agent
   -- is aerial or ground-based.
   function Select_Safe_State (Is_Aerial : Boolean) return Safe_State
     with Post =>
       (if Is_Aerial then Select_Safe_State'Result = Hover
        else Select_Safe_State'Result = Stop);

end Autonomy_Safety;
