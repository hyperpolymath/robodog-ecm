-- SPDX-License-Identifier: PMPL-1.0-or-later
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
--
-- ECM Safety — SPARK-proven signal power and frequency bounds.
--
-- DEFENSIVE USE ONLY — Wassenaar Category 11.
-- All operations are range-checked and proven free of runtime errors.

pragma SPARK_Mode (On);

package ECM_Safety
  with SPARK_Mode
is
   -- Frequency range in Hz (unsigned 64-bit to cover SHF band).
   subtype Frequency_Hz is Long_Long_Integer range 0 .. 30_000_000_000;

   -- Signal power in millwatts (non-negative, bounded).
   -- Maximum 1000 mW (1 W) — defensive simulation, not a transmitter.
   subtype Power_MW is Float range 0.0 .. 1000.0;

   -- Signal-to-noise ratio in dB (bounded to realistic range).
   subtype SNR_dB is Float range -20.0 .. 120.0;

   -- Frequency band enumeration matching the Rust/Idris2 definitions.
   type Frequency_Band is (HF, VHF, UHF, SHF);

   -- Return the lower bound of a frequency band.
   function Band_Lower (Band : Frequency_Band) return Frequency_Hz
     with Post =>
       (case Band is
          when HF  => Band_Lower'Result = 0,
          when VHF => Band_Lower'Result = 30_000_000,
          when UHF => Band_Lower'Result = 300_000_000,
          when SHF => Band_Lower'Result = 3_000_000_000);

   -- Return the upper bound of a frequency band.
   function Band_Upper (Band : Frequency_Band) return Frequency_Hz
     with Post =>
       (case Band is
          when HF  => Band_Upper'Result = 30_000_000,
          when VHF => Band_Upper'Result = 300_000_000,
          when UHF => Band_Upper'Result = 3_000_000_000,
          when SHF => Band_Upper'Result = 30_000_000_000);

   -- Check whether a frequency falls within a given band.
   function In_Band
     (Freq : Frequency_Hz;
      Band : Frequency_Band) return Boolean
     with Post =>
       In_Band'Result = (Freq >= Band_Lower (Band) and Freq < Band_Upper (Band));

   -- Validate that a signal's power is within safe simulation limits.
   -- Returns True if the power is within the allowed range.
   function Power_Safe (P : Power_MW) return Boolean
     with Post => Power_Safe'Result = (P <= 100.0);
   -- Note: 100 mW is the simulation safety limit. The subtype allows
   -- up to 1000 mW for calibration, but operational use is capped.

end ECM_Safety;
