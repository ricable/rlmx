// @aix/evolve — Dynamic function evolution for the @aix ecosystem

export type {
  FunctionStatus,
  AutoScoreMode,
  EvolvedFunction,
  ScoreResult,
  FeedbackDecision,
} from './types.js';

export { canTransition, EvolveError } from './types.js';

export { LifecycleManager } from './lifecycle.js';
export { ScoreEngine } from './scorer.js';
export { FeedbackEngine } from './feedback.js';
