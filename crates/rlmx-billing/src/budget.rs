//! Per-agent budget ledger (ADR-032).
//!
//! Tracks per-agent LLM inference costs with soft/hard limits and CAS-based
//! optimistic concurrency control on the ledger.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::{BillingError, BillingResult};

/// A single cost entry recording one LLM inference call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEntry {
    /// The agent that incurred this cost.
    pub agent_id: Uuid,
    /// The model used (e.g. "claude-3-opus").
    pub model: String,
    /// The provider (e.g. "anthropic", "openai").
    pub provider: String,
    /// Input tokens consumed.
    pub tokens_in: u64,
    /// Output tokens produced.
    pub tokens_out: u64,
    /// Cost in microcents (1 cent = 1_000_000 microcents).
    pub cost_microcents: u64,
    /// When this entry was recorded.
    pub timestamp: DateTime<Utc>,
}

/// Budget policy for a single agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetPolicy {
    /// Crossing this emits a warning but does not block.
    pub soft_limit_microcents: u64,
    /// Crossing this blocks the call entirely.
    pub hard_limit_microcents: u64,
    /// Optional cap on tokens per single call.
    pub per_call_max_tokens: Option<u64>,
}

/// Result of a budget check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDecision {
    /// Whether the agent is allowed to proceed.
    pub allowed: bool,
    /// Optional warning (e.g. soft limit crossed).
    pub warning: Option<String>,
    /// Remaining budget before hard limit (0 if over).
    pub remaining_microcents: u64,
}

/// In-memory per-agent budget ledger with CAS versioning.
pub struct BudgetLedger {
    per_agent: HashMap<Uuid, Vec<BudgetEntry>>,
    policies: HashMap<Uuid, BudgetPolicy>,
    versions: HashMap<Uuid, u64>,
    /// Cached running totals — updated on record(), read in O(1) by check().
    spent_cache: HashMap<Uuid, u64>,
}

impl BudgetLedger {
    /// Create an empty ledger.
    pub fn new() -> Self {
        Self {
            per_agent: HashMap::new(),
            policies: HashMap::new(),
            versions: HashMap::new(),
            spent_cache: HashMap::new(),
        }
    }

    /// Set or replace the budget policy for an agent.
    pub fn set_policy(&mut self, agent_id: Uuid, policy: BudgetPolicy) {
        self.policies.insert(agent_id, policy);
    }

    /// Get the budget policy for an agent, if any.
    pub fn get_policy(&self, agent_id: &Uuid) -> Option<&BudgetPolicy> {
        self.policies.get(agent_id)
    }

    /// Record a budget entry with optional CAS check.
    ///
    /// If `expected_version` is `Some(v)` and the current version for this
    /// agent does not equal `v`, the write is rejected.
    ///
    /// Returns the new version number on success.
    pub fn record(
        &mut self,
        entry: BudgetEntry,
        expected_version: Option<u64>,
    ) -> BillingResult<u64> {
        let agent_id = entry.agent_id;
        let current_version = self.versions.get(&agent_id).copied().unwrap_or(0);

        if let Some(expected) = expected_version {
            if expected != current_version {
                return Err(BillingError::Internal(format!(
                    "CAS conflict: expected version {}, found {}",
                    expected, current_version
                )));
            }
        }

        // ADR-032: enforce per-call token limit
        if let Some(policy) = self.policies.get(&agent_id) {
            if let Some(max_tokens) = policy.per_call_max_tokens {
                let total_tokens = entry.tokens_in + entry.tokens_out;
                if total_tokens > max_tokens {
                    return Err(BillingError::QuotaExceeded {
                        resource: "per_call_tokens".to_string(),
                        used: total_tokens,
                        limit: max_tokens,
                    });
                }
            }
        }

        let new_version = current_version + 1;
        self.versions.insert(agent_id, new_version);
        // Update spent cache in O(1) instead of recomputing on check()
        *self.spent_cache.entry(agent_id).or_insert(0) += entry.cost_microcents;
        self.per_agent.entry(agent_id).or_default().push(entry);

        Ok(new_version)
    }

