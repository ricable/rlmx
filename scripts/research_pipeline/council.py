"""
council.py — Polymorphic LLM-Council

Modes:
  PEER_REVIEW     — each member critiques an artifact; consensus arbitrates
  DIVERSIFICATION — each member generates a diverse alternative perspective

Mode switching (both active simultaneously):
  1. Stage-bound:    stage_mode_map maps pipeline stage → CouncilMode
  2. Fitness-triggered: if artifact novelty > novelty_threshold → PEER_REVIEW
  3. Per-call override: pass council_mode= kwarg to override both

Council composition is GEPA-evolvable: the GEPA meta-loop calls update_members().
"""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from typing import Any, Optional

import dspy

from .config import CouncilConfig, CouncilMode, CouncilMemberConfig
from .signatures import (
    CouncilConsensus,
    CouncilDiversify,
    CouncilPeerReview,
)

logger = logging.getLogger(__name__)


# ─── Per-member LM wrappers ───────────────────────────────────────────────────

def _build_lm(member: CouncilMemberConfig) -> dspy.LM:
    """
    Construct a DSPy LM for a council member.
    Supports: openai, anthropic, google, local (ollama), mistral.
    """
    provider = member.provider.value
    model = member.model

    if provider == "openai":
        return dspy.LM(f"openai/{model}", temperature=member.temperature)
    elif provider == "anthropic":
        return dspy.LM(f"anthropic/{model}", temperature=member.temperature)
    elif provider == "google":
        return dspy.LM(f"google/{model}", temperature=member.temperature)
    elif provider == "local":
        return dspy.LM(f"ollama_chat/{model}", temperature=member.temperature)
    elif provider == "mistral":
        return dspy.LM(f"mistral/{model}", temperature=member.temperature)
    else:
        raise ValueError(f"Unknown council provider: {provider}")


# ─── CouncilSession ───────────────────────────────────────────────────────────

@dataclass
class CouncilVote:
    member_id: str
    model: str
    mode: str
    score: float = 0.0
    output: dict[str, Any] = None

    def __post_init__(self):
        if self.output is None:
            self.output = {}


@dataclass
class CouncilResult:
    mode: CouncilMode
    votes: list[CouncilVote]
    consensus_score: float
    disagreement_level: float
    consensus_action: str          # promote | revise | reject | explore_further
    aggregated_feedback: str
    frontier_signal: bool
    revision_instructions: str = ""
    # Diversification mode extras
    new_directions: list[str] = None
    alternative_perspectives: list[str] = None

    def __post_init__(self):
        if self.new_directions is None:
            self.new_directions = []
        if self.alternative_perspectives is None:
            self.alternative_perspectives = []


# ─── Council ──────────────────────────────────────────────────────────────────

