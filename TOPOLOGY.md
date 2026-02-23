<!-- SPDX-License-Identifier: PMPL-1.0-or-later -->
<!-- TOPOLOGY.md — Project architecture map and completion dashboard -->
<!-- Last updated: 2026-02-19 -->

# Robodog ECM — Project Topology

## System Architecture

```
                        ┌─────────────────────────────────────────┐
                        │              OPERATOR / COMMAND         │
                        │        (Mission Control / SDR Interface)│
                        └───────────────────┬─────────────────────┘
                                            │
                                            ▼
                        ┌─────────────────────────────────────────┐
                        │           ROBODOG ECM PLATFORM          │
                        │                                         │
                        │  ┌───────────┐  ┌───────────────────┐  │
                        │  │ ECM       │  │  Cryptographic    │  │
                        │  │ Simulation│  │  Protocols        │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        │        │                 │              │
                        │  ┌─────▼─────┐  ┌────────▼──────────┐  │
                        │  │ Formation │  │  Defensive        │  │
                        │  │ Control   │  │  Autonomy         │  │
                        │  └─────┬─────┘  └────────┬──────────┘  │
                        └────────│─────────────────│──────────────┘
                                 │                 │
                                 ▼                 ▼
                        ┌─────────────────────────────────────────┐
                        │           AUTONOMOUS SYSTEMS            │
                        │      (Robot Swarms, Rescue Drones)      │
                        └─────────────────────────────────────────┘

                        ┌─────────────────────────────────────────┐
                        │          REPO INFRASTRUCTURE            │
                        │  Export Control     .machine_readable/  │
                        │  Justfile           0-AI-MANIFEST.a2ml  │
                        └─────────────────────────────────────────┘
```

## Completion Dashboard

```
COMPONENT                          STATUS              NOTES
─────────────────────────────────  ──────────────────  ─────────────────────────────────
CORE TECHNOLOGIES
  ECM Signal Analysis               █░░░░░░░░░  10%    SDR integration stubs
  Cryptographic Protocols           █░░░░░░░░░  10%    PQ algorithm design active
  Formation Control Algorithms      █░░░░░░░░░  10%    Distributed primitives stubs
  Defensive Autonomy Logic          █░░░░░░░░░  10%    Collision avoidance stubs

INFRASTRUCTURE
  Export Control Framework          ██████████ 100%    ITAR/EAR compliance verified
  .machine_readable/                ██████████ 100%    STATE tracking active
  0-AI-MANIFEST.a2ml                ██████████ 100%    AI entry point verified

REPO INFRASTRUCTURE
  Justfile Automation               ██████████ 100%    Standard build tasks
  Documented Use Cases              ██████████ 100%    Defensive justification complete

─────────────────────────────────────────────────────────────────────────────
OVERALL:                            ██░░░░░░░░  ~20%   Early Research Phase
```

## Key Dependencies

```
Export Control ──► Protocol Design ──► Signal Analysis ──► Simulation
     │                 │                   │                 │
     ▼                 ▼                   ▼                 ▼
Ethical Guide ──► Authz Framework ───► Threat Models ───► Recovery
```

## Update Protocol

This file is maintained by both humans and AI agents. When updating:

1. **After completing a component**: Change its bar and percentage
2. **After adding a component**: Add a new row in the appropriate section
3. **After architectural changes**: Update the ASCII diagram
4. **Date**: Update the `Last updated` comment at the top of this file

Progress bars use: `█` (filled) and `░` (empty), 10 characters wide.
Percentages: 0%, 10%, 20%, ... 100% (in 10% increments).
