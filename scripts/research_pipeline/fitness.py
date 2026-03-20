"""
fitness.py — 6-signal composite fitness engine with GEPA-evolvable weights

Signals:
  1. novelty              — divergence from existing literature embedding space
  2. falsifiability       — how testable/falsifiable the claim/hypothesis is
  3. citation_density     — breadth of literature the artifact connects to
  4. compression_ratio    — information density (how much a paper compresses a thread)
  5. council_disagreement — high variance = frontier territory worth exploring
  6. cross_domain_surprise — bridges two or more distant domains

FitnessEngine scores at each artifact level (Claim, HypothesisCard, MiniPaper).
GEPA calls update_weights() to evolve the composite formula.
"""

from __future__ import annotations

import math
from typing import TYPE_CHECKING, Any

import dspy

if TYPE_CHECKING:
    from .artifacts import Claim, HypothesisCard, MiniPaper, ResearchThread
    from .config import GEPAConfig


# ─── DSPy Scoring Signatures ──────────────────────────────────────────────────

class NoveltyScorer(dspy.Signature):
    """
    Score the novelty of a research artifact against the known literature landscape.
    A truly novel claim addresses a gap nobody has formally studied yet.
    Penalize incremental extensions of well-covered territory.
    """
    artifact_text: str = dspy.InputField(
        desc="Full text of the artifact to evaluate (claim, hypothesis, or abstract)"
    )
    literature_summary: str = dspy.InputField(
        desc="Concise summary of the most relevant existing work in this space"
    )
    novelty_score: float = dspy.OutputField(
        desc="Novelty score 0.0 (fully covered) to 1.0 (genuinely unexplored)"
    )
    novelty_explanation: str = dspy.OutputField(
        desc="One-paragraph explanation of what is or isn't novel about this artifact"
    )
    closest_existing_work: str = dspy.OutputField(
        desc="The closest existing work this artifact resembles, or 'none found'"
    )


class FalsifiabilityScorer(dspy.Signature):
    """
    Score how falsifiable and empirically testable a hypothesis is.
    A high-scoring hypothesis has clear falsification criteria and a
    concrete experimental path. Vague or unfalsifiable statements score low.
    """
    hypothesis_text: str = dspy.InputField(
        desc="The hypothesis or claim to evaluate for falsifiability"
    )
    falsification_criteria: str = dspy.InputField(
        desc="Stated falsification criteria (may be empty if not yet defined)"
    )
    falsifiability_score: float = dspy.OutputField(
        desc="Falsifiability score 0.0 (unfalsifiable) to 1.0 (clearly testable)"
    )
    test_design: str = dspy.OutputField(
        desc="A concrete experiment or test that would falsify this hypothesis"
    )
    blocking_issues: str = dspy.OutputField(
        desc="What prevents this from being testable, if anything"
    )


class CrossDomainSurpriseScorer(dspy.Signature):
    """
    Score the surprise value of a cross-domain connection.
    High scores go to insights that bridge two semantically distant domains
    in a non-obvious way that opens new research territory in both.
    Adjacent domain connections score lower than genuinely surprising bridges.
    """
    artifact_text: str = dspy.InputField(
        desc="Text of the artifact making the cross-domain connection"
    )
    primary_domain: str = dspy.InputField(
        desc="The primary domain of this research artifact"
    )
    bridged_domains: str = dspy.InputField(
        desc="Comma-separated list of other domains this artifact references"
    )
    surprise_score: float = dspy.OutputField(
        desc="Cross-domain surprise 0.0 (trivially adjacent) to 1.0 (genuinely surprising bridge)"
    )
    bridge_insight: str = dspy.OutputField(
        desc="The key insight that makes this cross-domain connection valuable"
    )
    distance_estimate: str = dspy.OutputField(
        desc="Estimated semantic distance between the domains: adjacent | moderate | distant | extreme"
    )


# ─── FitnessEngine ────────────────────────────────────────────────────────────

