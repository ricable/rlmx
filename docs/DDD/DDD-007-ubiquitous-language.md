# DDD-007: Ubiquitous Language Glossary

## Overview

This glossary defines every key term used across the RLMX codebase and
documentation. Terms are grouped by bounded context. All code, comments,
commit messages, and conversations should use these terms consistently.

---

## Kernel Syscall Context

| Term | Definition |
|------|------------|
| **Syscall** | One of 12 kernel operations that an agent can invoke: `VecInsert`, `VecSearch`, `VecDelete`, `GraphQuery`, `GraphCut`, `GraphDiffuse`, `ProcessFork`, `ProcessSend`, `ProcessRecv`, `StateMutate`, `AttentionSelect`, `HaltCheck`. Defined as `enum Syscall` in `syscall.rs`. |
| **Capability Token** | An HMAC-SHA256-signed credential (`CapabilityToken`) that authorizes a process to invoke specific syscalls. Contains: owner, granted permissions, scope, expiry, and signature. |
| **Syscall Permission** | A single permission variant (`enum SyscallPermission`) granting access to one syscall family, or `All` for unrestricted access. |
| **Process Fork** | The `ProcessFork` syscall that creates a child process with a scoped capability token and isolated memory view. Analogous to Unix `fork()` but with capability narrowing. |
| **Witness Chain** | An SHA-256-chained append-only audit log (`WitnessChain`) where each entry's `content_hash` covers all its fields and its `prev_hash` links to the previous entry. |
| **Witness** | A single entry in the witness chain, recording an action hash, reasoning chain hash, evidence references, timestamp, and chain linkage. |
| **Proof** | The output of `ProofEngine::validate()`. Contains `valid: bool`, `confidence`, and the `witness_id` of the appended witness. A proof is valid when confidence meets the threshold and evidence requirements are satisfied. |
| **Proof-Gated Mutation** | The invariant that every `StateMutate` syscall must pass through `ProofEngine::validate()` and append a witness before the mutation is accepted. |
| **KernelContext** | The aggregate root holding `Arc<Mutex<T>>` handles to all subsystems (memory, graph, processes, proof engine, capability manager). All syscall dispatch flows through this struct. |
| **Process** | A kernel-level execution unit (`struct Process`) with a unique `ProcessId`, parent reference, capability token, memory scope, status, and message channel. |
| **Memory Region** | The vector memory subsystem (`MemoryRegion`) providing brute-force cosine similarity search over 64-dimensional pseudo-embeddings with Hot/Warm/Cold tier classification. |
| **Segment Tier** | Classification of a memory segment by access frequency: `Hot` (frequently accessed), `Warm` (moderate), `Cold` (rarely accessed). |
| **Graph** | The in-memory property graph (`struct Graph`) supporting Cypher-like queries, Karger/Stoer-Wagner min-cut, and heat-kernel diffusion. |
| **Dispatch** | The function `dispatch(syscall, ctx)` that pattern-matches on a `Syscall` variant and routes to the appropriate subsystem handler. The central entry point of the kernel. |

## Agent Lifecycle Context

