use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::events::{self, DomainEvent, DomainEventBus};

/// How to aggregate results from multi-zone scatter queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GatherStrategy {
    /// Return the first successful result.
    First,
    /// Wait for all zones, merge results.
    All,
    /// Wait for a quorum (majority).
    Quorum,
    /// Custom merge function name.
    Custom(String),
}

/// Scheduling strategy selection.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum Strategy {
    /// Use the RLM (Reinforcement Learning Model) scheduler.
    Rlm,
    /// Use the TRM (Tree of Reasoning Models) scheduler with a specific model.
    Trm(String),
    /// Automatically select the best strategy based on query characteristics.
    #[default]
    Auto,
    /// Hybrid: use a triage model to classify, then dispatch.
    Hybrid { triage: String, threshold: f32 },
    /// Edge-local inference on the device.
    Edge,
    /// Cross-zone scatter-gather via swarm.
    Swarm {
        scatter_zones: Vec<String>,
        gather_strategy: GatherStrategy,
        timeout_ms: u64,
    },
}

impl Strategy {
    /// Create a default Swarm strategy targeting all zones.
    pub fn swarm_default() -> Self {
        Strategy::Swarm {
            scatter_zones: vec![
                "zone-a".into(),
                "zone-b".into(),
                "zone-c".into(),
                "zone-d".into(),
            ],
            gather_strategy: GatherStrategy::First,
            timeout_ms: 10_000,
        }
    }
}

/// Configuration for the kernel scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub default_strategy: Strategy,
    pub max_recursion_depth: usize,
    pub query_timeout: Duration,
    pub max_concurrent_processes: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            default_strategy: Strategy::Auto,
            max_recursion_depth: 10,
            query_timeout: Duration::from_secs(30),
            max_concurrent_processes: 64,
        }
    }
}

/// The core scheduler that dispatches queries to the appropriate strategy.
pub struct Scheduler {
    pub config: SchedulerConfig,
    /// Optional domain event bus for emitting `QueryRouted` events.
    pub event_bus: Option<DomainEventBus>,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            event_bus: None,
        }
    }

    /// Builder method: attach a domain event bus.
    pub fn with_event_bus(mut self, bus: DomainEventBus) -> Self {
        self.event_bus = Some(bus);
        self
    }

    /// Set or replace the domain event bus.
    pub fn set_event_bus(&mut self, bus: DomainEventBus) {
        self.event_bus = Some(bus);
    }

    /// Determine which strategy to use for a given query.
    /// Returns the resolved strategy (never Auto -- Auto is resolved here).
    /// Emits a `QueryRouted` domain event when an event bus is present.
    pub fn resolve_strategy(&self, query: &str, hint: Option<&Strategy>) -> Strategy {
        let strategy = hint.unwrap_or(&self.config.default_strategy);

        let resolved = match strategy {
            Strategy::Auto => self.auto_select(query),
            other => other.clone(),
        };

        // Emit QueryRouted event.
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);
        let query_hash = hasher.finish();

        let strategy_name = format!("{:?}", resolved);
        events::emit(
            &self.event_bus,
            DomainEvent::QueryRouted {
                query_hash,
                strategy: strategy_name,
                confidence: 1.0,
                timestamp: Utc::now(),
            },
        );

        resolved
    }

    /// Simple heuristic for auto-selecting a strategy.
    fn auto_select(&self, query: &str) -> Strategy {
        let len = query.len();
        let has_code = query.contains("```") || query.contains("fn ") || query.contains("def ");
        let is_question = query.trim_end().ends_with('?');

        if has_code || len > 500 {
            Strategy::Trm("default".into())
        } else if is_question && len < 200 {
            Strategy::Rlm
        } else {
            Strategy::Hybrid {
                triage: "auto-triage".into(),
                threshold: 0.5,
            }
        }
    }

    /// Check whether the current recursion depth is within limits.
    pub fn check_recursion_depth(&self, depth: usize) -> bool {
        depth < self.config.max_recursion_depth
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new(SchedulerConfig::default())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_event_bus_builder() {
        let (tx, _rx) = crate::events::create_event_bus();
        let scheduler = Scheduler::default().with_event_bus(tx);
        assert!(scheduler.event_bus.is_some());
    }

    #[test]
    fn test_set_event_bus() {
        let mut scheduler = Scheduler::default();
        assert!(scheduler.event_bus.is_none());
        let (tx, _rx) = crate::events::create_event_bus();
        scheduler.set_event_bus(tx);
        assert!(scheduler.event_bus.is_some());
    }

    #[test]
    fn test_query_routed_event_emitted() {
        let (tx, mut rx) = crate::events::create_event_bus();
        let scheduler = Scheduler::default().with_event_bus(tx);

        let _strategy = scheduler.resolve_strategy("What is Rust?", None);

        let event = rx.try_recv().expect("should receive QueryRouted event");
        match event {
            DomainEvent::QueryRouted {
                strategy,
                confidence,
                ..
            } => {
                assert!(strategy.contains("Rlm"), "expected Rlm, got {}", strategy);
                assert!((confidence - 1.0).abs() < f64::EPSILON);
            }
            other => panic!("expected QueryRouted, got {:?}", other),
        }
    }

    #[test]
    fn test_query_routed_event_with_explicit_hint() {
        let (tx, mut rx) = crate::events::create_event_bus();
        let scheduler = Scheduler::default().with_event_bus(tx);

        let _strategy = scheduler.resolve_strategy("test", Some(&Strategy::Rlm));

        let event = rx.try_recv().expect("should receive QueryRouted event");
        match event {
            DomainEvent::QueryRouted { strategy, .. } => {
                assert!(strategy.contains("Rlm"), "expected Rlm, got {}", strategy);
            }
            other => panic!("expected QueryRouted, got {:?}", other),
        }
    }

    #[test]
    fn test_resolve_without_event_bus() {
        let scheduler = Scheduler::default();
        // Should work fine without an event bus.
        let strategy = scheduler.resolve_strategy("What is Rust?", None);
        assert!(matches!(strategy, Strategy::Rlm));
    }

    #[test]
    fn test_heuristic_code_query() {
        let scheduler = Scheduler::default();
        let strategy = scheduler.resolve_strategy("```rust\nfn main() {}\n```", None);
        assert!(matches!(strategy, Strategy::Trm(_)));
    }

    #[test]
    fn test_heuristic_hybrid_fallback() {
        let scheduler = Scheduler::default();
        let strategy = scheduler.resolve_strategy("analyze this data set", None);
        assert!(matches!(strategy, Strategy::Hybrid { .. }));
    }
}