    /// Check whether an agent is within budget.
    pub fn check(&self, agent_id: &Uuid) -> BudgetDecision {
        let spent = self.total_spent(agent_id);

        let policy = match self.policies.get(agent_id) {
            Some(p) => p,
            None => {
                // No policy = unlimited
                return BudgetDecision {
                    allowed: true,
                    warning: None,
                    remaining_microcents: u64::MAX,
                };
            }
        };

        if spent >= policy.hard_limit_microcents {
            return BudgetDecision {
                allowed: false,
                warning: Some(format!(
                    "hard limit exceeded: spent {} >= limit {}",
                    spent, policy.hard_limit_microcents
                )),
                remaining_microcents: 0,
            };
        }

        let remaining = policy.hard_limit_microcents - spent;

        if spent >= policy.soft_limit_microcents {
            return BudgetDecision {
                allowed: true,
                warning: Some(format!(
                    "soft limit exceeded: spent {} >= limit {}",
                    spent, policy.soft_limit_microcents
                )),
                remaining_microcents: remaining,
            };
        }

        BudgetDecision {
            allowed: true,
            warning: None,
            remaining_microcents: remaining,
        }
    }

    /// Total cost in microcents for an agent. O(1) via cached running total.
    pub fn total_spent(&self, agent_id: &Uuid) -> u64 {
        self.spent_cache.get(agent_id).copied().unwrap_or(0)
    }

    /// Breakdown of costs by (model, provider, total_cost_microcents).
    pub fn report(&self, agent_id: &Uuid) -> Vec<(String, String, u64)> {
        let entries = match self.per_agent.get(agent_id) {
            Some(e) => e,
            None => return Vec::new(),
        };

        let mut breakdown: HashMap<(String, String), u64> = HashMap::new();
        for entry in entries {
            *breakdown
                .entry((entry.model.clone(), entry.provider.clone()))
                .or_default() += entry.cost_microcents;
        }

        let mut result: Vec<(String, String, u64)> = breakdown
            .into_iter()
            .map(|((model, provider), cost)| (model, provider, cost))
            .collect();
        result.sort_by(|a, b| b.2.cmp(&a.2));
        result
    }

    /// Get all entries for an agent.
    pub fn entries(&self, agent_id: &Uuid) -> &[BudgetEntry] {
        self.per_agent
            .get(agent_id)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

impl Default for BudgetLedger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn make_entry(agent_id: Uuid, model: &str, provider: &str, cost: u64) -> BudgetEntry {
        BudgetEntry {
            agent_id,
            model: model.to_string(),
            provider: provider.to_string(),
            tokens_in: 100,
            tokens_out: 50,
            cost_microcents: cost,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn new_ledger_is_empty() {
        let ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        assert_eq!(ledger.total_spent(&id), 0);
        assert!(ledger.entries(&id).is_empty());
        assert!(ledger.report(&id).is_empty());
    }

    #[test]
    fn record_without_cas() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let v = ledger.record(make_entry(id, "gpt-4", "openai", 5000), None).unwrap();
        assert_eq!(v, 1);
        assert_eq!(ledger.total_spent(&id), 5000);
    }

    #[test]
    fn record_with_valid_cas() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let v1 = ledger.record(make_entry(id, "gpt-4", "openai", 1000), Some(0)).unwrap();
        assert_eq!(v1, 1);
        let v2 = ledger.record(make_entry(id, "gpt-4", "openai", 2000), Some(1)).unwrap();
        assert_eq!(v2, 2);
    }

    #[test]
    fn cas_rejects_stale_version() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        // version is now 1, but we pass 0 (stale)
        let result = ledger.record(make_entry(id, "gpt-4", "openai", 2000), Some(0));
        assert!(result.is_err());
    }

