-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- ECM Safety — SPARK-proven implementations.

pragma SPARK_Mode (On);

package body ECM_Safety
  with SPARK_Mode
is

   function Band_Lower (Band : Frequency_Band) return Frequency_Hz is
   begin
      case Band is
         when HF  => return 0;
         when VHF => return 30_000_000;
         when UHF => return 300_000_000;
         when SHF => return 3_000_000_000;
      end case;
   end Band_Lower;

   function Band_Upper (Band : Frequency_Band) return Frequency_Hz is
   begin
      case Band is
         when HF  => return 30_000_000;
         when VHF => return 300_000_000;
         when UHF => return 3_000_000_000;
         when SHF => return 30_000_000_000;
      end case;
   end Band_Upper;

   function In_Band
     (Freq : Frequency_Hz;
      Band : Frequency_Band) return Boolean
   is
   begin
      return Freq >= Band_Lower (Band) and Freq < Band_Upper (Band);
   end In_Band;

   function Power_Safe (P : Power_MW) return Boolean is
   begin
      return P <= 100.0;
   end Power_Safe;

end ECM_Safety;