| Term | Definition |
|------|------------|
| **Agent** | A specialized kernel process with domain-specific behavior. Distinguished from a raw `Process` by having an `AgentType`, zone assignment, model tier, and participation in the permission matrix. |
| **Agent Type** | One of 12 specializations: `Coordinator`, `Researcher`, `Router`, `Experimenter`, `Worker`, `Monitor`, `Reviewer`, `Trainer`, `Validator`, `Replicator`, `Embedder`, `Analyst`. |
| **Coordinator** | The supervisory agent type (one per zone) that orchestrates task decomposition and spawns other agents. Holds the broadest capability token in its zone. |
| **Researcher** | Agent that generates hypotheses and designs experiments for the auto-research pipeline. Max 3 concurrent. |
| **Router** | Agent that wraps the TinyDancer neural router, handling fast query classification. |
| **Experimenter** | Agent that executes experiments in COW-branched isolation. Max 8 total. |
| **Worker** | General-purpose agent that executes concrete tasks (code generation, analysis, data processing). |
| **Monitor** | Agent that performs health checks, collects metrics, and detects anomalies. |
| **Reviewer** | Agent that validates outputs against quality gates before results are accepted. |
| **Trainer** | Agent that performs online learning: SONA adaptation, LoRA updates, DAG optimizer training. |
| **Validator** | Agent that audits proof chains and validates witness integrity. |
| **Replicator** | Agent that synchronizes state across nodes in the swarm. |
| **Embedder** | Agent that generates and indexes vector embeddings. |
| **Analyst** | Agent that synthesizes results from multiple workers into coherent reports. |
| **Permission Matrix** | A 12x12 boolean table defining which agent types can send messages to which other types. Enforced on every `ProcessSend` between agents. |
| **Agent Registry** | The aggregate root (`AgentRegistry`) that tracks all live agents, enforces concurrency limits, and validates permissions. |
| **Agent Spawner** | Factory that creates agents by deriving child capability tokens and issuing `ProcessFork` syscalls. |
| **Process Group** | A logical grouping of agents assembled by a Coordinator to work on a shared task. |
| **Model Tier** | The inference capability level assigned to an agent: `Small` (Haiku-class), `Medium` (Sonnet-class), `Large` (Opus-class), `Edge` (local GGUF), `Custom`. |

## Swarm Coordination Context

| Term | Definition |
|------|------------|
| **Swarm** | The distributed cluster of up to 25 nodes running RLMX agents. |
| **Swarm Node** | A physical or virtual machine participating in the swarm. Has a `NodeId`, zone assignment, hardware profile, and health status. |
| **Zone** | A failure domain and hardware grouping. Five zones: `ZoneA` (primary), `ZoneB` (secondary), `ZoneC` (edge), `Cloud` (GPU), `Browser` (WASM). |
| **PBFT** | Practical Byzantine Fault Tolerance. Used for critical consensus (leader election). Requires `3f + 1` nodes for `f` faults. |
| **Raft** | Log replication consensus. Used for steady-state coordination within zones. Requires majority quorum (`n/2 + 1`). |
| **Gossip** | Epidemic protocol for membership propagation and health status. Eventual consistency within 10 seconds. |
| **Leader** | The elected node within a consensus group that proposes and coordinates decisions. One leader per epoch. |
| **Epoch** | A monotonically increasing counter for consensus rounds. Each leader election increments the epoch. |
| **Heartbeat** | Periodic health check message sent between nodes. 3 consecutive misses marks a node `Unreachable`. |
| **Partition** | A network split where groups of nodes cannot communicate. Detected when >50% of a zone's nodes are unreachable. |
| **Zone Failover** | Automatic migration of agents from a failed zone to its designated failover target zone. |
| **Node Profile** | Hardware description of a node: CPU cores, memory, GPU availability, backend type, OS, architecture. |

## Inference Routing Context

| Term | Definition |
|------|------------|
| **Strategy** | The inference approach for a query. Enum: `Rlm` (recursive LLM), `Trm` (tiny recursive model), `Edge` (local GGUF), `Hybrid` (triage + dispatch), `Auto` (resolved by router). |
| **Tiny-Dancer** | The planned sub-millisecond neural router based on FastGRNN. Replaces the heuristic `auto_select()` with a learned routing function. Named for its small footprint and speed. |
| **FastGRNN** | Fast, Gated Recurrent Neural Network. A compact RNN architecture suitable for real-time inference on resource-constrained devices. The core of TinyDancer. |
| **Router Input** | The 6 features extracted from a query for routing: `query_length`, `has_code`, `is_question`, `trigram_entropy`, `edge_available`, `node_load`. |
| **Escalation** | Moving a query from a lower inference tier to a higher one when confidence is below threshold. Monotonic: only upward (`Edge -> Small -> Medium -> Large -> Cloud`). |
| **Local Engine** | The `LocalEngine` struct in `rlmx-ruvllm` that wraps `CandleBackend` for GGUF model inference. Feature-gated behind `ruvllm`. |
| **Edge Inference** | Running LLM inference locally on the device (RPi5, Mac, browser) rather than calling a remote API. Uses GGUF quantized models. |
| **GGUF** | A quantized model file format used by llama.cpp and ruvllm. RLMX discovers GGUF files in `~/.rlmx/models/`. |

