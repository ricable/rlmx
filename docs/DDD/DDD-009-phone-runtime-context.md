# DDD-009: Phone Runtime Bounded Context

## Overview

The Phone Runtime context manages the mobile device as the primary command
center (Zone A-Mobile). It owns background execution scheduling, notification
orchestration, lock screen widgets, battery-aware throttling, offline queuing,
and the engagement system (Life Score, Money Saved counter, streaks,
gamification). This context ensures the phone delivers continuous value while
respecting OS constraints and battery limits.

**Crate**: `crates/rlmx-phone/`

## Aggregate Root: PhoneRuntime

```rust
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
}
```

`PhoneRuntime` is the single point of truth for all device-local state: which
agents are running, what notifications are pending, current battery policy, and
engagement metrics. All mutations to device-scoped resources flow through
this aggregate, ensuring the battery budget, notification budget, and agent
concurrency limits remain consistent.

## Entities

### LightweightCoordinator

A stripped-down coordinator that runs within mobile OS background execution
limits. Unlike the full `Coordinator` agent (DDD-003), this variant caps
concurrency based on real-time battery and thermal readings.

```rust
pub struct LightweightCoordinator {
    pub active_agents: Vec<LocalAgent>,
    pub max_concurrent: u8,             // Battery-dependent (3-8)
    pub zone: Zone,                     // Always ZoneAMobile
    pub swarm_connection: Option<SwarmLink>,
}
```

### BackgroundScheduler

Manages OS-constrained background execution windows (BGTaskScheduler on iOS,
WorkManager on Android). Prioritizes agent work within the OS-granted time
budget.

```rust
pub struct BackgroundScheduler {
    pub scheduled_tasks: Vec<ScheduledTask>,
    pub next_window: Option<DateTime<Utc>>,
    pub os_budget_remaining: Duration,
    pub priority_queue: BinaryHeap<PrioritizedTask>,
}
```

### NotificationOrchestrator

Owns the notification pipeline from agent output to user-facing alert. Applies
fatigue modeling to suppress low-value notifications when the user is not
engaging.

```rust
pub struct NotificationOrchestrator {
    pub pending: VecDeque<Notification>,
    pub fatigue_model: FatigueModel,       // ML-based send/suppress
    pub daily_budget: NotificationBudget,
    pub user_response_history: Vec<NotificationResponse>,
}
```

### WidgetManager

Manages lock screen and home screen widget state. Widgets are the primary
passive information surface — users see them without opening the app.

```rust
pub struct WidgetManager {
    pub active_widgets: Vec<Widget>,
    pub last_update: HashMap<WidgetType, DateTime<Utc>>,
    pub update_interval: Duration,         // Minimum 60s
}
```

### UserEngagement

The gamification and value-tracking subsystem. Tracks Life Score, money saved,
streaks, achievements, and agent leveling. All savings claims must be backed
by a `ProofSeal` from the kernel's proof subsystem.

```rust
pub struct UserEngagement {
    pub life_score: LifeScore,
    pub money_saved: MoneySaved,
    pub streak: StreakState,
    pub achievements: Vec<Achievement>,
    pub agent_levels: HashMap<AgentId, u8>,
    pub user_level: u16,
    pub xp: u64,
    pub collection_grid: CollectionGrid,
}
```

### OfflineOutbox

Transactional outbox for requests generated while the device is offline.
Requests are durably queued and flushed in order when connectivity resumes.

```rust
pub struct OfflineOutbox {
    pub queued_requests: VecDeque<QueuedRequest>,
    pub max_queue_size: usize,             // Default 100
    pub retry_policy: RetryPolicy,
    pub last_sync: Option<DateTime<Utc>>,
}
```

## Value Objects

### DeviceCapabilities

```rust
pub struct DeviceCapabilities {
    pub gpu: GpuType,                      // WebGPU, Metal, None
    pub memory_mb: u32,
    pub battery_pct: f32,                  // 0.0-1.0
    pub network: NetworkType,              // WiFi, Cellular, Offline
    pub thermal_state: ThermalState,       // Nominal, Fair, Serious, Critical
    pub os: MobileOS,                      // iOS, Android
    pub background_policy: BackgroundPolicy,
}
```

### LifeScore

Composite metric reflecting the user's real-world improvements across four
domains. History is retained for trend visualization in widgets.

```rust
pub struct LifeScore {
    pub finance: f32,                      // 0-100
    pub health: f32,                       // 0-100
    pub time: f32,                         // 0-100
    pub safety: f32,                       // 0-100
    pub composite: f32,                    // Weighted average
    pub history: Vec<(DateTime<Utc>, f32)>,
}
```

