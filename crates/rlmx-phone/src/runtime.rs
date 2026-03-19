use uuid::Uuid;

use crate::background::BackgroundScheduler;
use crate::battery::{BatteryAwareScheduler, BatteryPolicy, DeviceCapabilities, ThermalState};
use crate::coordinator::LightweightCoordinator;
use crate::engagement::UserEngagement;
use crate::events::PhoneDomainEvent;
use crate::notifications::{NotificationOrchestrator, NotificationOutcome};
use crate::offline::OfflineOutbox;
use crate::widgets::WidgetManager;

/// Phone Runtime aggregate root (DDD-009).
///
/// Single point of truth for all device-local state. All mutations to
/// device-scoped resources flow through this aggregate.
#[derive(Debug)]
pub struct PhoneRuntime {
    pub device_id: Uuid,
    pub user_id: Uuid,
    pub coordinator: LightweightCoordinator,
    pub background: BackgroundScheduler,
    pub notifications: NotificationOrchestrator,
    pub widgets: WidgetManager,
    pub battery: BatteryAwareScheduler,
    pub offline_queue: OfflineOutbox,
    pub engagement: UserEngagement,
    pub capabilities: DeviceCapabilities,
    /// Collected domain events (drained by the caller after each operation).
    events: Vec<PhoneDomainEvent>,
}

impl PhoneRuntime {
    /// Create a new phone runtime for a device.
    pub fn new(device_id: Uuid, user_id: Uuid, capabilities: DeviceCapabilities) -> Self {
        let max_concurrent = BatteryAwareScheduler::new(capabilities.clone()).max_concurrent_agents();
        Self {
            device_id,
            user_id,
            coordinator: LightweightCoordinator::new(max_concurrent),
            background: BackgroundScheduler::new(std::time::Duration::from_secs(600)),
            notifications: NotificationOrchestrator::new(),
            widgets: WidgetManager::new(),
            battery: BatteryAwareScheduler::new(capabilities.clone()),
            offline_queue: OfflineOutbox::default(),
            engagement: UserEngagement::new(),
            capabilities,
            events: Vec::new(),
        }
    }

    /// Drain all pending domain events.
    pub fn drain_events(&mut self) -> Vec<PhoneDomainEvent> {
        std::mem::take(&mut self.events)
    }

    /// Initialize the runtime: spawn free agents, emit DeviceConnected.
    pub fn initialize(&mut self) {
        self.events.push(PhoneDomainEvent::DeviceConnected {
            device_id: self.device_id,
            capabilities: self.capabilities.clone(),
        });

        let ids = self.coordinator.spawn_free_agents();
        for id in &ids {
            self.events.push(PhoneDomainEvent::AgentStarted {
                agent_id: *id,
                agent_type: "free-agent".to_string(),
            });
        }

        tracing::info!(
            device_id = %self.device_id,
            agents = ids.len(),
            "phone runtime initialized"
        );
    }

    /// Start an agent (domain service: start_agent).
    pub fn start_agent(
        &mut self,
        agent_type: &str,
        is_free_tier: bool,
    ) -> Result<Uuid, PhoneError> {
        // Check battery allows agent execution (invariant 1).
        if !self.battery.inference_allowed() && !is_free_tier {
            return Err(PhoneError::BatteryTooLow);
        }

        let id = self
            .coordinator
            .start_agent(agent_type, is_free_tier)
            .map_err(|e| PhoneError::CoordinatorError(e.to_string()))?;

        self.events.push(PhoneDomainEvent::AgentStarted {
            agent_id: id,
            agent_type: agent_type.to_string(),
        });

        Ok(id)
    }

    /// Suspend an agent (domain service: suspend_agent).
    pub fn suspend_agent(&mut self, agent_id: Uuid, reason: &str) -> Result<(), PhoneError> {
        self.coordinator
            .suspend_agent(agent_id, reason)
            .map_err(|e| PhoneError::CoordinatorError(e.to_string()))?;

        self.events.push(PhoneDomainEvent::AgentSuspended {
            agent_id,
            reason: reason.to_string(),
        });

        Ok(())
    }

    /// Apply battery policy changes (domain service: apply_battery_policy).
    pub fn apply_battery_policy(&mut self, battery_pct: f32, thermal_state: ThermalState) {
        let (new_policy, changed) = self.battery.update(battery_pct, thermal_state);

        if changed {
            self.coordinator.apply_battery_policy(new_policy);

            // Adjust widget update interval.
            let interval_secs = self.battery.widget_update_interval_secs();
            self.widgets
                .set_update_interval(std::time::Duration::from_secs(interval_secs));

            // Suspend non-essential agents if critical.
            if new_policy == BatteryPolicy::Critical {
                let suspended = self.coordinator.suspend_non_essential();
                for id in &suspended {
                    self.events.push(PhoneDomainEvent::AgentSuspended {
                        agent_id: *id,
                        reason: "battery critical".to_string(),
                    });
                }
            }

            self.events.push(PhoneDomainEvent::BatteryPolicyChanged { new_policy });
            tracing::info!(?new_policy, battery_pct, ?thermal_state, "battery policy updated");
        }
    }

