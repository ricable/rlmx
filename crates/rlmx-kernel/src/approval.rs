//! Human-in-the-loop approval tiers for dangerous operations (ADR-037).
//!
//! Every operation is classified into one of four tiers:
//! - **Auto**: proceed immediately, no notification
//! - **Notify**: proceed immediately, notify human after
//! - **Confirm**: block until human approves or timeout
//! - **Escalate**: block, require admin-level approval
//!
//! Cost-based escalation: if `cost_estimate` exceeds a policy's `cost_threshold`,
//! the tier is automatically escalated one level.
//!
//! Timeout auto-deny: pending requests older than `timeout` are auto-denied on cleanup.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// The four approval tiers, ordered by strictness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord, Hash)]
pub enum ApprovalTier {
    /// Proceed immediately, no notification.
    Auto,
    /// Proceed immediately, notify human after.
    Notify,
    /// Block until human approves or timeout auto-denies.
    Confirm,
    /// Block, require admin-level approval.
    Escalate,
}

impl ApprovalTier {
    /// Escalate one level. Escalate stays at Escalate.
    pub fn escalate(self) -> Self {
        match self {
            Self::Auto => Self::Notify,
            Self::Notify => Self::Confirm,
            Self::Confirm => Self::Escalate,
            Self::Escalate => Self::Escalate,
        }
    }
}

/// A policy mapping an operation pattern to an approval tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Operation name or pattern to match.
    pub operation: String,
    /// Base tier for this operation.
    pub tier: ApprovalTier,
    /// If cost exceeds this threshold, escalate one tier.
    pub cost_threshold: Option<u64>,
    /// Human-readable description of why this tier was chosen.
    pub description: String,
}

/// A pending approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique request identifier.
    pub id: Uuid,
    /// The operation requesting approval.
    pub operation: String,
    /// The agent that initiated the request.
    pub agent_id: Uuid,
    /// The tier level of this request.
    pub tier: ApprovalTier,
    /// Estimated cost in microcents (optional).
    pub cost_estimate: Option<u64>,
    /// When the request was created.
    pub requested_at: DateTime<Utc>,
    /// Whether it was approved (None = still pending).
    pub decided: Option<bool>,
    /// Who decided (human operator ID or "system:timeout").
    pub decided_by: Option<String>,
    /// When the decision was made.
    pub decided_at: Option<DateTime<Utc>>,
}

/// A human decision on an approval request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalDecision {
    /// Whether the operation is approved.
    pub approved: bool,
    /// Identifier of the human who decided.
    pub decided_by: String,
}

/// Error type for approval operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum ApprovalError {
    #[error("approval request {0} not found")]
    NotFound(Uuid),
    #[error("approval request {0} already decided")]
    AlreadyDecided(Uuid),
    #[error("operation requires approval but no request was created")]
    RequiresApproval,
}

/// The approval gate holds policies, pending requests, and timeout config.
#[derive(Debug, Clone)]
pub struct ApprovalGate {
    policies: Vec<ApprovalPolicy>,
    pending: HashMap<Uuid, ApprovalRequest>,
    timeout: Duration,
}

impl Default for ApprovalGate {
    fn default() -> Self {
        Self::new()
    }
}

