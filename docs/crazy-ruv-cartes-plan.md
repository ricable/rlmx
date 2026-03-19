# RuVix — One Voice, Millions of Agents

> **You speak. Millions of AI agents act. Your life changes.**

---

## 1. THE VISION — One Voice, Millions of Agents

You hold your phone to your mouth. You say what you need. Within milliseconds, your words decompose into intents — finance, health, legal, shopping, career, travel — and scatter across millions of specialized AI agents running on your phone, your laptop, your home hub, and a global marketplace of narrow experts. They execute in parallel. They coordinate via swarm consensus. They stream results back — voice narrating key findings while visual cards, charts, and haptic pulses render simultaneously on your screen. In under 10 seconds, you have answers that would have taken a team of human experts days.

**This is not Siri.** Siri is one mediocre generalist that forgets you exist between requests.

**This is not a chatbot.** Chatbots wait for you to type. They give you text. They have no hands.

**This is a personal army of narrow experts** — each one laser-focused on its domain, each one learning YOUR specific patterns via micro-LoRA, each one getting smarter daily from federated knowledge across millions of users, each one capable of ACTING on your behalf — negotiating bills, scheduling appointments, filing claims, comparing prices, monitoring your health, protecting your privacy — while you live your life.

**RuVix is the cognitive immune system for modern life, activated by your voice.**

### The Great Equalizer

A minimum-wage worker holding a $200 Android phone gets the same intelligence as a Fortune 500 CEO's staff of executive assistants, financial advisors, lawyers, and health consultants — combined. The federated learning from millions of negotiations, millions of health trajectories, millions of financial optimizations means the newest user benefits from the collective intelligence of everyone who came before.

**Every human with a phone gets a personal staff of millions. For free.**

### Why Now

Three breakthroughs converged:

1. **On-device inference crossed the usability threshold.** Whisper-tiny runs STT in <100ms on a 2021 phone. TinyLlama 1.1B Q4 runs at 15 tok/s on a Raspberry Pi. WebGPU unlocks GPU compute in the browser. The phone is no longer a dumb terminal — it's an inference engine.

2. **Micro-LoRA + EWC++ made personal learning viable.** Rank-4 LoRA deltas personalize a base model to YOUR preferences in <50 interactions, consuming <5MB storage. EWC++ ensures learning your food preferences doesn't erase your clothing knowledge. Personal AI is finally personal.

3. **Swarm coordination matured.** PBFT, Raft, and Gossip consensus protocols running on commodity hardware let millions of agents coordinate in real-time. DAG consensus handles multi-domain decisions without deadlocking. The infrastructure exists to run an army.

### Market Timing

- 4.5B smartphones globally, each sitting idle 95% of the time — untapped compute
- AI assistant market: $14B today, projected $120B by 2030 — but current products (Siri, Alexa, Google) have flatlined in satisfaction scores since 2020
- Agent frameworks exploding (LangChain 10M+ downloads/mo) — developers are ready
- Voice is the fastest-growing interaction modality: 58% of US consumers use voice assistants weekly, but 72% report "frustration" — the market is primed for disruption

---

## 2. THE EXPERIENCE — What Day One Feels Like

### Installation to Value: 24 Hours

You install RuVix. You tap the mic button.

**"Set me up."**

Within 24 hours, without you doing anything else:

- **Your Bill Negotiator agent** has analyzed your electricity, internet, phone, and insurance bills, identified you're overpaying by **$847/year**, and drafted negotiation scripts. In 48 hours, it's on autopilot — calling providers via voice API, negotiating on your behalf, saving you money while you sleep.

- **Your Health Guardian agent** ingested your last 3 years of lab results (photos → multimodal OCR → structured data), built your personal health trajectory, and flagged that your vitamin D trend is heading toward deficiency 4 months from now. It adjusted your meal planning agent's suggestions and added a reminder to discuss with your doctor at your next visit (which your Calendar agent already scheduled based on the recommended interval).

- **Your Legal Shield agent** read every Terms of Service you clicked "accept" on this month, flagged 3 that contain arbitration clauses that waive your rights, and one that auto-renews in 6 days at a higher rate. It drafted a cancellation email and is waiting for your "send it" voice command.

- **Your Finance Pilot agent** detected that your savings account earns 0.01% APY, found 4 alternatives at 4.5%+, compared FDIC coverage and withdrawal limits, and presents a one-click migration plan. It coordinates with your Tax agent to understand the implications. It whispers through your earbuds: *"I found you $312 in annual interest. Want me to show you the options?"*

- **Your Shopping Sentinel** is running continuously — every purchase you make is compared against historical prices across 200+ retailers in <100ms. It vibrates your phone gently: *"That laptop drops 30% in 12 days. I'll remind you."* Predicted from 3 years of price history across federated user data.

- **47 other agents** are running silently, each one a narrow expert in its domain, each one learning YOUR specific patterns, each one getting smarter from anonymized knowledge federated across millions of other users.

All of this runs **on your own devices**. Your phone runs WASM agents. Your laptop runs native agents. Your $35 Raspberry Pi home hub runs the full kernel with edge inference. No data leaves your devices unless you explicitly escalate to cloud. **Your agents are YOURS.**

### Voice-First Interaction Examples

**Morning briefing** — Your alarm goes off. RuVix speaks:

> *"Good morning. Three things: your Comcast bill was reduced to $62, saving you $38/month — that's done. You have a dentist appointment at 2pm that conflicts with your standup — I moved the standup to 3pm and notified your team. And Whole Foods has organic avocados at $0.89, 40% below your usual price — want me to add them to your delivery?"*

**You say:** "Yes on the avocados. Cancel the dentist, reschedule next week."

Two intents. Two agents. Both execute before you finish brushing your teeth.

**Commute query** — Driving to work, you tap the steering wheel button:

**"What should I know today?"**

> *"Traffic is clear, 22 minutes. Your portfolio is up 1.2% — your auto-rebalancer is holding. You got 47 emails overnight — 3 need your attention, I drafted replies for all three. Your manager shared a doc about the Q3 roadmap — I summarized the parts that affect your team. And your daughter's school sent a permission slip due Friday — I filled it out, just needs your signature."*

Seven agents contributed to that 15-second briefing. Zero screens touched.

**The Mega-Example: "I want to move to Paris"**

You hold your phone up at dinner. You say:

**"I want to move to Paris."**

RuVix responds within 3 seconds:

> *"Starting Paris relocation analysis. I'm activating 47 agents across 9 domains. You'll see real-time progress on your screen."*

Your phone screen fills with a grid of progress cards:

| Domain | Agent(s) | Status |
|--------|----------|--------|
| 🛂 Visa & Immigration | Visa Analyst, Document Prep | Researching work permit requirements for your nationality... |
| 🏠 Housing | Real Estate Scout, Neighborhood Analyst | Scanning 12 arrondissements matching your budget + commute... |
| 💼 Career | Job Market Analyst, Resume Localizer, LinkedIn Optimizer | Mapping your skills to Paris job market, translating CV... |
| 🏫 Schools | School Researcher, Enrollment Advisor | Finding international schools near candidate neighborhoods... |
| 🏥 Healthcare | Insurance Comparator, Provider Finder | Comparing CPAM vs. private insurance for your family size... |
| 🏦 Banking | Account Migrator, Tax Analyst, Currency Optimizer | Modeling tax treaty implications, finding EUR accounts... |
| 🗣️ Language | Learning Path Designer, Tutor Matcher | Assessing your French level, designing 90-day intensive... |
| 📦 Logistics | Moving Coordinator, Customs Advisor | Estimating shipping costs for your inventory, customs reqs... |
| 📋 Life Admin | Utility Setup, Phone/Internet, Mail Redirect | Queuing EDF electricity, Free Mobile, La Poste redirect... |

Every card updates in real-time. The first results arrive in under 10 seconds. The full analysis completes in under 2 minutes. RuVix narrates key findings through your earbuds while visual cards render:

> *"Good news — your US passport qualifies for a VLS-TS long-stay visa. With your salary, you qualify for the Talent Passport track — that's the fast one. For housing, the 11th arrondissement hits your sweet spot: 15-minute bike commute, two international schools within 1km, and rent is 23% below your current SF apartment. I found 6 listings. Want me to schedule virtual tours?"*

**47 agents. 9 domains. One voice command. Two minutes.**

### The "1000 Lifts a Day" Model

Every agent interaction that saves you time, money, health risk, or cognitive load is a "lift." Average user gets:

| Swarm | Lifts/Day | Examples |
|-------|-----------|----------|
| Calendar/Email/Task | ~200 | Auto-triage, auto-schedule, auto-respond to routine emails, meeting prep summaries |
| Shopping/Finance | ~50 | Price alerts, bill monitoring, investment rebalancing signals, subscription audit |
| Health/Wellness | ~30 | Meal suggestions, activity nudges, medication reminders personalized to YOUR chronotype |
| Information Shield | ~500+ | Spam filtered, scams detected, privacy violations caught, dark patterns blocked, TOS flagged |
| Home/IoT | ~200+ | Energy optimization, appliance scheduling, security monitoring, package tracking |

**Total: 1,000+ micro-improvements per day**, each one invisible, each one compounding. After 30 days, users report the feeling isn't "I have an AI assistant" — it's **"I have superpowers."**

### Multimodal Response System

RuVix doesn't just talk back. Every response is multimodal:

- **Voice**: TTS with domain-specific personas (warm for health, authoritative for legal, excited for savings) — streams the first sentence while generating the rest
- **Visual cards**: Structured data rendered as swipeable cards with charts, comparisons, and action buttons
- **Haptics**: Gentle pulse for savings found, double-tap for urgent alerts, long buzz for blocked scams
- **Ambient indicators**: Lock screen widget glows green when agents are saving you money, amber when something needs attention

---

## 3. VOICE-FIRST ARCHITECTURE

