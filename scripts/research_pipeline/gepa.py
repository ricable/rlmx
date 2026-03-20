"""
gepa.py — GEPA: Generative Evolutionary Prompt Architecture

GEPA is the outer meta-loop. It evolves:
  1. DSPy signature instructions (docstrings + field descriptions)
  2. LLM-Council membership and composition
  3. Fitness signal weights

Architecture:
  - When per_level=True: 4 independent GEPA loops (one per artifact level)
    each with their own population, running the same evolution operators
  - Fitness signals flow upward: claim fitness informs hypothesis card GEPA,
    card fitness informs thread GEPA, thread fitness informs paper GEPA
  - The council and fitness weights are shared across all levels (evolved once)

Evolution operators:
  - Selection: tournament selection from population
  - Mutation:  GEPAMutateSignature (LLM-powered instruction mutation)
  - Crossover: GEPACrossover (LLM-powered instruction crossover)
  - Elitism:   top N survive unchanged each generation
"""

from __future__ import annotations

import json
import logging
import random
from dataclasses import dataclass, field
from typing import Any, Optional

import dspy

from .config import ArtifactLevel, GEPAConfig
from .council import Council
from .fitness import FitnessEngine
from .signatures import (
    GEPACrossover,
    GEPAEvolveCouncil,
    GEPAEvolveFitnessWeights,
    GEPAMutateSignature,
)

logger = logging.getLogger(__name__)


# ─── Population member ────────────────────────────────────────────────────────

@dataclass
class Individual:
    """One member of a GEPA population (one evolved signature instruction)."""
    signature_name: str
    instruction: str
    fitness: float = 0.0
    generation: int = 0
    parent_ids: list[str] = field(default_factory=list)
    mutation_type: str = ""
    id: str = field(default_factory=lambda: f"ind_{random.randint(10000,99999)}")


# ─── GEPALevel ────────────────────────────────────────────────────────────────