class Council:
    """
    Polymorphic LLM-Council.

    Usage:
        council = Council(config)
        result = council.evaluate(
            artifact_type="claim",
            artifact_text="...",
            artifact_novelty=0.8,
            stage="generation",
        )
    """

    def __init__(self, config: CouncilConfig) -> None:
        self.config = config
        self._peer_review  = dspy.ChainOfThought(CouncilPeerReview)
        self._diversify    = dspy.ChainOfThought(CouncilDiversify)
        self._consensus    = dspy.ChainOfThought(CouncilConsensus)

    # ── Public API ────────────────────────────────────────────────────────────

    def evaluate(
        self,
        artifact_type: str,
        artifact_text: str,
        artifact_novelty: float = 0.5,
        stage: str = "synthesis",
        council_mode: Optional[CouncilMode] = None,
        extra_context: str = "",
    ) -> CouncilResult:
        """
        Evaluate an artifact. Mode is determined by:
          1. Per-call override (council_mode kwarg) — highest priority
          2. Fitness-triggered (novelty threshold)
          3. Stage-bound map
          4. Default: PEER_REVIEW
        """
        mode = self._resolve_mode(stage, artifact_novelty, council_mode)
        votes: list[CouncilVote] = []

        for i, member in enumerate(self.config.members):
            member_id = f"{member.model}_{i}"
            try:
                vote = self._run_member(
                    member=member,
                    member_id=member_id,
                    mode=mode,
                    artifact_type=artifact_type,
                    artifact_text=artifact_text,
                    extra_context=extra_context,
                )
                votes.append(vote)
            except Exception as exc:
                logger.warning(f"Council member {member_id} failed: {exc}")
                votes.append(CouncilVote(
                    member_id=member_id,
                    model=member.model,
                    mode=mode.value,
                    score=0.5,
                    output={"error": str(exc)},
                ))

        if len(votes) < self.config.quorum:
            logger.warning(
                f"Council quorum not met: {len(votes)} < {self.config.quorum}"
            )

        result = self._aggregate(votes, mode, artifact_type, artifact_text)
        return result

    def update_members(self, new_members_json: str) -> None:
        """GEPA calls this to evolve council membership."""
        try:
            raw = json.loads(new_members_json)
            self.config.members = [
                CouncilMemberConfig(**m) for m in raw
            ]
            logger.info(f"Council updated: {len(self.config.members)} members")
        except Exception as exc:
            logger.error(f"Failed to update council members: {exc}")

    def member_summary(self) -> str:
        lines = []
        for i, m in enumerate(self.config.members):
            lines.append(
                f"  [{i}] {m.model} ({m.provider.value}) "
                f"temp={m.temperature} angle={m.diversity_angle.value} "
                f"role={m.reviewer_role}"
            )
        return "\n".join(lines)

    # ── Mode resolution ───────────────────────────────────────────────────────

    def _resolve_mode(
        self,
        stage: str,
        novelty: float,
        override: Optional[CouncilMode],
    ) -> CouncilMode:
        if override is not None and self.config.allow_override:
            return override
        if novelty >= self.config.novelty_threshold:
            return CouncilMode.PEER_REVIEW
        stage_mode = self.config.stage_mode_map.get(stage)
        if stage_mode is not None:
            return stage_mode
        return CouncilMode.PEER_REVIEW

    # ── Per-member execution ──────────────────────────────────────────────────

    def _run_member(
        self,
        member: CouncilMemberConfig,
        member_id: str,
        mode: CouncilMode,
        artifact_type: str,
        artifact_text: str,
        extra_context: str,
    ) -> CouncilVote:
        member_identity = (
            f"Model: {member.model} | Provider: {member.provider.value} | "
            f"Temp: {member.temperature} | Role: {member.reviewer_role}"
        )

        lm = _build_lm(member)

        with dspy.context(lm=lm):
            if mode == CouncilMode.PEER_REVIEW:
                pred = self._peer_review(
                    artifact_type=artifact_type,
                    artifact_text=artifact_text,
                    reviewer_perspective=(
                        f"{member_identity}\n"
                        f"Reviewer role: {member.reviewer_role}\n"
                        f"{extra_context}"
                    ),
                )
                output = {
                    "score":                 float(pred.score),
                    "critique":              pred.critique,
                    "suggested_improvements": pred.suggested_improvements,
                    "flagged_issues":        pred.flagged_issues,
                    "would_promote":         bool(pred.would_promote),
                }
                score = float(pred.score)

            else:  # DIVERSIFICATION
                pred = self._diversify(
                    artifact_type=artifact_type,
                    artifact_text=artifact_text,
                    diversity_angle=member.diversity_angle.value,
                    member_identity=member_identity,
                )
                output = {
                    "alternative_perspective": pred.alternative_perspective,
                    "new_directions":          pred.new_directions,
                    "bridging_insight":        pred.bridging_insight,
                    "challenge_to_original":   pred.challenge_to_original,
                }
                # Score diversification by novelty of perspective (self-assessed)
                score = 0.7  # default; consensus will weight by member weight

        return CouncilVote(
            member_id=member_id,
            model=member.model,
            mode=mode.value,
            score=score,
            output=output,
        )

    # ── Consensus aggregation ─────────────────────────────────────────────────

    def _aggregate(
        self,
        votes: list[CouncilVote],
        mode: CouncilMode,
        artifact_type: str,
        artifact_text: str,
    ) -> CouncilResult:
        council_outputs = [
            {"member_id": v.member_id, "model": v.model, **v.output}
            for v in votes
        ]

        pred = self._consensus(
            artifact_type=artifact_type,
            council_outputs_json=json.dumps(council_outputs, indent=2),
            council_mode=mode.value,
            artifact_text=artifact_text[:1000],  # truncate for context
        )

        # Collect diversification extras
        new_directions: list[str] = []
        alt_perspectives: list[str] = []
        if mode == CouncilMode.DIVERSIFICATION:
            for v in votes:
                raw_dirs = v.output.get("new_directions", "")
                if isinstance(raw_dirs, str):
                    try:
                        dirs = json.loads(raw_dirs)
                        new_directions.extend(dirs if isinstance(dirs, list) else [raw_dirs])
                    except json.JSONDecodeError:
                        new_directions.append(raw_dirs)
                elif isinstance(raw_dirs, list):
                    new_directions.extend(raw_dirs)

                alt = v.output.get("alternative_perspective", "")
                if alt:
                    alt_perspectives.append(alt)

        return CouncilResult(
            mode=mode,
            votes=votes,
            consensus_score=float(pred.consensus_score),
            disagreement_level=float(pred.disagreement_level),
            consensus_action=str(pred.consensus_action).strip(),
            aggregated_feedback=pred.aggregated_feedback,
            frontier_signal=bool(pred.frontier_signal),
            revision_instructions=pred.revision_instructions,
            new_directions=new_directions,
            alternative_perspectives=alt_perspectives,
        )