### The Voice Pipeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        VOICE-FIRST PIPELINE                             │
│                                                                         │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────────────┐    │
│  │ Always-  │   │ On-Device│   │ Intent   │   │  Multi-Intent    │    │
│  │ Listening│──▶│   STT    │──▶│ Classify │──▶│  Decomposition   │    │
│  │   VAD    │   │ (Whisper)│   │(TinyDancr│   │                  │    │
│  │ 500K CNN │   │  Q4 GGUF │   │ 18→32→5) │   │ "move to Paris"  │    │
│  └──────────┘   └──────────┘   └──────────┘   │  → 9 domains     │    │
│       │                                        │  → 47 agents     │    │
│       │              ┌─────────────────────────┘                  │    │
│       ▼              ▼                                             │    │
│  ┌──────────┐   ┌──────────────────────────────────────────┐      │    │
│  │ Wake Word│   │  Strategy::Swarm { scatter, gather, to } │      │    │
│  │ (custom, │   │  scatter: [ZoneA, ZoneB, ZoneC, ZoneD]   │      │    │
│  │ on-device│   │  gather: GatherStrategy::All             │      │    │
│  │  only)   │   │  timeout: 120_000ms                      │      │    │
│  └──────────┘   └──────────────────────────────────────────┘      │    │
│                              │                                     │    │
│                              ▼                                     │    │
│                 ┌────────────────────────┐                         │    │
│                 │  Streaming Response    │                         │    │
│                 │  ┌────────┐ ┌────────┐ │                         │    │
│                 │  │ TTS    │ │ Visual │ │                         │    │
│                 │  │ Voice  │ │ Cards  │ │                         │    │
│                 │  │ Stream │ │ Stream │ │                         │    │
│                 │  └────────┘ └────────┘ │                         │    │
│                 │  ┌────────┐ ┌────────┐ │                         │    │
│                 │  │Haptic  │ │Progress│ │                         │    │
│                 │  │Pulses  │ │  Cards │ │                         │    │
│                 │  └────────┘ └────────┘ │                         │    │
│                 └────────────────────────┘                         │    │
└─────────────────────────────────────────────────────────────────────────┘
```

### On-Device Speech-to-Text

Whisper-tiny Q4 via existing `rlmx-ruvllm` TieredEngine infrastructure:

- **39M parameters** quantized to 4-bit → ~25MB on device
- **<100ms** end-to-end latency on modern phones (A15+, Snapdragon 8 Gen 1+)
- Runs through `TieredEngine` escalation: phone STT (fast, 95% accuracy) → laptop STT (medium, 98%) → cloud STT (full Whisper-large, 99.5%) for critical commands
- Language detection automatic — supports 99 languages via federated language packs

### Always-Listening Voice Activity Detection

500K parameter CNN on `ruv-neural-signal` (already in dependency tree):

- **Energy-based + spectral feature** VAD running at <0.1% CPU on background thread
- Triggers on human speech patterns, filters ambient noise, TV, music
- **Battery impact**: <2% per day in always-listening mode (hardware-accelerated DSP on modern SoCs)
- **Privacy guarantee**: audio is processed in a 3-second rolling buffer. No audio is stored. No audio leaves device. VAD outputs a binary "speech detected" signal — only then does STT activate.

### Wake Word

- User-configurable keyword (default: "Hey RuVix" or just tap-to-talk)
- Runs entirely on-device via small keyword-spotting model (20K params)
- **Zero audio transmitted** until wake word detected
- Optional: no wake word needed — physical button trigger (volume button shortcut, AirPod squeeze, Apple Watch raise-to-speak)

### Intent Classification — TinyDancerRouter Expansion

The existing `TinyDancerRouter` (FastGRNN 14→32→5) expands from 14 to **18 input dimensions**:

```
Original 14 dims: [query_complexity, token_count, domain_signals(5), battery_level,
                    network_type, time_of_day, model_confidence_history(3), memory_pressure]

New 18 dims:      [+ speaker_confidence,    // STT confidence score (0-1)
                    + emotion_valence,       // detected emotion (-1 to 1)
                    + urgency_score,         // speech rate + keyword triggers
                    + ambient_noise_level]   // background noise dB estimate
```

The router now makes **voice-aware routing decisions**:

- Low STT confidence + high urgency → escalate to cloud STT before routing
- High emotion valence (anger/stress) → prioritize speed over accuracy, route to fastest available engine
- High ambient noise → increase TTS volume, simplify response to key facts only
- Low battery + voice query → compress response, skip visual cards, voice-only mode

Online SGD updates weights within 100 examples. After a week, the router knows: *"This user always asks complex medical questions at 9pm — pre-cache the Health Guardian agent and route to Medium tier."*

### Multi-Intent Decomposition

One utterance → multiple agent domains via `Strategy::Swarm`:

```rust
// "Cancel my dentist, reschedule next week, and add avocados to my delivery"
// Decomposes to 3 intents, 3 agents, executed in parallel:

Strategy::Swarm {
    scatter_zones: vec![ZoneA, ZoneD],  // phone + laptop agents
    gather_strategy: GatherStrategy::All,
    timeout_ms: 5_000,
    intents: vec![
        Intent { domain: "calendar", action: "cancel", entity: "dentist_apt_2pm" },
        Intent { domain: "calendar", action: "reschedule", entity: "dentist", params: "next_week" },
        Intent { domain: "shopping", action: "add_item", entity: "organic_avocados" },
    ]
}
```

### TTS with Voice Personas

Per-domain voice personality, streaming first sentence while generating rest:

| Domain | Persona | Style |
|--------|---------|-------|
| Finance | Confident advisor | Calm, precise, numbers emphasized |
| Health | Warm practitioner | Empathetic, measured, reassuring |
| Legal | Authority | Direct, clear, no ambiguity |
| Shopping | Excited friend | Upbeat when savings found, matter-of-fact otherwise |
| Calendar | Efficient assistant | Brief, action-oriented |
| Emergency | Alert | Urgent, clear, repetitive for comprehension |

TTS runs on-device for standard responses (<50 tokens). Cloud TTS for long-form narration (Paris analysis). Streaming: first sentence plays within 200ms of generation start.

### New SwarmEvent Variants for Voice

```rust
enum SwarmEvent {
    // Existing 9 variants...
    NodeJoined { ... },
    NodeLeft { ... },
    ConsensusReached { ... },
    TaskAssigned { ... },
    TaskCompleted { ... },
    HealthCheck { ... },
    MetricsUpdate { ... },
    SandboxEvent { ... },
    FleetEvent { ... },

    // New voice/multimodal variants:
    VoiceChunk {
        session_id: Uuid,
        audio_segment: Vec<u8>,    // opus-encoded chunk
        transcript: String,         // real-time partial transcript
        confidence: f32,
        is_final: bool,
    },
    AgentProgress {
        task_id: Uuid,
        agent_type: AgentType,
        domain: String,
        progress_pct: f32,          // 0.0 - 1.0
        status_text: String,        // "Scanning 12 arrondissements..."
        eta_ms: Option<u64>,
    },
    MultimodalResponse {
        session_id: Uuid,
        voice_chunk: Option<Vec<u8>>,   // streaming TTS audio
        visual_card: Option<CardData>,   // structured visual data
        haptic_pattern: Option<HapticPattern>,
        is_final: bool,
    },
}
```

### Conversation Memory

SONA PatternBank stores voice sessions with temporal-weighted VecSearch for recall:

- Every voice interaction recorded as `(transcript_embedding, intent, outcome, timestamp)`
- Temporal decay: recent interactions weighted 3x vs. 30-day-old
- Cross-session context: "Remember when I asked about Paris?" → VecSearch finds the session, restores full context
- Speaker identification: multi-user households distinguished by voice embedding (on-device, no cloud)

---

## 4. THE PHONE AS COMMAND CENTER

### Zone Model Inversion

The original architecture treated the phone as Zone D — the lowest priority, burst-only compute pool. **We invert this.**

```
BEFORE (infrastructure-first):              AFTER (voice-first):
┌──────────┐                                ┌──────────────────────┐
│ Zone A   │ Laptop (primary)               │ Zone A-Mobile        │ PHONE (primary)
│ Zone B   │ Cloud (inference)        →     │ Zone A-Desktop       │ Laptop (secondary)
│ Zone C   │ Home Hub (edge)                │ Zone B               │ Cloud (burst)
│ Zone D   │ Phone (burst)                  │ Zone C               │ Home Hub (sentinel)
└──────────┘                                └──────────────────────┘
```

**The phone is where the user IS.** It's the microphone. It's the screen. It's the always-with-you device. Everything else serves the phone.

### On-Device Agent Runtime

A lightweight **Coordinator agent variant** runs on phone:

- WASM kernel via `@ruvector/ruvllm-wasm` v2.0.2 — existing, production-ready
- Core inference primitives: WebGPU matmul, HNSW search, SONA adaptation
- **5 always-on agents** running with zero network required:
  1. **Email Triage** — classifies and prioritizes incoming email via on-device embedding
  2. **Calendar** — schedule management, conflict detection, smart suggestions
  3. **Weather+Commute** — combines weather API cache with learned commute patterns
  4. **News Digest** — personalized news ranking from cached RSS + learned preferences
  5. **Shopping Comparison** — price history lookup from federated cache

### Always-On Foreground Service

| Platform | Implementation | Battery Impact |
|----------|---------------|----------------|
| iOS | Background Audio session + BGProcessingTask | <3%/day (tested) |
| Android | Foreground Service with persistent notification | <2%/day (tested) |
| Both | Hardware-accelerated VAD bypasses main CPU | Negligible when idle |

### Lock Screen Widgets

Three widgets available:

1. **Agent Status**: "12 agents active · 3 tasks pending · Last: Saved $38 on Comcast"
2. **Money Saved**: "$847 saved this year" — big number, always updating, green pulse on new savings
3. **Quick Voice**: Tap-to-speak button directly from lock screen (iOS Live Activity / Android Widget)

### Push Notification Orchestration

3 priority tiers with ML-based fatigue prevention:

| Priority | Vibration | Sound | Example |
|----------|-----------|-------|---------|
| **Critical** | Strong | Alert tone | "Your credit card was used in a city you're not in" |
| **Actionable** | Gentle | Subtle chime | "Your laptop just dropped 30% — buy now?" |
| **Informational** | None | None | Batched into daily briefing: "3 subscriptions renewed today, all within budget" |

**Fatigue prevention**: ML model trained on user's notification response patterns. If user hasn't opened the last 5 Actionable notifications, auto-downgrade to Informational. If user always opens Shopping alerts within 1 minute, upgrade to Actionable. Learns per-user within 2 weeks.

### Offline-First Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    PHONE (Offline Mode)                       │
│                                                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐  │
│  │ 5 Free      │  │ SONA        │  │ Federated Cache     │  │
│  │ On-Device   │  │ PatternBank │  │ (50K patterns,      │  │
│  │ Agents      │  │ (personal)  │  │  updated weekly)    │  │
│  └─────────────┘  └─────────────┘  └─────────────────────┘  │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │ Outbox Queue: cloud-burst requests queued until online  │  │
│  │ → Paris analysis queued → executes when WiFi detected   │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

- 5 free agents run with **zero network**
- Complex queries queue in outbox, execute when connectivity restored
- Federated pattern cache updated weekly over WiFi (background, battery-aware)
- User always sees value, even in airplane mode

### Background Agent Execution

| Platform | API | Constraints | RuVix Strategy |
|----------|-----|-------------|----------------|
| iOS | BGProcessingTask | 30s-10min window, power + WiFi | Batch SONA training, federated sync, bill monitoring |
| iOS | BGAppRefreshTask | 30s window | Quick agent checks, notification prep |
| Android | WorkManager | 10min+ for long-running | Full agent execution, model updates |
| Android | Foreground Service | Unlimited (with notification) | Always-on VAD + coordinator |

---

## 5. VOICE → MILLIONS OF AGENTS — The Fan-Out Pipeline

### The Complete Flow

```
User speaks into phone mic
        │
        ▼
┌──────────────────┐
│  VAD (500K CNN)  │ ── silence → sleep
│  Speech detected │
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│  STT (Whisper    │    "I want to move to Paris"
│  Q4, on-device)  │    confidence: 0.97
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ TinyDancerRouter │    Input: 18-dim feature vector
│ (FastGRNN        │    Output: strategy + zone routing
│  18→32→5)        │    Decision: Strategy::Swarm (multi-domain)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Intent           │    Decomposition:
│ Decomposer       │    → visa (legal)
│                  │    → housing (real estate)
│                  │    → career (employment)
│                  │    → schools (education)
│                  │    → healthcare (insurance)
│                  │    → banking (finance)
│                  │    → language (learning)
│                  │    → logistics (moving)
│                  │    → life_admin (utilities)
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Coordinator      │    ProcessFork × 47
│ Agent            │    Spawn specialized agents per domain
│ (Zone A-Mobile)  │    Time-bounded capability tokens
└────────┬─────────┘
         │
    ┌────┼────┬────┬────┬────┬────┬────┬────┐
    ▼    ▼    ▼    ▼    ▼    ▼    ▼    ▼    ▼
  Visa  House  Job School Health Bank  Lang  Move  Admin
  Agent Agent Agent Agent Agent Agent Agent Agent Agent
  (×2)  (×3)  (×4)  (×2)  (×3)  (×4)  (×2)  (×3)  (×2)
    │    │    │    │    │    │    │    │    │
    └────┼────┴────┴────┴────┴────┴────┴────┘
         │
         ▼
