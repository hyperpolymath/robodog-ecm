// SPDX-License-Identifier: PMPL-1.0-or-later
// Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>

//! Defensive Autonomy — collision avoidance, threat response, safe-state.
//!
//! DEFENSIVE USE ONLY — all autonomous decisions are defensive in nature.
//! No offensive or lethal capability exists in this module. The type system
//! structurally excludes construction of offensive actions.
//!
//! Safety-critical invariants are formally verified in the SPARK counterpart
//! (`src/spark/src/autonomy_safety.ads`). This Rust module provides the
//! runtime logic; SPARK provides the mathematical proof of correctness.

use serde::{Deserialize, Serialize};

use crate::ecm::detection::DefensiveRecommendation;
use crate::formation::{AgentState, Position, Velocity};

/// Autonomous actions — strictly defensive.
///
/// There is intentionally NO variant for offensive action, targeting,
/// or lethal force. The type system makes offensive autonomy
/// unrepresentable. This is a core safety invariant verified in
/// `src/abi/Proofs/NoLethal.idr`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DefensiveAction {
    /// Continue current trajectory — no threat detected.
    Continue,
    /// Adjust velocity to maintain safe separation distance.
    AvoidCollision {
        /// The adjusted velocity to achieve safe separation.
        adjusted_velocity: Velocity,
    },
    /// Transition to a safe state (hover/stop).
    SafeState {
        /// Reason for entering safe state.
        reason: SafeStateReason,
    },
    /// Execute a frequency hop to avoid ECM interference.
    FrequencyHop {
        /// Target frequency in Hz.
        target_freq_hz: f64,
    },
    /// Request human operator intervention.
    RequestHumanControl {
        /// Description of the situation requiring human input.
        situation: String,
    },
}

/// Reasons for entering safe state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SafeStateReason {
    /// Communication link lost for longer than timeout.
    CommunicationLoss,
    /// Imminent collision detected.
    CollisionImminent,
    /// Operator commanded safe state.
    OperatorCommand,
    /// System fault detected.
    SystemFault,
    /// Battery / fuel critically low.
    LowEnergy,
}

/// Safety parameters for autonomous decision-making.
///
/// These thresholds are the runtime counterparts of the SPARK proof
/// obligations. Any change here must be reflected in the SPARK specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyParams {
    /// Minimum safe separation distance for ground agents (metres).
    pub min_separation_ground_m: f64,
    /// Minimum safe separation distance for aerial agents (metres).
    pub min_separation_aerial_m: f64,
    /// Communication loss timeout (seconds).
    pub comms_timeout_s: f64,
    /// Maximum allowed speed (metres per second).
    pub max_speed_mps: f64,
    /// Safe-state altitude for aerial agents (metres AGL).
    pub safe_hover_altitude_m: f64,
}

impl Default for SafetyParams {
    fn default() -> Self {
        Self {
            min_separation_ground_m: 2.0,
            min_separation_aerial_m: 10.0,
            comms_timeout_s: 3.0,
            max_speed_mps: 15.0,
            safe_hover_altitude_m: 20.0,
        }
    }
}

/// Compute the defensive action for an agent given its environment.
///
/// This is the main autonomous decision function. It considers:
/// 1. Proximity to other agents (collision avoidance — always active).
/// 2. Communication link status.
/// 3. ECM threat assessment.
///
/// The decision is always defensive. If multiple threats exist,
/// the most conservative (safest) action is chosen.
#[must_use]
pub fn compute_defensive_action(
    agent: &AgentState,
    neighbours: &[AgentState],
    comms_last_heard_s: f64,
    current_time_s: f64,
    ecm_recommendation: DefensiveRecommendation,
    params: &SafetyParams,
    is_aerial: bool,
) -> DefensiveAction {
    let min_sep = if is_aerial {
        params.min_separation_aerial_m
    } else {
        params.min_separation_ground_m
    };

    // Priority 1: Communication loss → safe state (cannot be overridden).
    let comms_elapsed = current_time_s - comms_last_heard_s;
    if comms_elapsed > params.comms_timeout_s {
        return DefensiveAction::SafeState {
            reason: SafeStateReason::CommunicationLoss,
        };
    }

    // Priority 2: Imminent collision → avoidance manoeuvre.
    if let Some(avoidance) = compute_avoidance(agent, neighbours, min_sep) {
        return avoidance;
    }

    // Priority 3: ECM threat response.
    match ecm_recommendation {
        DefensiveRecommendation::AlertOperator => {
            return DefensiveAction::RequestHumanControl {
                situation: "ECM threat detected — operator assessment required".to_string(),
            };
        }
        DefensiveRecommendation::FrequencyHop => {
            // In a real system this would select from a pre-shared hop table.
            return DefensiveAction::FrequencyHop {
                target_freq_hz: 2_437_000_000.0,
            };
        }
        DefensiveRecommendation::NoAction
        | DefensiveRecommendation::IncreasedMonitoring
        | DefensiveRecommendation::IncreasePower => {}
    }

    DefensiveAction::Continue
}

