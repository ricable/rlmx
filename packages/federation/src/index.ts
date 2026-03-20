// @aix/federation -- Federated learning cycle management for the @aix ecosystem
// Re-exports everything from submodules.

export {
  FederationError,
  type FederationErrorCode,
  invalidCycleStatus,
  aggregationThresholdNotMet,
  contributionRejected,
  noDomainContributions,
  packageNotFound,
  cycleNotFound,
  anonymizationFailed,
  distributionFailed,
  bootstrapFailed,
  privacyViolation,
} from './errors.js';

export type {
  FederationEvent,
  FederationEventType,
  FederationEventOf,
  CycleStartedEvent,
  ContributionReceivedEvent,
  AggregationCompletedEvent,
  PackageDistributedEvent,
  CycleCompletedEvent,
} from './events.js';

export { FEDERATION_EVENT_TYPES } from './events.js';

export type {
  Modality,
  EmotionBucket,
  AnonymizedPattern,
  LoraDelta,
  Contribution,
} from './contribution.js';

export {
  createContribution,
  withRegion,
  withSignature,
  validateContribution,
  generatePseudonym,
} from './contribution.js';

export type {
  AnonymizationConfig,
  VoiceEnrichedPattern,
} from './anonymizer.js';

export {
  FederationAnonymizer,
  DEFAULT_ANONYMIZATION_CONFIG,
} from './anonymizer.js';

export type { AggregatorConfig, AggregatedModel } from './aggregator.js';

export {
  FederatedAggregator,
  DEFAULT_AGGREGATOR_CONFIG,
} from './aggregator.js';

export type { FederationPackage } from './distribution.js';

export {
  createPackageFromAggregated,
  verifyPackageIntegrity,
  PackageDistributor,
} from './distribution.js';

export type { BootstrapResult, SonaAdapter } from './bootstrap.js';

export { bootstrapFromLatest, bootstrapFromPackage } from './bootstrap.js';

export type { CycleStatus } from './cycle.js';

export { FederationCycle } from './cycle.js';
