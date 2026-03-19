# ADR-013: Phone as Primary Command Center (Zone A-Mobile)

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The existing RLMX zone model (ADR-001) places the phone in Zone D — lowest priority, burst-only compute. The PRD demands inverting this: the phone becomes Zone A-Mobile, the PRIMARY interface where users speak commands and receive multimodal responses. The laptop becomes Zone A-Desktop (secondary). This requires a new `rlmx-phone` crate for mobile runtime management.

## Decision

### Invert the Zone Model

- Phone promoted from Zone D to Zone A-Mobile (primary command interface)
- Laptop becomes Zone A-Desktop (secondary, heavy compute)
- Cloud remains Zone B (burst)
- Home Hub remains Zone C (sentinel, privacy anchor)

### Introduce `rlmx-phone` crate

New crate `crates/rlmx-phone/` with:

- `PhoneRuntime` struct: coordinates all on-device capabilities
- `LightweightCoordinator`: on-device swarm coordinator managing local agents
- `BackgroundScheduler`: iOS BGProcessingTask/BGAppRefreshTask + Android WorkManager/ForegroundService abstraction
- `NotificationOrchestrator`: 3-tier priority (Critical/Actionable/Informational) with ML-based fatigue prevention
- `WidgetManager`: lock screen widgets (agent status, money saved counter, quick voice)
- `BatteryAwareScheduler`: throttle inference based on battery %, thermal state
- `OfflineOutbox`: queue cloud-burst requests when offline, auto-retry on connectivity
- `DeviceCapabilities`: runtime detection of GPU type, memory, battery, network, thermal state

### 5 Always-On Free Agents (on-device, zero network)

1. Email Triage (Router agent, WASM)
2. Calendar (Router agent, WASM)
3. Weather+Commute (Monitor agent, cached data)
4. News Digest (Router agent, cached RSS + learned preferences)
5. Shopping Comparison (Monitor agent, federated price cache)

### Background Execution Strategy

| Platform | API                  | Window        | RuVix Usage                    |
|----------|----------------------|---------------|--------------------------------|
| iOS      | Background Audio     | Continuous    | VAD always-listening           |
| iOS      | BGProcessingTask     | 30s-10min     | SONA training, federated sync  |
| iOS      | BGAppRefreshTask     | 30s           | Quick agent checks             |
| Android  | Foreground Service   | Unlimited     | VAD + coordinator              |
| Android  | WorkManager          | 10min+        | Full agent execution           |

### Lock Screen Widgets

1. **Agent Status**: active count, pending tasks, last action
2. **Money Saved**: cumulative savings counter with green pulse animation
3. **Quick Voice**: tap-to-speak from lock screen (iOS Live Activity / Android Widget)

### Push Notification System

3-tier priority with ML fatigue prevention:

- **Critical**: strong vibration + alert tone (security threats, fraud)
- **Actionable**: gentle vibration + chime (savings found, price drops)
- **Informational**: silent, batched into daily briefing

ML model trained on user's response patterns — auto-downgrades if user ignores consecutive notifications of a type. Learns within 2 weeks.

### Offline-First Architecture

- 5 free agents run with zero network connectivity
- SONA PatternBank (personal) cached on-device
- Federated pattern cache (50K patterns) updated weekly over WiFi
- Cloud-burst requests queued in OfflineOutbox, auto-execute when connectivity restored
- User always sees value, even in airplane mode

## Consequences

### Positive

- Phone becomes the always-with-you command center — matches actual user behavior
- Offline-first means value even without internet
- Battery-aware scheduling prevents poor user experience
- Widget system provides ambient awareness without opening the app

### Negative

- Zone model inversion requires updating all zone-routing logic in `rlmx-kernel` and `rlmx-swarm`
- iOS background execution constraints limit what agents can do between user sessions
- Phone storage constraints limit on-device SONA capacity (~50K patterns vs. unlimited on laptop/Pi)

### Risks

- Apple/Google may tighten background execution policies
- Battery drain from always-on VAD could cause negative app store reviews if not carefully tuned
- Different Android OEMs aggressively kill background services (Xiaomi, Samsung, Huawei)

## References

- ADR-001: Distributed Swarm Architecture (zone model)
- ADR-009: Browser WASM Compute Pool (existing phone WASM infrastructure)
- ADR-012: Voice-First Pipeline (voice integration)
- PRD Section 4: The Phone as Command Center

## Implementation Notes

Implemented in `crates/rlmx-phone/` with 10 source files and 47 tests. PhoneRuntime aggregate root. LightweightCoordinator with 5 always-on free agents (EmailTriage, Calendar, WeatherCommute, NewsDigest, ShoppingComparison). BackgroundScheduler (iOS/Android abstraction). NotificationOrchestrator (3-tier with fatigue prevention). BatteryAwareScheduler with DeviceCapabilities. OfflineOutbox (bounded queue, 100 max). All 9 DDD-009 invariants enforced. CLI: `cargo run -p rlmx-cli -- phone status|agents|battery`