    #[test]
    fn cas_rejects_future_version() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let result = ledger.record(make_entry(id, "gpt-4", "openai", 1000), Some(99));
        assert!(result.is_err());
    }

    #[test]
    fn no_policy_allows_unlimited() {
        let ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_none());
        assert_eq!(decision.remaining_microcents, u64::MAX);
    }

    #[test]
    fn hard_limit_blocks() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 10000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(!decision.allowed);
        assert!(decision.warning.is_some());
        assert_eq!(decision.remaining_microcents, 0);
    }

    #[test]
    fn hard_limit_blocks_when_exceeded() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 15000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(!decision.allowed);
    }

    #[test]
    fn soft_limit_warns_but_allows() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 7000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_some());
        assert_eq!(decision.remaining_microcents, 3000);
    }

    #[test]
    fn under_soft_limit_no_warning() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 3000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_none());
        assert_eq!(decision.remaining_microcents, 7000);
    }

    #[test]
    fn total_spent_sums_multiple_entries() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        ledger.record(make_entry(id, "claude-3", "anthropic", 2000), None).unwrap();
        ledger.record(make_entry(id, "gpt-4", "openai", 3000), None).unwrap();
        assert_eq!(ledger.total_spent(&id), 6000);
    }

    #[test]
    fn total_spent_zero_for_unknown_agent() {
        let ledger = BudgetLedger::new();
        assert_eq!(ledger.total_spent(&Uuid::new_v4()), 0);
    }

    #[test]
    fn report_breakdown_by_model_provider() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        ledger.record(make_entry(id, "gpt-4", "openai", 2000), None).unwrap();
        ledger.record(make_entry(id, "claude-3", "anthropic", 5000), None).unwrap();
        let report = ledger.report(&id);
        assert_eq!(report.len(), 2);
        // Sorted by cost descending
        assert_eq!(report[0], ("claude-3".to_string(), "anthropic".to_string(), 5000));
        assert_eq!(report[1], ("gpt-4".to_string(), "openai".to_string(), 3000));
    }

    #[test]
    fn report_empty_for_unknown_agent() {
        let ledger = BudgetLedger::new();
        assert!(ledger.report(&Uuid::new_v4()).is_empty());
    }

    #[test]
    fn entries_returns_all_for_agent() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        ledger.record(make_entry(id, "claude-3", "anthropic", 2000), None).unwrap();
        assert_eq!(ledger.entries(&id).len(), 2);
    }

    #[test]
    fn entries_empty_for_unknown_agent() {
        let ledger = BudgetLedger::new();
        assert!(ledger.entries(&Uuid::new_v4()).is_empty());
    }

    #[test]
    fn set_policy_replaces_existing() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 1000,
            hard_limit_microcents: 2000,
            per_call_max_tokens: None,
        });
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: Some(4096),
        });
        let policy = ledger.get_policy(&id).unwrap();
        assert_eq!(policy.soft_limit_microcents, 5000);
        assert_eq!(policy.hard_limit_microcents, 10000);
        assert_eq!(policy.per_call_max_tokens, Some(4096));
    }

    #[test]
    fn get_policy_returns_none_if_unset() {
        let ledger = BudgetLedger::new();
        assert!(ledger.get_policy(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn multiple_agents_independent() {
        let mut ledger = BudgetLedger::new();
        let a1 = Uuid::new_v4();
        let a2 = Uuid::new_v4();
        ledger.record(make_entry(a1, "gpt-4", "openai", 5000), None).unwrap();
        ledger.record(make_entry(a2, "claude-3", "anthropic", 3000), None).unwrap();
        assert_eq!(ledger.total_spent(&a1), 5000);
        assert_eq!(ledger.total_spent(&a2), 3000);
    }

    #[test]
    fn version_increments_per_agent() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let v1 = ledger.record(make_entry(id, "gpt-4", "openai", 100), None).unwrap();
        let v2 = ledger.record(make_entry(id, "gpt-4", "openai", 200), None).unwrap();
        let v3 = ledger.record(make_entry(id, "gpt-4", "openai", 300), None).unwrap();
        assert_eq!(v1, 1);
        assert_eq!(v2, 2);
        assert_eq!(v3, 3);
    }

    #[test]
    fn cas_version_per_agent_independent() {
        let mut ledger = BudgetLedger::new();
        let a1 = Uuid::new_v4();
        let a2 = Uuid::new_v4();
        let v1 = ledger.record(make_entry(a1, "gpt-4", "openai", 100), Some(0)).unwrap();
        let v2 = ledger.record(make_entry(a2, "gpt-4", "openai", 200), Some(0)).unwrap();
        assert_eq!(v1, 1);
        assert_eq!(v2, 1);
    }

    #[test]
    fn exact_soft_limit_triggers_warning() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 5000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_some());
    }

    #[test]
    fn exact_hard_limit_blocks() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 10000), None).unwrap();
        let decision = ledger.check(&id);
        assert!(!decision.allowed);
    }

    #[test]
    fn default_trait() {
        let ledger = BudgetLedger::default();
        assert_eq!(ledger.total_spent(&Uuid::new_v4()), 0);
    }

    #[test]
    fn report_single_model_single_provider() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        ledger.record(make_entry(id, "gpt-4", "openai", 2000), None).unwrap();
        let report = ledger.report(&id);
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].2, 3000);
    }

    #[test]
    fn report_three_distinct_models() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        ledger.record(make_entry(id, "claude-3", "anthropic", 2000), None).unwrap();
        ledger.record(make_entry(id, "llama-3", "meta", 500), None).unwrap();
        let report = ledger.report(&id);
        assert_eq!(report.len(), 3);
    }

    #[test]
    fn budget_entry_serialization() {
        let entry = make_entry(Uuid::new_v4(), "gpt-4", "openai", 1000);
        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: BudgetEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.model, "gpt-4");
        assert_eq!(deserialized.cost_microcents, 1000);
    }

    #[test]
    fn budget_policy_serialization() {
        let policy = BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: Some(4096),
        };
        let json = serde_json::to_string(&policy).unwrap();
        let deserialized: BudgetPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.soft_limit_microcents, 5000);
        assert_eq!(deserialized.per_call_max_tokens, Some(4096));
    }

    #[test]
    fn budget_decision_serialization() {
        let decision = BudgetDecision {
            allowed: true,
            warning: Some("soft limit".to_string()),
            remaining_microcents: 3000,
        };
        let json = serde_json::to_string(&decision).unwrap();
        let deserialized: BudgetDecision = serde_json::from_str(&json).unwrap();
        assert!(deserialized.allowed);
        assert_eq!(deserialized.remaining_microcents, 3000);
    }

    #[test]
    fn per_call_max_tokens_stored() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: Some(2048),
        });
        let policy = ledger.get_policy(&id).unwrap();
        assert_eq!(policy.per_call_max_tokens, Some(2048));
    }

    #[test]
    fn no_per_call_max_tokens() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 5000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        let policy = ledger.get_policy(&id).unwrap();
        assert!(policy.per_call_max_tokens.is_none());
    }

    #[test]
    fn check_after_multiple_small_entries_crossing_soft() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 3000,
            hard_limit_microcents: 10000,
            per_call_max_tokens: None,
        });
        for _ in 0..4 {
            ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        }
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_some());
        assert_eq!(decision.remaining_microcents, 6000);
    }

    #[test]
    fn check_after_multiple_entries_crossing_hard() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 3000,
            hard_limit_microcents: 5000,
            per_call_max_tokens: None,
        });
        for _ in 0..6 {
            ledger.record(make_entry(id, "gpt-4", "openai", 1000), None).unwrap();
        }
        let decision = ledger.check(&id);
        assert!(!decision.allowed);
        assert_eq!(decision.remaining_microcents, 0);
    }

    #[test]
    fn entry_preserves_token_counts() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let mut entry = make_entry(id, "gpt-4", "openai", 1000);
        entry.tokens_in = 500;
        entry.tokens_out = 250;
        ledger.record(entry, None).unwrap();
        let stored = &ledger.entries(&id)[0];
        assert_eq!(stored.tokens_in, 500);
        assert_eq!(stored.tokens_out, 250);
    }

    #[test]
    fn entry_preserves_timestamp() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        let ts = Utc::now();
        let mut entry = make_entry(id, "gpt-4", "openai", 1000);
        entry.timestamp = ts;
        ledger.record(entry, None).unwrap();
        let stored = &ledger.entries(&id)[0];
        assert_eq!(stored.timestamp, ts);
    }

    #[test]
    fn per_call_token_limit_allows_within_budget() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 100_000,
            hard_limit_microcents: 200_000,
            per_call_max_tokens: Some(1000),
        });
        let mut entry = make_entry(id, "gpt-4", "openai", 500);
        entry.tokens_in = 400;
        entry.tokens_out = 500; // total 900 <= 1000
        let result = ledger.record(entry, None);
        assert!(result.is_ok());
    }

    #[test]
    fn per_call_token_limit_rejects_exceeding() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 100_000,
            hard_limit_microcents: 200_000,
            per_call_max_tokens: Some(1000),
        });
        let mut entry = make_entry(id, "gpt-4", "openai", 500);
        entry.tokens_in = 600;
        entry.tokens_out = 500; // total 1100 > 1000
        let result = ledger.record(entry, None);
        assert!(result.is_err());
    }

    #[test]
    fn no_per_call_token_policy_allows_any_tokens() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        // No policy at all
        let mut entry = make_entry(id, "gpt-4", "openai", 500);
        entry.tokens_in = 999_999;
        entry.tokens_out = 999_999;
        let result = ledger.record(entry, None);
        assert!(result.is_ok());
    }

    #[test]
    fn zero_cost_entries_allowed() {
        let mut ledger = BudgetLedger::new();
        let id = Uuid::new_v4();
        ledger.set_policy(id, BudgetPolicy {
            soft_limit_microcents: 1000,
            hard_limit_microcents: 2000,
            per_call_max_tokens: None,
        });
        ledger.record(make_entry(id, "gpt-4", "openai", 0), None).unwrap();
        let decision = ledger.check(&id);
        assert!(decision.allowed);
        assert!(decision.warning.is_none());
        assert_eq!(decision.remaining_microcents, 2000);
    }
}
