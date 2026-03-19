//! Developer accounts: revenue sharing, payout tracking, marketplace publishing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::{BillingError, BillingResult};

/// Developer revenue share percentage (developer gets 70%).
const DEVELOPER_SHARE_PCT: u64 = 70;

/// Default payout threshold in cents ($50).
const DEFAULT_PAYOUT_THRESHOLD_CENTS: u64 = 5000;

/// Payout method for developer earnings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayoutMethod {
    StripeConnect(String),
    BankTransfer { routing: String, account: String },
}

/// A record of a single agent sale.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaleRecord {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub buyer_id: Uuid,
    pub gross_cents: u64,
    pub developer_cents: u64,
    pub platform_cents: u64,
    pub timestamp: DateTime<Utc>,
}

/// A payout record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRecord {
    pub id: Uuid,
    pub amount_cents: u64,
    pub processed_at: DateTime<Utc>,
}

/// A developer account with earnings tracking and marketplace publishing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperAccount {
    pub id: Uuid,
    pub publisher_name: String,
    pub payout_method: Option<PayoutMethod>,
    pub balance_cents: u64,
    pub lifetime_earned_cents: u64,
    pub payout_threshold_cents: u64,
    pub published_agents: Vec<Uuid>,
    pub sales: Vec<SaleRecord>,
    pub payouts: Vec<PayoutRecord>,
    pub created_at: DateTime<Utc>,
}

impl DeveloperAccount {
    /// Create a new developer account.
    pub fn new(publisher_name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            publisher_name: publisher_name.into(),
            payout_method: None,
            balance_cents: 0,
            lifetime_earned_cents: 0,
            payout_threshold_cents: DEFAULT_PAYOUT_THRESHOLD_CENTS,
            published_agents: Vec::new(),
            sales: Vec::new(),
            payouts: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Set the payout method.
    pub fn set_payout_method(&mut self, method: PayoutMethod) {
        self.payout_method = Some(method);
    }

    /// Record a sale with the standard 70/30 revenue split.
    pub fn record_sale(&mut self, agent_id: Uuid, buyer_id: Uuid, gross_cents: u64) -> SaleRecord {
        let developer_cents = gross_cents * DEVELOPER_SHARE_PCT / 100;
        let platform_cents = gross_cents - developer_cents;

        let record = SaleRecord {
            id: Uuid::new_v4(),
            agent_id,
            buyer_id,
            gross_cents,
            developer_cents,
            platform_cents,
            timestamp: Utc::now(),
        };

        self.balance_cents += developer_cents;
        self.lifetime_earned_cents += developer_cents;
        self.sales.push(record.clone());

        tracing::info!(
            developer = %self.id,
            agent = %agent_id,
            gross = gross_cents,
            dev_share = developer_cents,
            "sale recorded"
        );

        record
    }

    /// Calculate the pending payout amount.
    pub fn calculate_payout(&self) -> u64 {
        self.balance_cents
    }

    /// Request a payout. Returns the payout record if eligible.
    pub fn request_payout(&mut self) -> BillingResult<PayoutRecord> {
        if self.payout_method.is_none() {
            return Err(BillingError::NoPayoutMethod);
        }
        if self.balance_cents < self.payout_threshold_cents {
            return Err(BillingError::PayoutThresholdNotMet {
                balance_cents: self.balance_cents,
                threshold_cents: self.payout_threshold_cents,
            });
        }

        let amount = self.balance_cents;
        self.balance_cents = 0;

        let record = PayoutRecord {
            id: Uuid::new_v4(),
            amount_cents: amount,
            processed_at: Utc::now(),
        };
        self.payouts.push(record.clone());

        tracing::info!(
            developer = %self.id,
            amount_cents = amount,
            "payout processed"
        );

        Ok(record)
    }

    /// Register a published agent.
    pub fn publish_agent(&mut self, agent_id: Uuid) {
        if !self.published_agents.contains(&agent_id) {
            self.published_agents.push(agent_id);
        }
    }

    /// Total number of sales.
    pub fn total_sales(&self) -> usize {
        self.sales.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_developer_account() {
        let dev = DeveloperAccount::new("TestDev");
        assert_eq!(dev.publisher_name, "TestDev");
        assert_eq!(dev.balance_cents, 0);
        assert_eq!(dev.payout_threshold_cents, 5000);
    }

    #[test]
    fn record_sale_70_30_split() {
        let mut dev = DeveloperAccount::new("TestDev");
        let sale = dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 1000);
        assert_eq!(sale.developer_cents, 700);
        assert_eq!(sale.platform_cents, 300);
        assert_eq!(dev.balance_cents, 700);
        assert_eq!(dev.lifetime_earned_cents, 700);
    }

    #[test]
    fn payout_threshold_enforced() {
        let mut dev = DeveloperAccount::new("TestDev");
        dev.set_payout_method(PayoutMethod::StripeConnect("acct_123".into()));

        // $49 gross -> $34.30 dev share, below $50 threshold
        dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 4900);
        assert!(dev.request_payout().is_err());

        // Add more to exceed threshold
        dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 3000);
        let payout = dev.request_payout().unwrap();
        assert!(payout.amount_cents > 0);
        assert_eq!(dev.balance_cents, 0);
    }

    #[test]
    fn payout_requires_method() {
        let mut dev = DeveloperAccount::new("TestDev");
        dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 100_000);
        assert!(dev.request_payout().is_err());
    }

    #[test]
    fn publish_agent_idempotent() {
        let mut dev = DeveloperAccount::new("TestDev");
        let agent = Uuid::new_v4();
        dev.publish_agent(agent);
        dev.publish_agent(agent);
        assert_eq!(dev.published_agents.len(), 1);
    }

    #[test]
    fn multiple_sales_accumulate() {
        let mut dev = DeveloperAccount::new("TestDev");
        dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 1000);
        dev.record_sale(Uuid::new_v4(), Uuid::new_v4(), 2000);
        assert_eq!(dev.balance_cents, 2100); // 700 + 1400
        assert_eq!(dev.total_sales(), 2);
    }
}
