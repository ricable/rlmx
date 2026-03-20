"""
pipeline.py — ResearchPipeline: main orchestration

Full architecture:
  ResearchIntent → [Autoresearch.generate]
    → for each breadth slot:
        GenerateClaim → Council(stage=generation) → [refine if needed]
        → score_claim → fitness(claim)
    → [GEPA.evolve(CLAIM level)]

    → for each claim:
        [Autoresearch.execute(direction)]
        → GenerateHypothesisCard → Council(stage=execution) → score_card
        → fitness(card)
    → [GEPA.evolve(HYPOTHESIS_CARD level)]

    → WeaveResearchThread(cards) → ExtendResearchThread (if needed)
    → fitness(thread)
    → [GEPA.evolve(RESEARCH_THREAD level)]

    → [Autoresearch.synthesize]
    → SynthesizeMiniPaper → CouncilPeerReview(stage=synthesis)
    → fitness(paper) → quality gate check
    → [GEPA.evolve(MINI_PAPER level)]
    → [GEPA.evolve_council + GEPA.evolve_fitness_weights every K gens]

  Termination checked after each mini-paper:
    priority: budget → convergence → novelty_collapse → quality_gate

  Streaming: StreamEvent emitted after each artifact is produced.
"""

from __future__ import annotations

import asyncio
import json
import logging
import time
from collections.abc import AsyncGenerator
from typing import Any, Optional

import dspy

from .artifacts import (
    Claim,
    HypothesisCard,
    MiniPaper,
    ResearchState,
    ResearchThread,
)
from .autoresearch import AutoresearchEngine
from .config import ArtifactLevel, CouncilMode, ResearchIntent, StreamEvent
from .council import Council, CouncilResult
from .fitness import FitnessEngine
from .gepa import GEPA
from .modules import ModuleRegistry

logger = logging.getLogger(__name__)


class TerminationError(Exception):
    """Raised internally when a termination condition is met."""
    def __init__(self, reason: str) -> None:
        super().__init__(reason)
        self.reason = reason


