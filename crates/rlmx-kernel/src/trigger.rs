//! Trigger registry for event-to-function bindings (ADR-038).
//!
//! Triggers bind external events (HTTP, cron, domain events, channel messages)
//! to target functions. The registry handles CRUD, matching, and cycle prevention
//! via recursion depth tracking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// The type of event source that fires a trigger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerType {
    /// Incoming HTTP webhook.
    Http {
        /// URL path pattern (e.g., "/webhooks/github").
        path: String,
        /// HTTP method (GET, POST, etc.).
        method: String,
    },
    /// Cron-scheduled trigger.
    Schedule {
        /// Cron expression (e.g., "0 */5 * * * *").
        cron: String,
    },
    /// Domain event trigger.
    Event {
        /// The DomainEvent variant name to match (e.g., "StateMutated").
        domain_event: String,
    },
    /// Channel adapter trigger (ADR-039).
    Channel {
        /// Which channel adapter (e.g., "telegram", "discord").
        adapter: String,
        /// Content filter substring to match.
        filter: String,
    },
}

/// A binding from a trigger source to a target function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerBinding {
    /// Unique binding identifier.
    pub id: Uuid,
    /// The event source type.
    pub trigger_type: TriggerType,
    /// Target function name or process template to invoke.
    pub target_function: String,
    /// Optional transform expression for input mapping.
    pub transform: Option<String>,
    /// Whether this binding is active.
    pub enabled: bool,
    /// Maximum recursion depth before refusing to fire (cycle prevention).
    pub max_depth: u8,
}

/// Error type for trigger operations.
#[derive(Debug, Clone, thiserror::Error)]
pub enum TriggerError {
    #[error("trigger binding {0} not found")]
    NotFound(Uuid),
}

/// Registry of trigger bindings with matching and cycle prevention.
#[derive(Debug, Clone)]
pub struct TriggerRegistry {
    bindings: Vec<TriggerBinding>,
    active_depths: HashMap<String, u8>,
}