## Research & Evolution Context

| Term | Definition |
|------|------------|
| **Research Objective** | A goal for autonomous discovery. The aggregate root that tracks hypotheses, experiments, and the current best genome. |
| **Hypothesis** | A testable prediction generated by a Researcher agent. Contains a statement, predicted improvement, and confidence level. |
| **Experiment** | A controlled test of a hypothesis, executed by an Experimenter agent in a COW-branched environment. |
| **Genome** | A serializable configuration vector (`struct Genome`) encoding: feature weights, routing thresholds, prompt templates, and attention config. The unit of evolution. |
| **Fitness** | A multi-objective score (`FitnessScore`) combining accuracy, latency, and cost. Used to rank genomes and select survivors. |
| **Fitness Score** | `{ accuracy: f64, latency_ms: f64, cost: f64, combined: f64 }`. The `combined` field is the weighted aggregate used for comparison. |
| **Mutation** | A random modification to a genome. Types: `PointMutation` (perturb one weight), `Crossover` (combine two genomes), `Insertion` (add template), `Deletion` (remove weight). |
| **Cross-Pollination** | Transferring successful genome fragments from one research objective to another, enabling knowledge sharing across independent research tracks. |
| **Cloud Escalation** | Routing stalled research to Cloud zone nodes with GPU resources when local compute fails to make progress. Triggered by `stall_threshold`. |
| **Generation** | A monotonically increasing counter tracking evolutionary iterations within a research objective. |
| **COW Branch** | A copy-on-write fork of an RVF container (`BranchManager::create_branch()`) used to isolate experiment mutations from the main state. |

## Observation & Health Context

| Term | Definition |
|------|------------|
| **SONA** | Self-Optimizing Neural Architecture (`struct Sona`). Records successful (query, actions, result) patterns and applies micro-LoRA adaptations. The learning core of RLMX. |
| **Pattern Bank** | SONA's storage for learned patterns (`PatternBank`). Bounded capacity with lowest-quality eviction. Supports cosine similarity search over 64-dim embeddings. |
| **Pattern** | A recorded triple: `(query_embedding, actions_taken, result_quality)`. Stored in the Pattern Bank for future retrieval. |
| **Micro-LoRA** | Low-rank adaptation applied to inference pathway layers. SONA produces `LoraDelta` structs with rank-factored weight updates. Small and fast to apply. |
| **EWC++** | Elastic Weight Consolidation (improved). A regularization technique that penalizes changes to important weights, preventing catastrophic forgetting during online learning. Implemented via `FisherInformation` diagonal. |
| **DAG Optimizer** | The `DagOptimizer` that records execution history and learns optimal (strategy, attention mechanism) pairs for each query pattern. Self-improving over time. |
| **Nervous System** | The bio-inspired cognitive architecture in `rlmx-cognitive`: BTSP, HDC, WTA, Circadian Controller, Global Workspace. |
| **BTSP** | Behavioral Time-Scale Plasticity. One-shot memory system inspired by hippocampal learning. Stores patterns immediately without iterative training. |
| **Hyperdimensional Computing (HDC)** | Computing with 10,000-bit binary hypervectors. Uses XOR binding and Hamming similarity. Implemented in `HdcComputer`. |
| **Winner-Take-All (WTA)** | Competition network where neurons inhibit their neighbors. Returns the index of the winning (highest activation) neuron. |
| **Circadian Controller** | Schedules system phases by time-of-day: `Compute` (08-18h), `Learn` (18-22h), `Consolidate` (22-08h). Controls when adaptation and compaction occur. |
| **Global Workspace** | A limited-capacity attention buffer (4-7 items) inspired by Global Workspace Theory. Items compete for slots based on salience. |

## Container & Storage Context

