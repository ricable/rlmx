//! FederationCycle: the aggregate root for the Federated Learning bounded context.
//!
//! A `FederationCycle` represents one weekly federation round. It progresses
//! through four statuses: Collecting -> Aggregating -> Distributing -> Completed.
//!
//! The cycle owns:
//! - All contributions received during the collection window.
//! - Aggregated models per domain (once aggregation is complete).
//! - Distribution packages built from aggregated models.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use uuid::Uuid;

use rlmx_kernel::LifeDomain;

use crate::aggregator::{AggregatedModel, AggregatorConfig, FederatedAggregator};
use crate::contribution::Contribution;
use crate::distribution::{FederationPackage, PackageDistributor};
use crate::error::{FederationError, Result};

/// Status of a federation cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CycleStatus {
    /// Accepting contributions from devices.
    Collecting,
    /// Contributions closed; aggregation in progress.
    Aggregating,
    /// Aggregated models built; distributing packages.
    Distributing,
    /// Cycle complete; packages distributed.
    Completed,
}

/// The aggregate root for the Federated Learning bounded context (DDD-012).
///
/// Manages a single weekly federation round from contribution collection
/// through aggregation and distribution.
#[derive(Debug)]
pub struct FederationCycle {
    /// Unique identifier for this cycle.
    pub id: Uuid,
    /// Monotonically increasing cycle number.
    pub cycle_number: u64,
    /// Current status of this cycle.
    pub status: CycleStatus,
    /// All contributions received during the collection phase.
    pub contributions: Vec<Contribution>,
    /// Aggregated models keyed by domain (populated during Aggregating phase).
    pub aggregated_models: HashMap<LifeDomain, AggregatedModel>,
    /// Distribution packages (populated during Distributing phase).
    pub packages: Vec<FederationPackage>,
    /// When this cycle started.
    pub started_at: DateTime<Utc>,
    /// When this cycle completed (None if still in progress).
    pub completed_at: Option<DateTime<Utc>>,
    /// Aggregator configuration.
    aggregator_config: AggregatorConfig,
}

impl FederationCycle {
    /// Start a new federation cycle.
    pub fn start_cycle(cycle_number: u64, aggregator_config: AggregatorConfig) -> Self {
        let id = Uuid::new_v4();
        info!(cycle_id = %id, cycle_number, "starting federation cycle");

        Self {
            id,
            cycle_number,
            status: CycleStatus::Collecting,
            contributions: Vec::new(),
            aggregated_models: HashMap::new(),
            packages: Vec::new(),
            started_at: Utc::now(),
            completed_at: None,
            aggregator_config,
        }
    }

    /// Accept a contribution during the collection phase.
    pub fn accept_contribution(&mut self, contribution: Contribution) -> Result<()> {
        if self.status != CycleStatus::Collecting {
            return Err(FederationError::InvalidCycleStatus(self.id));
        }

        contribution.validate()?;

        debug!(
            cycle_id = %self.id,
            contributor = %contribution.user_pseudonym,
            domain = ?contribution.domain,
            patterns = contribution.pattern_count(),
            "accepted contribution"
        );

        self.contributions.push(contribution);
        Ok(())
    }

    /// Transition from Collecting to Aggregating, then perform aggregation.
    ///
    /// Returns the number of domains successfully aggregated.
    pub fn aggregate(&mut self) -> Result<usize> {
        if self.status != CycleStatus::Collecting {
            return Err(FederationError::InvalidCycleStatus(self.id));
        }

        self.status = CycleStatus::Aggregating;
        info!(
            cycle_id = %self.id,
            contribution_count = self.contributions.len(),
            "starting aggregation"
        );

        let aggregator = FederatedAggregator::new(self.aggregator_config.clone());
        self.aggregated_models = aggregator.aggregate_all(&self.contributions);

        let domain_count = self.aggregated_models.len();
        info!(
            cycle_id = %self.id,
            domains_aggregated = domain_count,
            "aggregation complete"
        );

        Ok(domain_count)
    }

    /// Transition from Aggregating to Distributing; build distribution packages.
    ///
    /// Returns the number of packages built.
    pub fn distribute(&mut self) -> Result<usize> {
        if self.status != CycleStatus::Aggregating {
            return Err(FederationError::InvalidCycleStatus(self.id));
        }

        self.status = CycleStatus::Distributing;
        let version = format!("{}.0.0", self.cycle_number);

        let mut built = 0;
        for model in self.aggregated_models.values() {
            match FederationPackage::from_aggregated(model, self.id, &version) {
                Ok(pkg) => {
                    self.packages.push(pkg);
                    built += 1;
                }
                Err(e) => {
                    warn!(
                        cycle_id = %self.id,
                        domain = ?model.domain,
                        error = %e,
                        "failed to build package for domain"
                    );
                }
            }
        }

        info!(
            cycle_id = %self.id,
            packages_built = built,
            "distribution packages ready"
        );

        Ok(built)
    }

    /// Mark the cycle as completed and publish packages to the distributor.
    pub fn complete(&mut self, distributor: &mut PackageDistributor) -> Result<()> {
        if self.status != CycleStatus::Distributing {
            return Err(FederationError::InvalidCycleStatus(self.id));
        }

        for pkg in &self.packages {
            distributor.publish(pkg.clone());
        }

        self.status = CycleStatus::Completed;
        self.completed_at = Some(Utc::now());

        info!(
            cycle_id = %self.id,
            cycle_number = self.cycle_number,
            packages = self.packages.len(),
            "federation cycle completed"
        );

        Ok(())
    }

