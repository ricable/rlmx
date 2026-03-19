//! Domain types and events for the marketplace bounded context.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export LifeDomain from kernel (source of truth per CLAUDE.md rule 9).
pub use rlmx_kernel::LifeDomain;

/// Device types supported by marketplace agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    Desktop,
    Mobile,
    Tablet,
    Browser,
    Edge,
    Wearable,
}

/// Model tier requirements for agents.
/// Local to marketplace to avoid circular dependency with rlmx-agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelTier {
    Small,
    ClaudeCode,
    Medium,
    Custom,
}

/// Permission declarations for agent listings.
/// Maps conceptually to kernel SyscallPermission but expressed as strings
/// in the marketplace context to allow extensibility.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(pub String);

impl Permission {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Returns true if this permission touches sensitive domains.
    pub fn is_sensitive(&self) -> bool {
        matches!(
            self.0.as_str(),
            "health_data" | "finance_data" | "legal_data"
        )
    }
}

/// Publisher identifier (newtype over Uuid).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PublisherId(pub Uuid);

impl PublisherId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for PublisherId {
    fn default() -> Self {
        Self::new()
    }
}

/// Developer type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeveloperType {
    Individual,
    Organization,
    Celebrity,
}

/// Severity levels for security checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

/// Marketplace domain events (DDD-010).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MarketplaceDomainEvent {
    AgentSubmitted {
        agent_id: Uuid,
        publisher_id: PublisherId,
    },
    ReviewStarted {
        agent_id: Uuid,
        submission_id: Uuid,
    },
    ReviewCompleted {
        agent_id: Uuid,
        status: ReviewStatus,
    },
    AgentPublished {
        agent_id: Uuid,
        domain: LifeDomain,
    },
    AgentInstalled {
        agent_id: Uuid,
        user_id: Uuid,
        device: DeviceType,
    },
    AgentUninstalled {
        agent_id: Uuid,
        user_id: Uuid,
    },
    AgentRated {
        agent_id: Uuid,
        user_id: Uuid,
        rating: f32,
        review: Option<String>,
    },
    AgentSuspended {
        agent_id: Uuid,
        reason: String,
    },
    PayoutProcessed {
        publisher_id: PublisherId,
        amount_cents: u64,
    },
    PackCreated {
        pack_id: Uuid,
        creator: PublisherId,
    },
}

/// Review status for agent submissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InReview,
    AutoPassed,
    FlaggedForHuman,
    Approved,
    Rejected,
}

/// Listing status for agents in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListingStatus {
    Draft,
    InReview,
    Published,
    Suspended,
}

/// Price model for agents.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum AgentPrice {
    Free,
    /// One-time purchase price in cents.
    OneTime(u64),
    /// Monthly subscription price in cents.
    Monthly(u64),
}

impl AgentPrice {
    /// Returns the price in cents (0 for free).
    pub fn amount_cents(&self) -> u64 {
        match self {
            AgentPrice::Free => 0,
            AgentPrice::OneTime(c) | AgentPrice::Monthly(c) => *c,
        }
    }
}

/// Payout method for publisher earnings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayoutMethod {
    StripeConnect(String),
    BankTransfer { routing: String, account: String },
}

/// Revenue split configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevenueSplit {
    pub creator_pct: u8,
    pub platform_pct: u8,
    pub base_dev_pct: u8,
}

impl RevenueSplit {
    /// Standard 70/30 developer/platform split.
    pub fn standard() -> Self {
        Self {
            creator_pct: 70,
            platform_pct: 30,
            base_dev_pct: 0,
        }
    }

    /// Celebrity/influencer 50/30/20 split.
    pub fn celebrity() -> Self {
        Self {
            creator_pct: 50,
            platform_pct: 30,
            base_dev_pct: 20,
        }
    }

    /// Validates that percentages sum to 100.
    pub fn is_valid(&self) -> bool {
        (self.creator_pct as u16 + self.platform_pct as u16 + self.base_dev_pct as u16) == 100
    }
}

/// Celebrity/Influencer agent pack (curated bundle of agents).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPack {
    pub id: Uuid,
    pub name: String,
    pub creator: PublisherId,
    /// Agent IDs included in this pack.
    pub agents: Vec<Uuid>,
    pub price: AgentPrice,
    /// Revenue split: 50/30/20 creator/platform/base-dev for celebrity packs.
    pub revenue_split: RevenueSplit,
    pub description: String,
}

impl AgentPack {
    /// Create a new agent pack with the celebrity revenue split.
    pub fn new(
        name: String,
        creator: PublisherId,
        agents: Vec<Uuid>,
        price: AgentPrice,
        description: String,
    ) -> Self {
        Self::with_split(
            name,
            creator,
            agents,
            price,
            description,
            RevenueSplit::celebrity(),
        )
    }

    /// Create a new agent pack with a custom revenue split.
    pub fn with_split(
        name: String,
        creator: PublisherId,
        agents: Vec<Uuid>,
        price: AgentPrice,
        description: String,
        split: RevenueSplit,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            creator,
            agents,
            price,
            revenue_split: split,
            description,
        }
    }
}

/// Security check types for the review pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCheckType {
    CapabilityMinimality,
    DataFlowVerification,
    FuzzTesting,
    NetworkPolicyCompliance,
    MalwareSignatureScan,
}

/// Result of a single security check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCheck {
    pub check_type: SecurityCheckType,
    pub passed: bool,
    pub details: String,
    pub severity: Severity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revenue_split_standard_is_valid() {
        assert!(RevenueSplit::standard().is_valid());
    }

    #[test]
    fn revenue_split_celebrity_is_valid() {
        assert!(RevenueSplit::celebrity().is_valid());
    }

    #[test]
    fn revenue_split_invalid() {
        let bad = RevenueSplit {
            creator_pct: 50,
            platform_pct: 50,
            base_dev_pct: 50,
        };
        assert!(!bad.is_valid());
    }

    #[test]
    fn agent_price_amount() {
        assert_eq!(AgentPrice::Free.amount_cents(), 0);
        assert_eq!(AgentPrice::OneTime(999).amount_cents(), 999);
        assert_eq!(AgentPrice::Monthly(499).amount_cents(), 499);
    }

    #[test]
    fn agent_pack_default_celebrity_split() {
        let pack = AgentPack::new(
            "CoolPack".into(),
            PublisherId::new(),
            vec![Uuid::new_v4(), Uuid::new_v4()],
            AgentPrice::OneTime(1999),
            "A curated pack".into(),
        );
        assert!(pack.revenue_split.is_valid());
        assert_eq!(pack.revenue_split.creator_pct, 50);
        assert_eq!(pack.revenue_split.platform_pct, 30);
        assert_eq!(pack.revenue_split.base_dev_pct, 20);
        assert_eq!(pack.agents.len(), 2);
    }

    #[test]
    fn agent_pack_custom_split() {
        let split = RevenueSplit {
            creator_pct: 60,
            platform_pct: 30,
            base_dev_pct: 10,
        };
        let pack = AgentPack::with_split(
            "CustomPack".into(),
            PublisherId::new(),
            vec![Uuid::new_v4()],
            AgentPrice::Monthly(499),
            "Custom split pack".into(),
            split,
        );
        assert!(pack.revenue_split.is_valid());
        assert_eq!(pack.revenue_split.creator_pct, 60);
    }

    #[test]
    fn sensitive_permissions() {
        assert!(Permission::new("health_data").is_sensitive());
        assert!(Permission::new("finance_data").is_sensitive());
        assert!(Permission::new("legal_data").is_sensitive());
        assert!(!Permission::new("vec_search").is_sensitive());
    }
}