| Term | Definition |
|------|------------|
| **RVF Container** | The sealed packaging unit (`RvfContainer`): manifest + typed segments + witness chain + optional Ed25519 signature. |
| **Seal** | Signing an RVF container with an Ed25519 key (`container.seal(signing_key)`). After sealing, the container carries a `ContainerSignature` covering the content hash. |
| **RBAC** | Role-Based Access Control. 6 roles: `Viewer`, `Operator`, `Engineer`, `Admin`, `Auditor`, `System`. Defined in `rlmx-rvf/src/rbac.rs`. |
| **Branch Manager** | Manages named COW branches of RVF containers. `create_branch()`, `merge()`, `diff()`. |
| **Segment** | A typed data unit within an RVF container (`RvfSegment`). Types: `Vec`, `Graph`, `Config`, `Model`, `Prompt`, `Evidence`. |

## Plugin & Domain Context

| Term | Definition |
|------|------------|
| **Domain Plugin** | The `DomainPlugin` trait (14 methods) in `rlmx-plugin`. The primary extension point for adding domain-specific ingest, strategy, safety, and graph schema. |
| **Safety Engine** | Enforces `ParameterBound` and `RateLimit` constraints from a plugin's `safety_constraints()`. |
| **Ingest Adapter** | A plugin-provided component that transforms domain data into kernel-compatible segments for `VecInsert`. |

## Voice Pipeline Context

| Term | Definition |
|------|------------|
| **Conversation Turn** | A single user-system exchange within a `VoiceSession`, identified by a unique ID. Contains transcript, audio duration, extracted intents, and optional multimodal response. Turns are entities (not value objects) because downstream consumers reference them by ID for follow-up resolution. |
| **Emotion Bucket** | One of 5 discrete quantization levels (`VeryNegative`, `Negative`, `Neutral`, `Positive`, `VeryPositive`) applied to continuous `f32` emotion valence before any data leaves the device. Prevents fingerprinting through high-precision emotion tracking. Enforced at the ACL boundary. |
| **Fan-Out** | The scatter-gather pattern used when a single utterance decomposes into multiple intents across different life domains. The voice context requests `Strategy::Swarm { scatter_zones, gather_strategy, timeout_ms }` to resolve intents in parallel across zones, then assembles results into a single `MultimodalResponse`. |
| **Federated Pattern** | A learned `(query_embedding, actions_taken, result_quality)` pattern from SONA's `PatternBank` that has been cleared for cross-device sharing. Speaker embeddings and raw audio are stripped; only text-derived features and scalar metrics are federated. |
| **Anonymized Pattern** | A federated pattern with additional PII removal: user IDs replaced with ephemeral hashes, timestamps coarsened to day granularity, and entity values generalized (e.g., specific dollar amounts become ranges). Used for population-level learning without individual traceability. |
| **Intent** | A structured action extracted from a transcript, containing: `LifeDomain`, action verb, extracted entities with character-span offsets, urgency score (0-1), and decomposition confidence (0-1). Intents below 0.3 confidence are discarded; ambiguous intents (0.3-0.6) trigger a clarification turn. |
| **Multi-Intent Decomposition** | Parsing a single utterance into multiple `Intent` structs targeting different life domains. E.g., "Cancel my dentist appointment and order more dog food" yields two intents in Health and Shopping domains. |
| **Response Mode** | How the system responds — `VoiceOnly`, `Visual`, `Multimodal`, or `Ambient` — auto-detected from device sensors (accelerometer, proximity, screen state, audio output route). Mode changes take effect on the next turn, not mid-response. |
| **Speaker Context** | Metadata about the speaker (confidence, emotion valence, urgency, noise level) passed to router. The embedding vector stays on-device; only scalar features cross context boundaries. |
| **VAD (Voice Activity Detection)** | 500K parameter CNN that detects human speech in ambient audio, triggering STT activation. Operates on a 3-second rolling ring buffer with adaptive silence threshold. |
| **Voice Persona** | Domain-specific TTS personality (e.g., warm for health, authoritative for legal). Each of the 12 life domains has a default persona; users can override per domain. Defined by `PersonaStyle` (Warm, Authoritative, Upbeat, Calm, Neutral), speech rate multiplier, and TTS model identifier. |
| **Voice Session** | The aggregate root of the Voice Interaction context. A bounded interaction from wake word through response delivery, owning the consistency boundary for conversation turns, intent state, emotion trajectory, and resource allocation (audio buffer, STT context, TTS stream). Auto-closes after 5 minutes of inactivity. |
| **Wake Word** | User-configurable keyword (default "Hey RuVix") processed on-device to activate the voice pipeline. Wake word detection is a prerequisite for STT activation (invariant: no continuous transcription without explicit trigger). |