    /// Check if this cycle's collection window has expired (7 days).
    pub fn collection_window_expired(&self) -> bool {
        let window = Duration::days(7);
        Utc::now() - self.started_at > window
    }

    /// Number of unique contributors across all domains.
    pub fn unique_contributor_count(&self) -> usize {
        let pseudonyms: std::collections::HashSet<&str> = self
            .contributions
            .iter()
            .map(|c| c.user_pseudonym.as_str())
            .collect();
        pseudonyms.len()
    }

    /// Number of contributions for a specific domain.
    pub fn domain_contribution_count(&self, domain: LifeDomain) -> usize {
        self.contributions
            .iter()
            .filter(|c| c.domain == domain)
            .count()
    }

    /// Get a summary of contributions per domain.
    pub fn contribution_summary(&self) -> HashMap<LifeDomain, usize> {
        let mut summary = HashMap::new();
        for c in &self.contributions {
            *summary.entry(c.domain).or_insert(0) += 1;
        }
        summary
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contribution::Contribution;
    use chrono::Utc;
    use rlmx_cognitive::voice_patterns::{AnonymizedPattern, Modality};
    use uuid::Uuid;

    fn low_threshold_config() -> AggregatorConfig {
        AggregatorConfig {
            min_contributors: 2,
            top_k_patterns: 100,
        }
    }

    fn make_contribution(domain: LifeDomain, pseudonym: &str) -> Contribution {
        let pattern = AnonymizedPattern {
            id: Uuid::new_v4(),
            sanitized_embedding: vec![0.1; 8],
            actions_taken: vec!["action".into()],
            result_quality: 0.85,
            emotion_bucket: None,
            noisy_urgency: 0.5,
            noisy_satisfaction: None,
            interaction_modality: Modality::Voice,
            timestamp: Utc::now(),
        };
        Contribution::new(pseudonym.into(), domain, vec![pattern], None)
    }

    #[test]
    fn test_cycle_start() {
        let cycle = FederationCycle::start_cycle(1, low_threshold_config());
        assert_eq!(cycle.cycle_number, 1);
        assert_eq!(cycle.status, CycleStatus::Collecting);
        assert!(cycle.contributions.is_empty());
        assert!(cycle.completed_at.is_none());
    }

    #[test]
    fn test_accept_contribution() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        let contrib = make_contribution(LifeDomain::Finance, "user-1");
        cycle.accept_contribution(contrib).unwrap();
        assert_eq!(cycle.contributions.len(), 1);
    }

    #[test]
    fn test_accept_contribution_wrong_status() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        // Manually move past Collecting.
        cycle.status = CycleStatus::Aggregating;
        let contrib = make_contribution(LifeDomain::Finance, "user-1");
        assert!(cycle.accept_contribution(contrib).is_err());
    }

    #[test]
    fn test_full_cycle() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());

        // Collect contributions.
        for i in 0..3 {
            cycle
                .accept_contribution(make_contribution(LifeDomain::Finance, &format!("user-{i}")))
                .unwrap();
        }

        // Aggregate.
        let domains = cycle.aggregate().unwrap();
        assert_eq!(domains, 1);
        assert_eq!(cycle.status, CycleStatus::Aggregating);
        assert!(cycle.aggregated_models.contains_key(&LifeDomain::Finance));

        // Distribute.
        let pkg_count = cycle.distribute().unwrap();
        assert_eq!(pkg_count, 1);
        assert_eq!(cycle.status, CycleStatus::Distributing);

        // Complete.
        let mut dist = PackageDistributor::new();
        cycle.complete(&mut dist).unwrap();
        assert_eq!(cycle.status, CycleStatus::Completed);
        assert!(cycle.completed_at.is_some());
        assert_eq!(dist.count(), 1);
    }

    #[test]
    fn test_unique_contributor_count() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "alice"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Health, "alice"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "bob"))
            .unwrap();
        assert_eq!(cycle.unique_contributor_count(), 2);
    }

    #[test]
    fn test_domain_contribution_count() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "a"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "b"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Health, "c"))
            .unwrap();
        assert_eq!(cycle.domain_contribution_count(LifeDomain::Finance), 2);
        assert_eq!(cycle.domain_contribution_count(LifeDomain::Health), 1);
        assert_eq!(cycle.domain_contribution_count(LifeDomain::Legal), 0);
    }

    #[test]
    fn test_contribution_summary() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "a"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Finance, "b"))
            .unwrap();
        cycle
            .accept_contribution(make_contribution(LifeDomain::Health, "c"))
            .unwrap();
        let summary = cycle.contribution_summary();
        assert_eq!(summary[&LifeDomain::Finance], 2);
        assert_eq!(summary[&LifeDomain::Health], 1);
    }

    #[test]
    fn test_aggregate_wrong_status() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        cycle.status = CycleStatus::Completed;
        assert!(cycle.aggregate().is_err());
    }

    #[test]
    fn test_distribute_wrong_status() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        assert!(cycle.distribute().is_err()); // Still in Collecting
    }

    #[test]
    fn test_complete_wrong_status() {
        let mut cycle = FederationCycle::start_cycle(1, low_threshold_config());
        let mut dist = PackageDistributor::new();
        assert!(cycle.complete(&mut dist).is_err()); // Still in Collecting
    }
}
