"""
signatures.py — All DSPy signatures for the autonomous research pipeline.

Organized by function:
  A. Artifact generation   (Claim → HypothesisCard → ResearchThread → MiniPaper)
  B. LLM-Council           (PeerReview, Diversify, Consensus)
  C. GEPA meta-signatures  (MutateSignature, Crossover, EvolveCouncil, EvolveFitnessWeights)
  D. Autoresearch          (Generate, Execute, Synthesize)

GEPA evolves the docstring (instruction) of any signature in section A, B, and D.
All OutputField descriptions are part of the instruction GEPA can mutate.
"""

from __future__ import annotations
import dspy


# ═══════════════════════════════════════════════════════════════════════════════
# A. ARTIFACT GENERATION SIGNATURES
# ═══════════════════════════════════════════════════════════════════════════════

# ─── A1. Claim ────────────────────────────────────────────────────────────────

class GenerateClaim(dspy.Signature):
    """
    Generate a single novel, falsifiable research claim.

    The claim must:
    - Address a genuine open question not already answered in the literature
    - Be falsifiable: there must exist a possible experiment or observation that could refute it
    - Be specific enough to motivate a concrete hypothesis card
    - Identify if it bridges multiple research domains (cross-domain bonus)

    Avoid incremental extensions of well-covered territory.
    Prefer claims that open new territory over claims that refine existing knowledge.
    """
    research_question: str = dspy.InputField(
        desc="Core research question or intent driving this pipeline run"
    )
    domain: str = dspy.InputField(
        desc="Primary research domain and any stated constraints"
    )
    existing_claims_json: str = dspy.InputField(
        desc="JSON array of already-generated claim texts — do not duplicate these"
    )
    literature_context: str = dspy.InputField(
        desc="Condensed literature landscape from autoresearch generation phase"
    )
    focus_angle: str = dspy.InputField(
        desc="Research angle: broad | narrow | contrarian | comparative"
    )

    claim: str = dspy.OutputField(
        desc="A single falsifiable research claim (1–3 sentences)"
    )
    rationale: str = dspy.OutputField(
        desc="Why this claim is worth investigating (2–4 sentences)"
    )
    confidence: float = dspy.OutputField(
        desc="Confidence 0.0–1.0 that this is a genuine open research question"
    )
    novelty_signal: str = dspy.OutputField(
        desc="What makes this claim novel relative to the stated literature context"
    )
    bridged_domains: str = dspy.OutputField(
        desc="Comma-separated list of other domains this claim connects to, or 'none'"
    )


class RefineClaimWithCouncil(dspy.Signature):
    """
    Refine a research claim based on aggregated LLM-Council feedback.

    Preserve the core insight of the original claim while addressing
    the council's concerns about falsifiability, novelty, or clarity.
    Do not dilute the claim into vague generality to satisfy concerns —
    instead sharpen it into a more precise, testable statement.
    """
    original_claim: str = dspy.InputField(
        desc="The original claim text"
    )
    council_critiques: str = dspy.InputField(
        desc="Aggregated critique from all council members"
    )
    council_suggestions: str = dspy.InputField(
        desc="Aggregated improvement suggestions from council"
    )
    frontier_signal: bool = dspy.InputField(
        desc="True if council disagreement was high (frontier territory — preserve boldness)"
    )

    refined_claim: str = dspy.OutputField(
        desc="Improved version of the claim"
    )
    changes_made: str = dspy.OutputField(
        desc="Bullet list of what was changed and why"
    )
    preserved_insight: str = dspy.OutputField(
        desc="The core insight preserved from the original claim"
    )


# ─── A2. HypothesisCard ───────────────────────────────────────────────────────