class GEPALevel:
    """
    One GEPA loop for a single artifact level.
    Maintains a population of signature instructions.
    """

    def __init__(
        self,
        level: ArtifactLevel,
        config: GEPAConfig,
        mutable_signatures: list[str],
    ) -> None:
        self.level = level
        self.config = config
        self.mutable_signatures = mutable_signatures
        # population: sig_name → list[Individual]
        self.populations: dict[str, list[Individual]] = {}
        self.generation = 0

        self._mutate   = dspy.ChainOfThought(GEPAMutateSignature)
        self._crossover = dspy.ChainOfThought(GEPACrossover)

    def initialize(self, base_instructions: dict[str, str]) -> None:
        """Seed population from the current signature instructions."""
        for sig_name, instruction in base_instructions.items():
            if sig_name not in self.mutable_signatures:
                continue
            self.populations[sig_name] = [
                Individual(
                    signature_name=sig_name,
                    instruction=instruction,
                    fitness=0.5,  # neutral prior
                    generation=0,
                )
            ]
        logger.info(
            f"GEPA[{self.level.value}] initialized with {len(self.populations)} signatures"
        )

    def update_fitness(self, sig_name: str, fitness: float) -> None:
        """Record observed fitness for the currently active individual."""
        if sig_name in self.populations:
            best = self._best(sig_name)
            if best:
                # Exponential moving average
                best.fitness = 0.7 * best.fitness + 0.3 * fitness

    def evolve(
        self,
        sig_name: str,
        recent_artifacts_json: str,
        failure_patterns: str,
    ) -> Optional[Individual]:
        """
        Run one evolution step for a signature.
        Returns the new best individual (to replace active instruction).
        """
        pop = self.populations.get(sig_name, [])
        if not pop:
            return None

        # Elitism: preserve top N
        sorted_pop = sorted(pop, key=lambda x: x.fitness, reverse=True)
        elite = sorted_pop[: self.config.elite_size]
        new_pop = list(elite)

        # Fill rest of population with mutations + crossovers
        while len(new_pop) < self.config.population_size:
            op = random.random()

            if op < self.config.crossover_rate and len(sorted_pop) >= 2:
                # Crossover
                parent_a, parent_b = self._tournament_select(sorted_pop, 2)
                child = self._do_crossover(parent_a, parent_b, sig_name)
                new_pop.append(child)

            elif op < self.config.crossover_rate + self.config.mutation_rate:
                # Mutation
                parent = self._tournament_select(sorted_pop, 1)[0]
                child = self._do_mutation(
                    parent, sig_name, recent_artifacts_json, failure_patterns
                )
                new_pop.append(child)

            else:
                # Clone with small noise
                parent = self._tournament_select(sorted_pop, 1)[0]
                clone = Individual(
                    signature_name=sig_name,
                    instruction=parent.instruction,
                    fitness=parent.fitness * 0.95,
                    generation=self.generation + 1,
                    parent_ids=[parent.id],
                    mutation_type="clone",
                )
                new_pop.append(clone)

        self.populations[sig_name] = new_pop
        self.generation += 1

        best = self._best(sig_name)
        if best:
            logger.info(
                f"GEPA[{self.level.value}] evolved {sig_name}: "
                f"gen={self.generation} best_fitness={best.fitness:.3f}"
            )
        return best

    def best_instruction(self, sig_name: str) -> Optional[str]:
        best = self._best(sig_name)
        return best.instruction if best else None

    def population_summary(self) -> dict[str, Any]:
        return {
            sig: {
                "size": len(pop),
                "best_fitness": max((i.fitness for i in pop), default=0.0),
                "avg_fitness": sum(i.fitness for i in pop) / max(len(pop), 1),
            }
            for sig, pop in self.populations.items()
        }

    # ── Private evolution operators ───────────────────────────────────────────

    def _do_mutation(
        self,
        parent: Individual,
        sig_name: str,
        recent_artifacts_json: str,
        failure_patterns: str,
    ) -> Individual:
        try:
            pred = self._mutate(
                signature_name=sig_name,
                current_instruction=parent.instruction,
                fitness_scores_json=recent_artifacts_json,
                failure_patterns=failure_patterns,
                top_performing_examples=recent_artifacts_json[:500],
            )
            return Individual(
                signature_name=sig_name,
                instruction=pred.mutated_instruction,
                fitness=parent.fitness + float(pred.expected_fitness_delta) * 0.5,
                generation=self.generation + 1,
                parent_ids=[parent.id],
                mutation_type=pred.mutation_type,
            )
        except Exception as exc:
            logger.warning(f"Mutation failed for {sig_name}: {exc}")
            return Individual(
                signature_name=sig_name,
                instruction=parent.instruction,
                fitness=parent.fitness * 0.9,
                generation=self.generation + 1,
                parent_ids=[parent.id],
                mutation_type="failed_clone",
            )

    def _do_crossover(
        self,
        parent_a: Individual,
        parent_b: Individual,
        sig_name: str,
    ) -> Individual:
        try:
            pred = self._crossover(
                parent_a_instruction=parent_a.instruction,
                parent_b_instruction=parent_b.instruction,
                parent_a_fitness=parent_a.fitness,
                parent_b_fitness=parent_b.fitness,
                signature_name=sig_name,
            )
            avg_fitness = (parent_a.fitness + parent_b.fitness) / 2
            return Individual(
                signature_name=sig_name,
                instruction=pred.child_instruction,
                fitness=avg_fitness * 1.05,  # slight optimism for crossover
                generation=self.generation + 1,
                parent_ids=[parent_a.id, parent_b.id],
                mutation_type="crossover",
            )
        except Exception as exc:
            logger.warning(f"Crossover failed for {sig_name}: {exc}")
            return Individual(
                signature_name=sig_name,
                instruction=parent_a.instruction,
                fitness=parent_a.fitness,
                generation=self.generation + 1,
                parent_ids=[parent_a.id],
                mutation_type="crossover_fallback",
            )

    @staticmethod
    def _tournament_select(
        pop: list[Individual], n: int, k: int = 3
    ) -> list[Individual]:
        selected = []
        for _ in range(n):
            contestants = random.sample(pop, min(k, len(pop)))
            winner = max(contestants, key=lambda x: x.fitness)
            selected.append(winner)
        return selected

    def _best(self, sig_name: str) -> Optional[Individual]:
        pop = self.populations.get(sig_name, [])
        return max(pop, key=lambda x: x.fitness) if pop else None


