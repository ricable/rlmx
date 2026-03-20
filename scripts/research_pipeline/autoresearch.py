"""
autoresearch.py — Karpathy-style autoresearch integration

Three phases, omnipresent across the pipeline:

  GENERATE  → maps the literature landscape, proposes research directions,
               surfaces blank spaces, generates seed search queries
  EXECUTE   → runs searches/retrievals, analyzes results, updates confidence
  SYNTHESIZE → integrates all execution findings into meta-narratives,
               identifies higher-order patterns, signals convergence

The AutoresearchEngine wraps all three phases and provides a unified
interface to the pipeline. It accumulates knowledge across iterations.

Karpathy autoresearch principles applied here:
  - Systematically enumerate known unknowns before diving into any one
  - Treat blank spaces as first-class outputs, not side effects
  - Execute conservatively: weak evidence should lower, not raise confidence
  - Synthesis is a separate cognitive act from execution: resist premature convergence
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass, field
from typing import Any, Callable, Optional

import dspy

from .signatures import (
    AutoresearchExecute,
    AutoresearchGenerate,
    AutoresearchSynthesize,
)

logger = logging.getLogger(__name__)


# ─── Search/Retrieval adapter ─────────────────────────────────────────────────

# Type alias: a callable that takes a query string and returns raw text results
SearchFn = Callable[[str], str]


def _default_search(query: str) -> str:
    """
    Placeholder search function.
    In production, replace with:
      - Semantic Scholar API
      - ArXiv API
      - Perplexity / Tavily
      - Local vector store retrieval
    """
    return f"[No search backend configured. Query: {query}]"


# ─── Data classes ─────────────────────────────────────────────────────────────

@dataclass
class ResearchDirection:
    title: str
    description: str
    priority_rank: int = 0
    confidence_before: float = 0.5
    confidence_after: float = 0.5
    search_queries: list[str] = field(default_factory=list)
    supporting_evidence: list[str] = field(default_factory=list)
    contradicting_evidence: list[str] = field(default_factory=list)
    key_papers: list[dict[str, Any]] = field(default_factory=list)
    execution_summary: str = ""
    executed: bool = False


@dataclass
class AutoresearchState:
    """Accumulated knowledge across all autoresearch iterations."""
    directions: list[ResearchDirection] = field(default_factory=list)
    blank_spaces: list[str] = field(default_factory=list)
    literature_map: str = ""
    syntheses: list[str] = field(default_factory=list)          # one per iteration
    meta_findings: list[list[str]] = field(default_factory=list)
    confidence_landscape: dict[str, float] = field(default_factory=dict)
    convergence_signals: list[str] = field(default_factory=list)
    strongest_findings: list[str] = field(default_factory=list)
    iteration: int = 0


# ─── AutoresearchEngine ───────────────────────────────────────────────────────

class AutoresearchEngine:
    """
    Full Karpathy-style autoresearch engine.

    Usage:
        engine = AutoresearchEngine(search_fn=my_search)

        # Phase 1: Generation — call before generating claims
        gen = engine.generate(intent_summary, existing_claims=[])

        # Phase 2: Execution — call when building hypothesis cards
        exec_ = engine.execute(direction_title, prior_confidence=0.5)

        # Phase 3: Synthesis — call before synthesizing mini-papers
        synth = engine.synthesize(intent_summary, iteration=1)
    """

    def __init__(
        self,
        search_fn: Optional[SearchFn] = None,
        max_queries_per_direction: int = 3,
    ) -> None:
        self.search_fn = search_fn or _default_search
        self.max_queries = max_queries_per_direction
        self.state = AutoresearchState()

        self._generate  = dspy.ChainOfThought(AutoresearchGenerate)
        self._execute   = dspy.ChainOfThought(AutoresearchExecute)
        self._synthesize = dspy.ChainOfThought(AutoresearchSynthesize)

    # ── Phase 1: Generate ─────────────────────────────────────────────────────

    def generate(
        self,
        intent_summary: str,
        existing_claims: list[str],
        focus_angle: str = "broad",
    ) -> AutoresearchState:
        """
        Map the literature landscape and propose research directions.
        Returns updated AutoresearchState.
        """
        logger.info("Autoresearch GENERATE phase starting")

        pred = self._generate(
            research_intent=intent_summary,
            literature_landscape=self.state.literature_map or "Unknown — first iteration.",
            existing_claims_json=json.dumps(existing_claims),
            focus_angle=focus_angle,
        )

        # Parse directions
        try:
            raw_dirs = json.loads(pred.proposed_directions)
        except (json.JSONDecodeError, TypeError):
            raw_dirs = [{"title": "Main direction", "description": pred.proposed_directions}]

        # Parse priority ranking
        try:
            priority = json.loads(pred.priority_ranking)
            rank_map = {p["title"]: i for i, p in enumerate(priority)} if priority else {}
        except (json.JSONDecodeError, TypeError):
            rank_map = {}

        # Parse search queries
        try:
            all_queries = json.loads(pred.search_queries)
        except (json.JSONDecodeError, TypeError):
            all_queries = [pred.search_queries] if pred.search_queries else []

        # Parse blank spaces
        try:
            blank_spaces = json.loads(pred.blank_spaces)
        except (json.JSONDecodeError, TypeError):
            blank_spaces = [pred.blank_spaces] if pred.blank_spaces else []

        # Build direction objects
        new_directions = []
        for raw in raw_dirs:
            if isinstance(raw, str):
                raw = {"title": raw, "description": raw}
            title = raw.get("title", "Untitled")
            # Assign queries round-robin across directions
            d_idx = len(new_directions)
            queries = all_queries[d_idx::len(raw_dirs)] if raw_dirs else []
            new_directions.append(ResearchDirection(
                title=title,
                description=raw.get("description", ""),
                priority_rank=rank_map.get(title, d_idx),
                search_queries=queries[: self.max_queries],
            ))

        # Update state
        self.state.directions.extend(new_directions)
        self.state.blank_spaces.extend(blank_spaces)
        self.state.literature_map = pred.literature_map or self.state.literature_map

        logger.info(
            f"Autoresearch GENERATE: {len(new_directions)} directions, "
            f"{len(blank_spaces)} blank spaces"
        )
        return self.state

    # ── Phase 2: Execute ──────────────────────────────────────────────────────

    def execute(
        self,
        direction_title: str,
        prior_confidence: float = 0.5,
    ) -> Optional[ResearchDirection]:
        """
        Run searches for a specific direction and analyze results.
        Returns the updated ResearchDirection, or None if not found.
        """
        direction = self._find_direction(direction_title)
        if direction is None:
            logger.warning(f"Direction not found: {direction_title}")
            return None

        if direction.executed:
            logger.debug(f"Direction already executed: {direction_title}")
            return direction

        logger.info(f"Autoresearch EXECUTE: {direction_title}")

        # Run search queries
        retrieved_parts: list[str] = []
        for query in direction.search_queries[: self.max_queries]:
            try:
                result = self.search_fn(query)
                retrieved_parts.append(f"Query: {query}\n{result}")
            except Exception as exc:
                logger.warning(f"Search failed for query '{query}': {exc}")
                retrieved_parts.append(f"Query: {query}\n[Search failed: {exc}]")

        retrieved_text = "\n\n---\n\n".join(retrieved_parts)

        pred = self._execute(
            research_direction=f"{direction.title}: {direction.description}",
            search_queries_used=json.dumps(direction.search_queries),
            retrieved_results=retrieved_text,
            prior_confidence=prior_confidence,
        )

        # Parse key papers
        try:
            key_papers = json.loads(pred.key_papers_json)
        except (json.JSONDecodeError, TypeError):
            key_papers = []

        # Parse evidence lists
        def _parse_list(raw: Any) -> list[str]:
            if isinstance(raw, list):
                return raw
            try:
                parsed = json.loads(raw)
                return parsed if isinstance(parsed, list) else [str(raw)]
            except (json.JSONDecodeError, TypeError):
                return [str(raw)] if raw else []

        direction.supporting_evidence    = _parse_list(pred.supporting_evidence)
        direction.contradicting_evidence = _parse_list(pred.contradicting_evidence)
        direction.key_papers             = key_papers if isinstance(key_papers, list) else []
        direction.confidence_before      = prior_confidence
        direction.confidence_after       = float(pred.confidence_update)
        direction.execution_summary      = pred.execution_summary
        direction.executed               = True

        logger.info(
            f"Autoresearch EXECUTE done: {direction_title} "
            f"confidence {prior_confidence:.2f} → {direction.confidence_after:.2f}"
        )
        return direction

    def execute_all(self) -> AutoresearchState:
        """Execute all not-yet-executed directions."""
        for direction in self.state.directions:
            if not direction.executed:
                self.execute(direction.title, prior_confidence=direction.confidence_before)
        return self.state

    # ── Phase 3: Synthesize ───────────────────────────────────────────────────

    def synthesize(
        self,
        intent_summary: str,
        iteration: int = 1,
    ) -> AutoresearchState:
        """
        Integrate all execution findings into meta-narratives.
        Updates AutoresearchState.syntheses, .meta_findings, .convergence_signals.
        """
        logger.info(f"Autoresearch SYNTHESIZE: iteration {iteration}")

        # Serialize executed directions
        dirs_data = []
        for d in self.state.directions:
            dirs_data.append({
                "title": d.title,
                "description": d.description,
                "confidence_before": d.confidence_before,
                "confidence_after": d.confidence_after,
                "supporting_evidence": d.supporting_evidence[:5],
                "contradicting_evidence": d.contradicting_evidence[:3],
                "execution_summary": d.execution_summary,
            })

        prior_synthesis = (
            self.state.syntheses[-1] if self.state.syntheses else ""
        )

        pred = self._synthesize(
            all_directions_json=json.dumps(dirs_data, indent=2),
            research_intent=intent_summary,
            iteration_number=iteration,
            prior_synthesis=prior_synthesis,
        )

        # Parse meta findings
        try:
            meta = json.loads(pred.meta_findings)
        except (json.JSONDecodeError, TypeError):
            meta = [pred.meta_findings] if pred.meta_findings else []

        # Parse confidence landscape
        try:
            landscape = json.loads(pred.confidence_landscape)
            # Update direction confidences
            for d in self.state.directions:
                if d.title in landscape:
                    d.confidence_after = float(landscape[d.title])
        except (json.JSONDecodeError, TypeError):
            landscape = {}

        self.state.syntheses.append(pred.synthesis_narrative)
        self.state.meta_findings.append(meta if isinstance(meta, list) else [])
        self.state.confidence_landscape.update(landscape)
        self.state.convergence_signals.append(pred.convergence_signal)
        self.state.strongest_findings.append(pred.strongest_finding)
        self.state.iteration = iteration

        logger.info(
            f"Autoresearch SYNTHESIZE done: convergence={pred.convergence_signal}"
        )
        return self.state

    # ── Accessors ─────────────────────────────────────────────────────────────

    def literature_context_for_claim(self) -> str:
        """Condensed literature summary for injection into GenerateClaim."""
        parts = [self.state.literature_map or ""]
        if self.state.blank_spaces:
            parts.append("Known blank spaces: " + "; ".join(self.state.blank_spaces[:3]))
        if self.state.strongest_findings:
            parts.append("Strongest findings: " + "; ".join(self.state.strongest_findings[-2:]))
        return "\n".join(p for p in parts if p)

    def execution_results_for_card(self, direction_title: str) -> str:
        """Execution results for injection into GenerateHypothesisCard."""
        d = self._find_direction(direction_title)
        if d is None or not d.executed:
            return "No execution results available."
        papers = "\n".join(
            f"- {p.get('title', 'Unknown')} (relevance={p.get('relevance_score', '?')}): "
            f"{p.get('key_finding', '')}"
            for p in d.key_papers[:5]
        )
        return (
            f"Summary: {d.execution_summary}\n\n"
            f"Key papers:\n{papers}\n\n"
            f"Supporting: {'; '.join(d.supporting_evidence[:3])}\n"
            f"Contradicting: {'; '.join(d.contradicting_evidence[:2])}"
        )

    def synthesis_for_paper(self) -> str:
        """Latest synthesis narrative for injection into SynthesizeMiniPaper."""
        if not self.state.syntheses:
            return "No synthesis available yet."
        latest = self.state.syntheses[-1]
        meta = self.state.meta_findings[-1] if self.state.meta_findings else []
        meta_str = "\n".join(f"- {m}" for m in meta[:5])
        return f"{latest}\n\nMeta-findings:\n{meta_str}"

    def reset_for_next_generation(self) -> None:
        """Keep accumulated knowledge but reset execution flags for fresh generation."""
        for d in self.state.directions:
            d.executed = False

    # ── Internal helpers ──────────────────────────────────────────────────────

    def _find_direction(self, title: str) -> Optional[ResearchDirection]:
        for d in self.state.directions:
            if d.title.lower() == title.lower():
                return d
        # Fuzzy fallback: partial match
        for d in self.state.directions:
            if title.lower() in d.title.lower() or d.title.lower() in title.lower():
                return d
        return None
