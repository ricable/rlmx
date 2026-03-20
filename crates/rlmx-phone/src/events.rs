use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::battery::{BatteryPolicy, DeviceCapabilities};
use crate::engagement::StreakReward;
use crate::notifications::NotificationPriority;

/// Domain events emitted by the Phone Runtime bounded context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhoneDomainEvent {
    DeviceConnected {
        device_id: Uuid,
        capabilities: DeviceCapabilities,
    },
    AgentStarted {
        agent_id: Uuid,
        agent_type: String,
    },
    AgentSuspended {
        agent_id: Uuid,
        reason: String,
    },
    NotificationSent {
        notification_id: Uuid,
        priority: NotificationPriority,
    },
    NotificationSuppressed {
        notification_id: Uuid,
        reason: String,
    },
    StreakUpdated {
        current: u32,
        reward: Option<StreakReward>,
    },
    LifeScoreUpdated {
        composite: f32,
        delta: f32,
    },
    SavingsRecorded {
        amount_cents: u64,
        domain: String,
        proof_id: Uuid,
    },
    OfflineRequestQueued {
        request_id: Uuid,
        domain: String,
    },
    OfflineRequestFlushed {
        count: u32,
    },
    BatteryPolicyChanged {
        new_policy: BatteryPolicy,
    },
    WidgetUpdated {
        widget_type: String,
    },
}