class GenerateHypothesisCard(dspy.Signature):
    """
    Expand a research claim into a full hypothesis card with experiment design.

    A strong hypothesis card:
    - States a formal, testable hypothesis derived from the claim
    - Proposes a concrete experiment that could validate or refute it
    - Specifies precise falsification criteria (not vague disconfirmation)
    - Grounds itself in related literature from autoresearch execution
    - Sketches a methodology even if high-level

    Prioritize depth and specificity over breadth.
    The hypothesis should be more precise than the claim it expands.
    """
    claim_text: str = dspy.InputField(
        desc="The claim to expand into a hypothesis card"
    )
    claim_rationale: str = dspy.InputField(
        desc="Rationale from the claim generation step"
    )
    domain: str = dspy.InputField(
        desc="Primary research domain"
    )
    autoresearch_results: str = dspy.InputField(
        desc="Papers, abstracts, and findings retrieved during autoresearch execution phase"
    )

    hypothesis: str = dspy.OutputField(
        desc="Formal hypothesis statement (If X, then Y, because Z)"
    )
    predicted_experiment: str = dspy.OutputField(
        desc="Concrete experiment or study design that would test this hypothesis"
    )
    falsification_criteria: str = dspy.OutputField(
        desc="Precise criteria: what results would falsify this hypothesis"
    )
    methodology_sketch: str = dspy.OutputField(
        desc="High-level methodology: datasets, models, metrics, baselines"
    )
    related_works: str = dspy.OutputField(
        desc="JSON array of related work strings (title + 1-sentence relevance note)"
    )
    confidence_score: float = dspy.OutputField(
        desc="Confidence 0.0–1.0 that this hypothesis is true and worth testing"
    )


class ScoreHypothesisCard(dspy.Signature):
    """
    Score a hypothesis card for research quality and identify gaps.

    Evaluate across: hypothesis precision, experiment feasibility,
    falsification clarity, literature grounding, and methodology soundness.
    """
    hypothesis: str = dspy.InputField()
    falsification_criteria: str = dspy.InputField()
    predicted_experiment: str = dspy.InputField()
    related_works: str = dspy.InputField()

    quality_score: float = dspy.OutputField(
        desc="Overall quality 0.0–1.0"
    )
    gaps: str = dspy.OutputField(
        desc="JSON array of specific gaps or weaknesses that need addressing"
    )
    strengths: str = dspy.OutputField(
        desc="JSON array of specific strengths worth preserving"
    )


# ─── A3. ResearchThread ───────────────────────────────────────────────────────

class WeaveResearchThread(dspy.Signature):
    """
    Connect a set of hypothesis cards into a directed research thread (DAG).

    A research thread is not a list — it is a narrative of scientific tension.
    Identify which cards support, refute, or extend each other.
    Surface the key tensions between cards: these are the most important outputs,
    as they point toward what remains genuinely unresolved.
    The synthesis_direction is the thread's answer to: "where does this go next?"
    """
    hypothesis_cards_json: str = dspy.InputField(
        desc="JSON array of hypothesis card objects (id, hypothesis, falsification_criteria)"
    )
    thread_topic: str = dspy.InputField(
        desc="Unifying topic or research question for this thread"
    )

    thread_structure: str = dspy.OutputField(
        desc="DAG description: JSON object mapping card_id → list of parent card_ids"
    )
    edge_types: str = dspy.OutputField(
        desc="JSON object mapping 'src_id→dst_id' → 'supports'|'refutes'|'extends'"
    )
    key_tensions: str = dspy.OutputField(
        desc="JSON array of 3–7 key tensions or contradictions between cards"
    )
    synthesis_direction: str = dspy.OutputField(
        desc="Where this thread is pointing: what remains unresolved and why it matters"
    )
    narrative_arc: str = dspy.OutputField(
        desc="The research story this thread tells (3–5 sentences)"
    )


class ExtendResearchThread(dspy.Signature):
    """
    Propose new hypothesis cards to extend an existing research thread.

    Extensions should resolve a key tension OR open a new one.
    Do not propose redundant cards that restate what is already in the thread.
    The best extensions are at the leaf nodes of the current DAG.
    """
    existing_thread_summary: str = dspy.InputField(
        desc="Summary of the existing thread structure and narrative"
    )
    key_tensions: str = dspy.InputField(
        desc="JSON array of unresolved tensions in the current thread"
    )
    synthesis_direction: str = dspy.InputField(
        desc="Current stated direction of the thread"
    )
    leaf_node_summaries: str = dspy.InputField(
        desc="Summaries of the current leaf (frontier) nodes"
    )

    proposed_extensions: str = dspy.OutputField(
        desc="JSON array of new hypothesis card sketches (claim, hypothesis, direction)"
    )
    extension_rationale: str = dspy.OutputField(
        desc="Why these specific extensions are the most valuable next steps"
    )
    tension_resolution: str = dspy.OutputField(
        desc="Which existing tensions each extension addresses"
    )


# ─── A4. MiniPaper ────────────────────────────────────────────────────────────