┌──────────────────┐
│ Progress         │    SwarmEvent::AgentProgress streams
│ Streaming        │    Real-time cards on phone screen
│                  │    Per-domain completion %
└────────┬─────────┘
         │
         ▼
┌──────────────────┐
│ Gather &         │    GatherStrategy::All — wait for all domains
│ Synthesize       │    Rank by actionability
│                  │    Generate narrative summary
└────────┬─────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌────────┐ ┌────────┐
│  TTS   │ │ Visual │
│ Stream │ │ Cards  │
│ (voice)│ │(screen)│
└────────┘ └────────┘
```

### The "Move to Paris" Walkthrough — 47 Agents, 9 Domains

**T+0s**: User says "I want to move to Paris."

**T+0.3s**: STT completes. Intent classified as multi-domain life event. Router selects `Strategy::Swarm` with `GatherStrategy::All`, 120s timeout.

**T+0.5s**: Coordinator spawns 47 agents across 9 domains via `ProcessFork`. Each gets a time-bounded capability token (24-hour expiry, domain-scoped permissions):

```
Domain: Visa & Immigration (2 agents)
├── Visa Analyst         — permissions: [web_search, read_user_nationality, read_employment]
│                          zone: B (cloud, needs current immigration databases)
│                          model: Medium (complex legal reasoning)
└── Document Prep        — permissions: [read_user_documents, generate_document]
                           zone: A-Mobile (template-based, runs on phone)

Domain: Housing (3 agents)
├── Real Estate Scout    — permissions: [web_search, read_user_budget, read_user_family_size]
│                          zone: B (scrapes listing sites)
├── Neighborhood Analyst — permissions: [web_search, read_user_preferences]
│                          zone: B (cross-references crime, schools, transit data)
└── Rent Calculator      — permissions: [read_financial_data, read_user_income]
                           zone: A-Desktop (financial modeling on laptop GPU)

Domain: Career (4 agents)
├── Job Market Analyst   — permissions: [web_search, read_user_resume, read_user_skills]
├── Resume Localizer     — permissions: [read_user_resume, generate_document]
├── LinkedIn Optimizer   — permissions: [read_social_profile, generate_content]
└── Salary Benchmarker   — permissions: [web_search, read_user_income]

Domain: Schools (2 agents)
├── School Researcher    — permissions: [web_search, read_user_children_ages]
└── Enrollment Advisor   — permissions: [web_search, read_user_location_prefs]

Domain: Healthcare (3 agents)
├── Insurance Comparator — permissions: [web_search, read_user_health_history]
├── Provider Finder      — permissions: [web_search, read_user_location_prefs]
└── Prescription Transfer— permissions: [read_user_medications]

Domain: Banking (4 agents)
├── Account Migrator     — permissions: [read_financial_data, web_search]
├── Tax Treaty Analyst   — permissions: [read_financial_data, read_user_nationality]
├── Currency Optimizer   — permissions: [read_financial_data]
└── FATCA/FBAR Advisor   — permissions: [read_financial_data, read_user_nationality]

Domain: Language (2 agents)
├── Level Assessor       — permissions: [read_user_language_history, assess_proficiency]
└── Learning Planner     — permissions: [web_search, read_user_schedule]

Domain: Logistics (3 agents)
├── Moving Coordinator   — permissions: [web_search, read_user_inventory]
├── Customs Advisor      — permissions: [web_search, read_user_nationality]
└── Timeline Planner     — permissions: [read_all_domain_outputs] // cross-domain

Domain: Life Admin (2 agents)
├── Utility Setup        — permissions: [web_search]
└── Address Management   — permissions: [read_user_address, generate_document]
```

**T+3s**: First results arrive. Phone screen populates progress cards. TTS begins:

> *"Good news — your US passport qualifies for a VLS-TS long-stay visa..."*

**T+30s**: 70% of agents complete. Housing domain returns 6 listings in 11th arrondissement. School domain returns 2 international schools within 1km. RuVix narrates key findings while cards render.

**T+120s**: All 47 agents complete. Coordinator synthesizes a comprehensive relocation plan with:
- Executive summary (voice narrated, 60 seconds)
- 9 domain cards with actionable next steps
- Estimated timeline: 4-6 months
- Estimated cost: $8,200 - $12,400 (broken down by domain)
- "Start my move" button that begins executing the plan

**All coordination via**:
- `qudag-dag`: DAG consensus on cross-domain dependencies ("can't enroll kids before knowing neighborhood")
- `ruvector-raft`: Consistent state across phone + laptop (both see same progress)
- `midstreamer-quic`: Sub-millisecond sync on home network

**After analysis completes**:
- Temporary agents terminate (capability tokens expire in 24h)
- Learned patterns (visa pathways, housing market data, school rankings) → federated back to help the next person who says "I want to move to Paris"

---

## 6. AGENT MARKETPLACE — App Store for Life

### 12 Life-Domain Categories

| Category | Example Agents | User Need |
|----------|---------------|-----------|
| 💰 Finance | Bill Negotiator, Tax Advisor, Investment Rebalancer, Debt Optimizer, Subscription Auditor | "Save me money, grow my wealth" |
| 🏥 Health | Health Guardian, Meal Planner, Fitness Coach, Medication Manager, Mental Health Check-in | "Keep me healthy, catch problems early" |
| ⚖️ Legal | Legal Shield, Contract Reviewer, Small Claims Advisor, IP Monitor, Dispute Mediator | "Protect my rights, understand fine print" |
| 💼 Career | Resume Optimizer, Interview Coach, Salary Negotiator, Skill Gap Analyzer, Network Builder | "Advance my career, earn more" |
| 📚 Education | Learning Path Designer, Tutor Matcher, Homework Helper, Certification Tracker, Study Optimizer | "Learn faster, learn smarter" |
| 🏠 Home | Energy Optimizer, Maintenance Predictor, Smart Home Coordinator, Home Value Tracker, Renovation Planner | "Run my home efficiently" |
| 🛒 Shopping | Price Sentinel, Deal Hunter, Return Optimizer, Warranty Tracker, Counterfeit Detector | "Never overpay again" |
| ✈️ Travel | Trip Planner, Flight Monitor, Hotel Optimizer, Visa Advisor, Currency Optimizer, Packing Assistant | "Travel smarter, save on every trip" |
| 👥 Social | Event Coordinator, Gift Recommender, Relationship Reminder, Social Media Optimizer | "Strengthen my relationships" |
| 🏛️ Government | Tax Filer, Benefits Finder, Permit Navigator, DMV Automator, Vote Information | "Navigate bureaucracy painlessly" |
| 🚗 Automotive | Maintenance Tracker, Insurance Optimizer, Gas Price Monitor, Recall Alerter, Resale Value Tracker | "Minimize car costs" |
| 🐾 Pet | Vet Appointment Manager, Pet Health Monitor, Food Optimizer, Pet Insurance Advisor, Breed-Specific Care | "Best life for my pets" |

### Built on RVF Container Format

Every agent is packaged as an RVF container using the existing format:

```rust
// From rlmx-rvf
struct RvfContainer {
    segments: Vec<Segment>,
}

enum SegmentType {
    Code,      // WASM bytecode for the agent logic
    Model,     // Quantized model weights (LoRA deltas or full small model)
    Config,    // Agent configuration, permissions, UI templates
    Witness,   // Ed25519 signed proof chain — publisher + marketplace co-signature
}
```

### Third-Party Developer Program

```
┌────────────────────────────────────────────────────────────────────┐
│                    DEVELOPER JOURNEY                                │
│                                                                     │
│  1. Install SDK        cargo install rlmx-agent-sdk                │
│  2. Scaffold           rlmx-agent new my-agent --domain finance    │
│  3. Develop            rlmx-agent dev  (hot-reload local testing)  │
│  4. Test               rlmx-agent test (sandboxed execution)       │
│  5. Security Audit     Automated: capability analysis, data flow    │
│                        verification, fuzzing                        │
│  6. Submit             rlmx-agent publish                           │
│  7. Review             24-48h automated + human review              │
│  8. Live               Listed in marketplace with analytics         │
│                                                                     │
│  Revenue: 70% developer / 30% platform                             │
│  Payouts: Monthly, threshold $50, Stripe Connect                   │
└────────────────────────────────────────────────────────────────────┘
```

**SDK includes**:
- `rlmx-agent-sdk` crate with kernel syscall bindings
- WASM + native build targets
- Test harness with simulated user interactions
- SONA integration for personalization
- Template agents for each of the 12 domains
- Documentation site with tutorials, API reference, best practices

### Marketplace Scale Targets

| Timeframe | First-Party Agents | Third-Party Agents | Active Developers |
|-----------|-------------------|--------------------|-------------------|
| Launch | 50 | 0 | 0 |
| Year 1 | 100 | 200 | 500 |
| Year 2 | 150 | 1,000 | 2,500 |
| Year 3 | 200 | 5,000 | 10,000 |
| Year 5 | 250 | 10,000+ | 25,000+ |

### Celebrity & Influencer Agent Packs

Curated agent configurations from trusted voices:

- **"Dave Ramsey's Budget Swarm"** — 5 finance agents configured for zero-based budgeting with debt snowball
- **"Tim Ferriss' Productivity Stack"** — 8 agents for 4-hour workweek optimization, Pareto analysis on every task
- **"Dr. Andrew Huberman's Health Protocol"** — 6 health agents configured for his morning routine, supplement tracking, sleep optimization
- Revenue share: 50% creator / 30% platform / 20% base agent developer

### Agent Install Flow

```
User: "Install the Tax Advisor agent"
        │
        ▼
rvf-runtime: Download RVF container from marketplace
        │ (SegmentType::Code + ::Model + ::Config + ::Witness)
        │
rvf-crypto: Verify publisher's Ed25519 signature + marketplace co-signature
        │
        ▼
ruvix-cap: Derive capability token from user's root token
        │ Caveats: [read_financial_data, read_tax_history,
        │           NO_health, NO_social, expires_april_15]
        │
        ▼
ruvector-cognitive-container: Create isolated cognitive container
        │ Own SONA bank (empty), own memory region, own vector namespace
        │
        ▼
ruv-swarm-core: Agent discovers existing agents via gossip
        │ "I'm Tax Advisor. Who handles finance? Who handles employment?"
        │ Finance agent: "I'll share W-2 summaries via ProcessSend"
        │ Employment agent: "I'll share employer benefits data"
        │
        ▼
TinyDancerRouter: Router learns new strategy weights
        │ "Tax queries should route to this agent, confidence 0.9"
        │ Online SGD updates W1/W2 within 100 examples
        │
        ▼
AGENT IS LIVE: "Let me review your situation..."
        │
        ▼
ruvector-sona: Within 50 interactions, agent has learned:
        - User files jointly with spouse
        - Has 3 dependents (2 children + elderly parent)
        - Has rental income from 1 property
        - Uses standard deduction (but should itemize!)
        MicroLoRA delta adapts base tax model to THIS user
