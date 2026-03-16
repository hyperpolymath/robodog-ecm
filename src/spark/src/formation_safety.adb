-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- Formation Safety — SPARK-proven implementations.

pragma SPARK_Mode (On);

package body Formation_Safety
  with SPARK_Mode
is

   function Distance_Squared
     (A, B : Position_3D) return Float
   is
      DX : constant Float := A.X - B.X;
      DY : constant Float := A.Y - B.Y;
      DZ : constant Float := Float (A.Z) - Float (B.Z);
   begin
      return DX * DX + DY * DY + DZ * DZ;
   end Distance_Squared;

   function Ground_Safe
     (A, B : Position_3D) return Boolean
   is
   begin
      return Distance_Squared (A, B) >= Min_Separation_Ground ** 2;
   end Ground_Safe;

   function Aerial_Safe
     (A, B : Position_3D) return Boolean
   is
   begin
      return Distance_Squared (A, B) >= Min_Separation_Aerial ** 2;
   end Aerial_Safe;

end Formation_Safety;
