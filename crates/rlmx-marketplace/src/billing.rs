//! Billing engine: revenue splitting, payout tracking, transaction records.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::domain::{AgentPrice, PayoutMethod, PublisherId, RevenueSplit};
use crate::error::MarketplaceError;

/// Default payout threshold in cents ($50).
const DEFAULT_PAYOUT_THRESHOLD_CENTS: u64 = 5000;

/// Developer revenue share percentage.
const DEVELOPER_SHARE_PCT: u64 = 70;

/// A single earning transaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningTransaction {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub user_id: Uuid,
    pub gross_cents: u64,
    pub developer_cents: u64,
    pub platform_cents: u64,
    pub timestamp: DateTime<Utc>,
}

/// Earnings account for a publisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarningsAccount {
    pub publisher_id: PublisherId,
    pub balance_cents: u64,
    pub lifetime_earned_cents: u64,
    pub payout_method: Option<PayoutMethod>,
    pub payout_threshold_cents: u64,
    pub transactions: Vec<EarningTransaction>,
}

impl EarningsAccount {
    pub fn new(publisher_id: PublisherId) -> Self {
        Self {
            publisher_id,
            balance_cents: 0,
            lifetime_earned_cents: 0,
            payout_method: None,
            payout_threshold_cents: DEFAULT_PAYOUT_THRESHOLD_CENTS,
            transactions: Vec::new(),
        }
    }

    /// Credit earnings from an agent sale.
    pub fn credit(&mut self, developer_cents: u64, transaction: EarningTransaction) {
        self.balance_cents += developer_cents;
        self.lifetime_earned_cents += developer_cents;
        self.transactions.push(transaction);
    }

    /// Returns true if balance meets payout threshold.
    pub fn eligible_for_payout(&self) -> bool {
        self.balance_cents >= self.payout_threshold_cents && self.payout_method.is_some()
    }

    /// Process a payout, zeroing the balance. Returns the payout amount.
    pub fn process_payout(&mut self) -> u64 {
        let amount = self.balance_cents;
        self.balance_cents = 0;
        tracing::info!(
            publisher_id = ?self.publisher_id,
            amount_cents = amount,
            "payout processed"
        );
        amount
    }
}

/// A payout record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutRecord {
    pub id: Uuid,
    pub publisher_id: PublisherId,
    pub amount_cents: u64,
    pub processed_at: DateTime<Utc>,
}

/// The billing engine manages revenue splitting and payouts.
#[derive(Debug, Default)]
pub struct BillingEngine {
    accounts: HashMap<PublisherId, EarningsAccount>,
    platform_revenue_cents: u64,
    payouts: Vec<PayoutRecord>,
}