class SynthesizeMiniPaper(dspy.Signature):
    """
    Synthesize a research thread into a structured mini-paper.

    The abstract must stand alone as a publishable research contribution.
    Findings should be the highest-confidence, highest-novelty outputs of the thread.
    Open questions are the thread's gift to future researchers: be specific.
    The contribution statement must be bold and falsifiable, not vague.
    Limitations must be honest — they increase credibility, not decrease it.
    """
    thread_summary: str = dspy.InputField(
        desc="Full summary of the research thread (topic, cards, tensions, DAG)"
    )
    all_cards_json: str = dspy.InputField(
        desc="JSON array of all hypothesis cards with their findings and evidence"
    )
    autoresearch_synthesis: str = dspy.InputField(
        desc="Synthesis narrative from autoresearch synthesis phase"
    )
    research_question: str = dspy.InputField(
        desc="Original research question / intent"
    )

    abstract: str = dspy.OutputField(
        desc="200-word abstract suitable for arXiv submission"
    )
    methodology: str = dspy.OutputField(
        desc="Methodology section: how the research was conducted"
    )
    findings: str = dspy.OutputField(
        desc="JSON array of 3–8 key findings, each with confidence score"
    )
    open_questions: str = dspy.OutputField(
        desc="JSON array of 3–6 specific open questions for future work"
    )
    contribution_statement: str = dspy.OutputField(
        desc="What this work uniquely contributes to the field (2–3 sentences)"
    )
    limitations: str = dspy.OutputField(
        desc="Honest limitations of this work and how they might be addressed"
    )


class CritiqueMiniPaper(dspy.Signature):
    """
    Provide structured peer review of a mini-paper from a specific reviewer perspective.

    Be rigorous but constructive. Major concerns must be substantive, not stylistic.
    The accept recommendation should reflect whether the contribution is real and sound,
    not whether it is impressive or novel-sounding.
    """
    abstract: str = dspy.InputField()
    methodology: str = dspy.InputField()
    findings: str = dspy.InputField()
    open_questions: str = dspy.InputField()
    contribution_statement: str = dspy.InputField()
    reviewer_role: str = dspy.InputField(
        desc="Reviewer's identity and expertise angle"
    )

    score: float = dspy.OutputField(
        desc="Overall quality score 0.0–1.0"
    )
    major_concerns: str = dspy.OutputField(
        desc="JSON array of major concerns that must be addressed before acceptance"
    )
    minor_concerns: str = dspy.OutputField(
        desc="JSON array of minor suggestions for improvement"
    )
    accept_recommendation: str = dspy.OutputField(
        desc="One of: 'accept' | 'major_revision' | 'reject' — with one-sentence justification"
    )
    highlight: str = dspy.OutputField(
        desc="The single most valuable contribution of this paper"
    )


# ═══════════════════════════════════════════════════════════════════════════════
# B. LLM-COUNCIL SIGNATURES
# ═══════════════════════════════════════════════════════════════════════════════

class CouncilPeerReview(dspy.Signature):
    """
    Peer review an artifact for quality, validity, and research value.

    As a peer reviewer, your job is critical scrutiny — not encouragement.
    Flag logical errors, unsupported claims, circular reasoning, and
    unfalsifiable statements. Do not penalize ambition; penalize sloppiness.
    High disagreement with other reviewers is valuable: maintain your position
    if you have principled reasons for it.
    """
    artifact_type: str = dspy.InputField(
        desc="claim | hypothesis_card | research_thread | mini_paper"
    )
    artifact_text: str = dspy.InputField(
        desc="Full serialized text of the artifact under review"
    )
    reviewer_perspective: str = dspy.InputField(
        desc="Reviewer's model identity, temperature setting, and area of expertise"
    )

    score: float = dspy.OutputField(
        desc="Quality score 0.0–1.0"
    )
    critique: str = dspy.OutputField(
        desc="Substantive critique (3–6 sentences, specific not generic)"
    )
    suggested_improvements: str = dspy.OutputField(
        desc="JSON array of actionable, prioritized improvement suggestions"
    )
    flagged_issues: str = dspy.OutputField(
        desc="JSON array of logical errors, unsupported claims, or methodological concerns"
    )
    would_promote: bool = dspy.OutputField(
        desc="True if this artifact should be promoted to the next pipeline stage"
    )


