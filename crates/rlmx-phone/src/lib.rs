//! # rlmx-phone
//!
//! Phone runtime bounded context (DDD-009) implementing the phone as primary
//! command center (ADR-013). Manages background execution, notification
//! orchestration, lock screen widgets, battery-aware scheduling, offline
//! queuing, and the engagement/gamification system.

pub mod background;
pub mod battery;
pub mod coordinator;
pub mod engagement;
pub mod events;
pub mod notifications;
pub mod offline;
pub mod runtime;
pub mod widgets;

// Re-export primary types.
pub use background::{BackgroundScheduler, PrioritizedTask, ScheduledTask, TaskStatus};
pub use battery::{
    BatteryAwareScheduler, BatteryPolicy, DeviceCapabilities, GpuType, MobileOS, NetworkType,
    ThermalState,
};
pub use coordinator::{FreeAgentType, LightweightCoordinator, LocalAgent};
pub use engagement::{
    Achievement, AgentCollection, CollectionGrid, CollectionSlot, LifeScore, MoneySaved,
    SavingsEvent, StreakReward, StreakState, UserEngagement,
};
pub use events::PhoneDomainEvent;
pub use notifications::{
    FatigueModel, Notification, NotificationBudget, NotificationOrchestrator, NotificationOutcome,
    NotificationPriority, NotificationResponse,
};
pub use offline::{OfflineOutbox, QueuedRequest, RetryPolicy};
pub use runtime::PhoneRuntime;
pub use widgets::{Widget, WidgetManager, WidgetType};