## Phone & Engagement Context

| Term | Definition |
|------|------------|
| **Achievement** | A named milestone unlocked by user or agent activity (e.g., "First Savings", "10-Day Streak", "All Domains Covered"). Tracked in `UserEngagement::achievements` with an `unlocked_at` timestamp. Achievements are write-once: once unlocked, they are never revoked. |
| **Agent Collection** | The set of all agents a user has installed or can install, visualized in the Collection Grid. Each agent in the collection has a level (1-10), install status, and domain assignment. Collecting agents across all 12 life domains is a gamification goal. |
| **Agent Level** | 1-10 capability level per agent, reflecting real SONA learning depth. Leveling is driven by actual pattern quality improvements in the `PatternBank`, not arbitrary XP. Higher levels unlock expanded capability tokens. |
| **Background Scheduler** | The OS-constrained background execution manager within `PhoneRuntime`. Abstracts `BGTaskScheduler` (iOS) and `WorkManager` (Android) behind a unified `ScheduledTask` interface. Prioritizes agent work within the OS-granted time budget via a `BinaryHeap<PrioritizedTask>`. |
| **Collection Grid** | Visual grid of all available agents (installed shown in color, uninstalled greyed). Dimensions are `rows x cols` slots; each slot maps to an `AgentListing` from the marketplace. |
| **Life Score** | Daily 0-100 composite score across Finance, Health, Time, Safety domains. Floor-clamped to 30 for active users to prevent discouragement. History retained for trend visualization in lock screen widgets. |
| **Lightweight Coordinator** | A stripped-down `Coordinator` agent that runs within mobile OS background execution limits. Unlike the full `Coordinator` (DDD-003), it caps agent concurrency (3-8) based on real-time battery and thermal readings. Always assigned to `ZoneAMobile`. |
| **Money Saved** | Real-time cumulative savings counter, ProofSeal-verified, displayed on lock screen widget. Every savings claim requires a valid `ProofSeal` from the kernel proof subsystem; unverified claims are rejected at the aggregate boundary. |
| **Notification Fatigue Prevention** | ML model that auto-downgrades notification priority based on user response patterns. If 5+ consecutive `Actionable` notifications are ignored, future `Actionable` notifications are downgraded to `Informational` until re-engagement. |
| **Notification Tier** | Three-level priority classification for phone notifications: `Critical` (strong vibration + alert for security/fraud), `Actionable` (gentle vibration + chime for savings/price drops), `Informational` (silent, batched into daily briefing). Fatigue model can auto-downgrade tiers. |
| **Offline Outbox** | Transactional outbox queue (max 100 entries, FIFO eviction) for requests generated while the device is offline. Requests are durably queued and flushed in order when connectivity resumes, with configurable `RetryPolicy` (max retries, base delay, backoff factor). |
| **Streak** | Consecutive daily engagement counter with progressive reward tiers (7-day through 90-day). One freeze per 30 days preserves the streak on a missed day. `longest` is never decremented; cumulative total is always preserved. |
| **Zone A-Mobile** | The phone as primary command interface, promoted from Zone D in the original architecture. The phone joins the swarm as a Zone A-Mobile node, participating in consensus and receiving scatter-gather work units. |

## Marketplace Context

