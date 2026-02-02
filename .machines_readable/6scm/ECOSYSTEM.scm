;; SPDX-License-Identifier: PMPL-1.0-or-later
;; SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
;; ECOSYSTEM.scm - Project relationship mapping

(ecosystem
  (version "1.0")
  (name "robodog-ecm")
  (type "research-platform")
  (purpose "Defensive technologies platform for Electronic Countermeasures (ECM), cryptography, and autonomous systems coordination for military defensive applications, search and rescue, and disaster response")

  (position-in-ecosystem
    (category "Autonomous Systems")
    (subcategory "Defensive Technologies / Electronic Countermeasures")
    (unique-value
      ("First open-source ECM platform for defensive autonomous systems"
       "Post-quantum cryptography for robot swarms"
       "Export-control-compliant defensive technology research"
       "Integration of formation control with secure communications")))

  (related-projects
    ((anvomidav
      (relationship "explicitly-separate")
      (description "Figure skating choreography - MUST remain separate due to export control (Russian collaboration)"))
     (robot-vacuum-cleaner
      (relationship "algorithm-donor")
      (description "Formation control algorithms applicable to defensive coordination"))
     (januskey
      (relationship "potential-integration")
      (description "Secure identity for autonomous systems authentication"))
     (affinescript
      (relationship "potential-implementation-language")
      (description "Considering AffineScript for safety-critical ECM code"))))

  (what-this-is
    ("Defensive electronic countermeasures research platform"
     "Post-quantum cryptography for autonomous systems"
     "Formation control for defensive coordination"
     "Export-control-compliant defensive technology"))

  (what-this-is-not
    ("NOT an offensive weapons system"
     "NOT for surveillance or individual tracking"
     "NOT exempt from export control regulations"
     "NOT related to anvomidav (figure skating) except algorithmic concepts"
     "NOT for export to restricted countries")))
