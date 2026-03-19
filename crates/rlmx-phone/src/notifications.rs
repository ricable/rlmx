use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

/// 3-tier notification priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationPriority {
    /// Strong vibration + alert tone (security threats, fraud).
    Critical,
    /// Gentle vibration + chime (savings found, price drops).
    Actionable,
    /// Silent, batched into daily briefing.
    Informational,
}

/// A notification pending delivery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub priority: NotificationPriority,
    pub domain: String,
    pub created_at: DateTime<Utc>,
}

/// Record of how the user responded to a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notification_id: Uuid,
    pub priority: NotificationPriority,
    pub responded: bool,
    pub response_time_ms: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

/// Daily notification budget to prevent spam.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationBudget {
    pub max_critical: u16,
    pub max_actionable: u16,
    pub max_informational: u16,
    pub sent_critical: u16,
    pub sent_actionable: u16,
    pub sent_informational: u16,
    pub reset_at: DateTime<Utc>,
}

impl NotificationBudget {
    pub fn new() -> Self {
        Self {
            max_critical: 10,
            max_actionable: 20,
            max_informational: 50,
            sent_critical: 0,
            sent_actionable: 0,
            sent_informational: 0,
            reset_at: Utc::now(),
        }
    }

    /// Check if the budget allows sending a notification of the given priority.
    pub fn allows(&self, priority: NotificationPriority) -> bool {
        match priority {
            NotificationPriority::Critical => self.sent_critical < self.max_critical,
            NotificationPriority::Actionable => self.sent_actionable < self.max_actionable,
            NotificationPriority::Informational => self.sent_informational < self.max_informational,
        }
    }

    /// Record that a notification was sent.
    pub fn record_sent(&mut self, priority: NotificationPriority) {
        match priority {
            NotificationPriority::Critical => self.sent_critical += 1,
            NotificationPriority::Actionable => self.sent_actionable += 1,
            NotificationPriority::Informational => self.sent_informational += 1,
        }
    }

    /// Reset the daily budget counters.
    pub fn reset(&mut self) {
        self.sent_critical = 0;
        self.sent_actionable = 0;
        self.sent_informational = 0;
        self.reset_at = Utc::now();
    }
}

impl Default for NotificationBudget {
    fn default() -> Self {
        Self::new()
    }
}

/// ML-based fatigue model (stub) that tracks user engagement patterns.
///
/// Invariant 3: if the user ignores 5+ consecutive Actionable notifications,
/// auto-downgrade future Actionable to Informational.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FatigueModel {
    /// Consecutive ignored Actionable notifications.
    pub consecutive_ignored: u32,
    /// Threshold before auto-downgrade kicks in.
    pub downgrade_threshold: u32,
    /// Whether Actionable notifications are currently downgraded.
    pub actionable_downgraded: bool,
}

impl FatigueModel {
    pub fn new() -> Self {
        Self {
            consecutive_ignored: 0,
            downgrade_threshold: 5,
            actionable_downgraded: false,
        }
    }

    /// Record a user response (or lack thereof) to an Actionable notification.
    pub fn record_response(&mut self, responded: bool) {
        if responded {
            self.consecutive_ignored = 0;
            self.actionable_downgraded = false;
        } else {
            self.consecutive_ignored += 1;
            if self.consecutive_ignored >= self.downgrade_threshold {
                self.actionable_downgraded = true;
                tracing::info!(
                    consecutive = self.consecutive_ignored,
                    "actionable notifications downgraded due to fatigue"
                );
            }
        }
    }

    /// Evaluate whether a notification should be suppressed or downgraded.
    pub fn evaluate(&self, priority: NotificationPriority) -> NotificationPriority {
        if priority == NotificationPriority::Actionable && self.actionable_downgraded {
            return NotificationPriority::Informational;
        }
        priority
    }
}

impl Default for FatigueModel {
    fn default() -> Self {
        Self::new()
    }
}

/// Outcome of sending a notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotificationOutcome {
    Sent,
    Suppressed {
        reason: String,
    },
    Downgraded {
        from: NotificationPriority,
        to: NotificationPriority,
    },
    BudgetExhausted,
}