| Term | Definition |
|------|------------|
| **Agent Listing** | A published agent in the marketplace with metadata, rating, price, RVF container hash, required permissions, supported devices, and minimum model tier. Status lifecycle: `Draft` -> `InReview` -> `Published` (or `Suspended`). |
| **Agent Pack** | Curated agent configuration bundle, often celebrity/influencer branded. Revenue split is 50/30/20 (creator/platform/base-developer). Each pack contains agent IDs with custom configurations. |
| **Life Domain** | One of 12 categories: Finance, Health, Legal, Career, Education, Home, Shopping, Travel, Social, Government, Automotive, Pet. Shared with the kernel context (`LifeDomain` enum defined in `rlmx-kernel`, re-exported by consuming crates). |
| **Marketplace Listing** | Synonym for Agent Listing. The canonical aggregate entity in the marketplace context representing a single distributable agent with its metadata, review status, and pricing. |
| **Publisher** | A registered developer or organization (`Individual`, `Organization`, or `Celebrity`) with verified identity, reputation score, and `EarningsAccount`. Able to submit agents to the marketplace for review and distribution. |
| **Review Pipeline** | The sequential security audit process for agent submissions. Consists of 5 automated checks (capability minimality, data flow verification, fuzz testing, network policy compliance, malware signature scan). Agents requesting health, finance, or legal permissions are escalated to mandatory human review. No bypasses or fast-tracks exist. |
| **Revenue Split** | 70/30 developer/platform for standard agents; 50/30/20 creator/platform/base-dev for agent packs. Platform always receives exactly 30%. Enforced at the `BillingEngine` level. |
| **Security Review Pipeline** | See **Review Pipeline**. |

---

## Personal Mesh Context (DDD-011)

| Term | Definition |
|------|------------|
| **Personal Mesh** | The aggregate root (`PersonalMesh`) managing a single user's fleet of devices. Identified by `MeshId`. Max 10 devices. |
| **Mesh Device** | A device registered in the mesh (`MeshDevice`). Has a `DeviceType` (Phone/Tablet/Desktop/Laptop/Watch/Speaker/TV/Hub), zone assignment, and capability profile. |
| **Device Zone** | The swarm zone a mesh device is assigned to: A-Mobile, A-Desktop, B-Cloud, C-Edge, D-Browser, E-Mesh. Determines what work the device receives. |
| **Sync Protocol** | The state synchronization mechanism (`SyncProtocol`) for keeping mesh devices in sync. Uses state vectors and conflict-free merging. |
| **Discovery Service** | The mechanism for finding new devices (`DiscoveryService`). Supports mDNS, BLE, and manual registration. |
| **Failover Policy** | Rules (`FailoverPolicy`) governing automatic degradation when devices disconnect. Levels: Normal → Degraded → Critical → Emergency. |
| **Mesh Degradation** | The state of reduced capability when devices are offline (`MeshDegradation`). Evaluated automatically based on fleet health. |

## Federated Learning Context (DDD-012)

| Term | Definition |
|------|------------|
| **Federation Cycle** | The aggregate root (`FederationCycle`). A weekly cycle that progresses through 4 phases: Collecting → Aggregating → Distributing → Completed. |
| **Contribution** | An anonymized pattern set (`Contribution`) submitted by a device during the Collecting phase. Uses pseudonymous keys, never linkable to user identity. |
| **Anonymizer** | The on-device anonymization pipeline (`Anonymizer`). Strips PII, buckets emotions into 5 levels, applies Laplace noise (ε=1.0). All anonymization happens before data leaves the device. |
| **Aggregator** | The cloud-side aggregation engine (`Aggregator`). Requires minimum 1000-user contributions before publishing any pattern. Prevents re-identification. |
| **Distribution** | A LoRA update package (`Distribution`) built from aggregated patterns. Distributed to all participating devices after aggregation. |
| **Bootstrap** | The process of seeding a new user's SONA PatternBank from federated patterns (`Bootstrap`). Gives new users a baseline without waiting for personal data. |
| **Aggregation Threshold** | The minimum number of unique user contributions (1000) required before patterns can be aggregated and distributed. A privacy invariant. |

## Subscription Billing Context (DDD-013)