class CouncilDiversify(dspy.Signature):
    """
    Generate a genuinely diverse alternative perspective on a research direction.

    Do not paraphrase the original — produce a meaningfully different angle.
    The diversity angle determines your lens: contrarian challenges the premise,
    adjacent_domain imports a framework from another field, scale_shift changes
    the level of analysis, temporal_shift changes the time horizon.

    The most valuable diversification outputs are ones that would NOT have been
    generated by the original pipeline without this council member's perspective.
    """
    artifact_type: str = dspy.InputField(
        desc="claim | hypothesis_card | research_thread | mini_paper"
    )
    artifact_text: str = dspy.InputField(
        desc="Full text of the artifact to diversify from"
    )
    diversity_angle: str = dspy.InputField(
        desc="Angle: contrarian | adjacent_domain | scale_shift | temporal_shift | reductionist | systems_thinking"
    )
    member_identity: str = dspy.InputField(
        desc="Council member identity (model, temperature, reviewer role)"
    )

    alternative_perspective: str = dspy.OutputField(
        desc="A genuinely different take on this research direction (4–8 sentences)"
    )
    new_directions: str = dspy.OutputField(
        desc="JSON array of 2–4 new research directions this perspective opens"
    )
    bridging_insight: str = dspy.OutputField(
        desc="The key insight connecting this perspective to the original"
    )
    challenge_to_original: str = dspy.OutputField(
        desc="The strongest challenge this perspective poses to the original direction"
    )


class CouncilConsensus(dspy.Signature):
    """
    Aggregate all council member outputs into a consensus decision.

    Do not simply average scores — identify patterns, outliers, and the
    most substantive points of agreement and disagreement.
    High disagreement level is a positive signal for frontier territory.
    The consensus_action must be actionable: promote means move to next level,
    revise means return with specific instructions, reject means discard,
    explore_further means spawn additional investigation threads.
    """
    artifact_type: str = dspy.InputField()
    council_outputs_json: str = dspy.InputField(
        desc="JSON array of all council member outputs (scores, critiques, suggestions)"
    )
    council_mode: str = dspy.InputField(
        desc="peer_review | diversification"
    )
    artifact_text: str = dspy.InputField(
        desc="The artifact being evaluated (for context)"
    )

    consensus_score: float = dspy.OutputField(
        desc="Aggregated weighted consensus score 0.0–1.0"
    )
    disagreement_level: float = dspy.OutputField(
        desc="Score variance across council members 0.0–1.0"
    )
    consensus_action: str = dspy.OutputField(
        desc="promote | revise | reject | explore_further"
    )
    aggregated_feedback: str = dspy.OutputField(
        desc="Synthesized actionable feedback representing the council's collective view"
    )
    frontier_signal: bool = dspy.OutputField(
        desc="True if high disagreement indicates this is genuinely frontier territory"
    )
    revision_instructions: str = dspy.OutputField(
        desc="Specific instructions for revision if consensus_action is 'revise', else empty"
    )


# ═══════════════════════════════════════════════════════════════════════════════
# C. GEPA META-SIGNATURES
# ═══════════════════════════════════════════════════════════════════════════════

class GEPAMutateSignature(dspy.Signature):
    """
    Propose a targeted mutation to a DSPy signature's instruction to improve research quality.

    A mutation should address a specific failure pattern observed in recent outputs.
    Good mutations: tighten scope, add constraint, shift emphasis, add example.
    Bad mutations: make instructions vague or generic to avoid failures.
    The mutation must preserve the signature's core function.
    """
    signature_name: str = dspy.InputField(
        desc="Name of the DSPy signature to mutate (e.g., 'GenerateClaim')"
    )
    current_instruction: str = dspy.InputField(
        desc="Current docstring / instruction of the signature"
    )
    fitness_scores_json: str = dspy.InputField(
        desc="JSON array of recent fitness scores from artifacts produced by this signature"
    )
    failure_patterns: str = dspy.InputField(
        desc="Common failure patterns in recent outputs (e.g., 'claims are too vague')"
    )
    top_performing_examples: str = dspy.InputField(
        desc="JSON array of top-performing artifact examples for reference"
    )

    mutated_instruction: str = dspy.OutputField(
        desc="Improved signature instruction (full docstring replacement)"
    )
    mutation_rationale: str = dspy.OutputField(
        desc="Specific reason why this mutation should improve performance"
    )
    mutation_type: str = dspy.OutputField(
        desc="expand | constrain | reframe | specialize | generalize"
    )
    expected_fitness_delta: float = dspy.OutputField(
        desc="Estimated fitness improvement 0.0–0.5 from this mutation"
    )