### MoneySaved

All savings claims are verified via `ProofSeal` from the kernel proof
subsystem. Unverified claims are rejected at the aggregate boundary.

```rust
pub struct MoneySaved {
    pub total_cents: u64,
    pub today_cents: u64,
    pub events: Vec<SavingsEvent>,         // ProofSeal-verified
}
```

### StreakState

```rust
pub struct StreakState {
    pub current: u32,
    pub longest: u32,
    pub last_active: DateTime<Utc>,
    pub freezes_remaining: u8,             // 1 per 30 days
    pub rewards_earned: Vec<StreakReward>,
}
```

### NotificationPriority

```rust
pub enum NotificationPriority {
    Critical,          // Strong vibration + alert (security, fraud)
    Actionable,        // Gentle vibration + chime (savings, price drops)
    Informational,     // Silent, batched into daily briefing
}
```

### Additional Value Objects

| Value Object | Definition |
|-------------|------------|
| `GpuType` | Enum: `WebGPU`, `Metal`, `None` |
| `NetworkType` | Enum: `WiFi`, `Cellular`, `Offline` |
| `ThermalState` | Enum: `Nominal`, `Fair`, `Serious`, `Critical` |
| `MobileOS` | Enum: `iOS`, `Android` |
| `BackgroundPolicy` | Enum: `Unrestricted`, `Adaptive`, `Restricted`, `Suspended` |
| `BatteryPolicy` | Enum: `Full`, `Balanced`, `LowPower`, `Critical` |
| `WidgetType` | Enum: `LifeScoreCard`, `MoneySavedCounter`, `AgentStatus`, `DailyBriefing`, `StreakTracker` |
| `RetryPolicy` | Struct: `{ max_retries: u8, base_delay: Duration, backoff_factor: f32 }` |
| `QueuedRequest` | Struct: `{ id: Uuid, domain: String, payload: Vec<u8>, queued_at: DateTime<Utc>, priority: u8 }` |
| `SavingsEvent` | Struct: `{ amount_cents: u64, domain: String, proof_id: Uuid, timestamp: DateTime<Utc> }` |
| `StreakReward` | Struct: `{ day: u32, reward_type: String, claimed: bool }` |
| `Achievement` | Struct: `{ id: String, name: String, unlocked_at: Option<DateTime<Utc>> }` |
| `CollectionGrid` | Struct: `{ slots: Vec<CollectionSlot>, rows: u8, cols: u8 }` |

## Domain Events

| Event | Trigger | Data |
|-------|---------|------|
| `DeviceConnected` | App launch or background wake | `{ device_id, capabilities }` |
| `AgentStarted` | Coordinator spawns local agent | `{ agent_id, agent_type }` |
| `AgentSuspended` | Battery/thermal/OS constraint | `{ agent_id, reason }` |
| `NotificationSent` | Orchestrator dispatches alert | `{ notification_id, priority }` |
| `NotificationSuppressed` | Fatigue model suppresses | `{ notification_id, reason }` |
| `StreakUpdated` | Daily check-in detected | `{ current, reward: Option<StreakReward> }` |
| `LifeScoreUpdated` | Domain metric changes | `{ composite, delta }` |
| `SavingsRecorded` | Verified savings event | `{ amount_cents, domain, proof_id }` |
| `OfflineRequestQueued` | Request while offline | `{ request_id, domain }` |
| `OfflineRequestFlushed` | Connectivity restored | `{ count }` |
| `BatteryPolicyChanged` | Battery threshold crossed | `{ new_policy }` |
| `WidgetUpdated` | Widget content refreshed | `{ widget_type }` |

```rust
pub enum PhoneDomainEvent {
    DeviceConnected { device_id: Uuid, capabilities: DeviceCapabilities },
    AgentStarted { agent_id: Uuid, agent_type: String },
    AgentSuspended { agent_id: Uuid, reason: String },
    NotificationSent { notification_id: Uuid, priority: NotificationPriority },
    NotificationSuppressed { notification_id: Uuid, reason: String },
    StreakUpdated { current: u32, reward: Option<StreakReward> },
    LifeScoreUpdated { composite: f32, delta: f32 },
    SavingsRecorded { amount_cents: u64, domain: String, proof_id: Uuid },
    OfflineRequestQueued { request_id: Uuid, domain: String },
    OfflineRequestFlushed { count: u32 },
    BatteryPolicyChanged { new_policy: BatteryPolicy },
    WidgetUpdated { widget_type: String },
}
```

## Domain Services

