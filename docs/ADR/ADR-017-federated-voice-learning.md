# ADR-017: Federated Voice Learning and Privacy

| Field    | Value                     |
|----------|---------------------------|
| Status   | Implemented               |
| Date     | 2026-03-19                |
| Authors  | Cedric                    |
| Replaces | —                         |

## Context

The PRD's voice-first architecture generates rich interaction data: voice emotion valence, urgency scores, speaker confidence, and multi-intent patterns. This data, when aggregated across millions of users via federated learning, dramatically improves agent routing and personalization. However, voice data is deeply personal — emotion patterns, speech characteristics, and conversational habits reveal sensitive information. The existing federated learning loop (SONA PatternBank → anonymize → RVF TransferPrior → aggregate → distribute) must be extended to handle voice-specific metadata while maintaining strict privacy guarantees.

## Decision

### Voice Metadata in Federated Learning

Extend the SONA pattern recording to include voice interaction metadata:

```rust
pub struct VoiceEnrichedPattern {
    // Existing
    query_embedding: Vec<f32>,     // 768-dim
    actions_taken: Vec<Action>,
    result_quality: f32,

    // Voice-specific metadata (NOT audio — metadata only)
    voice_trigger_emotion: Option<f32>,   // -1 to 1 valence
    urgency_level: f32,                    // 0-1
    interaction_modality: Modality,        // Voice, Text, Multimodal
    response_satisfaction: Option<f32>,    // inferred from follow-up behavior
}
```

### Privacy Invariants (Immutable Rules)

1. **No audio ever leaves the device.** STT runs on-device. Only text transcripts and metadata flow to SONA.
2. **No speaker embeddings are federated.** Speaker identification is device-only, never shared.
3. **Emotion valence is bucketed, not precise.** Before federation: continuous float → 5 buckets (very_negative, negative, neutral, positive, very_positive). Prevents emotion fingerprinting.
4. **Differential privacy on all federated voice metadata.** Laplace noise (ε=1.0) added to emotion, urgency, and satisfaction scores before upload.
5. **Minimum aggregation threshold.** Voice patterns only federated when ≥1000 users share the same domain+region bucket. Prevents small-group deanonymization.

### Anonymization Pipeline

```
On-device pattern recording:
    (transcript_embedding, actions, quality, emotion, urgency)
        │
        ▼
Strip PII from transcript embedding (re-embed without named entities)
Bucket emotion: f32 → {very_neg, neg, neutral, pos, very_pos}
Add Laplace noise (ε=1.0) to urgency and satisfaction
Drop speaker_embedding entirely
        │
        ▼
Package as RVF SegmentType::TransferPrior
Sign with pseudonymous contribution key (rotated monthly)
        │
        ▼
Upload to RVF Registry (Zone B) only over WiFi, battery > 50%
```

### Federated Voice Insights (Aggregate Only)

Examples of insights derived from federated voice data:

- "Voice-triggered requests with frustration emotion correlate with 12% higher savings found" → route frustrated users to Bill Negotiator first
- "Users who ask about health with urgency > 0.8 prefer immediate response over comprehensive analysis" → adjust GatherStrategy
- "Multi-intent utterances in the morning have 2.3x more calendar-related intents than evening" → pre-cache Calendar agent

### TinyDancerRouter Voice-Aware Federated Weights

The 4 new router input dimensions (speaker_confidence, emotion_valence, urgency_score, ambient_noise_level) get federated weight updates:

- Federated SegmentType::Config now includes updated 18-dim RouterInput weights
- Users benefit from aggregate routing intelligence without sharing individual patterns
- Router learns: "high urgency + finance domain → route to cloud for fastest response"

### User Controls

- **Opt-out of voice metadata federation** entirely (still get federated non-voice patterns)
- **Opt-out of all federation** (device-only learning, no upload, no download of federated patterns)
- **Data contribution credits**: users who opt in to voice metadata federation earn credits toward premium features
- **Transparency dashboard**: shows exactly what metadata was federated (domain, bucketed emotion, region — never content)

### EWC++ Extension for Voice Patterns

Elastic Weight Consolidation extended to protect voice-learned patterns:

- Voice interaction patterns (emotion → intent routing) form a separate Fisher Information diagonal
- Learning new voice patterns doesn't erase previously learned text patterns
- Cross-modal transfer: insights from voice interactions improve text-based routing and vice versa

## Consequences

### Positive

- Voice metadata improves routing and agent personalization across all users
- Strict privacy guarantees (no audio, differential privacy, minimum aggregation) build trust
- User opt-out controls respect autonomy
- Federated voice insights create data moat competitors can't replicate

### Negative

- Differential privacy noise reduces utility of federated voice patterns
- Minimum aggregation threshold means less-common languages/regions get fewer benefits
- Privacy controls add complexity to the federation pipeline

### Risks

- Regulatory landscape for voice data varies by jurisdiction (GDPR, CCPA, BIPA)
- Even bucketed emotion data could face legal challenges in some regions
- Pseudonymous contribution keys could theoretically be linked across months

## References

- ADR-012: Voice-First Pipeline (voice metadata sources)
- ADR-003: Neural Model Routing (TinyDancerRouter weights)
- ADR-006: Evolutionary Auto-Research (existing federated learning loop)
- PRD Section 3: Voice-First Architecture (privacy guarantees)
- PRD Section 11: Technical Architecture (self-learning loop)

## Implementation Notes

Implemented in `crates/rlmx-cognitive/src/voice_patterns.rs` with 22 tests. VoiceEnrichedPattern with emotion, urgency, modality, satisfaction. VoicePatternBank with temporal-weighted search (recent 3x weight). FederatedAnonymizer: strip_pii(), bucket_emotion() (5 levels), add_laplace_noise(epsilon=1.0). All 5 privacy invariants enforced (no audio leaves device, no speaker embeddings federated, emotion bucketed, differential privacy, minimum aggregation threshold).