/// Orchestrates notification delivery with fatigue prevention and budgeting.
#[derive(Debug)]
pub struct NotificationOrchestrator {
    pub pending: VecDeque<Notification>,
    pub fatigue_model: FatigueModel,
    pub daily_budget: NotificationBudget,
    pub user_response_history: Vec<NotificationResponse>,
}

impl NotificationOrchestrator {
    pub fn new() -> Self {
        Self {
            pending: VecDeque::new(),
            fatigue_model: FatigueModel::new(),
            daily_budget: NotificationBudget::new(),
            user_response_history: Vec::new(),
        }
    }

    /// Enqueue a notification for evaluation and delivery.
    pub fn enqueue(&mut self, notification: Notification) {
        self.pending.push_back(notification);
    }

    /// Process the next pending notification.
    /// Returns the outcome and the effective priority used.
    pub fn process_next(&mut self) -> Option<(Notification, NotificationOutcome)> {
        let notification = self.pending.pop_front()?;

        // Apply fatigue model.
        let effective_priority = self.fatigue_model.evaluate(notification.priority);

        // Check budget.
        if !self.daily_budget.allows(effective_priority) {
            tracing::debug!(
                notification_id = %notification.id,
                "notification budget exhausted"
            );
            return Some((notification, NotificationOutcome::BudgetExhausted));
        }

        // Record the send.
        self.daily_budget.record_sent(effective_priority);

        let outcome = if effective_priority != notification.priority {
            NotificationOutcome::Downgraded {
                from: notification.priority,
                to: effective_priority,
            }
        } else {
            NotificationOutcome::Sent
        };

        tracing::info!(
            notification_id = %notification.id,
            priority = ?effective_priority,
            "notification dispatched"
        );
        Some((notification, outcome))
    }

    /// Record how the user responded to a notification.
    pub fn record_response(&mut self, notification_id: Uuid, responded: bool) {
        let response = NotificationResponse {
            notification_id,
            priority: NotificationPriority::Actionable, // Simplified — fatigue only tracks actionable.
            responded,
            response_time_ms: None,
            timestamp: Utc::now(),
        };
        self.user_response_history.push(response);
        self.fatigue_model.record_response(responded);
    }

    /// Number of pending notifications.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for NotificationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_notification(priority: NotificationPriority) -> Notification {
        Notification {
            id: Uuid::new_v4(),
            title: "Test".into(),
            body: "Test body".into(),
            priority,
            domain: "finance".into(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_fatigue_downgrade() {
        let mut model = FatigueModel::new();
        for _ in 0..5 {
            model.record_response(false);
        }
        assert!(model.actionable_downgraded);
        assert_eq!(
            model.evaluate(NotificationPriority::Actionable),
            NotificationPriority::Informational
        );
    }

    #[test]
    fn test_fatigue_reset_on_response() {
        let mut model = FatigueModel::new();
        for _ in 0..5 {
            model.record_response(false);
        }
        assert!(model.actionable_downgraded);
        model.record_response(true);
        assert!(!model.actionable_downgraded);
    }

    #[test]
    fn test_critical_not_downgraded() {
        let model = FatigueModel {
            consecutive_ignored: 10,
            downgrade_threshold: 5,
            actionable_downgraded: true,
        };
        assert_eq!(
            model.evaluate(NotificationPriority::Critical),
            NotificationPriority::Critical
        );
    }

    #[test]
    fn test_budget_enforcement() {
        let mut budget = NotificationBudget::new();
        budget.max_actionable = 1;
        budget.record_sent(NotificationPriority::Actionable);
        assert!(!budget.allows(NotificationPriority::Actionable));
    }

    #[test]
    fn test_process_notification() {
        let mut orch = NotificationOrchestrator::new();
        orch.enqueue(make_notification(NotificationPriority::Critical));
        let (notif, outcome) = orch.process_next().unwrap();
        assert_eq!(notif.priority, NotificationPriority::Critical);
        assert_eq!(outcome, NotificationOutcome::Sent);
    }
}
