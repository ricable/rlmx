"""
research_pipeline — Autonomous AI Research Pipeline

GEPA (Generative Evolutionary Prompt Architecture) + DSPy + LLM-Council + Karpathy Autoresearch

Architecture:
  ResearchIntent → Claims → HypothesisCards → ResearchThreads → MiniPapers

  Outer loop:  GEPA evolves DSPy signatures, council membership, fitness weights
  Inner loop:  DSPy modules generate and refine artifacts at each level
  Council:     Polymorphic (peer_review ↔ diversification), heterogeneous, GEPA-evolvable
  Autoresearch: Karpathy-style generate/execute/synthesize phases, omnipresent
  Fitness:     6 signals (novelty, falsifiability, citation_density, compression_ratio,
               council_disagreement, cross_domain_surprise) with evolvable weights
  Termination: budget → convergence → novelty_collapse → quality_gate (priority order)
  Streaming:   StreamEvent yields for real-time artifact observation

Quick start:
    import dspy
    from research_pipeline import ResearchPipeline, ResearchIntent

    lm = dspy.LM("openai/gpt-4o", api_key="...")
    pipeline = ResearchPipeline(lm=lm)

    intent = ResearchIntent(
        question="What are the fundamental limits of chain-of-thought reasoning in LLMs?",
        domain="LLM reasoning",
    )

    # Streaming
    async for event in pipeline.run(intent):
        print(f"[{event.event_type}] {event.artifact_summary} (fitness={event.fitness_score:.3f})")

    # Batch
    state = await pipeline.run_batch(intent)
    for paper in state.best_papers(3):
        print(paper.summary())
"""

from .artifacts import (
    Claim,
    HypothesisCard,
    MiniPaper,
    ResearchState,
    ResearchThread,
)
from .autoresearch import AutoresearchEngine, AutoresearchState, ResearchDirection
from .config import (
    ArtifactLevel,
    CouncilConfig,
    CouncilMemberConfig,
    CouncilMode,
    CouncilProvider,
    DiversityAngle,
    GEPAConfig,
    ResearchIntent,
    StreamEvent,
    TerminationConfig,
)
from .council import Council, CouncilResult, CouncilVote
from .fitness import FitnessEngine
from .gepa import GEPA, GEPALevel, Individual
from .modules import ModuleRegistry
from .pipeline import ResearchPipeline, TerminationError

__all__ = [
    # Pipeline entry point
    "ResearchPipeline",
    "TerminationError",
    # Config
    "ResearchIntent",
    "CouncilConfig",
    "CouncilMemberConfig",
    "CouncilMode",
    "CouncilProvider",
    "DiversityAngle",
    "GEPAConfig",
    "TerminationConfig",
    "ArtifactLevel",
    "StreamEvent",
    # Artifacts
    "Claim",
    "HypothesisCard",
    "ResearchThread",
    "MiniPaper",
    "ResearchState",
    # Components
    "Council",
    "CouncilResult",
    "CouncilVote",
    "FitnessEngine",
    "GEPA",
    "GEPALevel",
    "Individual",
    "AutoresearchEngine",
    "AutoresearchState",
    "ResearchDirection",
    "ModuleRegistry",
]

__version__ = "0.1.0"