class GEPACrossover(dspy.Signature):
    """
    Produce a crossover offspring from two high-fitness signature instructions.

    A good crossover takes the precision of one parent and the breadth of another,
    or the constraint style of one and the example quality of another.
    The child should not simply concatenate both — it should synthesize them.
    Weigh contributions by relative fitness of each parent.
    """
    parent_a_instruction: str = dspy.InputField(
        desc="Instruction from parent A"
    )
    parent_b_instruction: str = dspy.InputField(
        desc="Instruction from parent B"
    )
    parent_a_fitness: float = dspy.InputField(
        desc="Fitness score of parent A (0.0–1.0)"
    )
    parent_b_fitness: float = dspy.InputField(
        desc="Fitness score of parent B (0.0–1.0)"
    )
    signature_name: str = dspy.InputField(
        desc="Which signature these instructions belong to"
    )

    child_instruction: str = dspy.OutputField(
        desc="Crossover instruction synthesizing the best of both parents"
    )
    crossover_rationale: str = dspy.OutputField(
        desc="What was taken from each parent and why"
    )
    dominant_parent: str = dspy.OutputField(
        desc="'A' or 'B' — which parent contributed more and why"
    )


class GEPAEvolveCouncil(dspy.Signature):
    """
    Propose changes to LLM-Council membership and configuration.

    GEPA can: add members, remove underperforming members, change model assignments,
    adjust temperatures, reassign diversity angles, and shift reviewer roles.
    Optimize for: maximum insight diversity + minimum redundancy.
    A council that always agrees is useless. A council that never converges is noise.
    """
    current_council_json: str = dspy.InputField(
        desc="JSON array of current council member configs"
    )
    research_intent_summary: str = dspy.InputField(
        desc="Summary of the current research intent and domain"
    )
    fitness_history_json: str = dspy.InputField(
        desc="JSON array of (generation, composite_fitness) tuples"
    )
    council_disagreement_history: str = dspy.InputField(
        desc="JSON array of disagreement scores across recent artifacts"
    )
    council_performance_notes: str = dspy.InputField(
        desc="Qualitative notes on which council members are adding vs. not adding value"
    )

    proposed_council_json: str = dspy.OutputField(
        desc="JSON array of new/modified council member configs"
    )
    evolution_rationale: str = dspy.OutputField(
        desc="Why these specific council changes should improve pipeline quality"
    )
    removed_members: str = dspy.OutputField(
        desc="JSON array of removed member models and reasons"
    )
    added_members: str = dspy.OutputField(
        desc="JSON array of new member configs and their intended contribution"
    )


class GEPAEvolveFitnessWeights(dspy.Signature):
    """
    Propose a new fitness weight distribution to better capture research quality.

    Early in a pipeline run, novelty and cross_domain_surprise should dominate
    (explore widely). Late in a run, falsifiability and compression_ratio should
    rise (converge on quality). citation_density matters more in execution phase.
    council_disagreement should be rewarded throughout as a frontier signal.
    Weights must sum to 1.0 after normalization.
    """
    current_weights_json: str = dspy.InputField(
        desc="JSON object of current fitness signal weights"
    )
    signal_performance_json: str = dspy.InputField(
        desc="JSON object mapping signal_name → correlation with final paper quality"
    )
    research_stage: str = dspy.InputField(
        desc="Current stage: early (gen 1–2) | mid (gen 3–4) | late (gen 5+)"
    )
    fitness_trend: str = dspy.InputField(
        desc="Is composite fitness trending up, flat, or down?"
    )

    proposed_weights_json: str = dspy.OutputField(
        desc="JSON object of new fitness signal weights (will be normalized to sum to 1.0)"
    )
    evolution_rationale: str = dspy.OutputField(
        desc="Why these weight changes improve fitness estimation at this stage"
    )
    stage_strategy: str = dspy.OutputField(
        desc="The broader search strategy these weights encode: explore | exploit | converge"
    )


# ═══════════════════════════════════════════════════════════════════════════════
# D. AUTORESEARCH SIGNATURES (Karpathy-style)
# ═══════════════════════════════════════════════════════════════════════════════