class FitnessEngine:
    """
    Composite fitness scorer.

    Weights are evolvable: GEPA calls update_weights() each generation.
    Each artifact level uses all applicable signals; missing signals
    are imputed at 0.5 (neutral) to avoid penalizing early-stage artifacts.
    """

    def __init__(self, config: "GEPAConfig") -> None:
        self.weights = self._normalize(config.fitness_weights.copy())
        self._novelty    = dspy.ChainOfThought(NoveltyScorer)
        self._falsify    = dspy.ChainOfThought(FalsifiabilityScorer)
        self._xdomain    = dspy.ChainOfThought(CrossDomainSurpriseScorer)

    # ── Public scoring API ────────────────────────────────────────────────────

    def score_claim(self, claim: "Claim", literature_summary: str = "") -> float:
        scores: dict[str, float] = {}

        nov = self._novelty(
            artifact_text=claim.text,
            literature_summary=literature_summary or "No prior literature provided.",
        )
        scores["novelty"] = self._clamp(nov.novelty_score)

        fals = self._falsify(
            hypothesis_text=claim.text + "\n" + claim.rationale,
            falsification_criteria="",
        )
        scores["falsifiability"] = self._clamp(fals.falsifiability_score)

        # Cross-domain from bridged_domains field
        if claim.bridged_domains:
            xd = self._xdomain(
                artifact_text=claim.text,
                primary_domain=claim.domain,
                bridged_domains=claim.bridged_domains,
            )
            scores["cross_domain_surprise"] = self._clamp(xd.surprise_score)
        else:
            scores["cross_domain_surprise"] = 0.3

        # Signals with insufficient data at claim level → neutral imputation
        scores["citation_density"]     = 0.5
        scores["compression_ratio"]    = 0.5
        scores["council_disagreement"] = claim.council_disagreement or 0.5

        composite = self._composite(scores)
        claim.fitness_breakdown = scores
        claim.fitness_score = composite
        return composite

    def score_hypothesis_card(
        self,
        card: "HypothesisCard",
        literature_summary: str = "",
    ) -> float:
        scores: dict[str, float] = {}

        # Novelty
        nov = self._novelty(
            artifact_text=card.hypothesis,
            literature_summary=literature_summary or " ".join(card.related_works[:3]),
        )
        scores["novelty"] = self._clamp(nov.novelty_score)

        # Falsifiability — full criteria available
        fals = self._falsify(
            hypothesis_text=card.hypothesis,
            falsification_criteria=card.falsification_criteria,
        )
        scores["falsifiability"] = self._clamp(fals.falsifiability_score)

        # Citation density: normalize against a target of 15 related works
        scores["citation_density"] = min(card.citation_count() / 15.0, 1.0)

        # Compression ratio: N/A until mini-paper level → neutral
        scores["compression_ratio"] = 0.5

        # Council disagreement from reviews
        if card.council_reviews:
            review_scores = [float(r.get("score", 0.5)) for r in card.council_reviews]
            scores["council_disagreement"] = min(self._variance(review_scores) * 5.0, 1.0)
        else:
            scores["council_disagreement"] = card.council_disagreement or 0.5

        # Cross-domain
        domains = ", ".join(card.related_works[:3])
        if domains:
            xd = self._xdomain(
                artifact_text=card.hypothesis,
                primary_domain=card.claim.domain,
                bridged_domains=domains,
            )
            scores["cross_domain_surprise"] = self._clamp(xd.surprise_score)
        else:
            scores["cross_domain_surprise"] = 0.3

        composite = self._composite(scores)
        card.fitness_breakdown = scores
        card.fitness_score = composite
        return composite

    def score_research_thread(self, thread: "ResearchThread") -> float:
        """Average card fitness + thread-level DAG coverage bonus."""
        if not thread.hypothesis_cards:
            return 0.0
        card_scores = [c.fitness_score for c in thread.hypothesis_cards if c.fitness_score > 0]
        base = sum(card_scores) / max(len(card_scores), 1)

        # DAG coverage bonus: more connected DAG = richer thread
        edges = sum(len(v) for v in thread.dag.values())
        coverage_bonus = min(edges / (len(thread.hypothesis_cards) * 2), 0.15)

        # Tension bonus: unresolved tensions = frontier
        tension_bonus = min(len(thread.key_tensions) * 0.02, 0.10)

        composite = min(base + coverage_bonus + tension_bonus, 1.0)
        thread.fitness_score = composite
        return composite

    def score_mini_paper(self, paper: "MiniPaper") -> float:
        scores: dict[str, float] = {}

        # Novelty from abstract
        all_works = paper.thread.all_related_works()
        nov = self._novelty(
            artifact_text=paper.abstract,
            literature_summary=" | ".join(all_works[:5]),
        )
        scores["novelty"] = self._clamp(nov.novelty_score)

        # Falsifiability from open questions
        open_q_text = " ".join(paper.open_questions[:3])
        fals = self._falsify(
            hypothesis_text=open_q_text or paper.abstract,
            falsification_criteria=paper.methodology,
        )
        scores["falsifiability"] = self._clamp(fals.falsifiability_score)

        # Citation density: all unique works across the thread
        scores["citation_density"] = min(len(all_works) / 25.0, 1.0)

        # Compression ratio: thread_words / paper_words (higher = better compression)
        thread_words = sum(
            len(c.hypothesis.split()) for c in paper.thread.hypothesis_cards
        )
        paper_words = paper.word_count()
        if paper_words > 0 and thread_words > 0:
            ratio = thread_words / paper_words
            # Ideal compression ~5–10x; normalize to 0–1
            scores["compression_ratio"] = min(ratio / 10.0, 1.0)
        else:
            scores["compression_ratio"] = 0.5

        # Council disagreement
        if paper.council_peer_review:
            review_scores = [
                float(v.get("score", 0.5))
                for v in paper.council_peer_review.values()
            ]
            scores["council_disagreement"] = min(self._variance(review_scores) * 5.0, 1.0)
        else:
            scores["council_disagreement"] = 0.5

        # Cross-domain
        all_domains = list({c.claim.domain for c in paper.thread.hypothesis_cards if c.claim.domain})
        if len(all_domains) > 1:
            xd = self._xdomain(
                artifact_text=paper.abstract,
                primary_domain=all_domains[0],
                bridged_domains=", ".join(all_domains[1:]),
            )
            scores["cross_domain_surprise"] = self._clamp(xd.surprise_score)
        else:
            scores["cross_domain_surprise"] = 0.3

        composite = self._composite(scores)
        paper.fitness_breakdown = scores
        paper.fitness_score = composite
        # Quality score = composite weighted by council consensus
        paper.quality_score = composite * 0.6 + paper.council_consensus_score * 0.4
        return composite

    # ── GEPA interface ────────────────────────────────────────────────────────

    def update_weights(self, new_weights: dict[str, float]) -> None:
        """GEPA calls this each generation to evolve the fitness formula."""
        self.weights = self._normalize(new_weights)

    def current_weights(self) -> dict[str, float]:
        return self.weights.copy()

    def signal_report(self, scores: dict[str, float]) -> str:
        """Human-readable breakdown for GEPA feedback."""
        lines = ["Fitness signal breakdown:"]
        for signal, score in scores.items():
            w = self.weights.get(signal, 0.0)
            lines.append(f"  {signal:25s} score={score:.3f}  weight={w:.3f}  contrib={score*w:.4f}")
        total = self._composite(scores)
        lines.append(f"  {'COMPOSITE':25s} {total:.4f}")
        return "\n".join(lines)

    # ── Internal helpers ──────────────────────────────────────────────────────

    def _composite(self, scores: dict[str, float]) -> float:
        total = sum(self.weights.get(k, 0.0) * v for k, v in scores.items())
        weight_sum = sum(self.weights.values())
        return self._clamp(total / max(weight_sum, 1e-9))

    @staticmethod
    def _normalize(weights: dict[str, float]) -> dict[str, float]:
        total = sum(weights.values())
        if total < 1e-9:
            n = len(weights)
            return {k: 1.0 / n for k in weights}
        return {k: v / total for k, v in weights.items()}

    @staticmethod
    def _variance(values: list[float]) -> float:
        if not values:
            return 0.0
        mean = sum(values) / len(values)
        return sum((v - mean) ** 2 for v in values) / max(len(values), 1)

    @staticmethod
    def _clamp(value: Any, lo: float = 0.0, hi: float = 1.0) -> float:
        try:
            return max(lo, min(hi, float(value)))
        except (TypeError, ValueError):
            return 0.5