# ─── GEPA (top-level orchestrator) ────────────────────────────────────────────

class GEPA:
    """
    Top-level GEPA orchestrator.

    - Manages per-level GEPALevel instances (when config.per_level=True)
    - Evolves council composition via GEPAEvolveCouncil
    - Evolves fitness weights via GEPAEvolveFitnessWeights
    - Applies evolved instructions to DSPy signatures at runtime via patch_signatures()
    """

    LEVEL_TO_SIGS: dict[ArtifactLevel, list[str]] = {
        ArtifactLevel.CLAIM: [
            "GenerateClaim",
            "RefineClaimWithCouncil",
            "AutoresearchGenerate",
        ],
        ArtifactLevel.HYPOTHESIS_CARD: [
            "GenerateHypothesisCard",
            "ScoreHypothesisCard",
            "AutoresearchExecute",
        ],
        ArtifactLevel.RESEARCH_THREAD: [
            "WeaveResearchThread",
            "ExtendResearchThread",
        ],
        ArtifactLevel.MINI_PAPER: [
            "SynthesizeMiniPaper",
            "CritiqueMiniPaper",
            "AutoresearchSynthesize",
        ],
    }

    def __init__(
        self,
        config: GEPAConfig,
        fitness_engine: FitnessEngine,
        council: Council,
    ) -> None:
        self.config = config
        self.fitness = fitness_engine
        self.council = council
        self.generation = 0

        self._evolve_council  = dspy.ChainOfThought(GEPAEvolveCouncil)
        self._evolve_weights  = dspy.ChainOfThought(GEPAEvolveFitnessWeights)

        # Per-level GEPA loops
        self.levels: dict[ArtifactLevel, GEPALevel] = {}
        if config.per_level:
            for level in ArtifactLevel:
                sigs = self.LEVEL_TO_SIGS.get(level, [])
                self.levels[level] = GEPALevel(level, config, sigs)
        else:
            # Single unified loop operating on all signatures
            all_sigs = config.mutable_signatures
            unified = GEPALevel(ArtifactLevel.CLAIM, config, all_sigs)
            for level in ArtifactLevel:
                self.levels[level] = unified

    def initialize(self, base_instructions: dict[str, str]) -> None:
        """Seed all GEPA levels from the current DSPy signature instructions."""
        initialized = set()
        for level, gepa_level in self.levels.items():
            level_sigs = {
                k: v for k, v in base_instructions.items()
                if k in self.LEVEL_TO_SIGS.get(level, [])
            }
            if level_sigs:
                gepa_level.initialize(level_sigs)
                initialized.update(level_sigs.keys())
        logger.info(f"GEPA initialized: {len(initialized)} signatures across {len(self.levels)} levels")

    def evolve_signatures(
        self,
        level: ArtifactLevel,
        recent_fitness_scores: list[float],
        failure_patterns: str = "",
    ) -> dict[str, str]:
        """
        Run one evolution step for all signatures at a given level.
        Returns dict of sig_name → new_best_instruction.
        """
        if not self.config.evolve_signatures:
            return {}

        gepa_level = self.levels[level]
        scores_json = json.dumps(recent_fitness_scores)
        new_instructions: dict[str, str] = {}

        for sig_name in self.LEVEL_TO_SIGS.get(level, []):
            if sig_name not in self.config.mutable_signatures:
                continue
            best = gepa_level.evolve(
                sig_name=sig_name,
                recent_artifacts_json=scores_json,
                failure_patterns=failure_patterns,
            )
            if best:
                new_instructions[sig_name] = best.instruction

        self.generation += 1
        return new_instructions

    def evolve_council(
        self,
        intent_summary: str,
        fitness_history: list[tuple[int, float]],
        disagreement_history: list[float],
        performance_notes: str = "",
    ) -> None:
        """Evolve council membership. Updates self.council in-place."""
        if not self.config.evolve_council:
            return

        current = [
            {
                "model": m.model,
                "provider": m.provider.value,
                "temperature": m.temperature,
                "diversity_angle": m.diversity_angle.value,
                "reviewer_role": m.reviewer_role,
                "weight": m.weight,
            }
            for m in self.council.config.members
        ]

        try:
            pred = self._evolve_council(
                current_council_json=json.dumps(current, indent=2),
                research_intent_summary=intent_summary,
                fitness_history_json=json.dumps(fitness_history),
                council_disagreement_history=json.dumps(disagreement_history),
                council_performance_notes=performance_notes,
            )
            self.council.update_members(pred.proposed_council_json)
            logger.info(
                f"GEPA evolved council: gen={self.generation} "
                f"removed={pred.removed_members} added={pred.added_members}"
            )
        except Exception as exc:
            logger.error(f"GEPA council evolution failed: {exc}")

    def evolve_fitness_weights(
        self,
        signal_performance: dict[str, float],
        research_stage: str,
        fitness_trend: str,
    ) -> None:
        """Evolve fitness weights. Updates self.fitness in-place."""
        if not self.config.evolve_fitness_weights:
            return

        try:
            pred = self._evolve_weights(
                current_weights_json=json.dumps(self.fitness.current_weights(), indent=2),
                signal_performance_json=json.dumps(signal_performance, indent=2),
                research_stage=research_stage,
                fitness_trend=fitness_trend,
            )
            new_weights = json.loads(pred.proposed_weights_json)
            self.fitness.update_weights(new_weights)
            logger.info(
                f"GEPA evolved fitness weights: stage={research_stage} "
                f"strategy={pred.stage_strategy}"
            )
        except Exception as exc:
            logger.error(f"GEPA fitness weight evolution failed: {exc}")

    def patch_signatures(
        self,
        modules_registry: dict[str, dspy.Module],
        new_instructions: dict[str, str],
    ) -> None:
        """
        Apply evolved instructions to live DSPy modules.

        DSPy signatures store their instruction in the class docstring;
        we patch via the predict.signature attribute on ChainOfThought modules.
        """
        for sig_name, instruction in new_instructions.items():
            module = modules_registry.get(sig_name)
            if module is None:
                logger.debug(f"No module registered for signature: {sig_name}")
                continue
            try:
                if hasattr(module, "predict") and hasattr(module.predict, "signature"):
                    # Patch ChainOfThought
                    module.predict.signature.__doc__ = instruction
                elif hasattr(module, "signature"):
                    module.signature.__doc__ = instruction
                logger.debug(f"Patched signature: {sig_name}")
            except Exception as exc:
                logger.warning(f"Failed to patch {sig_name}: {exc}")

    def research_stage(self, current_gen: int, total_gens: int) -> str:
        frac = current_gen / max(total_gens, 1)
        if frac < 0.33:
            return "early"
        elif frac < 0.67:
            return "mid"
        return "late"

    def fitness_trend(self, history: list[tuple[int, float]], window: int = 3) -> str:
        if len(history) < window + 1:
            return "unknown"
        recent = [h[1] for h in history[-window:]]
        delta = recent[-1] - recent[0]
        if delta > 0.02:
            return "up"
        elif delta < -0.02:
            return "down"
        return "flat"

    def full_report(self) -> dict[str, Any]:
        return {
            "generation": self.generation,
            "fitness_weights": self.fitness.current_weights(),
            "council_members": len(self.council.config.members),
            "level_populations": {
                level.value: gl.population_summary()
                for level, gl in self.levels.items()
            },
        }
