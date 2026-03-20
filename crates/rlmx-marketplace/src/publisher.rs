//! Publisher portal: developer registration, management, and earnings tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{DeveloperType, PublisherId};
use crate::error::MarketplaceError;

/// A registered publisher in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publisher {
    pub id: PublisherId,
    pub name: String,
    pub email: String,
    pub developer_type: DeveloperType,
    pub verified: bool,
    pub agents: Vec<Uuid>,
    pub reputation_score: f32,
    pub joined_at: DateTime<Utc>,
}

impl Publisher {
    /// Create a new unverified publisher.
    pub fn new(name: String, email: String, developer_type: DeveloperType) -> Self {
        Self {
            id: PublisherId::new(),
            name,
            email,
            developer_type,
            verified: false,
            agents: Vec::new(),
            reputation_score: 0.0,
            joined_at: Utc::now(),
        }
    }

    /// Add an agent to this publisher's portfolio.
    pub fn add_agent(&mut self, agent_id: Uuid) {
        if !self.agents.contains(&agent_id) {
            self.agents.push(agent_id);
        }
    }

    /// Remove an agent from this publisher's portfolio.
    pub fn remove_agent(&mut self, agent_id: &Uuid) {
        self.agents.retain(|a| a != agent_id);
    }

    /// Update reputation score based on marketplace-internal signals.
    pub fn update_reputation(&mut self, avg_rating: f32, review_pass_rate: f32) {
        // Weighted average: 60% ratings, 40% review pass rate
        self.reputation_score = avg_rating * 0.6 + (review_pass_rate * 5.0) * 0.4;
    }
}

/// The publisher portal manages all publisher accounts.
#[derive(Debug, Default)]
pub struct PublisherPortal {
    publishers: HashMap<PublisherId, Publisher>,
    email_index: HashMap<String, PublisherId>,
}

impl PublisherPortal {
    pub fn new() -> Self {
        Self {
            publishers: HashMap::new(),
            email_index: HashMap::new(),
        }
    }

    /// Register a new publisher.
    pub fn register(
        &mut self,
        name: String,
        email: String,
        developer_type: DeveloperType,
    ) -> Result<PublisherId, MarketplaceError> {
        if self.email_index.contains_key(&email) {
            return Err(MarketplaceError::DuplicateEmail(email));
        }

        let publisher = Publisher::new(name.clone(), email.clone(), developer_type);
        let id = publisher.id;

        tracing::info!(publisher_id = ?id, name = %name, "publisher registered");
        self.email_index.insert(email, id);
        self.publishers.insert(id, publisher);

        Ok(id)
    }

    /// Get a publisher by ID.
    pub fn get(&self, id: &PublisherId) -> Option<&Publisher> {
        self.publishers.get(id)
    }

    /// Get a mutable publisher by ID.
    pub fn get_mut(&mut self, id: &PublisherId) -> Option<&mut Publisher> {
        self.publishers.get_mut(id)
    }

    /// Verify a publisher.
    pub fn verify(&mut self, id: &PublisherId) -> Result<(), MarketplaceError> {
        let publisher = self
            .publishers
            .get_mut(id)
            .ok_or(MarketplaceError::PublisherNotFound(*id))?;
        publisher.verified = true;
        tracing::info!(publisher_id = ?id, "publisher verified");
        Ok(())
    }

    /// Look up a publisher by email.
    pub fn by_email(&self, email: &str) -> Option<&Publisher> {
        self.email_index
            .get(email)
            .and_then(|id| self.publishers.get(id))
    }

    /// Total number of publishers.
    pub fn count(&self) -> usize {
        self.publishers.len()
    }

    /// List all verified publishers.
    pub fn verified_publishers(&self) -> Vec<&Publisher> {
        self.publishers.values().filter(|p| p.verified).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_lookup() {
        let mut portal = PublisherPortal::new();
        let id = portal
            .register(
                "Acme".into(),
                "dev@acme.com".into(),
                DeveloperType::Organization,
            )
            .unwrap();

        let pub_ = portal.get(&id).unwrap();
        assert_eq!(pub_.name, "Acme");
        assert!(!pub_.verified);
    }

    #[test]
    fn duplicate_email_rejected() {
        let mut portal = PublisherPortal::new();
        portal
            .register("A".into(), "a@test.com".into(), DeveloperType::Individual)
            .unwrap();

        let result = portal.register("B".into(), "a@test.com".into(), DeveloperType::Individual);
        assert!(result.is_err());
    }

    #[test]
    fn verify_publisher() {
        let mut portal = PublisherPortal::new();
        let id = portal
            .register(
                "Dev".into(),
                "dev@test.com".into(),
                DeveloperType::Individual,
            )
            .unwrap();

        portal.verify(&id).unwrap();
        assert!(portal.get(&id).unwrap().verified);
    }

    #[test]
    fn lookup_by_email() {
        let mut portal = PublisherPortal::new();
        portal
            .register(
                "EmailDev".into(),
                "find@me.com".into(),
                DeveloperType::Individual,
            )
            .unwrap();

        let found = portal.by_email("find@me.com");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "EmailDev");
    }

    #[test]
    fn reputation_update() {
        let mut pub_ = Publisher::new(
            "TestDev".into(),
            "test@dev.com".into(),
            DeveloperType::Individual,
        );
        pub_.update_reputation(4.0, 0.9);
        // 4.0 * 0.6 + (0.9 * 5.0) * 0.4 = 2.4 + 1.8 = 4.2
        assert!((pub_.reputation_score - 4.2).abs() < 0.01);
    }
}