```

---

## 7. ADDICTIVE FREE TIER — So Good You Can't Leave

The free tier isn't a demo. It isn't crippled. **It's genuinely life-changing** — and that's the conversion strategy.

### Variable Reward Schedules

Borrowed from the best engagement science (Nir Eyal's Hook Model, adapted ethically for genuine value):

```
┌─────────────────────────────────────────────────────────────────┐
│                 VARIABLE REWARD SCHEDULE                         │
│                                                                   │
│  Trigger: Agent finds real savings                                │
│                                                                   │
│  Timing varies (unpredictable = dopamine):                        │
│  ├── Sometimes immediate: "Found $12 savings on your cable bill" │
│  ├── Sometimes delayed: "After 3 days of monitoring, your auto   │
│  │   insurance is $340/yr too high"                               │
│  └── Sometimes compounding: "This week's total savings: $127"    │
│                                                                   │
│  The reward is REAL (actual money saved, not points or badges)    │
│  The timing is VARIABLE (optimized for engagement, not deception) │
│  The value INCREASES over time (more data = more savings found)   │
└─────────────────────────────────────────────────────────────────┘
```

### Streak Mechanics

Daily engagement streaks that unlock real value, not just cosmetics:

| Streak | Reward | Psychology |
|--------|--------|-----------|
| 3-day | Badge: "Getting Started" | Commitment |
| 7-day | Temporary premium agent unlocked for 48h | Taste of value |
| 14-day | Choose 1 premium agent for 7 days | Investment |
| 30-day | Cosmetic: custom voice persona | Identity |
| 60-day | Priority federated learning updates | Tangible improvement |
| 90-day | **Free month of Plus** | Conversion trigger |
| 365-day | "Founding Member" badge + permanent 10% discount | Loyalty lock |

The 90-day streak → free Plus month is the keystone: by day 90, the user's SONA has learned so deeply that downgrading would feel like losing a part of themselves.

### Life Score Dashboard

A daily 0-100 score across 4 domains, displayed prominently:

```
┌─────────────────────────────────────────────┐
│           YOUR LIFE SCORE: 73/100            │
│           ▲ +4 from last week                │
│                                               │
│  💰 Finance   ████████░░  82  ▲+3            │
│  🏥 Health    ██████░░░░  61  ▼-2            │
│  ⏰ Time      ████████░░  78  ▲+8            │
│  🛡️ Safety    ███████░░░  71  ═              │
│                                               │
│  ─ ─ ─ ─ sparkline trends (30 days) ─ ─ ─   │
│  💡 "Your health score dipped because you    │
│     missed 2 medication reminders. Enable    │
│     voice reminders?"                         │
│                                               │
│  📈 Since joining: +18 points (+32%)         │
└─────────────────────────────────────────────┘
```

The Life Score is optimized to always have room for improvement, never feel discouraging, and always suggest a specific action to improve. It's the fitness tracker for your entire life.

### Money Saved Counter

The most powerful conversion tool in the app:

```
┌───────────────────────────────────┐
│     💰 $847 saved this year       │
│     $2.32 today so far            │
│     ▲ beating 73% of users        │
│                                    │
│     [See breakdown →]              │
└───────────────────────────────────┘
```

- **Always visible** in the app header
- **Lock screen widget** with live counter
- Updates in real-time when savings are found
- Cumulative since join date — **the number only goes up**
- Social context: "You're in the top 27% of savers in San Francisco"
- At Plus price of $4.99/mo ($60/yr), the counter showing $847 in savings = **14x ROI** visible to the user at all times

### Social Proof

Persistent social proof throughout the app:

- **Global**: "47.3M users saved $8.7B total" — in onboarding and footer
- **Local**: "12,342 users in San Francisco · Average savings: $1,247/yr"
- **Comparative**: "Users like you (30s, tech, SF) save an average of $1,891/yr"
- **Trending**: "Bill Negotiator just saved someone in your zip code $342 on their internet bill"

### Gamification — Agent Collection

```
┌─────────────────────────────────────────────────────────┐
│              MY AGENTS  (7 / 52 collected)               │
│                                                           │
│  ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐ ┌────┐     │
│  │ 📧 │ │ 📅 │ │ 🌤 │ │ 📰 │ │ 🛒 │ │ 💰 │ │ ⚖️ │     │
│  │Email│ │Cal │ │Wthr│ │News│ │Shop│ │Bill│ │Legl│     │
│  │ Lv5 │ │ Lv3│ │ Lv2│ │ Lv4│ │ Lv8│ │ Lv6│ │ Lv1│     │
│  └────┘ └────┘ └────┘ └────┘ └────┘ └────┘ └────┘     │
│                                                           │
│  ░░░░  ░░░░  ░░░░  ░░░░  ░░░░  ... (45 more to unlock)  │
│                                                           │
│  🏆 Achievements: 12/50                                   │
│  ├── "First Save" ✓     "Night Owl" ✓    "Polyglot" ░    │
│  ├── "Bill Slayer" ✓    "Health Nut" ░   "Tax Wizard" ░  │
│  └── "Privacy Pro" ✓    "Travel Pro" ░   "Family Hero" ░ │
│                                                           │
│  Level 14 ████████░░░░░░░░ (2,847 / 5,000 XP)           │
│                                                           │
│  Leaderboard (opt-in): #247 in San Francisco             │
└─────────────────────────────────────────────────────────┘
```

Agents level up as they learn your preferences. A Level 10 Shopping agent is noticeably better than Level 1 — it knows your sizes, brands, price sensitivity, and timing preferences. **Levels represent real capability improvement, not cosmetic progression.**

### Free Tier vs. Paid — The Conversion Ladder

| Feature | Free | Plus ($4.99/mo) | Pro ($14.99/mo) |
|---------|------|-----------------|-----------------|
| On-device agents | 5 | Unlimited | Unlimited |
| Cloud burst | None | 100/day | Unlimited |
| Agent marketplace | Browse only | Install & use | Install, use, & create |
| Voice interaction | Tap-to-talk | Always-listening | Always-listening + wake word |
| Daily briefing | Basic (3 items) | Full (unlimited) | Full + proactive insights |
| SONA learning | On-device only | + Federated | + Priority federated + export |
| Response mode | Voice or text | Voice + visual cards | Full multimodal + haptics |
| Streaks | Basic | Enhanced rewards | 2x XP, exclusive achievements |
| Life Score | 2 domains | 4 domains | 4 domains + historical analysis |

---

## 8. MONETIZATION — 7 Revenue Streams to $1B ARR

### Stream 1: Subscriptions (~$300M at scale)

```
┌─────────────┬───────────────┬──────────────────────────────────────────────────────┐
│    Tier     │     Price     │                    What You Get                       │
├─────────────┼───────────────┼──────────────────────────────────────────────────────┤
│ Free        │ $0            │ 5 on-device agents, tap-to-talk voice, basic Life    │
│             │               │ Score, community federated learning                   │
├─────────────┼───────────────┼──────────────────────────────────────────────────────┤
│ Plus        │ $4.99/mo      │ Unlimited agents, 100 cloud bursts/day, always-      │
│             │               │ listening, full marketplace, visual cards, 4-domain   │
│             │               │ Life Score, enhanced streaks                           │
├─────────────┼───────────────┼──────────────────────────────────────────────────────┤
│ Pro         │ $14.99/mo     │ Everything in Plus + unlimited cloud, agent creation  │
│             │               │ SDK, priority federated learning, full multimodal,    │
│             │               │ proactive insights, API access                        │
├─────────────┼───────────────┼──────────────────────────────────────────────────────┤
│ Family      │ $24.99/mo     │ Up to 6 members, shared family agents (grocery,      │
│             │               │ chores, budget), per-member privacy via capability    │
│             │               │ tokens, family leaderboard, shared Life Score         │
├─────────────┼───────────────┼──────────────────────────────────────────────────────┤
│ Business    │ $49.99/user/mo│ Team coordination, compliance (SOC2, HIPAA), custom  │
│             │               │ agent creation, advanced analytics, SLA, SSO,         │
│             │               │ audit trail                                            │
└─────────────┴───────────────┴──────────────────────────────────────────────────────┘
```

**Why $4.99 for Plus**: It's the mass conversion play. Less than a coffee. Impulse-buy threshold. The Money Saved counter already shows $50+ in value by the time most users hit the paywall. $4.99 feels like stealing.

**Subscription math at scale**:
- 50M total users (Year 5)
- 35M Free (funnel value: data, network effects, marketplace)
- 10M Plus × $4.99/mo × 12 = $599M gross → ~$300M net (after churn, refunds)
- 3M Pro × $14.99/mo × 12 = $540M gross → ~$270M net
- 1M Family × $24.99/mo × 12 = $300M gross → ~$150M net
- 200K Business × $49.99/mo × 12 = $120M gross → ~$60M net
- **Subscription total: ~$780M gross, ~$300M net at conservative conversion rates (20% free→paid)**

### Stream 2: Transaction-Based (~$200M at scale)

| Transaction Type | Revenue Model | Example |
|-----------------|---------------|---------|
| Savings found | 10% of savings (user keeps 90%) | Bill Negotiator saves $847 → RuVix earns $84.70 |
| Affiliate | Standard affiliate commission | User buys recommended product → 3-8% commission |
| Financial referral | Institution referral fee | User opens recommended savings account → $50-200 referral |
| Insurance | Insurer acquisition fee | User switches to recommended policy → $100-500 referral |

**User always nets positive**: the 10% savings share means the user ALWAYS saves more than they would without RuVix. Transparent. Opt-in. User can disable and still get the savings.

### Stream 3: Marketplace (~$150M at scale)

| Revenue Source | Model | Scale |
|---------------|-------|-------|
| Paid agent sales | 30% platform cut (App Store model) | 10K paid agents × varying prices |
| Promoted placement | $99-$999/mo for featured listing | Top of category, "Recommended" badge |
| Enterprise agent licensing | Per-seat licensing through marketplace | B2B agents sold through consumer platform |

### Stream 4: Voice Personas & Customization (~$50M at scale)

| Product | Price | Description |
|---------|-------|-------------|
| Celebrity voice packs | $2.99/mo each | Morgan Freeman narrates your morning briefing |
| Custom TTS voice | $9.99 one-time | Clone your own voice (on-device, privacy-first) |
| Agent avatars | $0.99-$4.99 | Visual personas for your agent grid |
| Branded experiences | Sponsorship | "Brought to you by AmEx" — branded finance agent skin |

### Stream 5: Enterprise API (~$100M at scale)

- Usage-based pricing: $0.001 per agent-call, $0.01 per cloud burst
- White-label deployments: $50K-$500K/yr for companies that want "their own RuVix"
- Custom agent development: professional services arm
- Healthcare, legal, and financial verticals with compliance packaging

### Stream 6: Data Insights (~$100M at scale)

- **Anonymized, aggregated** insights sold to enterprises
- Example: "In Q3, 67% of ISP customers in California successfully negotiated a 20%+ discount when mentioning competitor pricing"
- Users earn credits for opt-in data contribution
- Zero individual data sold — ever. Aggregate patterns only. Differential privacy enforced.

### Stream 7: Financial Products (~$100M at scale)

Agent-recommended financial products with transparent referral economics:

| Product | Referral Fee | User Benefit |
|---------|-------------|--------------|
| High-yield savings | $50-100 per account | 4.5% APY vs. 0.01% |
| Credit cards (optimized for user) | $75-200 per approval | Cashback/points optimized for spending patterns |
| Insurance (comparison + switch) | $100-500 per policy | Avg. $347/yr savings |
| Refinancing | $500-2000 per closing | Lower rate, lower payment |

**All recommendations are fiduciary-grade**: the agent recommends what's BEST for the user, not what pays the highest referral. Users see the referral fee transparently. Trust is the product.

### Revenue Stack Summary

```
┌─────────────────────────────────────────────────────────┐
│             REVENUE STACK — YEAR 5 TARGET                │
│                                                           │
│  Subscriptions          $300M  ████████████████           │
│  Transaction-based      $200M  ██████████                 │
│  Marketplace            $150M  ████████                   │
│  Enterprise API         $100M  █████                      │
│  Data Insights          $100M  █████                      │
│  Financial Products     $100M  █████                      │
│  Voice/Customization     $50M  ███                        │
│  ───────────────────────────────                          │
│  TOTAL                 $1.0B+  ████████████████████████   │
│                                                           │
│  No single stream > 30% of revenue (diversified)         │
│  3 streams are user-aligned (they save money to make us  │
│  money — incentives perfectly aligned)                    │
└─────────────────────────────────────────────────────────┘
```

---

## 9. VIRAL GROWTH ENGINE

### Shareable Savings Cards

Every savings event generates a shareable card:

```
┌─────────────────────────────────────────┐
│  🎉 My AI just saved me $342           │
│                                          │
│  RuVix found that my internet bill was  │
│  $62/mo too high and negotiated it down │
│  while I was sleeping.                   │
│                                          │
│  Total saved since joining: $847        │
│                                          │
│  [Try RuVix free →]  (referral link)    │
│                                          │
│  ── Optimized for Instagram/TikTok ──   │
│  ── Auto-sized for each platform ──     │
│  ── One-tap share from notification ──  │
└─────────────────────────────────────────┘
```

### Referral Tiers

| Referrals | Reward (for BOTH referrer and referred) |
|-----------|----------------------------------------|
| 1 invite | Both get premium agent unlocked for 30 days |
| 5 invites | 1 month Plus free |
| 10 invites | 3 months Plus free |
| 25 invites | **Lifetime Plus** |
| 100 invites | **Lifetime Pro** + "Ambassador" badge + early access to all new agents |

The referral reward for BOTH parties removes the "am I being sold?" friction. When your friend says "try this, we both get a free month" it's peer recommendation, not sales pitch.

### Family Plans with Shared Swarms

Family plans amplify virality:

- **Shared agent swarms**: Family grocery agent optimizes for everyone's preferences
- **Family leaderboard**: "Dad saved the most this week ($127) — Mom is #2 ($98)"
- **Chore coordination**: "Whose turn to take out the trash?" — agents track, remind, gamify
- **Per-member privacy**: Capability tokens ensure kids' agents can't see parents' financial data (and vice versa, configurable)
- **Family Life Score**: Combined dashboard showing household optimization

### Challenge Mode

Opt-in competitive challenges:

- "Beat your neighbor's energy savings this month" (anonymized comparison by zip code)
- "Most productive week: compete with friends on Calendar Efficiency score"
- "Savings Sprint: who can find the most savings in 7 days?"

### Creator/Influencer Program

Content creators can **publish agent packs** and earn:

| Revenue Split | Recipient | Why |
|--------------|-----------|-----|
| 50% | Creator (curated the pack, drives installs) | |
| 30% | Platform (distribution, infrastructure) | |
| 20% | Base agent developers (wrote the underlying agents) | |

A creator with 1M followers publishes "My Productivity Stack" → 50K installs at $4.99 → Creator earns $124,750. This creates a new revenue stream for creators AND drives platform adoption.

### Configuration Sharing

Users can share their agent configurations:

- **"Here's my productivity swarm"** — one-tap import link
- Share via URL, QR code, or AirDrop
- Configuration includes: which agents, their settings, SONA personality (anonymized)
- Does NOT include personal data — just the agent setup
- Shared configs get a public rating and install count

### Viral Loop Economics

```
User joins (free)
    → Uses app for 7 days (free tier is genuinely good)
    → Money Saved counter hits $50
    → Gets savings card notification: "Share your savings?"
    → Shares to Instagram: "My AI saved me $50 this week"
    → 3 friends install (avg. referral conversion: 3.2%)
    → Each friend sees value in 7 days, shares
    → Viral coefficient k = 1.4 (each user brings 1.4 new users)
    → With k > 1.0: organic exponential growth
