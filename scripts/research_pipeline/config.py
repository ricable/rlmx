"""
config.py — ResearchIntent, CouncilConfig, GEPAConfig, TerminationConfig

All configuration types for the autonomous research pipeline.
GEPA can mutate ResearchIntent fields (focus_angle, depth, council, gepa weights).
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from enum import Enum
from typing import Any, Optional


# ─── Enums ────────────────────────────────────────────────────────────────────

class ArtifactLevel(str, Enum):
    CLAIM            = "claim"
    HYPOTHESIS_CARD  = "hypothesis_card"
    RESEARCH_THREAD  = "research_thread"
    MINI_PAPER       = "mini_paper"


class CouncilMode(str, Enum):
    PEER_REVIEW     = "peer_review"
    DIVERSIFICATION = "diversification"


class CouncilProvider(str, Enum):
    OPENAI    = "openai"
    ANTHROPIC = "anthropic"
    GOOGLE    = "google"
    LOCAL     = "local"
    MISTRAL   = "mistral"


class DiversityAngle(str, Enum):
    CONTRARIAN      = "contrarian"
    ADJACENT_DOMAIN = "adjacent_domain"
    SCALE_SHIFT     = "scale_shift"       # micro ↔ macro
    TEMPORAL_SHIFT  = "temporal_shift"    # short-term ↔ long-term
    REDUCTIONIST    = "reductionist"
    SYSTEMS         = "systems_thinking"


class MutationType(str, Enum):
    EXPAND      = "expand"
    CONSTRAIN   = "constrain"
    REFRAME     = "reframe"
    SPECIALIZE  = "specialize"
    GENERALIZE  = "generalize"


# ─── Council ──────────────────────────────────────────────────────────────────

@dataclass
class CouncilMemberConfig:
    """One seat on the LLM-Council. GEPA can add/remove/modify seats."""
    model: str
    provider: CouncilProvider = CouncilProvider.OPENAI
    temperature: float = 0.7
    system_prompt: str = ""
    weight: float = 1.0
    # Diversity angle used when council is in DIVERSIFICATION mode
    diversity_angle: DiversityAngle = DiversityAngle.CONTRARIAN
    # Peer-review persona used when council is in PEER_REVIEW mode
    reviewer_role: str = "critical reviewer"


@dataclass
class CouncilConfig:
    """
    Polymorphic LLM-Council configuration.

    Mode switching:
      - stage_mode_map: stage name → CouncilMode  (stage-bound switching)
      - novelty_threshold: if artifact novelty > threshold → peer_review  (fitness-triggered)
      - Both can be active simultaneously; per-call override always wins.
    """
    members: list[CouncilMemberConfig] = field(default_factory=lambda: [
        CouncilMemberConfig(
            model="gpt-4o",
            provider=CouncilProvider.OPENAI,
            temperature=0.7,
            diversity_angle=DiversityAngle.CONTRARIAN,
            reviewer_role="skeptical methodologist",
        ),
        CouncilMemberConfig(
            model="claude-sonnet-4-6",
            provider=CouncilProvider.ANTHROPIC,
            temperature=0.8,
            diversity_angle=DiversityAngle.ADJACENT_DOMAIN,
            reviewer_role="cross-domain synthesizer",
        ),
        CouncilMemberConfig(
            model="gemini-1.5-pro",
            provider=CouncilProvider.GOOGLE,
            temperature=0.6,
            diversity_angle=DiversityAngle.SCALE_SHIFT,
            reviewer_role="empirical validator",
        ),
    ])

    # Stage-bound mode switching: {"generation": DIVERSIFICATION, "synthesis": PEER_REVIEW}
    stage_mode_map: dict[str, CouncilMode] = field(default_factory=lambda: {
        "generation": CouncilMode.DIVERSIFICATION,
        "execution":  CouncilMode.DIVERSIFICATION,
        "synthesis":  CouncilMode.PEER_REVIEW,
    })

    # Fitness-triggered: novelty above this → flip to PEER_REVIEW
    novelty_threshold: float = 0.72

    # Allow per-call override of mode (passed via council_mode kwarg)
    allow_override: bool = True

    # Minimum council members needed for consensus
    quorum: int = 2

    # High disagreement = frontier territory; promote for deeper investigation
    frontier_disagreement_threshold: float = 0.35


# ─── GEPA ─────────────────────────────────────────────────────────────────────

@dataclass
class GEPAConfig:
    """
    Generative Evolutionary Prompt Architecture configuration.

    GEPA runs as the outer meta-loop, evolving:
      1. DSPy signature instructions (docstrings + field descriptions)
      2. Council membership / composition / weights
      3. Fitness signal weights

    When per_level=True, each ArtifactLevel gets its own GEPA loop with
    its own population; fitness signals are passed upward through the hierarchy.
    """
    population_size: int = 8
    generations: int = 5
    mutation_rate: float = 0.30
    crossover_rate: float = 0.65
    elite_size: int = 2              # top N survive unchanged each generation

    # What GEPA is allowed to evolve
    evolve_signatures: bool = True
    evolve_council: bool = True
    evolve_fitness_weights: bool = True

    # Per-level separate GEPA loops (True = 4 independent loops that share fitness upward)
    per_level: bool = True

    # Fitness weights — evolvable by GEPA itself (must sum to 1.0 after normalization)
    fitness_weights: dict[str, float] = field(default_factory=lambda: {
        "novelty":              0.20,
        "falsifiability":       0.20,
        "citation_density":     0.15,
        "compression_ratio":    0.15,
        "council_disagreement": 0.15,
        "cross_domain_surprise":0.15,
    })

    # Signature mutation: which signatures GEPA is allowed to mutate
    mutable_signatures: list[str] = field(default_factory=lambda: [
        "GenerateClaim",
        "GenerateHypothesisCard",
        "WeaveResearchThread",
        "SynthesizeMiniPaper",
        "AutoresearchGenerate",
        "AutoresearchExecute",
        "AutoresearchSynthesize",
        "CouncilPeerReview",
        "CouncilDiversify",
    ])

    # How many DSPy examples to keep in the GEPA training buffer per signature
    example_buffer_size: int = 32


# ─── Termination ──────────────────────────────────────────────────────────────

@dataclass
class TerminationConfig:
    """
    Four termination conditions checked in priority order.

    Priority (highest → lowest):
      1. budget          — hard resource ceiling (calls / time / cost)
      2. convergence     — council disagreement collapsed (frontier exhausted)
      3. novelty_collapse — GEPA fitness stopped improving
      4. quality_gate    — mini-paper council score exceeded threshold (publishable)
    """
    priority: list[str] = field(default_factory=lambda: [
        "budget",
        "convergence",
        "novelty_collapse",
        "quality_gate",
    ])

    # Budget
    max_llm_calls: int = 300
    max_seconds: float = 7200.0
    max_cost_usd: float = 20.0

    # Convergence: avg council disagreement across last N papers
    convergence_window: int = 3
    convergence_disagreement_threshold: float = 0.08

    # Novelty collapse: fitness delta < epsilon for K consecutive generations
    novelty_collapse_generations: int = 3
    novelty_collapse_delta: float = 0.005

    # Quality gate: mini-paper peer-review score
    quality_gate_score: float = 0.88


# ─── ResearchIntent ───────────────────────────────────────────────────────────

@dataclass
class ResearchIntent:
    """
    The seed and governing object for a research pipeline run.
    GEPA can mutate: focus_angle, depth, council, gepa.fitness_weights.

    At least one of (question, seed_abstract, domain) must be provided.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))

    # Seed inputs (all optional; GEPA can mutate which ones are used)
    question: Optional[str] = None          # "What are the limits of CoT reasoning?"
    seed_abstract: Optional[str] = None     # Existing paper abstract to start from
    domain: Optional[str] = None            # "agentic memory"

    # Constraints (budget, horizon, etc.)
    constraints: dict[str, Any] = field(default_factory=dict)

    # GEPA-mutable research parameters
    focus_angle: str = "broad"              # broad | narrow | contrarian | comparative
    depth: int = 3                          # how many hypothesis cards per thread
    breadth: int = 4                        # how many claims to generate before threading

    # Sub-configs (all GEPA-evolvable)
    council: CouncilConfig = field(default_factory=CouncilConfig)
    gepa: GEPAConfig = field(default_factory=GEPAConfig)
    termination: TerminationConfig = field(default_factory=TerminationConfig)

    # Streaming: emit intermediate artifacts as they are produced
    stream: bool = True

    def validate(self) -> bool:
        return any([self.question, self.seed_abstract, self.domain])

    def summary(self) -> str:
        parts = []
        if self.question:
            parts.append(f"Q: {self.question}")
        if self.seed_abstract:
            parts.append(f"Abstract: {self.seed_abstract[:120]}...")
        if self.domain:
            parts.append(f"Domain: {self.domain}")
        return " | ".join(parts) or "(empty intent)"


# ─── Stream Events ────────────────────────────────────────────────────────────

@dataclass
class StreamEvent:
    """Emitted by the pipeline for real-time observation of intermediate artifacts."""
    event_type: str          # claim_produced | card_produced | thread_produced | paper_produced
                             # gepa_generation | council_vote | termination | error
    level: ArtifactLevel
    artifact_id: str
    artifact_summary: str
    fitness_score: float = 0.0
    generation: int = 0
    metadata: dict[str, Any] = field(default_factory=dict)
