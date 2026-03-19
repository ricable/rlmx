//! Featured engine: ML-ranked agent recommendations and trending.

use crate::domain::LifeDomain;
use crate::registry::AgentListing;
use uuid::Uuid;

/// A scored agent for ranking.
#[derive(Debug, Clone)]
pub struct ScoredAgent {
    pub agent_id: Uuid,
    pub score: f64,
}

/// The featured engine computes rankings and recommendations.
#[derive(Debug, Default)]
pub struct FeaturedEngine {
    /// Manual featured agent IDs (curated by platform).
    curated: Vec<Uuid>,
}

impl FeaturedEngine {
    pub fn new() -> Self {
        Self {
            curated: Vec::new(),
        }
    }

    /// Add an agent to the curated featured list.
    pub fn add_curated(&mut self, agent_id: Uuid) {
        if !self.curated.contains(&agent_id) {
            self.curated.push(agent_id);
            tracing::info!(agent_id = %agent_id, "added to curated featured");
        }
    }

    /// Remove an agent from the curated list.
    pub fn remove_curated(&mut self, agent_id: &Uuid) {
        self.curated.retain(|id| id != agent_id);
    }

    /// Get the curated featured list.
    pub fn curated_list(&self) -> &[Uuid] {
        &self.curated
    }

    /// Compute a feature score for an agent listing.
    /// Stub ML scoring: weighted combination of rating, installs, and recency.
    pub fn compute_score(listing: &AgentListing) -> f64 {
        let rating_score = listing.rating as f64 / 5.0;
        let install_score = (listing.install_count as f64).ln_1p() / 10.0;
        let recency_score = if let Some(published_at) = listing.published_at {
            let days_old = (chrono::Utc::now() - published_at).num_days().max(1) as f64;
            1.0 / days_old.sqrt()
        } else {
            0.0
        };

        // Weighted: 40% rating, 35% installs, 25% recency
        rating_score * 0.40 + install_score * 0.35 + recency_score * 0.25
    }

    /// Rank a set of listings by feature score, returning top N.
    pub fn rank(listings: &[&AgentListing], top_n: usize) -> Vec<ScoredAgent> {
        let mut scored: Vec<ScoredAgent> = listings
            .iter()
            .map(|l| ScoredAgent {
                agent_id: l.id,
                score: Self::compute_score(l),
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_n);
        scored
    }

    /// Get recommended agents for a specific domain.
    pub fn recommend_for_domain(
        listings: &[&AgentListing],
        domain: LifeDomain,
        top_n: usize,
    ) -> Vec<ScoredAgent> {
        let domain_listings: Vec<&&AgentListing> =
            listings.iter().filter(|l| l.domain == domain).collect();

        let refs: Vec<&AgentListing> = domain_listings.into_iter().copied().collect();
        Self::rank(&refs, top_n)
    }

    /// Compute trending agents based on recent install velocity.
    /// Stub: uses install count as proxy for velocity.
    pub fn trending(listings: &[&AgentListing], top_n: usize) -> Vec<ScoredAgent> {
        let mut scored: Vec<ScoredAgent> = listings
            .iter()
            .map(|l| ScoredAgent {
                agent_id: l.id,
                score: l.install_count as f64,
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_n);
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;

    fn make_listing_with_stats(
        name: &str,
        domain: LifeDomain,
        rating: f32,
        installs: u64,
    ) -> AgentListing {
        let mut listing = AgentListing::new_draft(
            name.to_string(),
            format!("{name} desc"),
            domain,
            PublisherId::new(),
            [0u8; 32],
            semver::Version::new(1, 0, 0),
            AgentPrice::Free,
            vec![],
            vec![DeviceType::Desktop],
            ModelTier::Small,
            1024,
        );
        listing.rating = rating;
        listing.install_count = installs;
        listing.published_at = Some(chrono::Utc::now());
        listing
    }

    #[test]
    fn score_increases_with_rating() {
        let low = make_listing_with_stats("Low", LifeDomain::Finance, 1.0, 100);
        let high = make_listing_with_stats("High", LifeDomain::Finance, 5.0, 100);

        assert!(FeaturedEngine::compute_score(&high) > FeaturedEngine::compute_score(&low));
    }

    #[test]
    fn rank_returns_top_n() {
        let a = make_listing_with_stats("A", LifeDomain::Finance, 4.5, 1000);
        let b = make_listing_with_stats("B", LifeDomain::Finance, 3.0, 500);
        let c = make_listing_with_stats("C", LifeDomain::Finance, 5.0, 2000);

        let listings: Vec<&AgentListing> = vec![&a, &b, &c];
        let ranked = FeaturedEngine::rank(&listings, 2);
        assert_eq!(ranked.len(), 2);
        // Highest scorer first
        assert!(ranked[0].score >= ranked[1].score);
    }

    #[test]
    fn curated_management() {
        let mut engine = FeaturedEngine::new();
        let id = Uuid::new_v4();
        engine.add_curated(id);
        assert_eq!(engine.curated_list().len(), 1);
        engine.remove_curated(&id);
        assert!(engine.curated_list().is_empty());
    }
}
