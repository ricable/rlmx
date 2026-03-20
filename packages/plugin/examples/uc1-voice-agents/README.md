# UC1 — One Voice, Millions of Agents

Voice-first cognition kernel where a single spoken command decomposes into intents
across 12 life domains, dispatching specialized agents on phone, laptop, home hub,
and cloud marketplace — all coordinated via swarm consensus.

## Key Concepts

- **Voice pipeline**: on-device STT (Whisper-tiny <100ms) → intent decomposition →
  agent dispatch → swarm consensus → voice + visual response in <10s
- **12 LifeDomains**: Finance, Health, Legal, Career, Education, Home, Shopping,
  Travel, Social, Government, Automotive, Pet
- **Agent zones**: Zone A-Mobile (phone, primary), Zone A-Desktop (laptop),
  Zone C/E (home hub, privacy anchor), Zone D (cloud burst)
- **Personalization**: micro-LoRA rank-4 deltas (<5MB) per user, EWC++ to prevent
  catastrophic forgetting across domains
- **Privacy**: audio never leaves device; only text intents reach SONA; federated
  learning with differential privacy (Laplace ε=1.0, 1000-user minimum)

## Swarm Topology

```
VoiceCoordinator (Zone A-Mobile)
├── Router → domain-specific Workers (Zone A)
├── Researcher agents (Zone A-Desktop)
├── MarketplaceManager (Zone D, on escalation)
└── Monitor + Reviewer (cross-zone)
```

## Example Interaction

**User says**: "I want to move to Paris"

47 agents activate across 9 domains: visa research, cost-of-living analysis,
job market scan, housing search, school district evaluation, healthcare transfer,
tax implications, shipping logistics, social network mapping — results stream
back as voice narration + visual cards within 10 seconds.

## Deployment Profiles

| Profile | Device | Inference | Agents |
|---------|--------|-----------|--------|
| phone | Android/iOS | WASM + WebGPU | 5 (free) to unlimited |
| laptop | macOS/Linux | NAPI-RS native | Full swarm |
| hub | RPi5/NUC | Edge (TinyLlama Q4) | Full kernel |
| cloud | SkyPilot GPU | Full models | Marketplace burst |

## Files

- `manifest.ts` — PluginManifest with UC1 metadata
- `agents.ts` — Agent type configs and zone mappings
- `deploy-profiles.ts` — Device deployment profiles

## Related

- ADR-001: Distributed swarm architecture
- ADR-003: Voice pipeline
- ADR-007: Marketplace
- ADR-024: Feature gates
- Original prose: `archives/docs/crazy-ruv-cartes-plan.md`
