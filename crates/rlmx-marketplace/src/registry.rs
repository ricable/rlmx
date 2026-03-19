//! Agent registry: catalog of all marketplace agents with CRUD and search.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{
    AgentPrice, DeviceType, LifeDomain, ListingStatus, ModelTier, Permission, PublisherId,
};
use crate::error::MarketplaceError;

/// A published agent listing in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListing {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub domain: LifeDomain,
    pub publisher: PublisherId,
    pub rvf_hash: [u8; 32],
    pub version: semver::Version,
    pub price: AgentPrice,
    pub rating: f32,
    pub rating_count: u64,
    pub install_count: u64,
    pub permissions_required: Vec<Permission>,
    pub supported_devices: Vec<DeviceType>,
    pub min_model_tier: ModelTier,
    pub size_bytes: u64,
    pub status: ListingStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AgentListing {
    /// Create a new draft listing.
    #[allow(clippy::too_many_arguments)]
    pub fn new_draft(
        name: String,
        description: String,
        domain: LifeDomain,
        publisher: PublisherId,
        rvf_hash: [u8; 32],
        version: semver::Version,
        price: AgentPrice,
        permissions: Vec<Permission>,
        devices: Vec<DeviceType>,
        min_model_tier: ModelTier,
        size_bytes: u64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            domain,
            publisher,
            rvf_hash,
            version,
            price,
            rating: 0.0,
            rating_count: 0,
            install_count: 0,
            permissions_required: permissions,
            supported_devices: devices,
            min_model_tier,
            size_bytes,
            status: ListingStatus::Draft,
            published_at: None,
            created_at: Utc::now(),
        }
    }

    /// Returns true if any required permission touches sensitive domains.
    pub fn requires_sensitive_permissions(&self) -> bool {
        self.permissions_required.iter().any(|p| p.is_sensitive())
    }

    /// Record a user rating and update the aggregate.
    pub fn add_rating(&mut self, score: f32) {
        let total = self.rating * self.rating_count as f32 + score;
        self.rating_count += 1;
        self.rating = total / self.rating_count as f32;
    }

    /// Increment install count.
    pub fn record_install(&mut self) {
        self.install_count += 1;
    }
}

/// Search/filter criteria for querying the registry.
#[derive(Debug, Clone, Default)]
pub struct ListingFilter {
    pub domain: Option<LifeDomain>,
    pub keyword: Option<String>,
    pub min_rating: Option<f32>,
    pub max_price_cents: Option<u64>,
    pub status: Option<ListingStatus>,
    pub publisher: Option<PublisherId>,
    pub device: Option<DeviceType>,
}

/// Sort order for listing search results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListingSort {
    Rating,
    Installs,
    Newest,
    PriceLow,
    PriceHigh,
}

