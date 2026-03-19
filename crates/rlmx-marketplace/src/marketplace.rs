//! Marketplace aggregate root: top-level orchestrator for all marketplace operations.

use uuid::Uuid;

use crate::analytics::MarketplaceAnalytics;
use crate::billing::BillingEngine;
use crate::domain::{
    AgentPrice, DeveloperType, DeviceType, LifeDomain, ListingStatus, MarketplaceDomainEvent,
    ModelTier, Permission, PublisherId, ReviewStatus,
};
use crate::error::MarketplaceError;
use crate::featured::FeaturedEngine;
use crate::publisher::PublisherPortal;
use crate::registry::{AgentListing, AgentRegistry, ListingFilter, ListingSort};
use crate::review::{ReviewDecision, ReviewPipeline};

/// The marketplace aggregate root. All mutations flow through this struct.
#[derive(Debug, Default)]
pub struct Marketplace {
    pub registry: AgentRegistry,
    pub billing: BillingEngine,
    pub review_pipeline: ReviewPipeline,
    pub publisher_portal: PublisherPortal,
    pub featured: FeaturedEngine,
    pub analytics: MarketplaceAnalytics,
    pub categories: Vec<LifeDomain>,
}

impl Marketplace {
    /// Create a new marketplace with all 12 life domain categories.
    pub fn new() -> Self {
        Self {
            registry: AgentRegistry::new(),
            billing: BillingEngine::new(),
            review_pipeline: ReviewPipeline::new(),
            publisher_portal: PublisherPortal::new(),
            featured: FeaturedEngine::new(),
            analytics: MarketplaceAnalytics::new(),
            categories: LifeDomain::all().to_vec(),
        }
    }

    /// Register a new publisher.
    pub fn register_publisher(
        &mut self,
        name: String,
        email: String,
        developer_type: DeveloperType,
    ) -> Result<PublisherId, MarketplaceError> {
        let id = self
            .publisher_portal
            .register(name, email, developer_type)?;
        self.billing.ensure_account(id);
        Ok(id)
    }

    /// Submit a new agent listing for review.
    /// Returns (listing_id, submission_id).
    #[allow(clippy::too_many_arguments)]
    pub fn submit_agent(
        &mut self,
        name: String,
        description: String,
        domain: LifeDomain,
        publisher_id: PublisherId,
        rvf_hash: [u8; 32],
        version: semver::Version,
        price: AgentPrice,
        permissions: Vec<Permission>,
        devices: Vec<DeviceType>,
        min_model_tier: ModelTier,
        size_bytes: u64,
    ) -> Result<(Uuid, Uuid, MarketplaceDomainEvent), MarketplaceError> {
        // Verify publisher exists
        let _publisher = self
            .publisher_portal
            .get(&publisher_id)
            .ok_or(MarketplaceError::PublisherNotFound(publisher_id))?;

        let listing = AgentListing::new_draft(
            name,
            description,
            domain,
            publisher_id,
            rvf_hash,
            version,
            price,
            permissions,
            devices,
            min_model_tier,
            size_bytes,
        );

        let listing_id = listing.id;
        self.registry.insert(listing);

        // Start review
        let submission_id = self.review_pipeline.submit(listing_id);
        self.registry
            .update_status(&listing_id, ListingStatus::InReview)?;

        // Add to publisher's portfolio
        if let Some(pub_) = self.publisher_portal.get_mut(&publisher_id) {
            pub_.add_agent(listing_id);
        }

        let event = MarketplaceDomainEvent::AgentSubmitted {
            agent_id: listing_id,
            publisher_id,
        };

        tracing::info!(
            agent_id = %listing_id,
            publisher_id = ?publisher_id,
            "agent submitted for review"
        );

        Ok((listing_id, submission_id, event))
    }

    /// Run automated review for a submission.
    pub fn run_review(
        &mut self,
        submission_id: Uuid,
    ) -> Result<(ReviewStatus, MarketplaceDomainEvent), MarketplaceError> {
        // Get the agent's permissions from the submission
        let agent_id = self
            .review_pipeline
            .get(&submission_id)
            .ok_or(MarketplaceError::SubmissionNotFound(submission_id))?
            .agent_id;

        let permissions: Vec<Permission> = self
            .registry
            .get(&agent_id)
            .map(|l| l.permissions_required.clone())
            .unwrap_or_default();

        let result = self
            .review_pipeline
            .run_automated_review(&submission_id, &permissions)?;

        let event = MarketplaceDomainEvent::ReviewCompleted {
            agent_id,
            status: result.status,
        };

        // If auto-passed, set listing to Published
        if result.status == ReviewStatus::AutoPassed {
            self.registry
                .update_status(&agent_id, ListingStatus::Published)?;
        }

        Ok((result.status, event))
    }