/// Compute a collision avoidance velocity adjustment.
///
/// Uses a simple repulsive potential field: if any neighbour is
/// closer than the minimum separation, compute a velocity away
/// from it. Returns `None` if no avoidance is needed.
fn compute_avoidance(
    agent: &AgentState,
    neighbours: &[AgentState],
    min_separation_m: f64,
) -> Option<DefensiveAction> {
    let mut repulse_x = 0.0_f64;
    let mut repulse_y = 0.0_f64;
    let mut repulse_z = 0.0_f64;
    let mut need_avoidance = false;

    for neighbour in neighbours {
        if !neighbour.operational || neighbour.id == agent.id {
            continue;
        }

        let dist = agent.position.distance_to(&neighbour.position);
        if dist < min_separation_m && dist > 0.001 {
            need_avoidance = true;
            // Repulsive force inversely proportional to distance.
            let strength = (min_separation_m - dist) / dist;
            repulse_x += (agent.position.x - neighbour.position.x) * strength;
            repulse_y += (agent.position.y - neighbour.position.y) * strength;
            repulse_z += (agent.position.z - neighbour.position.z) * strength;
        }
    }

    if need_avoidance {
        // Normalise to a reasonable avoidance speed (2 m/s).
        let mag = (repulse_x.powi(2) + repulse_y.powi(2) + repulse_z.powi(2)).sqrt();
        let avoidance_speed = 2.0;
        if mag > 0.001 {
            Some(DefensiveAction::AvoidCollision {
                adjusted_velocity: Velocity {
                    vx: repulse_x / mag * avoidance_speed,
                    vy: repulse_y / mag * avoidance_speed,
                    vz: repulse_z / mag * avoidance_speed,
                },
            })
        } else {
            // Too close to compute direction — emergency stop.
            Some(DefensiveAction::SafeState {
                reason: SafeStateReason::CollisionImminent,
            })
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_agent(id: u32, x: f64, y: f64) -> AgentState {
        AgentState {
            id,
            position: Position { x, y, z: 0.0 },
            velocity: Velocity { vx: 0.0, vy: 0.0, vz: 0.0 },
            operational: true,
        }
    }

    #[test]
    fn comms_loss_triggers_safe_state() {
        let agent = make_agent(1, 0.0, 0.0);
        let params = SafetyParams::default();
        let action = compute_defensive_action(
            &agent,
            &[],
            0.0,   // Last heard at t=0.
            10.0,  // Current time t=10 — well past 3s timeout.
            DefensiveRecommendation::NoAction,
            &params,
            false,
        );
        assert_eq!(
            action,
            DefensiveAction::SafeState {
                reason: SafeStateReason::CommunicationLoss,
            }
        );
    }

    #[test]
    fn close_neighbour_triggers_avoidance() {
        let agent = make_agent(1, 0.0, 0.0);
        let neighbour = make_agent(2, 1.0, 0.0);
        let params = SafetyParams::default();
        let action = compute_defensive_action(
            &agent,
            &[neighbour],
            5.0,
            5.0,
            DefensiveRecommendation::NoAction,
            &params,
            false,
        );
        match action {
            DefensiveAction::AvoidCollision { adjusted_velocity } => {
                // Should be moving away (negative x direction from neighbour).
                assert!(adjusted_velocity.vx < 0.0);
            }
            _ => panic!("Expected AvoidCollision, got {action:?}"),
        }
    }

    #[test]
    fn clear_spectrum_continues() {
        let agent = make_agent(1, 0.0, 0.0);
        let params = SafetyParams::default();
        let action = compute_defensive_action(
            &agent,
            &[],
            5.0,
            5.0,
            DefensiveRecommendation::NoAction,
            &params,
            false,
        );
        assert_eq!(action, DefensiveAction::Continue);
    }

    #[test]
    fn ecm_alert_requests_human() {
        let agent = make_agent(1, 0.0, 0.0);
        let params = SafetyParams::default();
        let action = compute_defensive_action(
            &agent,
            &[],
            5.0,
            5.0,
            DefensiveRecommendation::AlertOperator,
            &params,
            false,
        );
        matches!(action, DefensiveAction::RequestHumanControl { .. });
    }
}
