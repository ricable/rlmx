"""
modules.py — DSPy modules wrapping all signatures

Each module is a dspy.Module subclass that:
  1. Wraps the corresponding signature in dspy.ChainOfThought
  2. Handles JSON parsing of list/dict output fields
  3. Populates the corresponding artifact dataclass
  4. Is registered in MODULE_REGISTRY for GEPA.patch_signatures()

Module registry key = signature class name (e.g., "GenerateClaim").
"""

from __future__ import annotations

import json
import logging
from typing import Any, Optional

import dspy

from .artifacts import (
    Claim,
    HypothesisCard,
    MiniPaper,
    ResearchThread,
)
from .signatures import (
    ExtendResearchThread,
    GenerateClaim,
    GenerateHypothesisCard,
    RefineClaimWithCouncil,
    ScoreHypothesisCard,
    SynthesizeMiniPaper,
    WeaveResearchThread,
)

logger = logging.getLogger(__name__)


# ─── Helpers ──────────────────────────────────────────────────────────────────

def _parse_json_field(raw: Any, fallback: Any = None) -> Any:
    """Safely parse a JSON string field; return fallback on failure."""
    if isinstance(raw, (list, dict)):
        return raw
    if isinstance(raw, str):
        raw = raw.strip()
        try:
            return json.loads(raw)
        except json.JSONDecodeError:
            # Try wrapping in a list
            try:
                return json.loads(f"[{raw}]")
            except json.JSONDecodeError:
                pass
    return fallback if fallback is not None else raw


def _safe_float(value: Any, default: float = 0.5) -> float:
    try:
        return max(0.0, min(1.0, float(value)))
    except (TypeError, ValueError):
        return default


# ═══════════════════════════════════════════════════════════════════════════════
# A. ARTIFACT GENERATION MODULES
# ═══════════════════════════════════════════════════════════════════════════════

class GenerateClaimModule(dspy.Module):
    """Generates a novel, falsifiable research claim."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(GenerateClaim)

    def forward(
        self,
        research_question: str,
        domain: str,
        existing_claims: list[str],
        literature_context: str,
        focus_angle: str = "broad",
        source_intent_id: str = "",
        gepa_generation: int = 0,
    ) -> Claim:
        pred = self.predict(
            research_question=research_question,
            domain=domain,
            existing_claims_json=json.dumps(existing_claims),
            literature_context=literature_context,
            focus_angle=focus_angle,
        )
        return Claim(
            text=pred.claim,
            rationale=pred.rationale,
            confidence=_safe_float(pred.confidence),
            novelty_signal=pred.novelty_signal,
            bridged_domains=pred.bridged_domains,
            domain=domain,
            source_intent_id=source_intent_id,
            gepa_generation=gepa_generation,
        )


class RefineClaimModule(dspy.Module):
    """Refines a claim using council feedback."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(RefineClaimWithCouncil)

    def forward(
        self,
        claim: Claim,
        council_critiques: str,
        council_suggestions: str,
        frontier_signal: bool = False,
    ) -> Claim:
        pred = self.predict(
            original_claim=claim.text,
            council_critiques=council_critiques,
            council_suggestions=council_suggestions,
            frontier_signal=frontier_signal,
        )
        refined = Claim(
            text=pred.refined_claim,
            rationale=claim.rationale,
            confidence=claim.confidence,
            novelty_signal=claim.novelty_signal,
            bridged_domains=claim.bridged_domains,
            domain=claim.domain,
            source_intent_id=claim.source_intent_id,
            gepa_generation=claim.gepa_generation,
            metadata={**claim.metadata, "refined_from": claim.id},
        )
        return refined