### `start_agent(agent_type, config) -> Result<AgentId>`

1. Check battery and thermal state against policy thresholds.
2. Check `LightweightCoordinator::max_concurrent` limit.
3. Derive scoped capability token from coordinator's token.
4. Spawn agent via kernel `ProcessFork` syscall.
5. Register in `LightweightCoordinator::active_agents`.
6. Emit `AgentStarted` event.

### `suspend_agent(agent_id, reason) -> Result<()>`

1. Signal agent to checkpoint its state.
2. Persist checkpoint to local storage.
3. Terminate agent process.
4. Emit `AgentSuspended` event.

### `send_notification(notification) -> Result<NotificationOutcome>`

1. Evaluate `FatigueModel` against `user_response_history`.
2. If suppressed: queue into daily briefing, emit `NotificationSuppressed`.
3. If approved: check `daily_budget`, dispatch via OS notification API.
4. Emit `NotificationSent` event.
5. Return outcome (sent, suppressed, or budgeted).

### `record_savings(amount_cents, domain, proof_id) -> Result<()>`

1. Validate `proof_id` against kernel proof subsystem (`ProofSeal`).
2. If invalid: reject with `UnverifiedSavingsError`.
3. Append `SavingsEvent` to `MoneySaved::events`.
4. Update `today_cents` and `total_cents`.
5. Recalculate `LifeScore::finance`.
6. Emit `SavingsRecorded` and `LifeScoreUpdated` events.

### `check_streak() -> Result<StreakOutcome>`

1. Compare `StreakState::last_active` to current date.
2. If same day: no-op.
3. If next day: increment `current`, check for rewards, emit `StreakUpdated`.
4. If missed day and `freezes_remaining > 0`: decrement freeze, preserve streak.
5. If missed day and no freezes: reset `current` to 0 (preserve `longest`).

### `flush_offline_queue() -> Result<u32>`

1. Check `NetworkType` is not `Offline`.
2. Drain `OfflineOutbox::queued_requests` in FIFO order.
3. Send each request, applying `RetryPolicy` on transient failures.
4. Remove successfully sent requests.
5. Emit `OfflineRequestFlushed` with count.

### `apply_battery_policy(battery_pct, thermal_state) -> Result<()>`

1. Map `(battery_pct, thermal_state)` to `BatteryPolicy`.
2. If policy changed: adjust `LightweightCoordinator::max_concurrent`.
3. If `Critical` (< 5%): suspend all non-essential agents.
4. If `Serious` or `Critical` thermal: suspend compute-heavy agents.
5. Update `WidgetManager::update_interval` (increase in low-power).
6. Emit `BatteryPolicyChanged` event.

## Invariants

1. **Battery floor**: No agent execution when `battery_pct < 0.05` (except
   `Critical` notifications). `apply_battery_policy()` enforces this by
   suspending all agents at the 5% threshold.

2. **Thermal ceiling**: Suspend non-essential agents when `thermal_state` is
   `Serious` or `Critical`. Only the `LightweightCoordinator` and
   `NotificationOrchestrator` remain active.

3. **Notification fatigue**: If the user ignores 5+ consecutive `Actionable`
   notifications, auto-downgrade future `Actionable` to `Informational` until
   the user re-engages. `FatigueModel` tracks this.

4. **Streak grace**: One freeze per 30 days. Missed day after freeze exhaustion
   resets `current` to 0. `longest` is never decremented. Cumulative total is
   always preserved.

5. **Savings must be verified**: `MoneySaved` events require a valid `ProofSeal`
   from the kernel proof subsystem. `record_savings()` rejects unverified claims
   at the aggregate boundary.

6. **Offline queue bounded**: Maximum 100 queued requests. When the queue is
   full, the oldest request is evicted (FIFO). `OfflineOutbox::max_queue_size`
   enforces this.

7. **5 free agents always available**: Free-tier agents run on-device regardless
   of subscription state. The `LightweightCoordinator` never refuses to spawn
   the 5 base agent types.

8. **Widget update frequency**: Lock screen widgets update at most once per 60
   seconds to conserve battery. `WidgetManager::update_interval` enforces this
   minimum.

9. **Life Score floor**: Minimum composite score of 30 for any active user.
   `LifeScore::composite` is clamped to `[30.0, 100.0]` to prevent
   discouragement.

## Context Relationships