```

---

## 10. BUSINESS CASE

### Market Sizing

| Metric | Value | Calculation |
|--------|-------|-------------|
| **TAM** | $270B | 4.5B smartphone users × $60 avg annual willingness-to-pay for AI assistant |
| **SAM** | $48B | 800M English-speaking developed-market smartphone users × $60/yr |
| **SOM (Year 5)** | $1.0B+ | 50M users × $20 blended ARPU (incl. free users at $0 direct + transaction value) |

### Unit Economics

| Metric | Free User | Plus ($4.99/mo) | Pro ($14.99/mo) |
|--------|-----------|-----------------|-----------------|
| **CAC** | $2 (organic/referral) | $15 (free→paid conversion) | $25 (Plus→Pro upgrade) |
| **Monthly COGS** | $0.10 (on-device only) | $1.20 (cloud burst costs) | $3.50 (unlimited cloud) |
| **Gross Margin** | N/A | 76% | 77% |
| **Monthly Churn** | 8% (M1), 3% (M6+) | 4% | 2% |
| **LTV** | $0.80 (transaction value) | $189 (36-mo horizon) | $480 (36-mo horizon) |
| **LTV:CAC** | 0.4x (funnel value) | **12.6x** | **19.2x** |

Free users have positive ROI through: (1) network effects improving all agents, (2) federated learning contributions, (3) viral referrals (k=1.4), and (4) transaction-based revenue from savings found.

### Growth Trajectory

| Year | Total Users | Paid Users | Revenue | Key Milestone |
|------|------------|------------|---------|---------------|
| Y1 | 1M | 100K | $6M | Product-market fit, 50 first-party agents |
| Y2 | 8M | 1.2M | $60M | Marketplace launch, 1K agents, viral loop k>1 |
| Y3 | 25M | 5M | $300M | 7 revenue streams active, international expansion |
| Y4 | 40M | 10M | $700M | Enterprise tier, financial products, 5K agents |
| Y5 | 50M+ | 15M+ | $1.0B+ | 10K+ agents, market leader, IPO-ready |

### Competitive Landscape

```
┌──────────────┬──────────┬───────────┬──────────┬──────────┬───────────┐
│              │  RuVix   │ Siri/     │  Google  │  Rabbit  │  Humane   │
│              │  Mesh    │ Alexa     │  Asst    │  R1      │  AI Pin   │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Architecture │ Millions │ 1 general │ 1 general│ 1 general│ 1 general │
│              │ of       │ assistant │ assistant│ assistant│ assistant │
│              │ narrow   │           │          │          │           │
│              │ experts  │           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Financial    │ $847+/yr │ $0        │ $0       │ $0       │ $0        │
│ value to     │ savings  │           │          │          │           │
│ user         │ found    │           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Marketplace  │ 10K+     │ "Skills"  │ None     │ None     │ None      │
│              │ agents   │ (stale)   │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ On-device    │ SONA +   │ None      │ Limited  │ None     │ None      │
│ learning     │ EWC++    │           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Federated    │ Yes      │ No        │ No       │ No       │ No        │
│ learning     │ (privacy │           │          │          │           │
│              │ first)   │           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Runs on user │ Yes      │ Cloud     │ Cloud    │ Cloud    │ Cloud     │
│ device       │ (phone,  │ only      │ only     │ only     │ only      │
│              │ laptop,  │           │          │          │           │
│              │ Pi)      │           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Voice-first  │ Yes +    │ Voice     │ Voice    │ Voice    │ Voice     │
│              │ multi-   │ only      │ + screen │ only     │ + laser   │
│              │ modal    │           │          │          │ projector │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Privacy      │ On-device│ All cloud │ All cloud│ All cloud│ All cloud │
│              │ first,   │           │          │          │           │
│              │ user     │           │          │          │           │
│              │ owns data│           │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Hardware req │ Any      │ Locked to │ Android/ │ $199     │ $699      │
│              │ phone +  │ Apple/    │ Pixel    │ dedicated│ dedicated │
│              │ optional │ Amazon    │          │ device   │ device    │
│              │ $35 Pi   │ hardware  │          │          │           │
├──────────────┼──────────┼───────────┼──────────┼──────────┼───────────┤
│ Acts on your │ Yes      │ Very      │ Limited  │ Limited  │ Very      │
│ behalf       │ (negot., │ limited   │          │          │ limited   │
│              │ schedule,│           │          │          │           │
│              │ purchase)│           │          │          │           │
└──────────────┴──────────┴───────────┴──────────┴──────────┴───────────┘
```

### Network Effects — The Moat

RuVix has **double-sided network effects** that create an exponential moat:

```
MORE USERS                          MORE DEVELOPERS
    │                                      │
    ▼                                      ▼
Smarter agents                     More marketplace agents
(federated learning)               (better selection)
    │                                      │
    ▼                                      ▼
More value per user                More reasons to join
    │                                      │
    └──────────── FLYWHEEL ────────────────┘
                    │
                    ▼
           More users join
           (viral k = 1.4)
```

**User-side network effect**: Every user's anonymized patterns improve every other user's agents. 1M users = good agents. 50M users = incredible agents. This data moat is impossible to replicate without the user base.

**Developer-side network effect**: More users = bigger marketplace audience = more developers building agents = more reasons for users to join. This is the iOS App Store flywheel.

**Combined**: Unlike single-sided network effects (which competitors can replicate with money), double-sided network effects are self-reinforcing. The #1 player wins disproportionately.

### Defensibility

| Moat | Strength | Why |
|------|----------|-----|
| Federated learning data | Very strong | 50M users' anonymized patterns = impossible to replicate |
| SONA personalization | Strong | 365 days of per-user micro-LoRA = high switching cost |
| Marketplace ecosystem | Strong | 10K agents + 25K developers = ecosystem lock-in |
| Voice-first UX | Medium | UX can be copied, but the backend cannot |
| On-device architecture | Medium | Technical moat — competitors are cloud-first, hard to retrofit |

---

## 11. TECHNICAL ARCHITECTURE

### The Runtime Stack (Updated for Voice-First)

```
┌──────────────────────────────────────────────────────────────────────┐
│                      RuVix Mesh Service Layer                        │
│  Agent Marketplace │ Voice Pipeline │ User Dashboard │ Developer SDK │
├──────────────────────────────────────────────────────────────────────┤
│                      RLMX Cognition Kernel                           │
│  15 Syscalls │ Capability Tokens │ TinyDancerRouter │ ProofEngine    │
├────────────┬────────────┬──────────────┬─────────────────────────────┤
│ Phone      │  Laptop    │  Home Hub    │  Cloud Burst                │
│ (WASM)     │ (NAPI-RS)  │  (RPi5)      │  (NUC/Cloud)               │
│ WebGPU     │  Metal     │  Edge GGUF   │  vLLM / MLX                │
│ Zone       │  Zone      │  Zone C      │  Zone B                    │
│ A-Mobile   │  A-Desktop │  (Sentinel)  │  (Burst)                   │
│ ★ PRIMARY  │  Secondary │  Always-on   │  On-demand                 │
└────────────┴────────────┴──────────────┴─────────────────────────────┘
```

### 3 New Crates

**`rlmx-voice`** — Voice Pipeline

```rust
// crates/rlmx-voice/src/lib.rs
pub struct VoicePipeline {
    vad: VoiceActivityDetector,      // 500K param CNN, always-listening
    stt: SpeechToText,               // Whisper-tiny Q4 via TieredEngine
    intent: IntentClassifier,         // TinyDancerRouter 18→32→5
    decomposer: MultiIntentDecomposer,
    tts: TextToSpeech,               // On-device + cloud streaming
    session_memory: SessionMemory,    // SONA-backed conversation context
}

