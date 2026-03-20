"""
artifacts.py — Hierarchical research artifact data models

Artifact hierarchy:
  Claim → HypothesisCard → ResearchThread → MiniPaper

Claims compose into HypothesisCards.
HypothesisCards form directed acyclic graph threads.
Threads synthesize into MiniPapers.

ResearchState holds the full mutable state of a pipeline run.
"""

from __future__ import annotations

import uuid
from dataclasses import dataclass, field
from datetime import datetime
from typing import TYPE_CHECKING, Any, Optional

if TYPE_CHECKING:
    from .config import ResearchIntent


# ─── Level 1: Claim ───────────────────────────────────────────────────────────

@dataclass
class Claim:
    """
    Atomic unit: a single falsifiable research statement.
    Produced by GenerateClaim + autoresearch generation phase.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    text: str = ""
    rationale: str = ""
    confidence: float = 0.0
    novelty_signal: str = ""
    bridged_domains: str = ""
    domain: str = ""
    source_intent_id: str = ""
    created_at: datetime = field(default_factory=datetime.utcnow)

    # Fitness
    fitness_score: float = 0.0
    fitness_breakdown: dict[str, float] = field(default_factory=dict)

    # Council
    council_mode_used: str = ""
    council_disagreement: float = 0.0
    council_action: str = ""          # promote | revise | reject | explore_further

    # GEPA lineage
    gepa_generation: int = 0
    signature_version: str = "v0"

    metadata: dict[str, Any] = field(default_factory=dict)

    def summary(self) -> str:
        return f"[Claim {self.id[:8]}] {self.text[:120]}"


# ─── Level 2: HypothesisCard ──────────────────────────────────────────────────

@dataclass
class HypothesisCard:
    """
    Expanded claim: formal hypothesis + experiment design + literature links.
    Produced by GenerateHypothesisCard + autoresearch execution phase.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    claim: Claim = field(default_factory=Claim)
    hypothesis: str = ""
    predicted_experiment: str = ""
    falsification_criteria: str = ""
    methodology_sketch: str = ""
    related_works: list[str] = field(default_factory=list)
    confidence_score: float = 0.0
    created_at: datetime = field(default_factory=datetime.utcnow)

    # Fitness
    fitness_score: float = 0.0
    fitness_breakdown: dict[str, float] = field(default_factory=dict)

    # Council
    council_reviews: list[dict[str, Any]] = field(default_factory=list)
    council_disagreement: float = 0.0
    council_action: str = ""

    # Autoresearch execution results
    autoresearch_findings: str = ""
    supporting_evidence: list[str] = field(default_factory=list)
    contradicting_evidence: list[str] = field(default_factory=list)

    # GEPA lineage
    gepa_generation: int = 0
    signature_version: str = "v0"

    metadata: dict[str, Any] = field(default_factory=dict)

    def summary(self) -> str:
        return f"[Card {self.id[:8]}] {self.hypothesis[:120]}"

    def citation_count(self) -> int:
        return len(set(self.related_works))


# ─── Level 3: ResearchThread ──────────────────────────────────────────────────