impl ApprovalGate {
    /// Create a new gate with default policies and 5-minute timeout.
    pub fn new() -> Self {
        let policies = vec![
            // Auto-tier: read-only operations
            ApprovalPolicy {
                operation: "VecSearch".into(),
                tier: ApprovalTier::Auto,
                cost_threshold: None,
                description: "Read-only vector search".into(),
            },
            ApprovalPolicy {
                operation: "GraphQuery".into(),
                tier: ApprovalTier::Auto,
                cost_threshold: None,
                description: "Read-only graph query".into(),
            },
            ApprovalPolicy {
                operation: "AttentionSelect".into(),
                tier: ApprovalTier::Auto,
                cost_threshold: None,
                description: "Attention-based selection".into(),
            },
            // Notify-tier: mutations with low risk
            ApprovalPolicy {
                operation: "VecInsert".into(),
                tier: ApprovalTier::Notify,
                cost_threshold: Some(10_000),
                description: "Vector insertion, notify on completion".into(),
            },
            ApprovalPolicy {
                operation: "VecDelete".into(),
                tier: ApprovalTier::Notify,
                cost_threshold: Some(5_000),
                description: "Vector deletion, notify on completion".into(),
            },
            ApprovalPolicy {
                operation: "ProcessFork".into(),
                tier: ApprovalTier::Notify,
                cost_threshold: Some(50_000),
                description: "Agent process fork".into(),
            },
            ApprovalPolicy {
                operation: "ProcessSend".into(),
                tier: ApprovalTier::Notify,
                cost_threshold: None,
                description: "Inter-process message send".into(),
            },
            ApprovalPolicy {
                operation: "ArtifactWrite".into(),
                tier: ApprovalTier::Notify,
                cost_threshold: Some(100_000),
                description: "Artifact creation or mutation".into(),
            },
            // Confirm-tier: state-changing operations
            ApprovalPolicy {
                operation: "StateMutate".into(),
                tier: ApprovalTier::Confirm,
                cost_threshold: Some(500_000),
                description: "Kernel state mutation requires confirmation".into(),
            },
            ApprovalPolicy {
                operation: "MeshSync".into(),
                tier: ApprovalTier::Confirm,
                cost_threshold: None,
                description: "Cross-device mesh synchronization".into(),
            },
            ApprovalPolicy {
                operation: "FederationContribute".into(),
                tier: ApprovalTier::Confirm,
                cost_threshold: None,
                description: "Contribute patterns to federation".into(),
            },
            // Escalate-tier: billing and security-critical
            ApprovalPolicy {
                operation: "BillingUpgrade".into(),
                tier: ApprovalTier::Escalate,
                cost_threshold: None,
                description: "Billing tier change requires admin".into(),
            },
            ApprovalPolicy {
                operation: "BillingCharge".into(),
                tier: ApprovalTier::Escalate,
                cost_threshold: None,
                description: "Direct billing charge requires admin".into(),
            },
            ApprovalPolicy {
                operation: "SecurityOverride".into(),
                tier: ApprovalTier::Escalate,
                cost_threshold: None,
                description: "Security policy override requires admin".into(),
            },
        ];

        Self {
            policies,
            pending: HashMap::new(),
            timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Set the timeout duration for pending requests.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Add a custom policy.
    pub fn add_policy(&mut self, policy: ApprovalPolicy) {
        self.policies.push(policy);
    }

    /// Check which tier an operation falls into, considering cost escalation.
    pub fn check(&self, operation: &str, cost_estimate: Option<u64>) -> ApprovalTier {
        let policy = self
            .policies
            .iter()
            .find(|p| p.operation == operation);

        match policy {
            Some(p) => {
                let mut tier = p.tier;
                // Cost-based escalation
                if let (Some(threshold), Some(cost)) = (p.cost_threshold, cost_estimate) {
                    if cost > threshold {
                        tier = tier.escalate();
                    }
                }
                tier
            }
            // Unknown operations default to Confirm for safety.
            None => ApprovalTier::Confirm,
        }
    }

    /// Create an approval request for Confirm/Escalate operations.
    /// Returns the request ID.
    pub fn request(
        &mut self,
        operation: &str,
        agent_id: Uuid,
        cost_estimate: Option<u64>,
    ) -> Result<Uuid, ApprovalError> {
        let tier = self.check(operation, cost_estimate);
        match tier {
            ApprovalTier::Auto | ApprovalTier::Notify => {
                // No approval needed — these tiers don't block.
                Err(ApprovalError::RequiresApproval)
            }
            ApprovalTier::Confirm | ApprovalTier::Escalate => {
                let id = Uuid::new_v4();
                let req = ApprovalRequest {
                    id,
                    operation: operation.to_string(),
                    agent_id,
                    tier,
                    cost_estimate,
                    requested_at: Utc::now(),
                    decided: None,
                    decided_by: None,
                    decided_at: None,
                };
                self.pending.insert(id, req);
                Ok(id)
            }
        }
    }

    /// Record a human decision on a pending request.
    pub fn decide(
        &mut self,
        request_id: Uuid,
        decision: ApprovalDecision,
    ) -> Result<ApprovalRequest, ApprovalError> {
        let req = self
            .pending
            .get_mut(&request_id)
            .ok_or(ApprovalError::NotFound(request_id))?;

        if req.decided.is_some() {
            return Err(ApprovalError::AlreadyDecided(request_id));
        }

        req.decided = Some(decision.approved);
        req.decided_by = Some(decision.decided_by);
        req.decided_at = Some(Utc::now());

        Ok(req.clone())
    }

    /// Check if a request is still pending (undecided).
    pub fn is_pending(&self, request_id: Uuid) -> bool {
        self.pending
            .get(&request_id)
            .is_some_and(|r| r.decided.is_none())
    }

    /// Return all pending (undecided) requests.
    pub fn pending_requests(&self) -> Vec<&ApprovalRequest> {
        self.pending
            .values()
            .filter(|r| r.decided.is_none())
            .collect()
    }

    /// Auto-deny all requests older than timeout. Returns the denied requests.
    pub fn cleanup_expired(&mut self) -> Vec<ApprovalRequest> {
        let now = Utc::now();
        let timeout_ms = self.timeout.as_millis() as i64;
        let mut expired = Vec::new();

        for req in self.pending.values_mut() {
            if req.decided.is_some() {
                continue;
            }
            let elapsed = now
                .signed_duration_since(req.requested_at)
                .num_milliseconds();
            if elapsed >= timeout_ms {
                req.decided = Some(false);
                req.decided_by = Some("system:timeout".into());
                req.decided_at = Some(now);
                expired.push(req.clone());
            }
        }

        expired
    }

    /// Get all registered policies.
    pub fn get_policies(&self) -> &[ApprovalPolicy] {
        &self.policies
    }

    /// Get the configured timeout.
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn gate() -> ApprovalGate {
        ApprovalGate::new()
    }

    // --- Tier classification tests ---

    #[test]
    fn test_vec_search_is_auto() {
        let g = gate();
        assert_eq!(g.check("VecSearch", None), ApprovalTier::Auto);
    }

    #[test]
    fn test_graph_query_is_auto() {
        let g = gate();
        assert_eq!(g.check("GraphQuery", None), ApprovalTier::Auto);
    }

    #[test]
    fn test_attention_select_is_auto() {
        let g = gate();
        assert_eq!(g.check("AttentionSelect", None), ApprovalTier::Auto);
    }

    #[test]
    fn test_vec_insert_is_notify() {
        let g = gate();
        assert_eq!(g.check("VecInsert", None), ApprovalTier::Notify);
    }

    #[test]
    fn test_vec_delete_is_notify() {
        let g = gate();
        assert_eq!(g.check("VecDelete", None), ApprovalTier::Notify);
    }

    #[test]
    fn test_process_fork_is_notify() {
        let g = gate();
        assert_eq!(g.check("ProcessFork", None), ApprovalTier::Notify);
    }

    #[test]
    fn test_process_send_is_notify() {
        let g = gate();
        assert_eq!(g.check("ProcessSend", None), ApprovalTier::Notify);
    }

    #[test]
    fn test_artifact_write_is_notify() {
        let g = gate();
        assert_eq!(g.check("ArtifactWrite", None), ApprovalTier::Notify);
    }

    #[test]
    fn test_state_mutate_is_confirm() {
        let g = gate();
        assert_eq!(g.check("StateMutate", None), ApprovalTier::Confirm);
    }

    #[test]
    fn test_mesh_sync_is_confirm() {
        let g = gate();
        assert_eq!(g.check("MeshSync", None), ApprovalTier::Confirm);
    }

    #[test]
    fn test_federation_contribute_is_confirm() {
        let g = gate();
        assert_eq!(g.check("FederationContribute", None), ApprovalTier::Confirm);
    }

    #[test]
    fn test_billing_upgrade_is_escalate() {
        let g = gate();
        assert_eq!(g.check("BillingUpgrade", None), ApprovalTier::Escalate);
    }

    #[test]
    fn test_billing_charge_is_escalate() {
        let g = gate();
        assert_eq!(g.check("BillingCharge", None), ApprovalTier::Escalate);
    }

    #[test]
    fn test_security_override_is_escalate() {
        let g = gate();
        assert_eq!(g.check("SecurityOverride", None), ApprovalTier::Escalate);
    }

    #[test]
    fn test_unknown_operation_defaults_to_confirm() {
        let g = gate();
        assert_eq!(g.check("UnknownOp", None), ApprovalTier::Confirm);
    }

    // --- Cost-based escalation tests ---

    #[test]
    fn test_cost_below_threshold_no_escalation() {
        let g = gate();
        // VecInsert threshold is 10_000
        assert_eq!(g.check("VecInsert", Some(5_000)), ApprovalTier::Notify);
    }

    #[test]
    fn test_cost_at_threshold_no_escalation() {
        let g = gate();
        assert_eq!(g.check("VecInsert", Some(10_000)), ApprovalTier::Notify);
    }

    #[test]
    fn test_cost_above_threshold_escalates() {
        let g = gate();
        // VecInsert: Notify + cost > 10_000 => Confirm
        assert_eq!(g.check("VecInsert", Some(15_000)), ApprovalTier::Confirm);
    }

    #[test]
    fn test_cost_escalation_notify_to_confirm() {
        let g = gate();
        // VecDelete threshold is 5_000
        assert_eq!(g.check("VecDelete", Some(6_000)), ApprovalTier::Confirm);
    }

    #[test]
    fn test_cost_escalation_confirm_to_escalate() {
        let g = gate();
        // StateMutate: Confirm + cost > 500_000 => Escalate
        assert_eq!(
            g.check("StateMutate", Some(600_000)),
            ApprovalTier::Escalate
        );
    }

    #[test]
    fn test_escalate_tier_stays_escalate_on_cost() {
        // Even with cost, Escalate cannot go higher
        let g = gate();
        assert_eq!(
            g.check("BillingCharge", Some(999_999)),
            ApprovalTier::Escalate
        );
    }

    #[test]
    fn test_no_threshold_ignores_cost() {
        let g = gate();
        // ProcessSend has no cost_threshold
        assert_eq!(
            g.check("ProcessSend", Some(999_999)),
            ApprovalTier::Notify
        );
    }

    // --- Approval request/decide lifecycle ---

    #[test]
    fn test_request_confirm_tier_creates_pending() {
        let mut g = gate();
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        assert!(g.is_pending(id));
    }

    #[test]
    fn test_request_auto_tier_returns_error() {
        let mut g = gate();
        let result = g.request("VecSearch", Uuid::new_v4(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_request_notify_tier_returns_error() {
        let mut g = gate();
        let result = g.request("VecInsert", Uuid::new_v4(), None);
        assert!(result.is_err());
    }

    #[test]
    fn test_decide_approve() {
        let mut g = gate();
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        let req = g
            .decide(
                id,
                ApprovalDecision {
                    approved: true,
                    decided_by: "admin@example.com".into(),
                },
            )
            .unwrap();
        assert_eq!(req.decided, Some(true));
        assert_eq!(req.decided_by.as_deref(), Some("admin@example.com"));
        assert!(!g.is_pending(id));
    }

    #[test]
    fn test_decide_deny() {
        let mut g = gate();
        let id = g.request("MeshSync", Uuid::new_v4(), None).unwrap();
        let req = g
            .decide(
                id,
                ApprovalDecision {
                    approved: false,
                    decided_by: "ops@example.com".into(),
                },
            )
            .unwrap();
        assert_eq!(req.decided, Some(false));
    }

    #[test]
    fn test_decide_nonexistent_request() {
        let mut g = gate();
        let result = g.decide(
            Uuid::new_v4(),
            ApprovalDecision {
                approved: true,
                decided_by: "admin".into(),
            },
        );
        assert!(matches!(result, Err(ApprovalError::NotFound(_))));
    }

    #[test]
    fn test_decide_already_decided() {
        let mut g = gate();
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        g.decide(
            id,
            ApprovalDecision {
                approved: true,
                decided_by: "admin".into(),
            },
        )
        .unwrap();
        let result = g.decide(
            id,
            ApprovalDecision {
                approved: false,
                decided_by: "other".into(),
            },
        );
        assert!(matches!(result, Err(ApprovalError::AlreadyDecided(_))));
    }

    #[test]
    fn test_pending_requests_lists_undecided() {
        let mut g = gate();
        let agent = Uuid::new_v4();
        let id1 = g.request("StateMutate", agent, None).unwrap();
        let id2 = g.request("MeshSync", agent, None).unwrap();
        g.decide(
            id1,
            ApprovalDecision {
                approved: true,
                decided_by: "admin".into(),
            },
        )
        .unwrap();
        let pending = g.pending_requests();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, id2);
    }

    // --- Timeout auto-deny ---

    #[test]
    fn test_cleanup_expired_auto_denies() {
        let mut g = ApprovalGate::new().with_timeout(Duration::from_millis(0));
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        // With 0ms timeout, any request is immediately expired
        let expired = g.cleanup_expired();
        assert_eq!(expired.len(), 1);
        assert_eq!(expired[0].id, id);
        assert_eq!(expired[0].decided, Some(false));
        assert_eq!(expired[0].decided_by.as_deref(), Some("system:timeout"));
    }

    #[test]
    fn test_cleanup_does_not_expire_recent_requests() {
        let mut g = ApprovalGate::new().with_timeout(Duration::from_secs(3600));
        g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        let expired = g.cleanup_expired();
        assert!(expired.is_empty());
    }

    #[test]
    fn test_cleanup_skips_already_decided() {
        let mut g = ApprovalGate::new().with_timeout(Duration::from_millis(0));
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        g.decide(
            id,
            ApprovalDecision {
                approved: true,
                decided_by: "admin".into(),
            },
        )
        .unwrap();
        let expired = g.cleanup_expired();
        assert!(expired.is_empty());
    }

    // --- Custom policies ---

    #[test]
    fn test_add_custom_policy() {
        let mut g = gate();
        g.add_policy(ApprovalPolicy {
            operation: "CustomDanger".into(),
            tier: ApprovalTier::Escalate,
            cost_threshold: None,
            description: "Custom dangerous operation".into(),
        });
        assert_eq!(g.check("CustomDanger", None), ApprovalTier::Escalate);
    }

    #[test]
    fn test_get_policies_returns_all() {
        let g = gate();
        assert!(g.get_policies().len() >= 14);
    }

    #[test]
    fn test_with_timeout_sets_duration() {
        let g = ApprovalGate::new().with_timeout(Duration::from_secs(60));
        assert_eq!(g.timeout(), Duration::from_secs(60));
    }

    #[test]
    fn test_default_timeout_is_five_minutes() {
        let g = gate();
        assert_eq!(g.timeout(), Duration::from_secs(300));
    }

    // --- Tier escalation ---

    #[test]
    fn test_tier_escalation_chain() {
        assert_eq!(ApprovalTier::Auto.escalate(), ApprovalTier::Notify);
        assert_eq!(ApprovalTier::Notify.escalate(), ApprovalTier::Confirm);
        assert_eq!(ApprovalTier::Confirm.escalate(), ApprovalTier::Escalate);
        assert_eq!(ApprovalTier::Escalate.escalate(), ApprovalTier::Escalate);
    }

    #[test]
    fn test_tier_ordering() {
        assert!(ApprovalTier::Auto < ApprovalTier::Notify);
        assert!(ApprovalTier::Notify < ApprovalTier::Confirm);
        assert!(ApprovalTier::Confirm < ApprovalTier::Escalate);
    }

    // --- Serialization ---

    #[test]
    fn test_approval_tier_serde_roundtrip() {
        for tier in &[
            ApprovalTier::Auto,
            ApprovalTier::Notify,
            ApprovalTier::Confirm,
            ApprovalTier::Escalate,
        ] {
            let json = serde_json::to_string(tier).unwrap();
            let back: ApprovalTier = serde_json::from_str(&json).unwrap();
            assert_eq!(*tier, back);
        }
    }

    #[test]
    fn test_approval_request_serde_roundtrip() {
        let req = ApprovalRequest {
            id: Uuid::new_v4(),
            operation: "StateMutate".into(),
            agent_id: Uuid::new_v4(),
            tier: ApprovalTier::Confirm,
            cost_estimate: Some(42_000),
            requested_at: Utc::now(),
            decided: None,
            decided_by: None,
            decided_at: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        let back: ApprovalRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(req.id, back.id);
        assert_eq!(req.operation, back.operation);
    }

    #[test]
    fn test_approval_policy_serde_roundtrip() {
        let policy = ApprovalPolicy {
            operation: "TestOp".into(),
            tier: ApprovalTier::Notify,
            cost_threshold: Some(1000),
            description: "test".into(),
        };
        let json = serde_json::to_string(&policy).unwrap();
        let back: ApprovalPolicy = serde_json::from_str(&json).unwrap();
        assert_eq!(policy.operation, back.operation);
        assert_eq!(policy.tier, back.tier);
    }

    // --- Edge cases ---

    #[test]
    fn test_multiple_requests_same_operation() {
        let mut g = gate();
        let agent = Uuid::new_v4();
        let id1 = g.request("StateMutate", agent, None).unwrap();
        let id2 = g.request("StateMutate", agent, None).unwrap();
        assert_ne!(id1, id2);
        assert_eq!(g.pending_requests().len(), 2);
    }

    #[test]
    fn test_is_pending_after_decide() {
        let mut g = gate();
        let id = g.request("StateMutate", Uuid::new_v4(), None).unwrap();
        assert!(g.is_pending(id));
        g.decide(
            id,
            ApprovalDecision {
                approved: true,
                decided_by: "admin".into(),
            },
        )
        .unwrap();
        assert!(!g.is_pending(id));
    }

    #[test]
    fn test_is_pending_nonexistent() {
        let g = gate();
        assert!(!g.is_pending(Uuid::new_v4()));
    }

    #[test]
    fn test_request_escalate_tier() {
        let mut g = gate();
        let id = g
            .request("BillingUpgrade", Uuid::new_v4(), None)
            .unwrap();
        assert!(g.is_pending(id));
        let pending = g.pending_requests();
        assert!(pending.iter().any(|r| r.tier == ApprovalTier::Escalate));
    }

    #[test]
    fn test_cost_escalated_request_creates_higher_tier() {
        let mut g = gate();
        // VecInsert with cost > 10_000 escalates from Notify to Confirm
        let id = g
            .request("VecInsert", Uuid::new_v4(), Some(20_000))
            .unwrap();
        let req = g.pending.get(&id).unwrap();
        assert_eq!(req.tier, ApprovalTier::Confirm);
    }

    #[test]
    fn test_default_impl() {
        let g = ApprovalGate::default();
        assert_eq!(g.check("VecSearch", None), ApprovalTier::Auto);
    }
}
