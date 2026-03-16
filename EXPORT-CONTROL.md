<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# Export Control Compliance Framework

## Classification

This repository contains **dual-use defensive technologies** that may be
subject to export control regulations including:

- **UK Export Control Act 2002** and Strategic Export Control Lists
- **Wassenaar Arrangement** Category 5 (Information Security) and Category 11 (Military Electronics)
- **US ITAR** (International Traffic in Arms Regulations) — for reference only; UK law governs
- **US EAR** (Export Administration Regulations) — for reference only; UK law governs

### Technology Categories Present

| Category | Description | Control Status |
|----------|-------------|----------------|
| ECM Signal Analysis | Spectrum analysis, interference detection | Dual-use (Wassenaar Cat 5) |
| Post-Quantum Cryptography | Kyber, Dilithium, SPHINCS+ protocols | Potentially controlled (Cat 5A2) |
| Formation Control | Distributed coordination for robotic systems | Dual-use (Cat 11) |
| Autonomous Navigation | Collision avoidance, threat response | Dual-use (Cat 11) |

## Defensive Use Only Policy

### Permitted Uses

1. **Defensive electronic countermeasures** — protecting communications from interception
2. **Search and rescue coordination** — multi-robot disaster response
3. **Secure communications research** — post-quantum protocol design and analysis
4. **Academic research** — published, peer-reviewed defensive technology studies
5. **Civilian infrastructure protection** — spectrum monitoring and anomaly detection

### Explicitly Prohibited Uses

1. **Offensive weapons systems** — targeting, kill chains, lethal autonomy
2. **Mass surveillance** — tracking, profiling, or monitoring individuals
3. **Offensive signal jamming** — disrupting civilian or emergency communications
4. **Autonomous lethal decision-making** — any system that decides to use force
5. **Export to embargoed or sanctioned entities** — per UK/EU/UN sanctions lists

## Contributor Requirements

### Before Contributing

All contributors must:

1. **Acknowledge** this export control policy by including in their first commit message:
   ```
   Export-Control-Acknowledged: yes
   ```

2. **Verify** they are not acting on behalf of any entity on:
   - UK Consolidated List of Financial Sanctions Targets
   - EU Consolidated Sanctions List
   - UN Security Council Sanctions Committees lists

3. **Ensure** their contributions serve exclusively defensive purposes

### Code Review Requirements

All pull requests involving the following must receive explicit export control review:

- New cryptographic algorithm implementations
- Signal processing or spectrum analysis modules
- Autonomous decision-making logic
- Formation control or swarm coordination protocols
- Any hardware interface code (SDR, sensors, actuators)

## Technical Controls

### Source-Level Annotations

All modules that handle controlled technology must include the annotation:

```rust
//! DEFENSIVE USE ONLY — Electronic countermeasures for protective applications.
//! Export control classification: Wassenaar Category [X], UK ML [Y].
//! See EXPORT-CONTROL.md for compliance requirements.
```

### Build-Time Verification

The `Trustfile.a2ml` includes checks that:

- No offensive capability markers exist in source code
- Core modules declare defensive-use-only intent
- Export control documentation is present and current

### CI/CD Enforcement

The `hypatia-scan` workflow includes export control scanning rules that flag:

- Offensive terminology in code, comments, or documentation
- Undocumented cryptographic implementations
- Missing defensive-use annotations on controlled modules

## Licensing Interaction

The PMPL-1.0-or-later license does **not** override export control obligations.
Even though the source code is publicly available under an open license, users
remain responsible for compliance with all applicable export control laws in
their jurisdiction.

## Contact

For export control queries:

- **Author:** Jonathan D.A. Jewell
- **Email:** j.d.a.jewell@open.ac.uk
- **Institution:** The Open University

## Document History

| Date | Change |
|------|--------|
| 2026-03-16 | Initial export control framework for v0.1 |