| Term | Definition |
|------|------------|
| **Subscription** | The aggregate root (`Subscription`). Represents a user's billing relationship. Has a tier, status, and billing period. |
| **Subscription Tier** | One of 6 tiers (`SubscriptionTier`): Free, Personal ($9.99), Pro ($19.99), Family ($29.99), Developer ($49.99), Enterprise (custom). Each tier has specific `TierLimits` and `TierFeatures`. |
| **Family Group** | A shared subscription (`FamilyGroup`) for the Family tier. Max 6 members. Each member has a `FamilyRole` (Owner/Adult/Child) and `PrivacyBoundary`. |
| **Developer Account** | A publisher account (`DeveloperAccount`) for the Developer tier. Earns 70/30 revenue split on marketplace agent sales. Payouts at $50 threshold. |
| **Tier Capability Token** | A token (`TierCapabilityToken`) derived from the subscription tier. Contains `TierCaveat` entries that enforce tier-specific limits (agent count, cloud burst, federation access). |
| **Tier Capability Enforcer** | The enforcement engine (`TierCapabilityEnforcer`). Derives capability tokens from tiers and validates feature access. Never hardcode tier checks — always go through the enforcer. |
| **Usage Metrics** | Per-period tracking (`UsageMetrics`) of API calls, agent spawns, storage, and inference tokens consumed. Used for billing and tier limit enforcement. |

## NAPI & WASM Binding Contexts (ADR-020, ADR-021)

| Term | Definition |
|------|------------|
| **NAPI Kernel** | The Node.js binding facade (`NapiKernel`). Exposes full kernel syscall surface as a native addon. Feature-gated behind `napi`. |
| **WASM Kernel** | The browser/WebView binding (`rlmx-wasm`). Exposes a reduced syscall subset suitable for Zone D browser agents. Feature-gated behind `wasm`. |

---

## Anti-Patterns (terms to avoid)

| Avoid | Use Instead | Reason |
|-------|-------------|--------|
| "task" (when meaning agent work) | "syscall" or "query" | "Task" is overloaded. In RLMX, the kernel dispatches syscalls, not tasks. Agents execute queries. The `task` field on `Process` is a description string, not a first-class entity. |
| "node" (when meaning agent) | "agent" or "swarm node" | "Node" refers exclusively to a `SwarmNode` in the cluster. Agents run on nodes but are not nodes. |
| "model" (ambiguous) | "strategy", "model tier", "GGUF model", or "model spec" | Specify which model concept: the inference strategy, the capability tier, the physical GGUF file, or the `ModelSpec` configuration. |
| "token" (ambiguous) | "capability token" or "bearer token" or "auth token" | "Capability token" = `CapabilityToken` (kernel security). "Bearer token" = HTTP auth token for MCP. "Auth token" = `McpConfig.auth_token`. |
| "branch" (ambiguous) | "COW branch" or "git branch" | "COW branch" = `BranchManager` RVF fork. Use "git branch" for version control. |
| "memory" (ambiguous) | "vector memory", "memory region", "RAM", or "SONA pattern bank" | "Vector memory" = `MemoryRegion` (kernel). "SONA pattern bank" = `PatternBank` (cognitive). "RAM" = physical memory. |
| "chain" (ambiguous) | "witness chain" or "proof chain" | Always qualify. "Witness chain" = `WitnessChain` (append-only log). "Proof chain" is informal shorthand for the same thing. |
| "router" (ambiguous) | "TinyDancer router", "Router agent", or "HTTP router" | "TinyDancer" = FastGRNN neural router. "Router agent" = `AgentType::Router`. "HTTP router" = MCP server's request dispatcher. |
| "plugin" (as verb) | "extend" or "register" | Use "register a domain plugin" or "extend via `DomainPlugin` trait". |
| "run" (vague) | "dispatch", "execute", "spawn", or "generate" | Be specific: "dispatch a syscall", "execute an experiment", "spawn an agent", "generate text". |
| "device" (ambiguous) | "mesh device" or "swarm node" | "Mesh device" = physical hardware in the user's fleet. "Swarm node" = logical participant in consensus. A mesh device runs one or more swarm nodes. |
| "tier" (ambiguous) | "subscription tier" or "model tier" | "Subscription tier" = billing plan (Free/Personal/Pro/etc). "Model tier" = inference capability (Small/Medium/Remote). |
| "cycle" (ambiguous) | "federation cycle" or "circadian cycle" | "Federation cycle" = weekly learning aggregation. "Circadian cycle" = daily scheduling pattern in NervousSystem. |
| "family" (ambiguous) | "family group" or "agent family" | "Family group" = billing Family tier sharing. "Agent family" = related agent types (never used formally). |