/// The agent registry: in-memory catalog of all agent listings.
#[derive(Debug, Default)]
pub struct AgentRegistry {
    listings: HashMap<Uuid, AgentListing>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            listings: HashMap::new(),
        }
    }

    /// Insert a new listing. Returns the listing ID.
    pub fn insert(&mut self, listing: AgentListing) -> Uuid {
        let id = listing.id;
        tracing::info!(listing_id = %id, name = %listing.name, "listing inserted");
        self.listings.insert(id, listing);
        id
    }

    /// Get a listing by ID.
    pub fn get(&self, id: &Uuid) -> Option<&AgentListing> {
        self.listings.get(id)
    }

    /// Get a mutable listing by ID.
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut AgentListing> {
        self.listings.get_mut(id)
    }

    /// Remove a listing.
    pub fn remove(&mut self, id: &Uuid) -> Option<AgentListing> {
        tracing::info!(listing_id = %id, "listing removed");
        self.listings.remove(id)
    }

    /// Update the status of a listing.
    pub fn update_status(
        &mut self,
        id: &Uuid,
        status: ListingStatus,
    ) -> Result<(), MarketplaceError> {
        let listing = self
            .listings
            .get_mut(id)
            .ok_or(MarketplaceError::ListingNotFound(*id))?;
        tracing::info!(listing_id = %id, old = ?listing.status, new = ?status, "status updated");
        listing.status = status;
        if status == ListingStatus::Published && listing.published_at.is_none() {
            listing.published_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Search listings with filters and sorting.
    pub fn search(&self, filter: &ListingFilter, sort: ListingSort) -> Vec<&AgentListing> {
        let mut results: Vec<&AgentListing> = self
            .listings
            .values()
            .filter(|l| {
                if let Some(d) = &filter.domain {
                    if l.domain != *d {
                        return false;
                    }
                }
                if let Some(kw) = &filter.keyword {
                    let kw_lower = kw.to_lowercase();
                    if !l.name.to_lowercase().contains(&kw_lower)
                        && !l.description.to_lowercase().contains(&kw_lower)
                    {
                        return false;
                    }
                }
                if let Some(min_r) = filter.min_rating {
                    if l.rating < min_r {
                        return false;
                    }
                }
                if let Some(max_p) = filter.max_price_cents {
                    if l.price.amount_cents() > max_p {
                        return false;
                    }
                }
                if let Some(s) = &filter.status {
                    if l.status != *s {
                        return false;
                    }
                }
                if let Some(p) = &filter.publisher {
                    if l.publisher != *p {
                        return false;
                    }
                }
                if let Some(d) = &filter.device {
                    if !l.supported_devices.contains(d) {
                        return false;
                    }
                }
                true
            })
            .collect();

        match sort {
            ListingSort::Rating => results.sort_by(|a, b| {
                b.rating
                    .partial_cmp(&a.rating)
                    .unwrap_or(std::cmp::Ordering::Equal)
            }),
            ListingSort::Installs => results.sort_by(|a, b| b.install_count.cmp(&a.install_count)),
            ListingSort::Newest => results.sort_by(|a, b| b.created_at.cmp(&a.created_at)),
            ListingSort::PriceLow => {
                results.sort_by(|a, b| a.price.amount_cents().cmp(&b.price.amount_cents()))
            }
            ListingSort::PriceHigh => {
                results.sort_by(|a, b| b.price.amount_cents().cmp(&a.price.amount_cents()))
            }
        }

        results
    }

    /// Total number of listings.
    pub fn count(&self) -> usize {
        self.listings.len()
    }

    /// All listings for a given publisher.
    pub fn by_publisher(&self, publisher_id: &PublisherId) -> Vec<&AgentListing> {
        self.listings
            .values()
            .filter(|l| l.publisher == *publisher_id)
            .collect()
    }

    /// Auto-suspend agents with rating below 2.0 after 50+ ratings (invariant 6).
    pub fn enforce_rating_policy(&mut self) -> Vec<Uuid> {
        let to_suspend: Vec<Uuid> = self
            .listings
            .values()
            .filter(|l| {
                l.status == ListingStatus::Published && l.rating_count >= 50 && l.rating < 2.0
            })
            .map(|l| l.id)
            .collect();

        for id in &to_suspend {
            if let Some(listing) = self.listings.get_mut(id) {
                listing.status = ListingStatus::Suspended;
                tracing::warn!(listing_id = %id, rating = listing.rating, "auto-suspended due to low rating");
            }
        }

        to_suspend
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_listing(name: &str, domain: LifeDomain, price: AgentPrice) -> AgentListing {
        AgentListing::new_draft(
            name.to_string(),
            format!("{name} description"),
            domain,
            PublisherId::new(),
            [0u8; 32],
            semver::Version::new(1, 0, 0),
            price,
            vec![],
            vec![DeviceType::Desktop],
            ModelTier::Small,
            1024,
        )
    }

    #[test]
    fn insert_and_get() {
        let mut reg = AgentRegistry::new();
        let listing = make_listing("TestAgent", LifeDomain::Finance, AgentPrice::Free);
        let id = listing.id;
        reg.insert(listing);
        assert!(reg.get(&id).is_some());
        assert_eq!(reg.count(), 1);
    }

    #[test]
    fn remove_listing() {
        let mut reg = AgentRegistry::new();
        let listing = make_listing("ToRemove", LifeDomain::Health, AgentPrice::Free);
        let id = listing.id;
        reg.insert(listing);
        assert!(reg.remove(&id).is_some());
        assert_eq!(reg.count(), 0);
    }

    #[test]
    fn search_by_domain() {
        let mut reg = AgentRegistry::new();
        reg.insert(make_listing(
            "FinAgent",
            LifeDomain::Finance,
            AgentPrice::Free,
        ));
        reg.insert(make_listing(
            "HealthAgent",
            LifeDomain::Health,
            AgentPrice::Free,
        ));

        let filter = ListingFilter {
            domain: Some(LifeDomain::Finance),
            ..Default::default()
        };
        let results = reg.search(&filter, ListingSort::Rating);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].domain, LifeDomain::Finance);
    }

    #[test]
    fn search_by_keyword() {
        let mut reg = AgentRegistry::new();
        reg.insert(make_listing(
            "BudgetTracker",
            LifeDomain::Finance,
            AgentPrice::Free,
        ));
        reg.insert(make_listing(
            "FitnessCoach",
            LifeDomain::Health,
            AgentPrice::Free,
        ));

        let filter = ListingFilter {
            keyword: Some("budget".to_string()),
            ..Default::default()
        };
        let results = reg.search(&filter, ListingSort::Rating);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_by_max_price() {
        let mut reg = AgentRegistry::new();
        reg.insert(make_listing(
            "Cheap",
            LifeDomain::Finance,
            AgentPrice::OneTime(500),
        ));
        reg.insert(make_listing(
            "Expensive",
            LifeDomain::Finance,
            AgentPrice::OneTime(5000),
        ));

        let filter = ListingFilter {
            max_price_cents: Some(1000),
            ..Default::default()
        };
        let results = reg.search(&filter, ListingSort::PriceLow);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "Cheap");
    }

    #[test]
    fn rating_aggregation() {
        let mut listing = make_listing("Rated", LifeDomain::Pet, AgentPrice::Free);
        listing.add_rating(4.0);
        listing.add_rating(5.0);
        assert!((listing.rating - 4.5).abs() < 0.01);
        assert_eq!(listing.rating_count, 2);
    }

    #[test]
    fn enforce_rating_policy_suspends_low_rated() {
        let mut reg = AgentRegistry::new();
        let mut listing = make_listing("BadAgent", LifeDomain::Social, AgentPrice::Free);
        listing.status = ListingStatus::Published;
        listing.rating = 1.5;
        listing.rating_count = 60;
        let id = listing.id;
        reg.insert(listing);

        let suspended = reg.enforce_rating_policy();
        assert_eq!(suspended.len(), 1);
        assert_eq!(suspended[0], id);
        assert_eq!(reg.get(&id).unwrap().status, ListingStatus::Suspended);
    }

    #[test]
    fn update_status_sets_published_at() {
        let mut reg = AgentRegistry::new();
        let listing = make_listing("PubTest", LifeDomain::Career, AgentPrice::Free);
        let id = listing.id;
        reg.insert(listing);

        reg.update_status(&id, ListingStatus::Published).unwrap();
        assert!(reg.get(&id).unwrap().published_at.is_some());
    }
}
