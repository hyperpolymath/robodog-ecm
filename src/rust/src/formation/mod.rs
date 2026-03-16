// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Formation Control — distributed coordination for multi-robot systems.
//!
//! DEFENSIVE USE ONLY — formation algorithms for search-and-rescue
//! coordination and disaster response. No offensive manoeuvre capability.
//!
//! Safety-critical distance invariants are verified in the SPARK
//! counterpart (`src/spark/src/formation_safety.ads`).

use serde::{Deserialize, Serialize};

/// 3D position in a local coordinate frame (metres).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// East displacement (metres).
    pub x: f64,
    /// North displacement (metres).
    pub y: f64,
    /// Altitude (metres above ground level).
    pub z: f64,
}

impl Position {
    /// Euclidean distance to another position.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2)
            + (self.y - other.y).powi(2)
            + (self.z - other.z).powi(2))
        .sqrt()
    }
}

/// 3D velocity vector (metres per second).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Velocity {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
}

impl Velocity {
    /// Speed (magnitude of velocity vector).
    #[must_use]
    pub fn speed(&self) -> f64 {
        (self.vx.powi(2) + self.vy.powi(2) + self.vz.powi(2)).sqrt()
    }
}

/// State of a single agent in the formation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    /// Unique agent identifier.
    pub id: u32,
    /// Current position.
    pub position: Position,
    /// Current velocity.
    pub velocity: Velocity,
    /// Whether the agent is operational.
    pub operational: bool,
}

/// Formation geometry templates.
///
/// Each template defines the relative positions of agents in the
/// formation. Actual positions are computed by scaling and rotating
/// the template to the desired heading and spacing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormationShape {
    /// Single-file line formation.
    Line,
    /// V-shaped wedge (SAR sweep pattern).
    Wedge,
    /// Circular perimeter (area defence).
    Circle,
    /// Diamond formation (balanced coverage).
    Diamond,
    /// Grid pattern (area search).
    Grid,
}

/// Formation parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormationParams {
    /// Desired shape.
    pub shape: FormationShape,
    /// Desired spacing between adjacent agents (metres).
    /// Must be >= minimum safe separation distance.
    pub spacing_m: f64,
    /// Formation heading (degrees, 0 = North, clockwise).
    pub heading_deg: f64,
    /// Centre position of the formation.
    pub centre: Position,
}

/// Compute desired positions for each agent in the formation.
///
/// Returns a vector of `(agent_id, desired_position)` pairs.
/// The autonomy module is responsible for actually moving agents
/// to these positions while respecting safety constraints.
#[must_use]
pub fn compute_formation_positions(
    agent_ids: &[u32],
    params: &FormationParams,
) -> Vec<(u32, Position)> {
    let n = agent_ids.len();
    if n == 0 {
        return Vec::new();
    }

    let heading_rad = params.heading_deg.to_radians();
    let cos_h = heading_rad.cos();
    let sin_h = heading_rad.sin();

    let offsets = match params.shape {
        FormationShape::Line => compute_line_offsets(n, params.spacing_m),
        FormationShape::Wedge => compute_wedge_offsets(n, params.spacing_m),
        FormationShape::Circle => compute_circle_offsets(n, params.spacing_m),
        FormationShape::Diamond => compute_diamond_offsets(n, params.spacing_m),
        FormationShape::Grid => compute_grid_offsets(n, params.spacing_m),
    };

    agent_ids
        .iter()
        .zip(offsets.iter())
        .map(|(&id, &(dx, dy))| {
            // Rotate offset by formation heading.
            let rx = dx * cos_h - dy * sin_h;
            let ry = dx * sin_h + dy * cos_h;
            (
                id,
                Position {
                    x: params.centre.x + rx,
                    y: params.centre.y + ry,
                    z: params.centre.z,
                },
            )
        })
        .collect()
}

/// Line formation: agents spaced along the lateral axis.
fn compute_line_offsets(n: usize, spacing: f64) -> Vec<(f64, f64)> {
    let start = -((n as f64 - 1.0) / 2.0) * spacing;
    (0..n).map(|i| (start + i as f64 * spacing, 0.0)).collect()
}

/// Wedge (V) formation: leader at front, wings spread back.
fn compute_wedge_offsets(n: usize, spacing: f64) -> Vec<(f64, f64)> {
    let mut offsets = vec![(0.0, 0.0)]; // Leader at origin.
    for i in 1..n {
        let side = if i % 2 == 1 { 1.0 } else { -1.0 };
        let rank = i.div_ceil(2) as f64;
        offsets.push((side * rank * spacing, -rank * spacing));
    }
    offsets
}

