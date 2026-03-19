//! Safety constraint enforcement engine.

use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::types::SafetyConstraint;

/// Result of a safety check.
#[derive(Debug, Clone, PartialEq)]
pub enum SafetyResult {
    /// Action is allowed to proceed.
    Allowed,
    /// Action is rejected with a reason.
    Rejected {
        /// Reason for rejection.
        reason: String,
    },
    /// Action requires human approval.
    RequiresApproval {
        /// Reason approval is needed.
        reason: String,
    },
}

/// Entry tracking rate limit state.
#[derive(Debug)]
#[allow(dead_code)]
struct RateLimitEntry {
    /// Timestamps of recent invocations.
    timestamps: Vec<Instant>,
    /// Maximum count in window.
    max_count: u32,
    /// Window duration.
    window: Duration,
}

/// Safety engine that enforces constraints on plugin actions.
pub struct SafetyEngine {
    /// In-memory rate limit counters keyed by action name.
    rate_limits: Mutex<HashMap<String, RateLimitEntry>>,
}

impl SafetyEngine {
    /// Create a new SafetyEngine.
    pub fn new() -> Self {
        Self {
            rate_limits: Mutex::new(HashMap::new()),
        }
    }

    /// Check an action against a set of safety constraints.
    ///
    /// Returns the most restrictive result (Rejected > RequiresApproval > Allowed).
    pub fn check(
        &self,
        action: &str,
        params: &serde_json::Value,
        constraints: &[SafetyConstraint],
    ) -> SafetyResult {
        let mut result = SafetyResult::Allowed;

        for constraint in constraints {
            let check_result = match constraint {
                SafetyConstraint::ParameterBound { param, min, max } => {
                    self.check_parameter_bound(params, param, *min, *max)
                }
                SafetyConstraint::KpiGuard {
                    metric,
                    max_degradation,
                } => self.check_kpi_guard(metric, *max_degradation),
                SafetyConstraint::HumanEscalation { condition, actions } => {
                    self.check_human_escalation(action, condition, actions)
                }
                SafetyConstraint::RateLimit {
                    action: rate_action,
                    max_count,
                    window_secs,
                } => {
                    if action == rate_action {
                        self.check_rate_limit(rate_action, *max_count, *window_secs)
                    } else {
                        SafetyResult::Allowed
                    }
                }
            };

            // Keep the most restrictive result.
            result = Self::most_restrictive(result, check_result);
        }

        result
    }

    /// Check a parameter value is within bounds.
    fn check_parameter_bound(
        &self,
        params: &serde_json::Value,
        param_name: &str,
        min: f64,
        max: f64,
    ) -> SafetyResult {
        match params.get(param_name) {
            None => SafetyResult::Rejected {
                reason: format!(
                    "Required parameter '{}' is missing (expected numeric value in [{}, {}])",
                    param_name, min, max
                ),
            },
            Some(value) => match value.as_f64() {
                None => SafetyResult::Rejected {
                    reason: format!(
                        "Parameter '{}' has non-numeric type (expected numeric value in [{}, {}], got {})",
                        param_name, min, max, value
                    ),
                },
                Some(num) if num < min || num > max => SafetyResult::Rejected {
                    reason: format!(
                        "Parameter '{}' value {} is outside bounds [{}, {}]",
                        param_name, num, min, max
                    ),
                },
                Some(_) => SafetyResult::Allowed,
            },
        }
    }

    /// Check KPI guard (placeholder - would integrate with monitoring).
    fn check_kpi_guard(&self, metric: &str, max_degradation: f64) -> SafetyResult {
        // In a real implementation, this would query current KPI values
        // and predict potential degradation. For now, always allow.
        let _ = (metric, max_degradation);
        SafetyResult::Allowed
    }

    /// Check if human escalation is required.
    fn check_human_escalation(
        &self,
        action: &str,
        condition: &str,
        actions: &[String],
    ) -> SafetyResult {
        // Check if the current action is in the list of actions this constraint applies to.
        if actions.iter().any(|a| a == action) {
            return SafetyResult::RequiresApproval {
                reason: format!("Human approval required: {}", condition),
            };
        }
        SafetyResult::Allowed
    }

    /// Check rate limiting for an action.
    fn check_rate_limit(&self, action: &str, max_count: u32, window_secs: u64) -> SafetyResult {
        let mut rate_limits = self.rate_limits.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();
        let window = Duration::from_secs(window_secs);

        let entry = rate_limits
            .entry(action.to_string())
            .or_insert_with(|| RateLimitEntry {
                timestamps: Vec::new(),
                max_count,
                window,
            });

        // Remove expired timestamps.
        entry
            .timestamps
            .retain(|ts| now.duration_since(*ts) < window);

        // Check if we're over the limit.
        if entry.timestamps.len() >= max_count as usize {
            return SafetyResult::Rejected {
                reason: format!(
                    "Rate limit exceeded for '{}': {} calls in {} seconds (max {})",
                    action,
                    entry.timestamps.len(),
                    window_secs,
                    max_count
                ),
            };
        }

        // Record this invocation.
        entry.timestamps.push(now);
        SafetyResult::Allowed
    }

    /// Return the more restrictive of two safety results.
    fn most_restrictive(a: SafetyResult, b: SafetyResult) -> SafetyResult {
        match (&a, &b) {
            (SafetyResult::Rejected { .. }, _) => a,
            (_, SafetyResult::Rejected { .. }) => b,
            (SafetyResult::RequiresApproval { .. }, _) => a,
            (_, SafetyResult::RequiresApproval { .. }) => b,
            _ => SafetyResult::Allowed,
        }
    }
}

impl Default for SafetyEngine {
    fn default() -> Self {
        Self::new()
    }
}