class GenerateHypothesisCardModule(dspy.Module):
    """Expands a claim into a full hypothesis card."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(GenerateHypothesisCard)

    def forward(
        self,
        claim: Claim,
        autoresearch_results: str = "",
        gepa_generation: int = 0,
    ) -> HypothesisCard:
        pred = self.predict(
            claim_text=claim.text,
            claim_rationale=claim.rationale,
            domain=claim.domain,
            autoresearch_results=autoresearch_results or "No execution results available.",
        )
        related_works = _parse_json_field(pred.related_works, fallback=[])
        if isinstance(related_works, list):
            works = [str(w) for w in related_works]
        else:
            works = [str(related_works)]

        return HypothesisCard(
            claim=claim,
            hypothesis=pred.hypothesis,
            predicted_experiment=pred.predicted_experiment,
            falsification_criteria=pred.falsification_criteria,
            methodology_sketch=pred.methodology_sketch,
            related_works=works,
            confidence_score=_safe_float(pred.confidence_score),
            gepa_generation=gepa_generation,
        )


class ScoreHypothesisCardModule(dspy.Module):
    """Scores a hypothesis card for quality."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(ScoreHypothesisCard)

    def forward(self, card: HypothesisCard) -> dict[str, Any]:
        pred = self.predict(
            hypothesis=card.hypothesis,
            falsification_criteria=card.falsification_criteria,
            predicted_experiment=card.predicted_experiment,
            related_works=json.dumps(card.related_works[:10]),
        )
        gaps     = _parse_json_field(pred.gaps, fallback=[])
        strengths = _parse_json_field(pred.strengths, fallback=[])
        return {
            "quality_score": _safe_float(pred.quality_score),
            "gaps":          gaps if isinstance(gaps, list) else [gaps],
            "strengths":     strengths if isinstance(strengths, list) else [strengths],
        }


class WeaveResearchThreadModule(dspy.Module):
    """Connects hypothesis cards into a research thread DAG."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(WeaveResearchThread)

    def forward(
        self,
        cards: list[HypothesisCard],
        topic: str,
        gepa_generation: int = 0,
    ) -> ResearchThread:
        cards_data = [
            {
                "id": c.id,
                "hypothesis": c.hypothesis,
                "falsification_criteria": c.falsification_criteria,
                "confidence_score": c.confidence_score,
            }
            for c in cards
        ]
        pred = self.predict(
            hypothesis_cards_json=json.dumps(cards_data, indent=2),
            thread_topic=topic,
        )

        # Parse DAG
        dag_raw = _parse_json_field(pred.thread_structure, fallback={})
        dag: dict[str, list[str]] = {}
        if isinstance(dag_raw, dict):
            for k, v in dag_raw.items():
                dag[k] = v if isinstance(v, list) else [v]

        # Parse edge types
        edges_raw = _parse_json_field(pred.edge_types, fallback={})
        edge_types: dict[tuple[str, str], str] = {}
        if isinstance(edges_raw, dict):
            for key, val in edges_raw.items():
                parts = key.split("→") if "→" in key else key.split("->")
                if len(parts) == 2:
                    edge_types[(parts[0].strip(), parts[1].strip())] = str(val)

        # Parse tensions
        tensions_raw = _parse_json_field(pred.key_tensions, fallback=[])
        tensions = tensions_raw if isinstance(tensions_raw, list) else [tensions_raw]

        thread = ResearchThread(
            topic=topic,
            hypothesis_cards=cards,
            dag=dag,
            edge_types=edge_types,
            key_tensions=[str(t) for t in tensions],
            synthesis_direction=pred.synthesis_direction,
            narrative_arc=pred.narrative_arc,
            gepa_generation=gepa_generation,
        )
        # Initialize dag entries for all cards
        for card in cards:
            if card.id not in thread.dag:
                thread.dag[card.id] = []

        return thread


class ExtendResearchThreadModule(dspy.Module):
    """Proposes hypothesis card extensions for an existing thread."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(ExtendResearchThread)

    def forward(self, thread: ResearchThread) -> list[dict[str, str]]:
        leaf_summaries = "\n".join(
            f"- {c.hypothesis[:100]}"
            for c in thread.hypothesis_cards
            if c.id in thread.leaf_nodes()
        )
        pred = self.predict(
            existing_thread_summary=f"{thread.topic}: {thread.narrative_arc}",
            key_tensions=json.dumps(thread.key_tensions),
            synthesis_direction=thread.synthesis_direction,
            leaf_node_summaries=leaf_summaries,
        )
        extensions = _parse_json_field(pred.proposed_extensions, fallback=[])
        return extensions if isinstance(extensions, list) else []


