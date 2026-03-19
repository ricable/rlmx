/**
 * @aix/plugin — Plugin registry for the @aix ecosystem.
 *
 * Ports the Rust rlmx-plugin crate to TypeScript, providing:
 * - PluginRegistry for managing domain plugin lifecycle
 * - DomainPlugin interface as the core extension point
 * - Safety constraint enforcement engine
 * - Dynamic import-based plugin loading
 * - Type definitions for strategies, actions, safety, and more
 *
 * @packageDocumentation
 */

// Types
export type {
  Strategy,
  StrategyPreference,
  ActionParam,
  ActionDefinition,
  SafetyConstraint,
  SafetyResult,
  EmbeddingConfig,
  TrmModelConfig,
  NodeType,
  EdgeType,
  GraphSchema,
  SegmentTypeDefinition,
  ContextSegment,
  EvaluationResult,
  DomainEvaluator,
  IngestAdapter,
  PluginManifest,
  PluginInfo,
  PluginEvent,
  PluginEventListener,
  DomainPlugin,
} from './types.js';

// Enums and classes (must be value exports)
export {
  ParamType,
  DistanceMetric,
  Tier,
  PluginStatus,
  ActionDefinitionBuilder,
  actionDef,
} from './types.js';

// Registry
export { PluginRegistry } from './registry.js';

// Loader
export { loadPlugin, loadPlugins } from './loader.js';
export type { PluginModule } from './loader.js';

// Errors
export {
  PluginError,
  PluginNotFoundError,
  PluginAlreadyRegisteredError,
  IngestError,
  ConfigError,
  ActionError,
  SafetyViolationError,
  UnsupportedFormatError,
  ManifestValidationError,
  PluginLoadError,
} from './errors.js';