class ResearchPipeline:
    """
    Autonomous research pipeline.

    Usage (batch):
        pipeline = ResearchPipeline(lm=dspy.LM("openai/gpt-4o"))
        state = await pipeline.run_batch(intent)

    Usage (streaming):
        async for event in pipeline.run(intent):
            print(event.event_type, event.artifact_summary)
    """

    def __init__(
        self,
        lm: Optional[dspy.LM] = None,
        search_fn=None,
        lm_call_cost_estimate: float = 0.002,  # $ per call
    ) -> None:
        if lm is not None:
            dspy.configure(lm=lm)

        self.call_cost = lm_call_cost_estimate
        self.modules  = ModuleRegistry()
        self._search_fn = search_fn

    # ── Public API ────────────────────────────────────────────────────────────

    async def run(
        self,
        intent: ResearchIntent,
    ) -> AsyncGenerator[StreamEvent, None]:
        """
        Streaming run: yields StreamEvents as artifacts are produced.
        Completes when a termination condition is met.
        """
        if not intent.validate():
            raise ValueError("ResearchIntent must have at least one of: question, seed_abstract, domain")

        state   = ResearchState(intent=intent)
        fitness = FitnessEngine(intent.gepa)
        council = Council(intent.council)
        gepa    = GEPA(intent.gepa, fitness, council)
        ar      = AutoresearchEngine(search_fn=self._search_fn)

        # Initialize GEPA from current signature instructions
        gepa.initialize(self.modules.base_instructions())

        intent_summary = intent.summary()
        llm_budget     = intent.termination.max_llm_calls

        try:
            async for event in self._main_loop(
                state=state,
                intent=intent,
                fitness=fitness,
                council=council,
                gepa=gepa,
                ar=ar,
                intent_summary=intent_summary,
                llm_budget=llm_budget,
            ):
                yield event
        except TerminationError as exc:
            state.terminated = True
            state.termination_reason = exc.reason
            yield StreamEvent(
                event_type="termination",
                level=ArtifactLevel.MINI_PAPER,
                artifact_id="terminal",
                artifact_summary=f"Pipeline terminated: {exc.reason}",
                generation=state.generation,
                metadata={
                    "reason": exc.reason,
                    "papers_produced": len(state.mini_papers),
                    "total_claims": len(state.claims),
                    "best_papers": [p.summary() for p in state.best_papers(3)],
                },
            )

    async def run_batch(self, intent: ResearchIntent) -> ResearchState:
        """Batch run: collect all events and return final state."""
        state = ResearchState(intent=intent)
        async for _event in self.run(intent):
            pass
        return state

    # ── Main loop ─────────────────────────────────────────────────────────────

    async def _main_loop(
        self,
        state: ResearchState,
        intent: ResearchIntent,
        fitness: FitnessEngine,
        council: Council,
        gepa: GEPA,
        ar: AutoresearchEngine,
        intent_summary: str,
        llm_budget: int,
    ) -> AsyncGenerator[StreamEvent, None]:
        for generation in range(intent.gepa.generations):
            state.generation = generation
            logger.info(f"=== Pipeline generation {generation} ===")

            # ── Phase 1: Autoresearch.generate ────────────────────────────────
            ar.generate(
                intent_summary=intent_summary,
                existing_claims=[c.text for c in state.claims],
                focus_angle=intent.focus_angle,
            )
            self._tick_calls(state, n=2)
            self._check_termination(state, intent)

            # ── Phase 2: Claim generation ─────────────────────────────────────
            new_claims: list[Claim] = []
            for _ in range(intent.breadth):
                claim = self._generate_claim(state, intent, ar, generation)
                self._tick_calls(state)

                # Council (generation stage → DIVERSIFICATION mode by default)
                council_result = council.evaluate(
                    artifact_type="claim",
                    artifact_text=f"{claim.text}\n\nRationale: {claim.rationale}",
                    artifact_novelty=claim.confidence,
                    stage="generation",
                )
                self._tick_calls(state, n=len(intent.council.members))
                self._apply_council_to_claim(claim, council_result)

                # Refine if council says revise
                if council_result.consensus_action == "revise":
                    claim = self.modules.refine_claim(
                        claim=claim,
                        council_critiques=council_result.aggregated_feedback,
                        council_suggestions="; ".join(
                            v.output.get("suggested_improvements", "")
                            for v in council_result.votes
                        ),
                        frontier_signal=council_result.frontier_signal,
                    )
                    self._tick_calls(state)

                # Score
                fitness.score_claim(claim, ar.literature_context_for_claim())
                self._tick_calls(state)

                if council_result.consensus_action != "reject":
                    state.claims.append(claim)
                    new_claims.append(claim)
                    state.fitness_history["claim"].append((generation, claim.fitness_score))

                    yield StreamEvent(
                        event_type="claim_produced",
                        level=ArtifactLevel.CLAIM,
                        artifact_id=claim.id,
                        artifact_summary=claim.summary(),
                        fitness_score=claim.fitness_score,
                        generation=generation,
                        metadata={
                            "council_action": council_result.consensus_action,
                            "frontier": council_result.frontier_signal,
                            "disagreement": council_result.disagreement_level,
                        },
                    )

                self._check_termination(state, intent)

            # GEPA: evolve claim-level signatures
            if new_claims and intent.gepa.evolve_signatures:
                claim_scores = [c.fitness_score for c in new_claims]
                failure_patterns = self._detect_failures(new_claims, "claim")
                new_instructions = gepa.evolve_signatures(
                    level=ArtifactLevel.CLAIM,
                    recent_fitness_scores=claim_scores,
                    failure_patterns=failure_patterns,
                )
                gepa.patch_signatures(self.modules.as_dict(), new_instructions)
                self._tick_calls(state, n=len(new_instructions))

                yield StreamEvent(
                    event_type="gepa_generation",
                    level=ArtifactLevel.CLAIM,
                    artifact_id=f"gepa_claim_gen{generation}",
                    artifact_summary=f"GEPA evolved {len(new_instructions)} claim signatures",
                    generation=generation,
                    metadata={"new_instructions": list(new_instructions.keys())},
                )

            # ── Phase 3: Hypothesis card generation ───────────────────────────
            new_cards: list[HypothesisCard] = []
            for claim in new_claims:
                # Autoresearch.execute for this claim's direction
                ar_direction = ar.state.directions[0] if ar.state.directions else None
                ar_results = ""
                if ar_direction:
                    executed = ar.execute(
                        direction_title=ar_direction.title,
                        prior_confidence=claim.confidence,
                    )
                    if executed:
                        ar_results = ar.execution_results_for_card(ar_direction.title)
                    self._tick_calls(state, n=len(ar_direction.search_queries or []) + 1)

                card = self.modules.generate_card(
                    claim=claim,
                    autoresearch_results=ar_results,
                    gepa_generation=generation,
                )
                self._tick_calls(state)

                # Council (execution stage → DIVERSIFICATION or PEER_REVIEW)
                card_text = (
                    f"Hypothesis: {card.hypothesis}\n"
                    f"Experiment: {card.predicted_experiment}\n"
                    f"Falsification: {card.falsification_criteria}"
                )
                council_result = council.evaluate(
                    artifact_type="hypothesis_card",
                    artifact_text=card_text,
                    artifact_novelty=claim.fitness_score,
                    stage="execution",
                )
                self._tick_calls(state, n=len(intent.council.members))
                self._apply_council_to_card(card, council_result)

                # Score
                fitness.score_hypothesis_card(card, ar.literature_context_for_claim())
                self._tick_calls(state)

                # Score quality
                quality = self.modules.score_card(card)
                card.metadata["quality"] = quality
                self._tick_calls(state)

                if council_result.consensus_action != "reject":
                    state.hypothesis_cards.append(card)
                    new_cards.append(card)
                    state.fitness_history["hypothesis_card"].append(
                        (generation, card.fitness_score)
                    )

                    yield StreamEvent(
                        event_type="card_produced",
                        level=ArtifactLevel.HYPOTHESIS_CARD,
                        artifact_id=card.id,
                        artifact_summary=card.summary(),
                        fitness_score=card.fitness_score,
                        generation=generation,
                        metadata={
                            "council_action": council_result.consensus_action,
                            "frontier": council_result.frontier_signal,
                            "new_directions": council_result.new_directions[:3],
                        },
                    )

                self._check_termination(state, intent)

            # GEPA: evolve card-level signatures
            if new_cards and intent.gepa.evolve_signatures:
                card_scores = [c.fitness_score for c in new_cards]
                failure_patterns = self._detect_failures(new_cards, "card")
                new_instructions = gepa.evolve_signatures(
                    level=ArtifactLevel.HYPOTHESIS_CARD,
                    recent_fitness_scores=card_scores,
                    failure_patterns=failure_patterns,
                )
                gepa.patch_signatures(self.modules.as_dict(), new_instructions)
                self._tick_calls(state, n=len(new_instructions))

            # ── Phase 4: Research thread weaving ──────────────────────────────
            if len(new_cards) >= 2:
                thread_topic = intent.question or intent.domain or "Research thread"
                thread = self.modules.weave_thread(
                    cards=new_cards,
                    topic=thread_topic,
                    gepa_generation=generation,
                )
                self._tick_calls(state)

                # Optionally extend thread with new cards
                if len(new_cards) < intent.depth:
                    extensions = self.modules.extend_thread(thread)
                    self._tick_calls(state)
                    thread.metadata["proposed_extensions"] = extensions

                fitness.score_research_thread(thread)

                # Council on thread (execution stage)
                thread_text = (
                    f"Topic: {thread.topic}\n"
                    f"Narrative: {thread.narrative_arc}\n"
                    f"Tensions: {'; '.join(thread.key_tensions[:3])}\n"
                    f"Direction: {thread.synthesis_direction}"
                )
                thread_council = council.evaluate(
                    artifact_type="research_thread",
                    artifact_text=thread_text,
                    artifact_novelty=thread.fitness_score,
                    stage="execution",
                )
                self._tick_calls(state, n=len(intent.council.members))
                thread.council_disagreement = thread_council.disagreement_level
                thread.council_action = thread_council.consensus_action
                state.council_disagreement_history.append(thread_council.disagreement_level)

                state.research_threads.append(thread)
                state.fitness_history["research_thread"].append(
                    (generation, thread.fitness_score)
                )

                yield StreamEvent(
                    event_type="thread_produced",
                    level=ArtifactLevel.RESEARCH_THREAD,
                    artifact_id=thread.id,
                    artifact_summary=thread.summary(),
                    fitness_score=thread.fitness_score,
                    generation=generation,
                    metadata={
                        "tensions": thread.key_tensions[:3],
                        "direction": thread.synthesis_direction[:100],
                        "frontier": thread_council.frontier_signal,
                    },
                )

                # GEPA: evolve thread-level signatures
                if intent.gepa.evolve_signatures:
                    gepa.evolve_signatures(
                        level=ArtifactLevel.RESEARCH_THREAD,
                        recent_fitness_scores=[thread.fitness_score],
                        failure_patterns=self._detect_failures([thread], "thread"),
                    )
                    self._tick_calls(state)

                self._check_termination(state, intent)

                # ── Phase 5: Autoresearch.synthesize + MiniPaper ──────────────
                ar.synthesize(intent_summary=intent_summary, iteration=generation + 1)
                self._tick_calls(state, n=2)

                paper = self.modules.synthesize_paper(
                    thread=thread,
                    autoresearch_synthesis=ar.synthesis_for_paper(),
                    research_question=intent.question or "",
                    gepa_generation=generation,
                )
                self._tick_calls(state)

                # Council peer-review (synthesis stage → PEER_REVIEW by default)
                paper_text = (
                    f"Abstract: {paper.abstract}\n"
                    f"Methodology: {paper.methodology}\n"
                    f"Findings: {json.dumps(paper.findings)}\n"
                    f"Contribution: {paper.contribution_statement}"
                )
                paper_council = council.evaluate(
                    artifact_type="mini_paper",
                    artifact_text=paper_text,
                    artifact_novelty=thread.fitness_score,
                    stage="synthesis",
                )
                self._tick_calls(state, n=len(intent.council.members))

                # Record council results on paper
                for vote in paper_council.votes:
                    paper.council_peer_review[vote.member_id] = vote.output
                paper.council_consensus_score = paper_council.consensus_score
                paper.council_action = paper_council.consensus_action
                state.council_disagreement_history.append(paper_council.disagreement_level)

                # Score paper (quality = fitness * 0.6 + council * 0.4)
                fitness.score_mini_paper(paper)
                self._tick_calls(state)

                # Quality gate check
                paper.passed_quality_gate = (
                    paper.quality_score >= intent.termination.quality_gate_score
                )

                state.mini_papers.append(paper)
                state.fitness_history["mini_paper"].append(
                    (generation, paper.fitness_score)
                )

                yield StreamEvent(
                    event_type="paper_produced",
                    level=ArtifactLevel.MINI_PAPER,
                    artifact_id=paper.id,
                    artifact_summary=paper.summary(),
                    fitness_score=paper.fitness_score,
                    generation=generation,
                    metadata={
                        "quality_score": paper.quality_score,
                        "passed_gate": paper.passed_quality_gate,
                        "council_action": paper_council.consensus_action,
                        "open_questions": paper.open_questions[:2],
                    },
                )

                # GEPA: evolve paper-level signatures
                if intent.gepa.evolve_signatures:
                    new_instructions = gepa.evolve_signatures(
                        level=ArtifactLevel.MINI_PAPER,
                        recent_fitness_scores=[paper.fitness_score],
                        failure_patterns="",
                    )
                    gepa.patch_signatures(self.modules.as_dict(), new_instructions)
                    self._tick_calls(state, n=len(new_instructions))

            # ── GEPA: evolve council + fitness weights every 2 generations ────
            if generation > 0 and generation % 2 == 0:
                if intent.gepa.evolve_council:
                    claim_hist = state.fitness_history["claim"]
                    gepa.evolve_council(
                        intent_summary=intent_summary,
                        fitness_history=claim_hist,
                        disagreement_history=state.council_disagreement_history,
                    )
                    self._tick_calls(state, n=2)

                if intent.gepa.evolve_fitness_weights:
                    stage = gepa.research_stage(generation, intent.gepa.generations)
                    trend = gepa.fitness_trend(state.fitness_history.get("mini_paper", []))
                    signal_perf = self._estimate_signal_correlations(state)
                    gepa.evolve_fitness_weights(
                        signal_performance=signal_perf,
                        research_stage=stage,
                        fitness_trend=trend,
                    )
                    self._tick_calls(state, n=2)

            self._check_termination(state, intent)

    # ── Termination ───────────────────────────────────────────────────────────

    def _check_termination(
        self,
        state: ResearchState,
        intent: ResearchIntent,
    ) -> None:
        tc = intent.termination
        for condition in tc.priority:
            if condition == "budget":
                elapsed = state.elapsed_seconds()
                if state.llm_call_count >= tc.max_llm_calls:
                    raise TerminationError(
                        f"Budget: LLM calls ({state.llm_call_count}) >= {tc.max_llm_calls}"
                    )
                if elapsed >= tc.max_seconds:
                    raise TerminationError(
                        f"Budget: time ({elapsed:.0f}s) >= {tc.max_seconds:.0f}s"
                    )
                if state.total_cost_usd >= tc.max_cost_usd:
                    raise TerminationError(
                        f"Budget: cost (${state.total_cost_usd:.2f}) >= ${tc.max_cost_usd:.2f}"
                    )

            elif condition == "convergence":
                if state.council_converged(tc):
                    raise TerminationError(
                        "Convergence: council disagreement dropped below threshold — frontier exhausted"
                    )

            elif condition == "novelty_collapse":
                if state.novelty_collapsed(tc):
                    raise TerminationError(
                        f"Novelty collapse: fitness delta < {tc.novelty_collapse_delta} "
                        f"for {tc.novelty_collapse_generations} consecutive generations"
                    )

            elif condition == "quality_gate":
                if any(p.passed_quality_gate for p in state.mini_papers):
                    best = max(state.mini_papers, key=lambda p: p.quality_score)
                    raise TerminationError(
                        f"Quality gate: paper {best.id[:8]} reached score {best.quality_score:.3f} "
                        f">= {tc.quality_gate_score}"
                    )

    # ── Helpers ───────────────────────────────────────────────────────────────

    def _tick_calls(self, state: ResearchState, n: int = 1) -> None:
        state.llm_call_count += n
        state.total_cost_usd += n * self.call_cost

    @staticmethod
    def _apply_council_to_claim(claim: Claim, result: CouncilResult) -> None:
        claim.council_mode_used  = result.mode.value
        claim.council_disagreement = result.disagreement_level
        claim.council_action     = result.consensus_action
        claim.metadata["council_feedback"] = result.aggregated_feedback
        claim.metadata["council_frontier"] = result.frontier_signal

    @staticmethod
    def _apply_council_to_card(card: HypothesisCard, result: CouncilResult) -> None:
        card.council_reviews = [
            {"score": v.score, **v.output}
            for v in result.votes
        ]
        card.council_disagreement = result.disagreement_level
        card.council_action = result.consensus_action
        if result.new_directions:
            card.metadata["council_new_directions"] = result.new_directions

    @staticmethod
    def _detect_failures(artifacts: list, kind: str) -> str:
        """Simple failure pattern detection for GEPA feedback."""
        low = [a for a in artifacts if getattr(a, "fitness_score", 0.5) < 0.4]
        if not low:
            return f"No significant failures detected in {kind} generation."
        patterns = []
        if kind == "claim":
            vague = [a for a in low if len(getattr(a, "text", "").split()) < 8]
            if vague:
                patterns.append(f"{len(vague)} claims are too short/vague")
        elif kind == "card":
            no_experiment = [
                a for a in low
                if not getattr(a, "predicted_experiment", "").strip()
            ]
            if no_experiment:
                patterns.append(f"{len(no_experiment)} cards lack concrete experiments")
        return "; ".join(patterns) if patterns else f"{len(low)} low-fitness {kind}s produced"

    @staticmethod
    def _estimate_signal_correlations(state: ResearchState) -> dict[str, float]:
        """
        Estimate how each fitness signal correlates with final paper quality.
        Naive implementation: average signal value from paper fitness breakdowns.
        A real implementation would compute Pearson correlation against quality_score.
        """
        if not state.mini_papers:
            return {
                "novelty": 0.5, "falsifiability": 0.5,
                "citation_density": 0.5, "compression_ratio": 0.5,
                "council_disagreement": 0.5, "cross_domain_surprise": 0.5,
            }
        signals = [
            "novelty", "falsifiability", "citation_density",
            "compression_ratio", "council_disagreement", "cross_domain_surprise"
        ]
        result = {}
        for sig in signals:
            vals = [
                p.fitness_breakdown.get(sig, 0.5)
                for p in state.mini_papers
                if p.fitness_breakdown
            ]
            result[sig] = sum(vals) / max(len(vals), 1)
        return result