    /// Complete human review for a flagged submission.
    pub fn complete_human_review(
        &mut self,
        submission_id: Uuid,
        reviewer_id: String,
        decision: ReviewDecision,
        notes: String,
    ) -> Result<(ReviewStatus, MarketplaceDomainEvent), MarketplaceError> {
        let agent_id = self
            .review_pipeline
            .get(&submission_id)
            .ok_or(MarketplaceError::SubmissionNotFound(submission_id))?
            .agent_id;

        let status = self.review_pipeline.complete_human_review(
            &submission_id,
            reviewer_id,
            decision,
            notes,
        )?;

        if status == ReviewStatus::Approved {
            self.registry
                .update_status(&agent_id, ListingStatus::Published)?;
        }

        let event = MarketplaceDomainEvent::ReviewCompleted { agent_id, status };

        Ok((status, event))
    }

    /// Install an agent for a user.
    pub fn install_agent(
        &mut self,
        agent_id: Uuid,
        user_id: Uuid,
        device: DeviceType,
    ) -> Result<MarketplaceDomainEvent, MarketplaceError> {
        let listing = self
            .registry
            .get(&agent_id)
            .ok_or(MarketplaceError::ListingNotFound(agent_id))?;

        let publisher_id = listing.publisher;
        let price = listing.price;
        let domain = listing.domain;

        // Record billing if paid
        if let Some(tx) = self
            .billing
            .record_sale(&publisher_id, agent_id, user_id, &price)?
        {
            self.analytics.record_revenue(agent_id, tx.gross_cents);
        }

        // Update install count
        if let Some(listing) = self.registry.get_mut(&agent_id) {
            listing.record_install();
        }

        // Track analytics
        self.analytics.record_install(agent_id, user_id, domain);

        Ok(MarketplaceDomainEvent::AgentInstalled {
            agent_id,
            user_id,
            device,
        })
    }

    /// Uninstall an agent for a user.
    pub fn uninstall_agent(
        &mut self,
        agent_id: Uuid,
        user_id: Uuid,
    ) -> Result<MarketplaceDomainEvent, MarketplaceError> {
        // Verify the listing exists
        let _listing = self
            .registry
            .get(&agent_id)
            .ok_or(MarketplaceError::ListingNotFound(agent_id))?;

        self.analytics.record_uninstall(agent_id, user_id);

        tracing::info!(agent_id = %agent_id, user_id = %user_id, "agent uninstalled");

        Ok(MarketplaceDomainEvent::AgentUninstalled { agent_id, user_id })
    }

    /// Rate an agent.
    /// Enforces DDD-010 invariant 6: auto-suspends agents with rating < 2.0
    /// after 50+ ratings and returns both the rating event and any suspension events.
    pub fn rate_agent(
        &mut self,
        agent_id: Uuid,
        user_id: Uuid,
        rating: f32,
        review: Option<String>,
    ) -> Result<Vec<MarketplaceDomainEvent>, MarketplaceError> {
        let listing = self
            .registry
            .get_mut(&agent_id)
            .ok_or(MarketplaceError::ListingNotFound(agent_id))?;

        listing.add_rating(rating);

        let mut events = vec![MarketplaceDomainEvent::AgentRated {
            agent_id,
            user_id,
            rating,
            review,
        }];

        // Enforce rating policy on the just-rated agent only (not full scan).
        if listing.rating_count >= 50
            && listing.rating < 2.0
            && listing.status == ListingStatus::Published
        {
            listing.status = ListingStatus::Suspended;
            events.push(MarketplaceDomainEvent::AgentSuspended {
                agent_id,
                reason: "Auto-suspended: rating below 2.0 after 50+ ratings".into(),
            });
        }

        Ok(events)
    }

    /// Search the registry.
    pub fn search(&self, filter: &ListingFilter, sort: ListingSort) -> Vec<&AgentListing> {
        self.registry.search(filter, sort)
    }

    /// Get all 12 life domain categories.
    pub fn categories(&self) -> &[LifeDomain] {
        &self.categories
    }

    /// Get featured agents.
    pub fn featured_agents(&self, top_n: usize) -> Vec<crate::featured::ScoredAgent> {
        let filter = ListingFilter {
            status: Some(ListingStatus::Published),
            ..Default::default()
        };
        let published = self.registry.search(&filter, ListingSort::Rating);
        FeaturedEngine::rank(&published, top_n)
    }