| Related Context | Relationship | Description |
|----------------|-------------|-------------|
| Voice Interaction (DDD-008) | Partner | Voice pipeline runs within phone runtime; shares `DeviceCapabilities` and `BatteryPolicy` |
| Kernel Syscall (DDD-002) | Upstream | Phone dispatches syscalls through WASM kernel; kernel defines `ProcessFork`, `ProcessSend` |
| Agent Lifecycle (DDD-003) | Upstream | Phone starts/stops agents based on device resources; agents conform to DDD-003 types |
| Swarm Coordination (DDD-004) | Upstream | Phone joins swarm as Zone A-Mobile node; receives scatter-gather tasks |
| Inference Routing (DDD-005) | Upstream | Phone routes inference through `TieredEngine`; local models run via `LocalEngine` |
| Marketplace (DDD-010) | Downstream | Phone installs and runs marketplace agents; engagement system tracks agent levels |

## Anti-Corruption Layer

The Phone Runtime ACL translates between platform-specific OS APIs and the
domain model:

- **OS Background Scheduling**: `BackgroundScheduler` abstracts
  `BGTaskScheduler` (iOS) and `WorkManager` (Android) behind a unified
  `ScheduledTask` interface. Platform-specific callbacks are translated to
  `PhoneDomainEvent` variants.

- **Device Capabilities**: `DeviceCapabilities` normalizes iOS
  `ProcessInfo.thermalState` and Android `PowerManager` readings into a
  platform-agnostic `ThermalState` enum. Battery percentage is normalized
  to `0.0-1.0` regardless of OS API format.

- **Notification APIs**: `NotificationOrchestrator` wraps `UNUserNotification`
  (iOS) and `NotificationManager` (Android) behind the `Notification` struct.
  Platform-specific notification categories and channels are mapped to
  `NotificationPriority`.

- **Engagement Privacy**: Life Score, streaks, and savings are computed and
  stored locally on-device. Only anonymized, aggregated leaderboard data
  leaves the device. The ACL strips all PII before any network transmission.

- **Offline Outbox**: `QueuedRequest` structs are platform-neutral serialized
  payloads. The ACL handles OS-specific network reachability detection and
  translates connectivity callbacks into `flush_offline_queue()` triggers.

## File Map (planned)

| File | Types |
|------|-------|
| `crates/rlmx-phone/src/lib.rs` | Public re-exports |
| `crates/rlmx-phone/src/runtime.rs` | `PhoneRuntime` aggregate root |
| `crates/rlmx-phone/src/coordinator.rs` | `LightweightCoordinator`, agent lifecycle |
| `crates/rlmx-phone/src/background.rs` | `BackgroundScheduler`, `ScheduledTask`, `PrioritizedTask` |
| `crates/rlmx-phone/src/notifications.rs` | `NotificationOrchestrator`, `FatigueModel`, `NotificationBudget` |
| `crates/rlmx-phone/src/widgets.rs` | `WidgetManager`, `WidgetType`, widget update logic |
| `crates/rlmx-phone/src/battery.rs` | `BatteryAwareScheduler`, `BatteryPolicy`, thermal mapping |
| `crates/rlmx-phone/src/offline.rs` | `OfflineOutbox`, `QueuedRequest`, `RetryPolicy` |
| `crates/rlmx-phone/src/engagement.rs` | `UserEngagement`, `LifeScore`, `MoneySaved`, `StreakState` |
| `crates/rlmx-phone/src/capabilities.rs` | `DeviceCapabilities`, `GpuType`, `NetworkType`, `ThermalState`, `MobileOS` |
| `crates/rlmx-phone/src/events.rs` | `PhoneDomainEvent` enum (12 variants) |
| `crates/rlmx-phone/src/acl.rs` | Anti-corruption layer: OS abstraction traits |

## Implementation Status

**Status**: Implemented in `crates/rlmx-phone/`

| Module | File | Tests |
|--------|------|-------|
| PhoneRuntime (aggregate root) | `src/runtime.rs` | 5 |
| LightweightCoordinator | `src/coordinator.rs` | 5 |
| BackgroundScheduler | `src/background.rs` | 3 |
| NotificationOrchestrator | `src/notifications.rs` | 5 |
| WidgetManager | `src/widgets.rs` | 4 |
| BatteryAwareScheduler | `src/battery.rs` | 8 |
| OfflineOutbox | `src/offline.rs` | 4 |
| UserEngagement | `src/engagement.rs` | 13 |
| PhoneDomainEvent | `src/events.rs` | — |

**Total**: 47 tests, all passing. All 9 DDD-009 invariants enforced.

Mobile app implemented in `mobile/` (React Native 0.84.1, TypeScript):
- 8 screens, 11 components, 3 hooks, 4 services
- Android APK builds successfully with JDK 21
