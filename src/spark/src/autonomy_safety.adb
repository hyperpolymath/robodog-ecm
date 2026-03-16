-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Autonomy Safety — SPARK-proven implementations.

pragma SPARK_Mode (On);

package body Autonomy_Safety
  with SPARK_Mode
is

   function Comms_Lost
     (Last_Heard : Time_S;
      Now        : Time_S) return Boolean
   is
   begin
      return Now - Last_Heard > Comms_Timeout;
   end Comms_Lost;

   function Speed_Safe (S : Speed_MPS) return Boolean is
   begin
      return S <= Max_Safe_Speed;
   end Speed_Safe;

   function Select_Safe_State (Is_Aerial : Boolean) return Safe_State is
   begin
      if Is_Aerial then
         return Hover;
      else
         return Stop;
      end if;
   end Select_Safe_State;

end Autonomy_Safety;
