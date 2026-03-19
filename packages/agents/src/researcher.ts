/**
 * Auto-research with mutation strategies (DDD-005 / ADR-006).
 *
 * This is a simplified TypeScript port of the Rust ResearcherAgent
 * and ResearchObjective. It provides the same hypothesis-driven
 * research loop with mutation strategy tracking.
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type HypothesisStatus = 'Proposed' | 'Testing' | 'Confirmed' | 'Rejected';

export type ResearchStatus = 'Active' | 'Stalled' | 'Escalated' | 'Completed';

export interface Hypothesis {
  id: string;
  description: string;
  confidence: number;
  status: HypothesisStatus;
}

export interface Finding {
  hypothesisId: string;
  evidence: string;
  score: number;
  timestamp: Date;
}

export interface ResearchSummary {
  topic: string;
  hypothesesTested: number;
  findings: Finding[];
  bestFinding?: Finding;
  durationSecs: number;
}

export interface TrainingConfig {
  learningRate: number;
  batchSize: number;
  maxSteps: number;
  backend: string;
}

export interface MutationStrategy {
  id: string;
  parentId?: string;
  generation: number;
  featureWeights: Map<string, number>;
  routingThresholds: number[];
  promptTemplates: string[];
  trainingConfig: TrainingConfig;
  fitness?: number;
}

// ---------------------------------------------------------------------------
// ResearchObjective — aggregate root for an evolutionary auto-research run
// ---------------------------------------------------------------------------

export class ResearchObjective {
  readonly id: string;
  readonly goal: string;
  hypotheses: Hypothesis[] = [];
  experiments: string[] = [];
  bestGenome?: MutationStrategy;
  generation = 0;
  status: ResearchStatus = 'Active';

  constructor(goal: string) {
    this.id = crypto.randomUUID();
    this.goal = goal;
  }

  addExperiment(experimentId: string): void {
    this.experiments.push(experimentId);
  }

  /**
   * Update the best genome if the candidate has better (lower) fitness.
   */
  updateBestGenome(candidate: MutationStrategy): void {
    if (candidate.fitness === undefined) {
      return;
    }
    if (
      this.bestGenome === undefined ||
      this.bestGenome.fitness === undefined ||
      candidate.fitness < this.bestGenome.fitness
    ) {
      this.bestGenome = candidate;
    }
  }

  advanceGeneration(): void {
    this.generation++;
  }

  markStalled(): void {
    this.status = 'Stalled';
  }

  markEscalated(): void {
    this.status = 'Escalated';
  }

  markCompleted(): void {
    this.status = 'Completed';
  }
}

// ---------------------------------------------------------------------------
// ResearcherAgent
// ---------------------------------------------------------------------------

/**
 * Research agent -- generates hypotheses, gathers evidence, synthesizes findings.
 */
export class ResearcherAgent {
  readonly id: string;
  readonly parentId: string;
  readonly topic: string;
  hypotheses: Hypothesis[] = [];
  findings: Finding[] = [];
  private readonly startedAt: Date;

  constructor(parentId: string, topic: string) {
    this.id = crypto.randomUUID();
    this.parentId = parentId;
    this.topic = topic;
    this.startedAt = new Date();
  }

  /** Generate three hypotheses based on the research topic. */
  generateHypotheses(): Hypothesis[] {
    this.hypotheses = [
      {
        id: crypto.randomUUID(),
        description: `Primary approach to ${this.topic}`,
        confidence: 0.7,
        status: 'Proposed',
      },
      {
        id: crypto.randomUUID(),
        description: `Alternative approach to ${this.topic}`,
        confidence: 0.5,
        status: 'Proposed',
      },
      {
        id: crypto.randomUUID(),
        description: `Null hypothesis for ${this.topic}`,
        confidence: 0.3,
        status: 'Proposed',
      },
    ];
    return [...this.hypotheses];
  }

  /**
   * Run the research process: test each hypothesis and gather findings.
   */
  async research(): Promise<Finding[]> {
    if (this.hypotheses.length === 0) {
      this.generateHypotheses();
    }

    const findings: Finding[] = [];
    for (const hypothesis of this.hypotheses) {
      hypothesis.status = 'Testing';
      const score = hypothesis.confidence * 0.9;

      const finding: Finding = {
        hypothesisId: hypothesis.id,
        evidence: `Evidence for: ${hypothesis.description}`,
        score,
        timestamp: new Date(),
      };

      hypothesis.status = score > 0.5 ? 'Confirmed' : 'Rejected';
      findings.push(finding);
    }

    this.findings = findings;
    return [...findings];
  }

  /** Synthesize all findings into a research summary. */
  synthesize(): ResearchSummary {
    const bestFinding = this.findings.reduce<Finding | undefined>(
      (best, f) => (!best || f.score > best.score ? f : best),
      undefined,
    );

    const durationSecs = (Date.now() - this.startedAt.getTime()) / 1000;

    return {
      topic: this.topic,
      hypothesesTested: this.hypotheses.length,
      findings: [...this.findings],
      bestFinding,
      durationSecs,
    };
  }
}