@dataclass
class ResearchThread:
    """
    Directed acyclic graph of HypothesisCards connected by supporting/refuting edges.
    Produced by WeaveResearchThread + autoresearch synthesis pass.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    topic: str = ""
    hypothesis_cards: list[HypothesisCard] = field(default_factory=list)

    # DAG: node_id (HypothesisCard.id) → list of parent node_ids
    dag: dict[str, list[str]] = field(default_factory=dict)

    # Edge types: (src_id, dst_id) → "supports" | "refutes" | "extends"
    edge_types: dict[tuple[str, str], str] = field(default_factory=dict)

    key_tensions: list[str] = field(default_factory=list)
    synthesis_direction: str = ""
    narrative_arc: str = ""
    created_at: datetime = field(default_factory=datetime.utcnow)

    # Fitness
    fitness_score: float = 0.0
    fitness_breakdown: dict[str, float] = field(default_factory=dict)

    # Council
    council_disagreement: float = 0.0
    council_action: str = ""

    # GEPA lineage
    gepa_generation: int = 0
    signature_version: str = "v0"

    metadata: dict[str, Any] = field(default_factory=dict)

    def summary(self) -> str:
        return (
            f"[Thread {self.id[:8]}] {self.topic[:80]} "
            f"({len(self.hypothesis_cards)} cards)"
        )

    def all_related_works(self) -> list[str]:
        works = []
        for card in self.hypothesis_cards:
            works.extend(card.related_works)
        return list(set(works))

    def leaf_nodes(self) -> list[str]:
        """Cards with no children (frontier of the DAG)."""
        parents = set()
        for parent_list in self.dag.values():
            parents.update(parent_list)
        return [c.id for c in self.hypothesis_cards if c.id not in parents]


# ─── Level 4: MiniPaper ───────────────────────────────────────────────────────

@dataclass
class MiniPaper:
    """
    Full research output: structured like an arXiv abstract.
    Produced by SynthesizeMiniPaper + autoresearch synthesis phase.
    Gated by council peer-review before promotion.
    """
    id: str = field(default_factory=lambda: str(uuid.uuid4()))
    thread: ResearchThread = field(default_factory=ResearchThread)

    # Sections
    abstract: str = ""
    methodology: str = ""
    findings: list[str] = field(default_factory=list)
    open_questions: list[str] = field(default_factory=list)
    contribution_statement: str = ""
    limitations: str = ""
    created_at: datetime = field(default_factory=datetime.utcnow)

    # Fitness
    fitness_score: float = 0.0
    fitness_breakdown: dict[str, float] = field(default_factory=dict)

    # Council peer-review: member_id → {score, critique, recommendation}
    council_peer_review: dict[str, dict[str, Any]] = field(default_factory=dict)
    council_consensus_score: float = 0.0
    council_action: str = ""          # accept | major_revision | reject

    # Quality gate result
    passed_quality_gate: bool = False
    quality_score: float = 0.0

    # Autoresearch synthesis
    autoresearch_synthesis: str = ""

    # GEPA lineage
    gepa_generation: int = 0
    signature_version: str = "v0"

    metadata: dict[str, Any] = field(default_factory=dict)

    def summary(self) -> str:
        return (
            f"[Paper {self.id[:8]}] "
            f"{self.abstract[:100]}... "
            f"(quality={self.quality_score:.2f})"
        )

    def word_count(self) -> int:
        text = " ".join([
            self.abstract, self.methodology,
            " ".join(self.findings),
            " ".join(self.open_questions),
            self.contribution_statement,
        ])
        return len(text.split())


# ─── ResearchState ────────────────────────────────────────────────────────────

@dataclass
class ResearchState:
    """
    Full mutable state of a single research pipeline run.
    Passed through every stage; streamed as events are produced.
    """
    intent: "ResearchIntent"

    # Artifact stores (append-only during a run)
    claims: list[Claim] = field(default_factory=list)
    hypothesis_cards: list[HypothesisCard] = field(default_factory=list)
    research_threads: list[ResearchThread] = field(default_factory=list)
    mini_papers: list[MiniPaper] = field(default_factory=list)

    # Resource tracking
    llm_call_count: int = 0
    total_cost_usd: float = 0.0
    start_time: datetime = field(default_factory=datetime.utcnow)

    # GEPA tracking
    generation: int = 0
    # level → list of (generation, fitness_score) tuples
    fitness_history: dict[str, list[tuple[int, float]]] = field(default_factory=lambda: {
        "claim": [], "hypothesis_card": [], "research_thread": [], "mini_paper": []
    })

    # Per-level GEPA populations: level → list of (instruction_str, fitness)
    gepa_populations: dict[str, list[dict[str, Any]]] = field(default_factory=dict)

    # Council disagreement history for convergence detection
    council_disagreement_history: list[float] = field(default_factory=list)

    # Streaming event log
    stream_events: list[dict[str, Any]] = field(default_factory=list)

    # Termination
    terminated: bool = False
    termination_reason: str = ""

    def elapsed_seconds(self) -> float:
        return (datetime.utcnow() - self.start_time).total_seconds()

    def best_papers(self, n: int = 3) -> list[MiniPaper]:
        return sorted(self.mini_papers, key=lambda p: p.quality_score, reverse=True)[:n]

    def latest_fitness(self, level: str) -> Optional[float]:
        history = self.fitness_history.get(level, [])
        return history[-1][1] if history else None

    def novelty_collapsed(self, config: Any) -> bool:
        """Check if fitness delta < epsilon for K consecutive generations."""
        history = self.fitness_history.get("claim", [])
        k = config.novelty_collapse_generations
        eps = config.novelty_collapse_delta
        if len(history) < k + 1:
            return False
        recent = [h[1] for h in history[-(k + 1):]]
        deltas = [abs(recent[i+1] - recent[i]) for i in range(k)]
        return all(d < eps for d in deltas)

    def council_converged(self, config: Any) -> bool:
        """Check if avg council disagreement dropped below threshold."""
        window = config.convergence_window
        threshold = config.convergence_disagreement_threshold
        if len(self.council_disagreement_history) < window:
            return False
        recent = self.council_disagreement_history[-window:]
        return sum(recent) / len(recent) < threshold
