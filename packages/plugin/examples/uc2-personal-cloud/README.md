# UC2 — Personal Agent Cloud

Multi-device mesh topology where agents run across phone (WASM), laptop (NAPI-RS),
and home hub (RPi5) with federated learning, differential privacy, and tiered billing.

## Key Concepts

- **Mesh topology**: every device is a peer; home hub (Zone C/E) is the sole
  long-term privacy anchor; max 1 MeshCoordinator per mesh
- **Federated learning**: RVF containers carry anonymized micro-LoRA deltas;
  differential privacy (Laplace ε=1.0, 1000-user minimum); emotion bucketed
  to 5 levels; no speaker embeddings
- **Billing tiers**: Free (5 agents) → Personal ($9.99) → Family ($19.99) →
  Pro ($29.99) → Enterprise (custom); capability tokens enforce limits
- **1000 lifts/day model**: ~200 calendar/email, ~50 shopping/finance,
  ~30 health/wellness, ~500 information shield, ~200 home/IoT

## Mesh Topology

```
Phone (Zone A-Mobile)  ←→  Laptop (Zone A-Desktop)
        ↕                          ↕
    Home Hub (Zone C/E, privacy anchor)
        ↕
    Cloud (Zone D, burst only)
```

## Deployment Profiles

| Profile | Device | Runtime | Role |
|---------|--------|---------|------|
| phone | Android/iOS | WASM/WebGPU | Primary interaction |
| laptop | macOS/Linux | NAPI-RS | Heavy compute |
| hub | RPi5/NUC | Native + edge | Privacy anchor, federation |
| cloud | SkyPilot | GPU inference | Burst escalation |

## Files

- `manifest.ts` — PluginManifest with UC2 metadata
- `agents.ts` — Mesh topology agent configurations
- `deploy-profiles.ts` — Device-centric deployment profiles

## Related

- ADR-009: WASM compute pool
- ADR-013: Mesh networking
- ADR-014: Federation protocol
- ADR-015: Billing and tiers
- Original prose: `archives/docs/crazy-ruv-cartes-plan-copy.md`