    /// Record verified savings (domain service: record_savings).
    pub fn record_savings(
        &mut self,
        amount_cents: u64,
        domain: &str,
        proof_id: Uuid,
    ) {
        self.engagement.money_saved.record(amount_cents, domain, proof_id);

        // Recalculate finance score (rough heuristic).
        let finance_bump = (amount_cents as f32 / 100.0).min(5.0);
        let current = self.engagement.life_score.finance;
        let delta = self
            .engagement
            .life_score
            .update_finance((current + finance_bump).min(100.0));

        self.events.push(PhoneDomainEvent::SavingsRecorded {
            amount_cents,
            domain: domain.to_string(),
            proof_id,
        });

        if delta.abs() > f32::EPSILON {
            self.events.push(PhoneDomainEvent::LifeScoreUpdated {
                composite: self.engagement.life_score.composite,
                delta,
            });
        }

        // Check savings achievements.
        let total = self.engagement.money_saved.total_cents;
        if total >= 100 {
            self.engagement.unlock_achievement("first_save");
        }
        if total >= 10000 {
            self.engagement.unlock_achievement("save_100");
        }
        if total >= 100000 {
            self.engagement.unlock_achievement("save_1000");
        }
        if total >= 1000000 {
            self.engagement.unlock_achievement("save_10000");
        }
    }

    /// Check streak (domain service: check_streak).
    pub fn check_streak(&mut self) {
        let now = chrono::Utc::now();
        if let Some(reward) = self.engagement.streak.check_in(now) {
            self.events.push(PhoneDomainEvent::StreakUpdated {
                current: self.engagement.streak.current,
                reward: Some(reward),
            });
        } else if self.engagement.streak.current > 0 {
            self.events.push(PhoneDomainEvent::StreakUpdated {
                current: self.engagement.streak.current,
                reward: None,
            });
        }
    }

    /// Process next pending notification (domain service: send_notification).
    pub fn process_notification(&mut self) -> Option<NotificationOutcome> {
        let (notif, outcome) = self.notifications.process_next()?;

        match &outcome {
            NotificationOutcome::Sent => {
                self.events.push(PhoneDomainEvent::NotificationSent {
                    notification_id: notif.id,
                    priority: notif.priority,
                });
            }
            NotificationOutcome::Downgraded { to, .. } => {
                self.events.push(PhoneDomainEvent::NotificationSent {
                    notification_id: notif.id,
                    priority: *to,
                });
            }
            NotificationOutcome::Suppressed { reason } => {
                self.events.push(PhoneDomainEvent::NotificationSuppressed {
                    notification_id: notif.id,
                    reason: reason.clone(),
                });
            }
            NotificationOutcome::BudgetExhausted => {
                self.events.push(PhoneDomainEvent::NotificationSuppressed {
                    notification_id: notif.id,
                    reason: "daily budget exhausted".to_string(),
                });
            }
        }

        Some(outcome)
    }

    /// Queue an offline request (domain service helper).
    pub fn queue_offline_request(&mut self, domain: &str, payload: Vec<u8>, priority: u8) -> Uuid {
        let id = self.offline_queue.enqueue(domain, payload, priority);
        self.events.push(PhoneDomainEvent::OfflineRequestQueued {
            request_id: id,
            domain: domain.to_string(),
        });
        id
    }

    /// Flush offline queue (domain service: flush_offline_queue).
    pub fn flush_offline_queue(&mut self) -> u32 {
        let network = self.capabilities.network;
        let flushed = self.offline_queue.flush(network);
        let count = flushed.len() as u32;
        if count > 0 {
            self.events
                .push(PhoneDomainEvent::OfflineRequestFlushed { count });
        }
        count
    }
}

/// Phone runtime errors.
#[derive(Debug, thiserror::Error)]
pub enum PhoneError {
    #[error("battery too low to start non-free agents")]
    BatteryTooLow,

    #[error("coordinator error: {0}")]
    CoordinatorError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_runtime() -> PhoneRuntime {
        PhoneRuntime::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            DeviceCapabilities::default_ios(),
        )
    }

    #[test]
    fn test_initialize_spawns_free_agents() {
        let mut rt = test_runtime();
        rt.initialize();
        assert_eq!(rt.coordinator.running_count(), 5);
        let events = rt.drain_events();
        // 1 DeviceConnected + 5 AgentStarted.
        assert_eq!(events.len(), 6);
    }

    #[test]
    fn test_battery_critical_suspends_non_essential() {
        let mut rt = test_runtime();
        rt.initialize();
        rt.drain_events();

        // Start a non-free agent.
        rt.start_agent("premium", false).unwrap();
        assert_eq!(rt.coordinator.running_count(), 6);

        // Battery drops to critical.
        rt.apply_battery_policy(0.03, ThermalState::Nominal);

        // Non-essential agent should be suspended.
        assert_eq!(rt.coordinator.running_count(), 5);
        let events = rt.drain_events();
        assert!(events.iter().any(|e| matches!(e, PhoneDomainEvent::BatteryPolicyChanged { .. })));
    }

    #[test]
    fn test_record_savings() {
        let mut rt = test_runtime();
        rt.record_savings(1500, "shopping", Uuid::new_v4());
        assert_eq!(rt.engagement.money_saved.total_cents, 1500);
        let events = rt.drain_events();
        assert!(events.iter().any(|e| matches!(e, PhoneDomainEvent::SavingsRecorded { .. })));
    }

    #[test]
    fn test_offline_queue_and_flush() {
        let mut rt = test_runtime();
        rt.queue_offline_request("test", vec![1, 2, 3], 5);
        assert_eq!(rt.offline_queue.queue_len(), 1);

        let count = rt.flush_offline_queue();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_cannot_start_agent_when_critical() {
        let mut rt = test_runtime();
        rt.apply_battery_policy(0.03, ThermalState::Nominal);
        rt.drain_events();

        let result = rt.start_agent("premium", false);
        assert!(result.is_err());
    }
}