impl BillingEngine {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            platform_revenue_cents: 0,
            payouts: Vec::new(),
        }
    }

    /// Ensure an earnings account exists for a publisher.
    pub fn ensure_account(&mut self, publisher_id: PublisherId) {
        self.accounts
            .entry(publisher_id)
            .or_insert_with(|| EarningsAccount::new(publisher_id));
    }

    /// Set payout method for a publisher.
    pub fn set_payout_method(
        &mut self,
        publisher_id: &PublisherId,
        method: PayoutMethod,
    ) -> Result<(), MarketplaceError> {
        let account = self
            .accounts
            .get_mut(publisher_id)
            .ok_or(MarketplaceError::PublisherNotFound(*publisher_id))?;
        account.payout_method = Some(method);
        Ok(())
    }

    /// Process a sale with standard 70/30 revenue split.
    pub fn record_sale(
        &mut self,
        publisher_id: &PublisherId,
        agent_id: Uuid,
        user_id: Uuid,
        price: &AgentPrice,
    ) -> Result<Option<EarningTransaction>, MarketplaceError> {
        let gross = price.amount_cents();
        if gross == 0 {
            return Ok(None);
        }

        let developer_cents = gross * DEVELOPER_SHARE_PCT / 100;
        let platform_cents = gross - developer_cents;

        let tx = EarningTransaction {
            id: Uuid::new_v4(),
            agent_id,
            user_id,
            gross_cents: gross,
            developer_cents,
            platform_cents,
            timestamp: Utc::now(),
        };

        let account = self
            .accounts
            .get_mut(publisher_id)
            .ok_or(MarketplaceError::PublisherNotFound(*publisher_id))?;

        account.credit(developer_cents, tx.clone());
        self.platform_revenue_cents += platform_cents;

        tracing::info!(
            agent_id = %agent_id,
            gross = gross,
            dev_share = developer_cents,
            platform_share = platform_cents,
            "sale recorded"
        );

        Ok(Some(tx))
    }

    /// Process a sale with a custom revenue split (for agent packs).
    pub fn record_split_sale(
        &mut self,
        creator_id: &PublisherId,
        base_dev_id: Option<&PublisherId>,
        agent_id: Uuid,
        user_id: Uuid,
        gross_cents: u64,
        split: &RevenueSplit,
    ) -> Result<(), MarketplaceError> {
        if gross_cents == 0 {
            return Ok(());
        }

        let creator_cents = gross_cents * split.creator_pct as u64 / 100;
        let platform_cents = gross_cents * split.platform_pct as u64 / 100;
        let base_dev_cents = gross_cents - creator_cents - platform_cents;

        // Credit creator
        let creator_tx = EarningTransaction {
            id: Uuid::new_v4(),
            agent_id,
            user_id,
            gross_cents,
            developer_cents: creator_cents,
            platform_cents,
            timestamp: Utc::now(),
        };
        let account = self
            .accounts
            .get_mut(creator_id)
            .ok_or(MarketplaceError::PublisherNotFound(*creator_id))?;
        account.credit(creator_cents, creator_tx);

        // Credit base developer if applicable
        if let Some(dev_id) = base_dev_id {
            if base_dev_cents > 0 {
                let dev_tx = EarningTransaction {
                    id: Uuid::new_v4(),
                    agent_id,
                    user_id,
                    gross_cents,
                    developer_cents: base_dev_cents,
                    platform_cents: 0,
                    timestamp: Utc::now(),
                };
                let dev_account = self
                    .accounts
                    .get_mut(dev_id)
                    .ok_or(MarketplaceError::PublisherNotFound(*dev_id))?;
                dev_account.credit(base_dev_cents, dev_tx);
            }
        }

        self.platform_revenue_cents += platform_cents;
        Ok(())
    }

    /// Run monthly payout cycle. Returns list of processed payouts.
    pub fn run_payout_cycle(&mut self) -> Vec<PayoutRecord> {
        let mut payouts = Vec::new();

        let eligible: Vec<PublisherId> = self
            .accounts
            .values()
            .filter(|a| a.eligible_for_payout())
            .map(|a| a.publisher_id)
            .collect();

        for pid in eligible {
            if let Some(account) = self.accounts.get_mut(&pid) {
                let amount = account.process_payout();
                let record = PayoutRecord {
                    id: Uuid::new_v4(),
                    publisher_id: pid,
                    amount_cents: amount,
                    processed_at: Utc::now(),
                };
                payouts.push(record.clone());
                self.payouts.push(record);
            }
        }

        payouts
    }

    /// Get earnings account for a publisher.
    pub fn get_account(&self, publisher_id: &PublisherId) -> Option<&EarningsAccount> {
        self.accounts.get(publisher_id)
    }

    /// Total platform revenue collected.
    pub fn platform_revenue(&self) -> u64 {
        self.platform_revenue_cents
    }

    /// All payout records.
    pub fn payout_history(&self) -> &[PayoutRecord] {
        &self.payouts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_revenue_split() {
        let mut engine = BillingEngine::new();
        let pid = PublisherId::new();
        engine.ensure_account(pid);

        let tx = engine
            .record_sale(&pid, Uuid::new_v4(), Uuid::new_v4(), &AgentPrice::OneTime(1000))
            .unwrap()
            .unwrap();

        assert_eq!(tx.developer_cents, 700);
        assert_eq!(tx.platform_cents, 300);
        assert_eq!(engine.get_account(&pid).unwrap().balance_cents, 700);
        assert_eq!(engine.platform_revenue(), 300);
    }

    #[test]
    fn free_agent_no_transaction() {
        let mut engine = BillingEngine::new();
        let pid = PublisherId::new();
        engine.ensure_account(pid);

        let result = engine
            .record_sale(&pid, Uuid::new_v4(), Uuid::new_v4(), &AgentPrice::Free)
            .unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn payout_threshold_enforced() {
        let mut engine = BillingEngine::new();
        let pid = PublisherId::new();
        engine.ensure_account(pid);
        engine
            .set_payout_method(&pid, PayoutMethod::StripeConnect("acct_123".into()))
            .unwrap();

        // Below threshold: $49 gross -> $34.30 dev share
        engine
            .record_sale(&pid, Uuid::new_v4(), Uuid::new_v4(), &AgentPrice::OneTime(4900))
            .unwrap();

        let payouts = engine.run_payout_cycle();
        assert!(payouts.is_empty()); // Below $50 threshold

        // Push above threshold
        engine
            .record_sale(&pid, Uuid::new_v4(), Uuid::new_v4(), &AgentPrice::OneTime(3000))
            .unwrap();

        let payouts = engine.run_payout_cycle();
        assert_eq!(payouts.len(), 1);
        assert_eq!(engine.get_account(&pid).unwrap().balance_cents, 0);
    }

    #[test]
    fn no_payout_without_method() {
        let mut engine = BillingEngine::new();
        let pid = PublisherId::new();
        engine.ensure_account(pid);

        // Add enough balance but no payout method
        engine
            .record_sale(&pid, Uuid::new_v4(), Uuid::new_v4(), &AgentPrice::OneTime(10000))
            .unwrap();

        let payouts = engine.run_payout_cycle();
        assert!(payouts.is_empty());
    }
}