class SynthesizeMiniPaperModule(dspy.Module):
    """Synthesizes a research thread into a mini-paper."""

    def __init__(self) -> None:
        super().__init__()
        self.predict = dspy.ChainOfThought(SynthesizeMiniPaper)

    def forward(
        self,
        thread: ResearchThread,
        autoresearch_synthesis: str = "",
        research_question: str = "",
        gepa_generation: int = 0,
    ) -> MiniPaper:
        thread_summary = (
            f"Topic: {thread.topic}\n"
            f"Narrative: {thread.narrative_arc}\n"
            f"Tensions: {'; '.join(thread.key_tensions[:3])}\n"
            f"Direction: {thread.synthesis_direction}"
        )
        cards_data = [
            {
                "id": c.id,
                "hypothesis": c.hypothesis,
                "falsification_criteria": c.falsification_criteria,
                "confidence_score": c.confidence_score,
                "supporting_evidence": c.supporting_evidence[:3],
            }
            for c in thread.hypothesis_cards
        ]
        pred = self.predict(
            thread_summary=thread_summary,
            all_cards_json=json.dumps(cards_data, indent=2),
            autoresearch_synthesis=autoresearch_synthesis or "No synthesis available.",
            research_question=research_question,
        )

        findings_raw   = _parse_json_field(pred.findings, fallback=[])
        open_q_raw     = _parse_json_field(pred.open_questions, fallback=[])

        findings = [str(f) for f in (findings_raw if isinstance(findings_raw, list) else [findings_raw])]
        open_qs  = [str(q) for q in (open_q_raw  if isinstance(open_q_raw,  list) else [open_q_raw])]

        return MiniPaper(
            thread=thread,
            abstract=pred.abstract,
            methodology=pred.methodology,
            findings=findings,
            open_questions=open_qs,
            contribution_statement=pred.contribution_statement,
            limitations=pred.limitations,
            autoresearch_synthesis=autoresearch_synthesis,
            gepa_generation=gepa_generation,
        )


# ═══════════════════════════════════════════════════════════════════════════════
# MODULE REGISTRY
# ═══════════════════════════════════════════════════════════════════════════════

class ModuleRegistry:
    """
    Central registry of all DSPy modules.
    GEPA uses this registry to patch signature instructions at runtime.
    """

    def __init__(self) -> None:
        self.generate_claim         = GenerateClaimModule()
        self.refine_claim           = RefineClaimModule()
        self.generate_card          = GenerateHypothesisCardModule()
        self.score_card             = ScoreHypothesisCardModule()
        self.weave_thread           = WeaveResearchThreadModule()
        self.extend_thread          = ExtendResearchThreadModule()
        self.synthesize_paper       = SynthesizeMiniPaperModule()

        # Expose as dict for GEPA.patch_signatures()
        self._registry: dict[str, dspy.Module] = {
            "GenerateClaim":          self.generate_claim,
            "RefineClaimWithCouncil": self.refine_claim,
            "GenerateHypothesisCard": self.generate_card,
            "ScoreHypothesisCard":    self.score_card,
            "WeaveResearchThread":    self.weave_thread,
            "ExtendResearchThread":   self.extend_thread,
            "SynthesizeMiniPaper":    self.synthesize_paper,
        }

    def __getitem__(self, key: str) -> Optional[dspy.Module]:
        return self._registry.get(key)

    def as_dict(self) -> dict[str, dspy.Module]:
        return dict(self._registry)

    def base_instructions(self) -> dict[str, str]:
        """Extract current docstrings for GEPA initialization."""
        result = {}
        for name, module in self._registry.items():
            try:
                sig = None
                if hasattr(module, "predict") and hasattr(module.predict, "signature"):
                    sig = module.predict.signature
                elif hasattr(module, "signature"):
                    sig = module.signature
                if sig and sig.__doc__:
                    result[name] = sig.__doc__
            except Exception:
                pass
        return result