pub struct VoiceSession {
    id: Uuid,
    speaker_embedding: Vec<f32>,     // Speaker identification
    conversation_turns: Vec<Turn>,
    active_intents: Vec<Intent>,
    response_mode: ResponseMode,     // VoiceOnly, Visual, Multimodal
}

pub enum ResponseMode {
    VoiceOnly,      // Driving, eyes-free
    Visual,         // Looking at phone, text preferred
    Multimodal,     // Full experience: voice + cards + haptics
    Ambient,        // Lock screen, minimal
}
```

**`rlmx-phone`** — Mobile Runtime

```rust
// crates/rlmx-phone/src/lib.rs
pub struct PhoneRuntime {
    coordinator: LightweightCoordinator,  // On-device swarm coordinator
    background: BackgroundScheduler,       // iOS BGTask / Android WorkManager
    notifications: NotificationOrchestrator,
    widgets: WidgetManager,               // Lock screen + home screen
    battery: BatteryAwareScheduler,       // Throttle inference on low battery
    offline_queue: OfflineOutbox,         // Queue cloud requests for later
}

pub struct DeviceCapabilities {
    gpu: GpuType,                // WebGPU, Metal, None
    memory_mb: u32,
    battery_pct: f32,
    network: NetworkType,        // WiFi, Cellular, Offline
    thermal_state: ThermalState, // Nominal, Fair, Serious, Critical
}
```

**`rlmx-marketplace`** — Agent Marketplace

```rust
// crates/rlmx-marketplace/src/lib.rs
pub struct Marketplace {
    registry: AgentRegistry,        // All published agents
    billing: BillingEngine,         // Stripe Connect integration
    review: ReviewSystem,           // Automated security audit + human review
    publisher: PublisherPortal,     // Developer dashboard
    analytics: MarketplaceAnalytics,
    featured: FeaturedEngine,       // ML-ranked featured/recommended agents
}

pub struct AgentListing {
    id: Uuid,
    name: String,
    domain: LifeDomain,            // 1 of 12 categories
    publisher: PublisherId,
    rvf_hash: [u8; 32],           // Content-addressed RVF container
    price: AgentPrice,             // Free, OneTime($), Monthly($)
    rating: f32,
    install_count: u64,
    permissions_required: Vec<Permission>,
    supported_devices: Vec<DeviceType>,
}
```

### Kernel Expansion

**Syscalls: 12 → 15** (+VoiceTranscribe, VoiceSynthesize, IntentRoute)

```rust
pub enum Syscall {
    // Existing 12
    VecSearch, VecInsert, StateMutate, GraphQuery,
    ProcessFork, ProcessSend, ProcessHalt, HaltCheck,
    ProofSeal, ProofVerify, ProofWitness, StrategyRoute,

    // New voice syscalls
    VoiceTranscribe {
        audio: AudioBuffer,
        language_hint: Option<String>,
    },
    VoiceSynthesize {
        text: String,
        persona: VoicePersona,
        streaming: bool,
    },
    IntentRoute {
        transcript: String,
        speaker_context: SpeakerContext,
        decompose: bool,  // true = multi-intent decomposition
    },
}
```

**Domain Events: 6 → 8** (+VoiceSessionStarted, VoiceResponseStreaming)

**SwarmEvent: 9 → 12** (+VoiceChunk, AgentProgress, MultimodalResponse)

### Router Expansion

TinyDancerRouter input dimensions: 14 → 18

```rust
pub struct RouterInput {
    // Original 14
    pub query_complexity: f32,
    pub token_count: f32,
    pub domain_signals: [f32; 5],
    pub battery_level: f32,
    pub network_type: f32,
    pub time_of_day: f32,
    pub model_confidence_history: [f32; 3],
    pub memory_pressure: f32,

    // New voice features
    pub speaker_confidence: f32,      // STT confidence (0-1)
    pub emotion_valence: f32,         // Detected emotion (-1 to 1)
    pub urgency_score: f32,           // Speech rate + keyword triggers
    pub ambient_noise_level: f32,     // Background noise dB
}
```

### SandboxProfile Expansion

New fields for mobile-aware sandboxing:

```rust
pub struct SandboxProfile {
    // Existing fields
    pub name: String,
    pub agent_type: AgentType,
    pub resources: ResourceEnvelope,
    pub network: NetworkPolicy,
    pub zone: Zone,
    pub model_tier: ModelTier,

    // New mobile-aware fields
    pub device_class: DeviceClass,         // Phone, Tablet, Laptop, Pi, Cloud
    pub background_policy: BackgroundPolicy, // Foreground, Background, Suspended
    pub battery_threshold: f32,             // Min battery % to run (0.0-1.0)
    pub network_requirement: NetworkRequirement, // Any, WiFi, Offline
    pub thermal_budget: ThermalBudget,      // Max thermal contribution
}
```

### 8 New MCP Tools

```
rlmx_marketplace_search    — Search agents by domain, keyword, rating
rlmx_marketplace_install   — Install agent from marketplace
rlmx_marketplace_uninstall — Remove installed agent
rlmx_marketplace_rate      — Rate and review an agent
rlmx_marketplace_publish   — Publish agent to marketplace (developer)
rlmx_marketplace_earnings  — View developer earnings dashboard
rlmx_marketplace_featured  — Get featured/recommended agents
rlmx_marketplace_categories — List all 12 life-domain categories
```

### Device-to-Zone Mapping (Updated)

**Phone (WASM/WebGPU) = Zone A-Mobile (PRIMARY)**

- `@ruvector/ruvllm-wasm` v2.0.2 runs in Progressive Web App or native WebView
- `ParallelInference`: WebGPU matmul for embedding generation, route scoring
- `SonaInstantWasm`: <1ms adaptation to user corrections ("no, I don't like sushi")
- `HnswRouterWasm`: ANN pattern matching against user's preference embeddings
- `MicroLoraWasm`: Per-user LoRA deltas running in the browser — YOUR model, YOUR device
- `KvCacheWasm`: Two-tier cache for conversation context across agent sessions
- `BrowserComputePool`: Opt-in idle cycle contribution (battery-aware)
- **NEW: Voice pipeline** — VAD, STT, intent classification, TTS all on-device
- **NEW: Always-on coordinator** — lightweight swarm management
- **NEW: Lock screen widgets** — active agent count, savings counter

**Laptop (NAPI-RS) = Zone A-Desktop (Secondary)**

- `rlmx-napi` crate compiles kernel to native Node.js addon
- Powers: Electron desktop app, VS Code extension, CLI, Claude Code integration
- `dispatch()` calls kernel syscalls with zero HTTP overhead — sub-millisecond
- Metal GPU (Mac) or CUDA (Linux) for local inference up to 8B models via MLX
- SONA master pattern bank — all learned preferences consolidated here
- Heavy inference tasks delegated here from phone when available

```javascript
// The NAPI-RS API every agent developer uses
import { RuVixKernel } from '@ruvix/mesh-native';

const kernel = new RuVixKernel({
  zone: 'A-Desktop',
  user_id: 'u_abc123',
  model_dir: '~/.ruvix/models',
  sona_capacity: 50_000,  // 50K learned patterns
});

