K9!
# SPDX-License-Identifier: PMPL-1.0-or-later
# Robodog-ECM — Template Hunt (K9/Nickel).
#
# This component defines the formal strategy for "Hunting" — 
//! automated reconnaissance and dependency discovery within the 
//! electronic countermeasure ecosystem.

leash = 'Kennel # Combined evaluation and execution monitoring.

pedigree = {
  schema_version = "1.0.0",
  component_type = "recon-strategy",
  description = "Defines the search patterns for dependency-based vulnerability hunting.",
}

# SEARCH PATTERNS: Defines the identifiers and markers to look for.
patterns = {
  # CRITICAL: Detect usage of deprecated or insecure crypto primitives.
  insecure_crypto = [
    "md5", "sha1", "des", "rc4",
  ],
  # POLICY: Find instances of non-standard FFI boundaries.
  unverified_ffi = [
    "extern \"C\"", "unsafe",
  ],
}

# EXECUTION HOOKS: Procedures to run when a pattern is matched.
hooks = {
  on_match = fun match_context =>
    let severity = if std.array.contains match_context.pattern patterns.insecure_crypto then 'Critical else 'Warning in
    {
      action = 'Flag,
      severity = severity,
      message = "Pattern '%{match_context.pattern}' detected in %{match_context.file}",
    }
}