    /// Run monthly payout cycle.
    pub fn run_payout_cycle(&mut self) -> Vec<MarketplaceDomainEvent> {
        let payouts = self.billing.run_payout_cycle();
        payouts
            .into_iter()
            .map(|p| MarketplaceDomainEvent::PayoutProcessed {
                publisher_id: p.publisher_id,
                amount_cents: p.amount_cents,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_agent_lifecycle() {
        let mut mp = Marketplace::new();
        assert_eq!(mp.categories().len(), 12);

        // Register publisher
        let pub_id = mp
            .register_publisher(
                "TestDev".into(),
                "dev@test.com".into(),
                DeveloperType::Individual,
            )
            .unwrap();

        // Submit agent
        let (agent_id, sub_id, _event) = mp
            .submit_agent(
                "BudgetBot".into(),
                "Personal budget assistant".into(),
                LifeDomain::Finance,
                pub_id,
                [1u8; 32],
                semver::Version::new(1, 0, 0),
                AgentPrice::OneTime(999),
                vec![Permission::new("vec_search")],
                vec![DeviceType::Desktop, DeviceType::Mobile],
                ModelTier::Small,
                2048,
            )
            .unwrap();

        // Run review (should auto-pass, no sensitive permissions)
        let (status, _event) = mp.run_review(sub_id).unwrap();
        assert_eq!(status, ReviewStatus::AutoPassed);

        // Verify listing is published
        let listing = mp.registry.get(&agent_id).unwrap();
        assert_eq!(listing.status, ListingStatus::Published);

        // Install
        let _event = mp
            .install_agent(agent_id, Uuid::new_v4(), DeviceType::Desktop)
            .unwrap();

        let listing = mp.registry.get(&agent_id).unwrap();
        assert_eq!(listing.install_count, 1);

        // Rate
        let events = mp
            .rate_agent(agent_id, Uuid::new_v4(), 4.5, Some("Great!".into()))
            .unwrap();
        assert_eq!(events.len(), 1); // Just the rating event, no suspension

        let listing = mp.registry.get(&agent_id).unwrap();
        assert!((listing.rating - 4.5).abs() < 0.01);
    }

    #[test]
    fn sensitive_agent_requires_human_review() {
        let mut mp = Marketplace::new();
        let pub_id = mp
            .register_publisher(
                "HealthDev".into(),
                "health@dev.com".into(),
                DeveloperType::Organization,
            )
            .unwrap();

        let (_agent_id, sub_id, _) = mp
            .submit_agent(
                "HealthTracker".into(),
                "Health monitoring agent".into(),
                LifeDomain::Health,
                pub_id,
                [2u8; 32],
                semver::Version::new(1, 0, 0),
                AgentPrice::Monthly(499),
                vec![Permission::new("health_data")],
                vec![DeviceType::Mobile],
                ModelTier::Medium,
                4096,
            )
            .unwrap();

        let (status, _) = mp.run_review(sub_id).unwrap();
        assert_eq!(status, ReviewStatus::FlaggedForHuman);

        // Human approves
        let (status, _) = mp
            .complete_human_review(
                sub_id,
                "doc_reviewer".into(),
                ReviewDecision::Approve,
                "HIPAA compliant".into(),
            )
            .unwrap();
        assert_eq!(status, ReviewStatus::Approved);
    }

    #[test]
    fn uninstall_agent_tracks_analytics() {
        let mut mp = Marketplace::new();
        let pub_id = mp
            .register_publisher(
                "UninstDev".into(),
                "uninst@dev.com".into(),
                DeveloperType::Individual,
            )
            .unwrap();

        let (agent_id, sub_id, _) = mp
            .submit_agent(
                "RemoveMe".into(),
                "Agent to uninstall".into(),
                LifeDomain::Shopping,
                pub_id,
                [3u8; 32],
                semver::Version::new(1, 0, 0),
                AgentPrice::Free,
                vec![],
                vec![DeviceType::Desktop],
                ModelTier::Small,
                1024,
            )
            .unwrap();

        mp.run_review(sub_id).unwrap();

        let user_id = Uuid::new_v4();
        mp.install_agent(agent_id, user_id, DeviceType::Desktop)
            .unwrap();

        let event = mp.uninstall_agent(agent_id, user_id).unwrap();
        match event {
            MarketplaceDomainEvent::AgentUninstalled {
                agent_id: eid,
                user_id: uid,
            } => {
                assert_eq!(eid, agent_id);
                assert_eq!(uid, user_id);
            }
            _ => panic!("Expected AgentUninstalled event"),
        }

        // Verify analytics recorded the uninstall
        let metrics = mp.analytics.agent_metrics(&agent_id).unwrap();
        assert_eq!(metrics.total_uninstalls, 1);
    }

    #[test]
    fn low_rating_auto_suspends() {
        let mut mp = Marketplace::new();
        let pub_id = mp
            .register_publisher(
                "BadDev".into(),
                "bad@dev.com".into(),
                DeveloperType::Individual,
            )
            .unwrap();

        let (agent_id, sub_id, _) = mp
            .submit_agent(
                "BadAgent".into(),
                "Poorly rated agent".into(),
                LifeDomain::Social,
                pub_id,
                [4u8; 32],
                semver::Version::new(1, 0, 0),
                AgentPrice::Free,
                vec![],
                vec![DeviceType::Desktop],
                ModelTier::Small,
                512,
            )
            .unwrap();

        mp.run_review(sub_id).unwrap();

        // Simulate 50+ bad ratings to trigger auto-suspension
        // Directly set the listing stats to avoid 50+ rate_agent calls
        if let Some(listing) = mp.registry.get_mut(&agent_id) {
            listing.rating = 1.8;
            listing.rating_count = 49;
        }

        // The 50th rating should trigger the policy
        let events = mp.rate_agent(agent_id, Uuid::new_v4(), 1.0, None).unwrap();

        // Should have both the rating event and the suspension event
        assert_eq!(events.len(), 2);
        assert!(events
            .iter()
            .any(|e| matches!(e, MarketplaceDomainEvent::AgentSuspended { .. })));

        // Verify listing is suspended
        let listing = mp.registry.get(&agent_id).unwrap();
        assert_eq!(listing.status, ListingStatus::Suspended);
    }
}