impl Default for TriggerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TriggerRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            bindings: Vec::new(),
            active_depths: HashMap::new(),
        }
    }

    /// Create a new trigger binding and return its ID.
    pub fn bind(
        &mut self,
        trigger_type: TriggerType,
        target_function: impl Into<String>,
        transform: Option<String>,
    ) -> Uuid {
        let id = Uuid::new_v4();
        self.bindings.push(TriggerBinding {
            id,
            trigger_type,
            target_function: target_function.into(),
            transform,
            enabled: true,
            max_depth: 5,
        });
        id
    }

    /// Remove a binding by ID.
    pub fn unbind(&mut self, id: Uuid) -> Result<(), TriggerError> {
        let pos = self
            .bindings
            .iter()
            .position(|b| b.id == id)
            .ok_or(TriggerError::NotFound(id))?;
        self.bindings.remove(pos);
        Ok(())
    }

    /// Enable a binding.
    pub fn enable(&mut self, id: Uuid) -> Result<(), TriggerError> {
        let binding = self
            .bindings
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(TriggerError::NotFound(id))?;
        binding.enabled = true;
        Ok(())
    }

    /// Disable a binding.
    pub fn disable(&mut self, id: Uuid) -> Result<(), TriggerError> {
        let binding = self
            .bindings
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(TriggerError::NotFound(id))?;
        binding.enabled = false;
        Ok(())
    }

    /// List all bindings.
    pub fn list(&self) -> &[TriggerBinding] {
        &self.bindings
    }

    /// Get a binding by ID.
    pub fn get(&self, id: Uuid) -> Option<&TriggerBinding> {
        self.bindings.iter().find(|b| b.id == id)
    }

    /// Find all enabled bindings matching a domain event type.
    pub fn match_event(&self, event_type: &str) -> Vec<&TriggerBinding> {
        self.bindings
            .iter()
            .filter(|b| {
                b.enabled
                    && matches!(&b.trigger_type, TriggerType::Event { domain_event } if domain_event == event_type)
            })
            .collect()
    }

    /// Find all enabled bindings matching an HTTP path and method.
    pub fn match_http(&self, path: &str, method: &str) -> Vec<&TriggerBinding> {
        self.bindings
            .iter()
            .filter(|b| {
                b.enabled
                    && matches!(
                        &b.trigger_type,
                        TriggerType::Http { path: p, method: m }
                        if p == path && m.eq_ignore_ascii_case(method)
                    )
            })
            .collect()
    }

    /// Find all enabled bindings matching a channel adapter and content.
    pub fn match_channel(&self, adapter: &str, content: &str) -> Vec<&TriggerBinding> {
        self.bindings
            .iter()
            .filter(|b| {
                b.enabled
                    && matches!(
                        &b.trigger_type,
                        TriggerType::Channel { adapter: a, filter }
                        if a == adapter && content.contains(filter.as_str())
                    )
            })
            .collect()
    }

    /// Check if a function can be invoked (depth < max_depth).
    /// Returns `false` if the function has reached its recursion limit.
    pub fn check_depth(&self, function: &str) -> bool {
        let depth = self.active_depths.get(function).copied().unwrap_or(0);
        // Find the binding for this function to get max_depth
        let max = self
            .bindings
            .iter()
            .find(|b| b.target_function == function)
            .map_or(5, |b| b.max_depth);
        depth < max
    }

    /// Increment the recursion depth for a function.
    pub fn enter_depth(&mut self, function: &str) {
        let depth = self.active_depths.entry(function.to_string()).or_insert(0);
        *depth += 1;
    }

    /// Decrement the recursion depth for a function.
    pub fn exit_depth(&mut self, function: &str) {
        if let Some(depth) = self.active_depths.get_mut(function) {
            *depth = depth.saturating_sub(1);
            if *depth == 0 {
                self.active_depths.remove(function);
            }
        }
    }

    /// Set max_depth for a specific binding.
    pub fn set_max_depth(&mut self, id: Uuid, max_depth: u8) -> Result<(), TriggerError> {
        let binding = self
            .bindings
            .iter_mut()
            .find(|b| b.id == id)
            .ok_or(TriggerError::NotFound(id))?;
        binding.max_depth = max_depth;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn registry() -> TriggerRegistry {
        TriggerRegistry::new()
    }

    // --- CRUD tests ---

    #[test]
    fn test_bind_creates_binding() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handle_state_mutation",
            None,
        );
        assert!(r.get(id).is_some());
    }

    #[test]
    fn test_unbind_removes_binding() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "handler",
            None,
        );
        r.unbind(id).unwrap();
        assert!(r.get(id).is_none());
    }

    #[test]
    fn test_unbind_nonexistent_returns_error() {
        let mut r = registry();
        assert!(r.unbind(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_enable_disable() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "handler",
            None,
        );
        r.disable(id).unwrap();
        assert!(!r.get(id).unwrap().enabled);
        r.enable(id).unwrap();
        assert!(r.get(id).unwrap().enabled);
    }

    #[test]
    fn test_enable_nonexistent() {
        let mut r = registry();
        assert!(r.enable(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_disable_nonexistent() {
        let mut r = registry();
        assert!(r.disable(Uuid::new_v4()).is_err());
    }

    #[test]
    fn test_list_returns_all() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "A".into(),
            },
            "a",
            None,
        );
        r.bind(
            TriggerType::Event {
                domain_event: "B".into(),
            },
            "b",
            None,
        );
        assert_eq!(r.list().len(), 2);
    }

    #[test]
    fn test_binding_has_transform() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Http {
                path: "/hook".into(),
                method: "POST".into(),
            },
            "handler",
            Some(".body.data".into()),
        );
        assert_eq!(
            r.get(id).unwrap().transform.as_deref(),
            Some(".body.data")
        );
    }

    #[test]
    fn test_binding_default_max_depth() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "handler",
            None,
        );
        assert_eq!(r.get(id).unwrap().max_depth, 5);
    }

    #[test]
    fn test_set_max_depth() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "handler",
            None,
        );
        r.set_max_depth(id, 3).unwrap();
        assert_eq!(r.get(id).unwrap().max_depth, 3);
    }

    // --- Event matching ---

    #[test]
    fn test_match_event_finds_matching() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handler1",
            None,
        );
        r.bind(
            TriggerType::Event {
                domain_event: "AgentSpawned".into(),
            },
            "handler2",
            None,
        );
        let matches = r.match_event("StateMutated");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].target_function, "handler1");
    }

    #[test]
    fn test_match_event_skips_disabled() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handler",
            None,
        );
        r.disable(id).unwrap();
        assert!(r.match_event("StateMutated").is_empty());
    }

    #[test]
    fn test_match_event_no_match() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_event("AgentSpawned").is_empty());
    }

    #[test]
    fn test_match_event_multiple_bindings() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handler1",
            None,
        );
        r.bind(
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            "handler2",
            None,
        );
        assert_eq!(r.match_event("StateMutated").len(), 2);
    }

    // --- HTTP matching ---

    #[test]
    fn test_match_http_finds_matching() {
        let mut r = registry();
        r.bind(
            TriggerType::Http {
                path: "/webhooks/github".into(),
                method: "POST".into(),
            },
            "github_handler",
            None,
        );
        let matches = r.match_http("/webhooks/github", "POST");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_match_http_case_insensitive_method() {
        let mut r = registry();
        r.bind(
            TriggerType::Http {
                path: "/api/hook".into(),
                method: "POST".into(),
            },
            "handler",
            None,
        );
        assert_eq!(r.match_http("/api/hook", "post").len(), 1);
    }

    #[test]
    fn test_match_http_wrong_method() {
        let mut r = registry();
        r.bind(
            TriggerType::Http {
                path: "/api/hook".into(),
                method: "POST".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_http("/api/hook", "GET").is_empty());
    }

    #[test]
    fn test_match_http_wrong_path() {
        let mut r = registry();
        r.bind(
            TriggerType::Http {
                path: "/api/hook".into(),
                method: "POST".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_http("/api/other", "POST").is_empty());
    }

    #[test]
    fn test_match_http_skips_disabled() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Http {
                path: "/hook".into(),
                method: "POST".into(),
            },
            "handler",
            None,
        );
        r.disable(id).unwrap();
        assert!(r.match_http("/hook", "POST").is_empty());
    }

    // --- Channel matching ---

    #[test]
    fn test_match_channel_finds_matching() {
        let mut r = registry();
        r.bind(
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "help".into(),
            },
            "help_handler",
            None,
        );
        let matches = r.match_channel("telegram", "I need help please");
        assert_eq!(matches.len(), 1);
    }

    #[test]
    fn test_match_channel_no_match_wrong_adapter() {
        let mut r = registry();
        r.bind(
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "help".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_channel("discord", "I need help").is_empty());
    }

    #[test]
    fn test_match_channel_no_match_wrong_content() {
        let mut r = registry();
        r.bind(
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "help".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_channel("telegram", "goodbye").is_empty());
    }

    #[test]
    fn test_match_channel_empty_filter_matches_all() {
        let mut r = registry();
        r.bind(
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "".into(),
            },
            "catch_all",
            None,
        );
        assert_eq!(r.match_channel("telegram", "anything").len(), 1);
    }

    #[test]
    fn test_match_channel_skips_disabled() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "help".into(),
            },
            "handler",
            None,
        );
        r.disable(id).unwrap();
        assert!(r.match_channel("telegram", "help me").is_empty());
    }

    // --- Depth tracking (cycle prevention) ---

    #[test]
    fn test_check_depth_initially_ok() {
        let r = registry();
        assert!(r.check_depth("my_function"));
    }

    #[test]
    fn test_enter_exit_depth() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "my_fn",
            None,
        );
        r.enter_depth("my_fn");
        assert!(r.check_depth("my_fn")); // depth=1 < 5
        r.exit_depth("my_fn");
        assert!(r.check_depth("my_fn")); // depth=0
    }

    #[test]
    fn test_depth_reaches_max() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "my_fn",
            None,
        );
        r.set_max_depth(id, 3).unwrap();
        r.enter_depth("my_fn"); // 1
        r.enter_depth("my_fn"); // 2
        r.enter_depth("my_fn"); // 3
        assert!(!r.check_depth("my_fn")); // 3 >= 3
    }

    #[test]
    fn test_depth_recovers_after_exit() {
        let mut r = registry();
        let id = r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "my_fn",
            None,
        );
        r.set_max_depth(id, 2).unwrap();
        r.enter_depth("my_fn");
        r.enter_depth("my_fn");
        assert!(!r.check_depth("my_fn"));
        r.exit_depth("my_fn");
        assert!(r.check_depth("my_fn"));
    }

    #[test]
    fn test_exit_depth_saturates_at_zero() {
        let mut r = registry();
        r.exit_depth("nonexistent"); // Should not panic
        r.enter_depth("fn");
        r.exit_depth("fn");
        r.exit_depth("fn"); // Should not underflow
    }

    #[test]
    fn test_independent_function_depths() {
        let mut r = registry();
        r.enter_depth("fn_a");
        r.enter_depth("fn_a");
        r.enter_depth("fn_b");
        // fn_a has depth 2, fn_b has depth 1
        assert!(r.check_depth("fn_a"));
        assert!(r.check_depth("fn_b"));
    }

    // --- Serde ---

    #[test]
    fn test_trigger_type_serde_roundtrip() {
        let types = vec![
            TriggerType::Http {
                path: "/hook".into(),
                method: "POST".into(),
            },
            TriggerType::Schedule {
                cron: "0 */5 * * * *".into(),
            },
            TriggerType::Event {
                domain_event: "StateMutated".into(),
            },
            TriggerType::Channel {
                adapter: "telegram".into(),
                filter: "help".into(),
            },
        ];
        for tt in &types {
            let json = serde_json::to_string(tt).unwrap();
            let back: TriggerType = serde_json::from_str(&json).unwrap();
            assert_eq!(*tt, back);
        }
    }

    #[test]
    fn test_trigger_binding_serde_roundtrip() {
        let binding = TriggerBinding {
            id: Uuid::new_v4(),
            trigger_type: TriggerType::Event {
                domain_event: "Test".into(),
            },
            target_function: "handler".into(),
            transform: Some(".data".into()),
            enabled: true,
            max_depth: 5,
        };
        let json = serde_json::to_string(&binding).unwrap();
        let back: TriggerBinding = serde_json::from_str(&json).unwrap();
        assert_eq!(binding.id, back.id);
        assert_eq!(binding.target_function, back.target_function);
    }

    #[test]
    fn test_default_impl() {
        let r = TriggerRegistry::default();
        assert!(r.list().is_empty());
    }

    // --- Does not match non-trigger-type bindings ---

    #[test]
    fn test_match_event_ignores_http_bindings() {
        let mut r = registry();
        r.bind(
            TriggerType::Http {
                path: "/hook".into(),
                method: "POST".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_event("Http").is_empty());
    }

    #[test]
    fn test_match_http_ignores_event_bindings() {
        let mut r = registry();
        r.bind(
            TriggerType::Event {
                domain_event: "Test".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_http("/hook", "POST").is_empty());
    }

    #[test]
    fn test_match_channel_ignores_schedule_bindings() {
        let mut r = registry();
        r.bind(
            TriggerType::Schedule {
                cron: "0 * * * *".into(),
            },
            "handler",
            None,
        );
        assert!(r.match_channel("telegram", "hello").is_empty());
    }
}