class AutoresearchGenerate(dspy.Signature):
    """
    GENERATION PHASE: Propose research directions and seed search queries.

    Karpathy-style autoresearch: systematically map the known unknowns.
    Identify the blank spaces — areas where the literature thins out or where
    existing work reaches a natural frontier. Prioritize directions that are:
    (1) underexplored, (2) feasible to investigate with LLM-based methods,
    (3) likely to produce falsifiable hypotheses.

    The blank_spaces output is the most important: these are the voids in
    knowledge that this pipeline is designed to illuminate.
    """
    research_intent: str = dspy.InputField(
        desc="Full research intent summary (question + domain + constraints)"
    )
    literature_landscape: str = dspy.InputField(
        desc="High-level map of what is known in this domain"
    )
    existing_claims_json: str = dspy.InputField(
        desc="JSON array of already-generated claims to avoid duplicating"
    )
    focus_angle: str = dspy.InputField(
        desc="Research angle: broad | narrow | contrarian | comparative"
    )

    proposed_directions: str = dspy.OutputField(
        desc="JSON array of 4–8 research directions to pursue, each with title + description"
    )
    priority_ranking: str = dspy.OutputField(
        desc="JSON array of direction titles ranked by expected value, with justification"
    )
    search_queries: str = dspy.OutputField(
        desc="JSON array of 6–12 search/retrieval queries to execute across directions"
    )
    blank_spaces: str = dspy.OutputField(
        desc="JSON array of the most important unknown territories: gaps nobody has studied"
    )
    literature_map: str = dspy.OutputField(
        desc="Concise map of what IS known: key papers, findings, and their relationships"
    )


class AutoresearchExecute(dspy.Signature):
    """
    EXECUTION PHASE: Analyze retrieved results and extract structured evidence.

    Given raw search/retrieval results, extract what actually supports or
    contradicts the research direction. Separate signal from noise.
    Be conservative: if evidence is weak or tangential, say so.
    The key_papers output should include only papers that materially affect
    the hypothesis confidence — not every tangentially related paper.
    """
    research_direction: str = dspy.InputField(
        desc="The specific direction being investigated"
    )
    search_queries_used: str = dspy.InputField(
        desc="The queries that were executed"
    )
    retrieved_results: str = dspy.InputField(
        desc="Raw retrieved content: paper abstracts, snippets, or search results"
    )
    prior_confidence: float = dspy.InputField(
        desc="Confidence in the direction before execution (from generation phase)"
    )

    analyzed_findings: str = dspy.OutputField(
        desc="Structured analysis: what the retrieved results actually show"
    )
    supporting_evidence: str = dspy.OutputField(
        desc="JSON array of specific evidence items supporting this direction"
    )
    contradicting_evidence: str = dspy.OutputField(
        desc="JSON array of specific evidence items contradicting this direction"
    )
    key_papers_json: str = dspy.OutputField(
        desc="JSON array of key papers: {title, relevance_score, key_finding, supports: bool}"
    )
    confidence_update: float = dspy.OutputField(
        desc="Updated confidence 0.0–1.0 after seeing retrieved evidence"
    )
    execution_summary: str = dspy.OutputField(
        desc="One-paragraph summary of what was found and what it means"
    )


class AutoresearchSynthesize(dspy.Signature):
    """
    SYNTHESIS PHASE: Integrate all executed research into coherent meta-findings.

    This is the Karpathy autoresearch synthesis step: look across all directions
    and find the patterns that individual directions cannot see.
    Meta-findings are higher-order insights: things that are true across multiple
    directions, unexpected connections, and structural features of the research space.
    The convergence_signal tells the pipeline whether to keep exploring or consolidate.
    """
    all_directions_json: str = dspy.InputField(
        desc="JSON array of all investigated directions with their execution findings"
    )
    research_intent: str = dspy.InputField(
        desc="Original research intent"
    )
    iteration_number: int = dspy.InputField(
        desc="Current pipeline iteration number"
    )
    prior_synthesis: str = dspy.InputField(
        desc="Synthesis narrative from previous iteration (empty on first iteration)"
    )

    synthesis_narrative: str = dspy.OutputField(
        desc="Coherent narrative connecting all findings across directions (4–8 sentences)"
    )
    meta_findings: str = dspy.OutputField(
        desc="JSON array of higher-order patterns visible across multiple directions"
    )
    recommended_next_steps: str = dspy.OutputField(
        desc="JSON array of specific next research steps ranked by expected value"
    )
    confidence_landscape: str = dspy.OutputField(
        desc="JSON object mapping direction_title → updated_confidence"
    )
    convergence_signal: str = dspy.OutputField(
        desc="expanding | converging | stagnating — with one-sentence justification"
    )
    strongest_finding: str = dspy.OutputField(
        desc="The single most robust and surprising finding from this synthesis pass"
    )