// An agent is just a function that calls kernel syscalls
export async function billNegotiatorAgent(context) {
  // Search for similar past negotiations from federated learning
  const patterns = await kernel.dispatch('VecSearch', {
    query: context.bill_embedding,
    k: 20,
    filters: { type: 'negotiation', domain: context.provider_type }
  });

  // Route through TinyDancer — decides: local inference or cloud?
  const strategy = await kernel.route(context.query);

  // Generate negotiation script
  const script = await kernel.infer(strategy, {
    prompt: buildNegotiationPrompt(context, patterns),
    max_tokens: 2048,
  });

  // Record outcome for learning
  await kernel.dispatch('StateMutate', {
    action: 'record_negotiation',
    params: { result: script, bill: context.bill },
    proof: await kernel.prove(script),
  });

  return script;
}
```

**Home Hub (Raspberry Pi 5) = Zone C (Sentinel)**

- Cross-compiled `rlmx-cli` with ruvllm feature, deployed via systemd
- Always-on sentinel: runs 24/7 on 5W power draw
- TinyLlama 1.1B Q4 for instant local inference (email classification, IoT commands)
- Gossip protocol syncs state with phone and laptop when they're on home network
- IoT integration: reads smart home sensors, controls devices via ProcessSend to IoT agents
- **Privacy anchor**: The home hub is the ONLY device that stores long-term personal data. Phone and laptop sync TO it. If you lose your phone, your agent state lives on.
- Local RVF container store: all federated knowledge packages cached here

**Cloud (NUC Cluster or Cloud VMs) = Zone B (Burst)**

- For queries that exceed device capability (complex tax scenarios, medical literature review)
- Raft consensus for multi-user coordination (family plans, enterprise teams)
- Large model inference: 70B models via vLLM for high-stakes decisions
- RVF registry: serves federated learning packages to all users' devices
- User data NEVER stored in cloud — only anonymized, aggregated patterns

### All 40+ Ruvnet Crates in Action

#### Phase 1: Core Vector & Storage — User Memory

| Crate | Role in RuVix Mesh |
|-------|--------------------|
| `ruvector-core` | Every user preference, every learned pattern, every agent memory is a vector in HNSW-indexed storage. *"Show me restaurants like the one I loved in Barcelona"* = cosine similarity over 768-dim embeddings across 50K+ memories. |
| `ruvector-graph` | User's life graph: people, places, products, services, appointments, health metrics — all connected. *"Who recommended that dentist?"* = 2-hop graph traversal. |
| `ruvector-filter` | Bloom filter on phone: instantly reject 90% of spam/scam patterns without inference. Saves battery. |
| `ruvector-collections` | Namespace isolation per agent domain. Shopping agent can't read health data. Enforced at the vector level. |
| `rvf-types` | Wire format for agent packages distributed via marketplace. Every agent is an RVF container. |
| `rvf-runtime` | Loads and executes agent packages on any device. Install an agent = download RVF container, rvf-runtime extracts code + model + config + permissions. |
| `rvf-wire` | Serialization for device-to-device sync (laptop ↔ home hub) and cloud-to-device federation. |
| `rvf-index` | Index over installed agents, cached patterns, and federated knowledge packages. Fast lookup: *"which agent handles insurance claims?"* |
| `rvf-quant` | 4-bit quantized vectors on phone (8x memory savings). Your 50K preference vectors fit in 25MB instead of 200MB. |
| `rvf-crypto` | Every agent decision is Ed25519 signed. Witness chain proves: *"my Bill Negotiator agreed to $X/mo on my behalf at time T."* Legal standing. |

#### Phase 2: Swarm Consensus & Networking — Agent Coordination

| Crate | Role in RuVix Mesh |
|-------|--------------------|
| `qudag-dag` | When 5 agents coordinate on a complex decision (e.g., "should I take this job offer?" involves Finance, Health Insurance, Real Estate, Tax, Career agents), DAG consensus ensures all agents' inputs are considered without deadlocking. |
| `qudag-network` | Network layer for cross-device agent communication. Laptop agent talks to phone agent via encrypted channel. |
| `qudag-crypto` | Cryptographic proofs that agent coordination wasn't tampered with. Important for enterprise audit trails. |
| `ruvector-raft` | Family plan: 6 family members' agents share a grocery list, chore schedule, budget. Raft ensures consistency — if Mom's agent adds "buy milk" and Dad's agent adds "buy oat milk" simultaneously, Raft resolves the conflict. |
| `midstreamer-quic` | Low-latency QUIC transport between home hub (Pi) and laptop (Mac) on LAN. Sub-millisecond agent state sync when you walk in the door. |
| `ruv-swarm-core` | The primitive for "joining the swarm." When you install a new agent, it discovers existing agents, negotiates capabilities, and integrates. When you open the app on a new device, it joins the mesh automatically. |
| `ruv-swarm-transport` | Transport abstraction: QUIC on LAN (Pi ↔ Mac), WebSocket for phone (WASM ↔ cloud), HTTP for marketplace downloads. |

#### Phase 3: Enhanced Inference & Attention — The Intelligence

| Crate | Role in RuVix Mesh |
|-------|--------------------|
| `ruvector-attention` | Flash Attention on Mac GPU: the Health Guardian agent processes your 50-page medical history in 200ms instead of 1.5s. **7.4x speedup.** |
| `ruvector-attn-mincut` | Attention min-cut: your email agent processes a 50-email inbox by cutting irrelevant threads. **60% token reduction** = faster + cheaper. |
| `ruvector-sona` | The core of personalization. Every agent records (input, action, outcome_quality) triples. Over time, your Shopping agent's SONA bank contains 10K+ patterns. Micro-LoRA adapts the base model to YOU. EWC++ ensures learning about food preferences doesn't erase knowledge about clothing preferences. |
| `ruvector-solver` | Linear programming for budget optimization. Your Finance agent solves: *"maximize savings subject to: rent ≤ 30% income, food ≥ $X, discretionary ≥ $Y."* |
| `ruvector-gnn` | Graph neural network over your life graph. *"Based on your spending graph, social graph, and health graph, the GNN predicts you'll need $X for dental work in Q3 based on similar users' patterns."* |
| `ruvector-tiny-dancer-core` | The routing brain on every device. FastGRNN (18→32→5) decides in <1ms: should this query run on phone (WASM), laptop (Metal), home hub (edge GGUF), or cloud (vLLM)? Now voice-aware with speaker confidence, emotion, urgency, and noise level inputs. |
| `midstreamer-scheduler` | Schedules agent tasks across devices. Tax agent needs 8B model? Schedule on laptop when plugged in and idle. Shopping comparison needs parallelism? Fan out to phone + laptop + cloud. |
| `cognitum-gate-tilezero` | Sparse attention for long-context agents. Legal Shield reading a 40-page contract? Tile-zero gates skip boilerplate, focus compute on novel clauses. |
| `cognitum-gate-kernel` | Fused GPU kernels for gated attention. Makes Mac's Metal inference 30% faster on long documents. |

#### Phase 4: Edge & Neural — The Sensory System

| Crate | Role in RuVix Mesh |
|-------|--------------------|
| `ruvector-cognitive-container` | Each agent runs in its own cognitive container — isolated SONA state, isolated memory, isolated capability token. A compromised Shopping agent can't access Health data. |
| `ruvector-memopt` | Phone memory is precious. Enables gradient checkpointing for SONA adaptation on mobile, memory-mapped vectors that spill to storage, and aggressive cache eviction. |
| `ruv-neural-core` | Core neural primitives shared across all devices. Same matmul code compiles to Metal (Mac), WASM SIMD (phone), NEON (Pi), AVX-512 (NUC). |
| `ruv-neural-signal` | Signal processing for IoT data on home hub AND **voice activity detection** on phone. Smart thermostat anomaly detection + always-listening VAD from the same 500K param CNN. |
| `ruv-neural-graph` | Message passing over the user's life graph for the GNN. Each entity (person, product, event) gets a learned embedding updated with every interaction. |
| `ruv-neural-embed` | Embedding generation everywhere. Every email, every receipt, every photo, every voice command gets a 768-dim embedding stored in ruvector-core. |
| `ruv-neural-memory` | Differentiable episodic memory. Your Calendar agent remembers not just WHAT happened but HOW you felt about it (inferred from voice emotion valence). *"You always cancel Thursday evening plans — should I stop accepting them?"* |
| `cognitum-rs` | High-level cognitive framework on Mac/NUC. Wraps SONA + NervousSystem + DagOptimizer. Powers the Coordinator agent's meta-learning: *"which agents need retraining? which patterns are stale? which user preferences have shifted?"* |

#### Phase 5: Bare-Metal & Verification — Trust and Safety

| Crate | Role in RuVix Mesh |
|-------|--------------------|
| `ruvix-hal` | Hardware abstraction for home hub Pi. Direct GPIO access for USB-connected sensors (temperature, air quality, motion). |
| `ruvix-types` | Shared types across the stack. ProcessId, CapabilityToken, AgentId — consistent from phone WASM to cloud NUC. |
| `ruvix-cap` | The trust layer. Every agent has a capability token with Ed25519 signature, Macaroon-style caveats, and time-bounded delegation. Your Bill Negotiator gets: *"can make calls, can read billing data, CANNOT access health data, CANNOT spend > $0, expires in 24 hours."* Hierarchical narrowing: child agents get FEWER permissions than parents. |
| `ruvix-region` | Memory region isolation on home hub. Health data lives in region "health" with ACL. Shopping agent process physically cannot address that memory region. Hardware-enforced privacy. |
| `ruvix-nucleus` | Minimal kernel for ultra-low-power home sentinel mode. When the Pi drops to "overnight mode," ruvix-nucleus runs without Linux — just security monitoring + smoke detector integration. 0.5W power draw. |
| `ruvix-sched` | Cooperative scheduler for bare-metal mode. Interrupt-driven: doorbell ring → wake security agent → classify → alert or ignore. |
| `ruvix-bcm2711` | BCM2711 drivers for Pi peripherals. Direct I2C for environmental sensor cluster, SPI for e-ink display showing daily summary. |
| `ruvector-verified` | Formal verification of critical paths. Proves: (1) capability token derivation always narrows permissions, (2) no agent can escalate beyond parent's permissions, (3) witness chain integrity maintained. Mathematical proof, not just a promise. |

### Sandbox Orchestration — FleetManifest (Updated)

```json
{
  "fleet_name": "user_abc123_personal_mesh",
  "sandboxes": [
    {
      "profile": "voice-coordinator",
      "agent_type": "Coordinator",
      "resources": { "cpu_cores": 2, "memory_mb": 512, "gpu": "WebGPU", "max_runtime_s": null },
      "network": "EgressOnly",
      "zone": "zone-a-mobile",
      "model_tier": "Small",
      "device": "phone",
      "device_class": "Phone",
      "background_policy": "Foreground",
      "battery_threshold": 0.05,
      "network_requirement": "Any",
      "thermal_budget": "Low"
    },
    {
      "profile": "desktop-coordinator",
      "agent_type": "Coordinator",
      "resources": { "cpu_cores": 2, "memory_mb": 4096, "gpu": "Metal", "max_runtime_s": null },
      "network": "ClusterOnly",
      "zone": "zone-a-desktop",
      "model_tier": "Medium",
      "device": "laptop",
      "device_class": "Laptop",
      "background_policy": "Background",
      "battery_threshold": 0.15,
      "network_requirement": "Any",
      "thermal_budget": "High"
    },
    {
      "profile": "email-triage",
      "agent_type": "Router",
      "resources": { "cpu_cores": 1, "memory_mb": 512, "gpu": "None", "max_runtime_s": null },
      "network": "EgressOnly",
      "zone": "zone-a-mobile",
      "model_tier": "Small",
      "device": "phone-wasm",
      "device_class": "Phone",
      "background_policy": "Background",
      "battery_threshold": 0.10,
      "network_requirement": "Any",
      "thermal_budget": "Low"
    },
    {
      "profile": "health-guardian",
      "agent_type": "Analyst",
      "resources": { "cpu_cores": 2, "memory_mb": 2048, "gpu": "None", "max_runtime_s": null },
      "network": "Isolated",
      "zone": "zone-c",
      "model_tier": "Small",
      "device": "home-hub-pi",
      "device_class": "Pi",
      "background_policy": "Foreground",
      "battery_threshold": 0.0,
      "network_requirement": "Offline",
      "thermal_budget": "Medium"
    },
    {
      "profile": "bill-negotiator",
      "agent_type": "Worker",
      "resources": { "cpu_cores": 1, "memory_mb": 1024, "gpu": "Metal", "max_runtime_s": 3600 },
      "network": "EgressOnly",
      "zone": "zone-a-desktop",
      "model_tier": "Medium",
      "device": "laptop",
      "device_class": "Laptop",
      "background_policy": "Background",
      "battery_threshold": 0.20,
      "network_requirement": "WiFi",
      "thermal_budget": "Medium"
    },
    {
      "profile": "shopping-sentinel",
      "agent_type": "Monitor",
      "resources": { "cpu_cores": 1, "memory_mb": 256, "gpu": "WebGPU", "max_runtime_s": null },
      "network": "EgressOnly",
      "zone": "zone-a-mobile",
      "model_tier": "Small",
      "device": "phone-wasm",
      "device_class": "Phone",
      "background_policy": "Background",
      "battery_threshold": 0.15,
      "network_requirement": "Any",
      "thermal_budget": "Low"
    },
    {
      "profile": "privacy-anchor",
      "agent_type": "Validator",
      "resources": { "cpu_cores": 4, "memory_mb": 4096, "gpu": "None", "max_runtime_s": null },
      "network": "Isolated",
      "zone": "zone-c",
      "model_tier": "Small",
      "device": "home-hub-pi",
      "device_class": "Pi",
      "background_policy": "Foreground",
      "battery_threshold": 0.0,
      "network_requirement": "Offline",
      "thermal_budget": "High",
      "note": "Stores all long-term personal data. Never connects to cloud."
    }
  ]
}
```

### Kernel Distribution Per Device (Updated)

| Device | Binary | Build | Delivery |
|--------|--------|-------|----------|
| Phone (iOS/Android) | `@ruvector/ruvllm-wasm` + `rlmx-wasm` kernel + `rlmx-voice` pipeline | `wasm-pack build -p rlmx-wasm --target web` | PWA or WebView in native app |
| Laptop (Mac) | `rlmx-napi` → `@ruvix/mesh-native` npm package | `cargo build -p rlmx-napi --target aarch64-apple-darwin --features "metal,napi"` | npm install in Electron app |
| Laptop (Linux) | `rlmx-napi` → `@ruvix/mesh-native` npm package | `cargo build -p rlmx-napi --target x86_64-unknown-linux-gnu --features "napi"` | npm install |
| Home Hub (Pi 5) | `rlmx-cli` native binary | `cargo build -p rlmx-cli --target aarch64-unknown-linux-gnu --features "ruvllm"` | RVF container flashed to SD card, systemd service |
| Home Hub (bare-metal) | `ruvix-nucleus` + kernel subset | `cargo build --target aarch64-unknown-none --features "bare-metal"` | Direct flash, no OS |
| Cloud (NUC) | `rlmx-cli` full | `cargo build -p rlmx-cli --features "ruvllm"` | Docker/Kubernetes |

### The Self-Learning Loop

#### Per-User Learning (Private, On-Device)

```
Every agent interaction (including voice):
    │
    ▼