/// Circle formation: agents evenly distributed on a ring.
fn compute_circle_offsets(n: usize, spacing: f64) -> Vec<(f64, f64)> {
    if n <= 1 {
        return vec![(0.0, 0.0)];
    }
    // Compute radius from desired spacing between adjacent agents.
    let angle_step = 2.0 * std::f64::consts::PI / n as f64;
    let radius = spacing / (2.0 * (angle_step / 2.0).sin());
    (0..n)
        .map(|i| {
            let angle = i as f64 * angle_step;
            (radius * angle.cos(), radius * angle.sin())
        })
        .collect()
}

/// Diamond formation: 4-point diamond, extras fill inner ring.
fn compute_diamond_offsets(n: usize, spacing: f64) -> Vec<(f64, f64)> {
    let mut offsets = Vec::with_capacity(n);
    // Cardinal points of the diamond.
    let cardinals = [
        (0.0, spacing),    // North (leader).
        (spacing, 0.0),    // East.
        (0.0, -spacing),   // South.
        (-spacing, 0.0),   // West.
    ];
    for cardinal in cardinals.iter().take(n.min(4)) {
        offsets.push(*cardinal);
    }
    if n > 4 {
        for i in 4..n {
            // Extra agents fill the centre.
            let inner_idx = i - 4;
            let angle = inner_idx as f64 * std::f64::consts::PI / 3.0;
            let r = spacing * 0.5;
            offsets.push((r * angle.cos(), r * angle.sin()));
        }
    }
    offsets
}

/// Grid formation: rectangular grid for area search.
fn compute_grid_offsets(n: usize, spacing: f64) -> Vec<(f64, f64)> {
    let cols = (n as f64).sqrt().ceil() as usize;
    let mut offsets = Vec::with_capacity(n);
    for i in 0..n {
        let row = i / cols;
        let col = i % cols;
        let x = col as f64 * spacing - ((cols as f64 - 1.0) / 2.0) * spacing;
        let y = -(row as f64 * spacing);
        offsets.push((x, y));
    }
    offsets
}

/// Check that all agents satisfy the minimum safe separation distance.
///
/// Returns a list of agent ID pairs that violate the constraint.
/// This is the Rust-side check; the authoritative proof is in SPARK.
#[must_use]
pub fn check_separation(agents: &[AgentState], min_distance_m: f64) -> Vec<(u32, u32)> {
    let mut violations = Vec::new();
    for i in 0..agents.len() {
        for j in (i + 1)..agents.len() {
            if agents[i].operational && agents[j].operational {
                let dist = agents[i].position.distance_to(&agents[j].position);
                if dist < min_distance_m {
                    violations.push((agents[i].id, agents[j].id));
                }
            }
        }
    }
    violations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_formation_spacing() {
        let ids = vec![1, 2, 3];
        let params = FormationParams {
            shape: FormationShape::Line,
            spacing_m: 10.0,
            heading_deg: 0.0,
            centre: Position { x: 0.0, y: 0.0, z: 0.0 },
        };
        let positions = compute_formation_positions(&ids, &params);
        assert_eq!(positions.len(), 3);

        // Middle agent should be near centre.
        let mid = &positions[1].1;
        assert!((mid.x).abs() < 0.001);
    }

    #[test]
    fn circle_formation_equidistant() {
        let ids = vec![1, 2, 3, 4];
        let params = FormationParams {
            shape: FormationShape::Circle,
            spacing_m: 10.0,
            heading_deg: 0.0,
            centre: Position { x: 0.0, y: 0.0, z: 0.0 },
        };
        let positions = compute_formation_positions(&ids, &params);

        // All agents should be the same distance from centre.
        let radii: Vec<f64> = positions
            .iter()
            .map(|(_, p)| (p.x.powi(2) + p.y.powi(2)).sqrt())
            .collect();
        for r in &radii[1..] {
            assert!((r - radii[0]).abs() < 0.001);
        }
    }

    #[test]
    fn separation_violation_detected() {
        let agents = vec![
            AgentState {
                id: 1,
                position: Position { x: 0.0, y: 0.0, z: 0.0 },
                velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
                operational: true,
            },
            AgentState {
                id: 2,
                position: Position { x: 1.0, y: 0.0, z: 0.0 },
                velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
                operational: true,
            },
        ];
        let violations = check_separation(&agents, 2.0);
        assert_eq!(violations, vec![(1, 2)]);
    }

    #[test]
    fn no_violation_when_far_enough() {
        let agents = vec![
            AgentState {
                id: 1,
                position: Position { x: 0.0, y: 0.0, z: 0.0 },
                velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
                operational: true,
            },
            AgentState {
                id: 2,
                position: Position { x: 10.0, y: 0.0, z: 0.0 },
                velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
                operational: true,
            },
        ];
        let violations = check_separation(&agents, 2.0);
        assert!(violations.is_empty());
    }
}
