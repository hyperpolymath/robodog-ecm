K9!
# SPDX-License-Identifier: PMPL-1.0-or-later
# K9 Yard-level template: Verified configuration with logic.
# Security Level: Yard (Nickel evaluation only, no physical execution)
#
# This template is used for components that require logic (functions, 
# recursive records) but must still be strictly attenuated.

{
  pedigree = {
    schema_version = "1.0.0",
    # ATTENUATION: Only allows the pure functional evaluation of Nickel code.
    security = {
      leash = 'Yard,
      trust_level = "evaluated-logic",
      allow_network = false,
      allow_filesystem_write = false,
      allow_subprocess = false,
    },
    metadata = {
      name = "Logic-Enhanced Template",
      version = "1.0.0",
      description = "Template for components requiring functional transformations.",
    },
  },

  # CONFIGURATION: Supports dynamic generation and verification functions.
  config = {
    # LOGIC: Pure functions are permitted at the Yard level.
    calculate_total = fun a b => a + b,
    
    settings = {
      val_1 = 10,
      val_2 = 20,
      total = calculate_total val_1 val_2,
    },
  },
}