ruvector-sona: record_pattern(query_embedding, actions_taken, result_quality)
    + voice metadata: emotion_valence, urgency, speaker_confidence
    │
    ▼
When patterns > 100 for an agent domain:
    MicroLoRA: Compute rank-4 LoRA delta from recent patterns
    FisherInformation: Update EWC++ diagonal (prevent catastrophic forgetting)
    │
    ▼
Apply delta to local model weights
    Agent is now personalized to THIS user's voice + preferences
    │
    ▼
Every 24 hours (on home hub, plugged in, idle):
    TinyDancerRouter: retrain from routing history (now 18-dim with voice features)
    DagOptimizer: update strategy success rates
    NervousSystem: circadian adjustment + voice usage patterns
    cognitum-rs: meta-learning — which agents need retraining?
    VoicePipeline: update speaker embedding, tune VAD sensitivity
```

#### Federated Learning (Anonymous, Aggregate)

```
Weekly federation cycle:
    │
    ▼
Each user's home hub (Zone C):
    Anonymize patterns: strip PII, keep domain + strategy + outcome
    + anonymized voice interaction patterns (not audio — metadata only)
    Package as RVF SegmentType::TransferPrior
    Sign with pseudonymous contribution key
    │
    ▼
Upload to RVF Registry (Zone B, NUC cluster):
    rvf-index: Index by domain, region, demographic bucket
    ruvector-gnn: Train aggregate GNN on 1M+ anonymized patterns
    ruvector-sona: Merge patterns into federated PatternBank (top-10K per domain)
    │
    ▼
Produce federated update packages:
    SegmentType::Overlay (aggregated LoRA deltas per domain)
    SegmentType::Pattern (merged pattern bank)
    SegmentType::Config (updated TinyDancerRouter weights, now 18-dim)
    │
    ▼
Distribute to all users' devices:
    Each user's ruvector-sona applies federated LoRA with EWC++
    Local knowledge preserved, global knowledge incorporated
    Voice pipeline improvements distributed automatically
```

**Result**: A user who just installed the Shopping agent gets 50K learned patterns from 1M users on day one. Their agent is IMMEDIATELY smart. And when they speak, the router already knows the optimal inference path for their device.

### How Millions of Users' Agents Federate — Example

```
User A's Bill Negotiator successfully reduces Comcast bill by 40%
    (triggered by voice: "Why is my internet bill so high?")
    │
    ▼
ruvector-sona: Records pattern
    (input=comcast_bill_features, actions=negotiation_script, quality=0.95)
    + voice_trigger: urgency=0.7, emotion=frustration
    │
    ▼
Anonymization layer strips PII, keeps:
    {provider_type: "ISP", region: "CA", strategy: "competitor_mention",
     discount: 40%, voice_trigger_emotion: "frustration"}
    │
    ▼
rvf-types: Package as SegmentType::TransferPrior
rvf-crypto: Sign with user's contribution key (pseudonymous)
rvf-wire: Serialize for transport
    │
    ▼
midstreamer-quic: Upload to cloud RVF registry (Zone B)
    │
    ▼
RVF Registry aggregates across 100K users' ISP negotiations
    ruvector-gnn: Graph neural network finds:
    "mention competitor + ask for retention dept = 67% success rate in CA"
    "voice-triggered requests (frustration) correlate with 12% higher savings"
    │
    ▼
rvf-types: Package as federated Pattern + Overlay (aggregated LoRA delta)
    │
    ▼
User B installs Bill Negotiator for the first time
    rvf-runtime: Loads federated pattern pack
    ruvector-sona: Imports 10K anonymized patterns
    MicroLoRA: Applies aggregated LoRA delta
    → Agent is IMMEDIATELY good at negotiation
    Even with zero personal history, User B benefits from 100K users' experience
    │
    ▼
User B says: "Help me lower my cable bill"
    → Bill Negotiator already knows the optimal strategy for their region
    → First negotiation succeeds. Pattern recorded. Federation cycle continues.
```

---

## 12. IMPLEMENTATION PLAN

### Phase 1: Voice Foundation (Months 1-3)

**Goal**: User can speak into phone and get intelligent routed responses.

| Task | Crate | Deliverable |
|------|-------|-------------|
| Voice Activity Detection | `rlmx-voice` | 500K CNN model, <0.1% CPU background |
| On-device STT | `rlmx-voice` + `rlmx-ruvllm` | Whisper-tiny Q4, <100ms latency |
| Router expansion | `rlmx-kernel` | TinyDancerRouter 14→18 dim with voice features |
| Intent classification | `rlmx-voice` | Single + multi-intent decomposition |
| On-device TTS | `rlmx-voice` | Streaming TTS with persona system |
| Phone app prototype | `rlmx-phone` | PWA with tap-to-talk, basic visual cards |
| New syscalls | `rlmx-kernel` | VoiceTranscribe, VoiceSynthesize, IntentRoute |

**Exit criteria**: User speaks → STT → router → agent → TTS response in <3 seconds on iPhone 13+.

### Phase 2: Phone Runtime (Months 3-6)

**Goal**: Phone is the primary command center with always-on agents.

| Task | Crate | Deliverable |
|------|-------|-------------|
| Zone A-Mobile promotion | `rlmx-kernel`, `rlmx-swarm` | Phone as primary coordinator |
| Always-on foreground service | `rlmx-phone` | iOS + Android background execution |
| Lock screen widgets | `rlmx-phone` | Agent status, money saved, quick voice |
| 5 free on-device agents | `rlmx-agents` | Email, Calendar, Weather, News, Shopping |
| Notification orchestration | `rlmx-phone` | 3-tier priority + ML fatigue prevention |
| Offline queue | `rlmx-phone` | Cloud request outbox + auto-retry |
| Life Score v1 | `rlmx-cognitive` | 4-domain scoring with sparkline trends |
| Streak mechanics | `rlmx-phone` | Daily streaks with progressive rewards |
| Money Saved counter | `rlmx-phone` | Real-time counter with lock screen widget |

**Exit criteria**: 5 agents running on-device with zero network. Always-listening VAD. Lock screen widgets active. Streaks and Life Score functional.

### Phase 3: Marketplace (Months 6-9)

**Goal**: Third-party agents available, full marketplace operational.

| Task | Crate | Deliverable |
|------|-------|-------------|
| Marketplace backend | `rlmx-marketplace` | Registry, search, install, rate, publish |
| 8 MCP tools | `rlmx-mcp` | marketplace_search, _install, _uninstall, _rate, _publish, _earnings, _featured, _categories |
| Developer SDK | `rlmx-agent-sdk` | WASM + native build targets, test harness, docs |
| Agent review pipeline | `rlmx-marketplace` | Automated security audit + human review |
| 50 first-party agents | `rlmx-agents` | Covering all 12 life domains |
| RVF packaging | `rlmx-rvf` | Agent → RVF container build pipeline |
| Billing integration | `rlmx-marketplace` | Stripe Connect, 70/30 split, monthly payouts |
| Celebrity packs v1 | `rlmx-marketplace` | 3-5 influencer agent configurations |

**Exit criteria**: Marketplace live with 50 first-party + 200 third-party agents. Developer SDK public. Revenue flowing to developers.

### Phase 4: Growth & Monetization (Months 9-12)

**Goal**: All 7 revenue streams active. Viral loop k > 1.0.

| Task | Component | Deliverable |
|------|-----------|-------------|
| Referral system | `rlmx-phone` | Shareable savings cards, referral tiers, tracking |
| Family plans | `rlmx-swarm`, `rlmx-phone` | Shared agents, family leaderboard, privacy boundaries |
| Transaction revenue | `rlmx-marketplace` | 10% savings share, affiliate integration |
| Voice personas | `rlmx-voice` | Celebrity voice packs, custom TTS cloning |
| Enterprise API | `rlmx-mcp` | Usage-based pricing, white-label, SSO |
| Data insights | `rlmx-cognitive` | Anonymized aggregate reporting, differential privacy |
| Financial products | `rlmx-marketplace` | Referral integrations with banks, insurers |
| Challenge mode | `rlmx-phone` | Competitive savings challenges, zip-code leaderboards |
| Agent gamification | `rlmx-phone` | Agent levels, collection grid, achievements |

**Exit criteria**: All 7 revenue streams generating. Viral coefficient k > 1.0. Path to $6M ARR in Year 1 confirmed.

### Verification

```bash
cargo build --workspace                                    # Stub build clean
cargo build --workspace --all-features                     # Full build with all crates
cargo build -p rlmx-voice --features "whisper,tts"         # Voice pipeline builds
cargo build -p rlmx-phone --target wasm32-unknown-unknown  # Phone runtime WASM
cargo build -p rlmx-marketplace                            # Marketplace builds
cargo build -p rlmx-napi --features "napi"                 # NAPI-RS builds
cargo build -p rlmx-wasm --target wasm32-unknown-unknown   # WASM builds
cargo test --workspace                                     # All tests pass
cargo clippy --workspace -- -D warnings                    # Zero warnings
```

---

## What Makes This "Change Every Human Life Forever"

1. **Knowledge equity.** A minimum-wage worker gets the same bill-negotiation intelligence as a Fortune 500 CEO's executive assistant. The federated learning from millions of negotiations means everyone gets the best strategy. **Voice makes it accessible to everyone** — no typing, no screens, no technical literacy required.

2. **Time liberation.** 1,000+ micro-lifts per day. Email triaged before you see it. Bills optimized while you sleep. Health monitored passively. Shopping decisions pre-computed. The average user saves 2+ hours/day of cognitive load. **Voice makes it instant** — speak and it's done.

3. **Privacy by architecture.** Your data lives on YOUR devices. The Pi home hub is the privacy anchor. Federated learning uses anonymized aggregates. Capability tokens with formal verification mean agents mathematically CANNOT access unauthorized data. **Voice audio never leaves your device** — STT runs locally.

4. **Compound intelligence.** Your agents get smarter every day — from your interactions (SONA) AND from millions of other users (federated LoRA). After a year, your personal mesh has seen patterns you'd never encounter alone. **Voice emotion data accelerates personalization** — the agents learn not just WHAT you want but HOW you feel about it.

5. **Instant expertise.** Need to move to Paris? A specialized swarm of 47 agents assembles in seconds with patterns from similar movers. Need to file a claim? Legal, health, and insurance agents coordinate via DAG consensus. **One voice command activates an army.**

6. **Hardware democracy.** Runs on a $35 Pi + any phone + any laptop. No $2,000 GPU required. WASM/WebGPU inference on commodity hardware. The TieredEngine escalates to cloud only when needed. Edge-first by default. **The voice pipeline runs entirely on-device** — no monthly API costs for basic functionality.

---

*Built on the RLMX Cognition Kernel. 15 syscalls. 36 MCP tools. 12 agent types. 47+ crates. One voice. Millions of agents. Your life, upgraded.*
