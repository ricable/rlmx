/**
 * Plugin type definitions — ports rlmx-plugin types.rs, traits.rs,
 * adapter.rs, evaluator.rs, context.rs, and safety.rs to TypeScript.
 */

// ---------------------------------------------------------------------------
// Strategy types
// ---------------------------------------------------------------------------

/** Strategy for handling a task — maps to Rust Strategy enum. */
export type Strategy =
  | { kind: 'rlm' }
  | { kind: 'trm'; modelName: string }
  | { kind: 'auto' }
  | { kind: 'hybrid'; triage: string; threshold: number };

/** A preference mapping task patterns to strategies. */
export interface StrategyPreference {
  /** Regex or keyword pattern to match incoming tasks. */
  taskPattern: string;
  /** The strategy to use when the pattern matches. */
  strategy: Strategy;
  /** Optional TRM model name (when strategy is trm or hybrid). */
  trmModel?: string;
  /** Rationale for why this strategy is preferred. */
  rationale: string;
}

// ---------------------------------------------------------------------------
// Action types
// ---------------------------------------------------------------------------

/** Parameter type for action definitions. */
export enum ParamType {
  String = 'String',
  Float = 'Float',
  Int = 'Int',
  Bool = 'Bool',
  StringArray = 'StringArray',
}

/** A parameter in an action definition. */
export interface ActionParam {
  /** Parameter name. */
  name: string;
  /** Parameter type. */
  paramType: ParamType;
  /** Whether this parameter is required. */
  required: boolean;
  /** Description of the parameter. */
  description: string;
}

/** Definition of an action that a plugin can execute. */
export interface ActionDefinition {
  /** Unique name of the action. */
  name: string;
  /** Human-readable description. */
  description: string;
  /** Parameters accepted by this action. */
  params: ActionParam[];
}

// ---------------------------------------------------------------------------
// Safety types
// ---------------------------------------------------------------------------

/** Safety constraint applied to plugin actions. */
export type SafetyConstraint =
  | {
      kind: 'parameterBound';
      param: string;
      min: number;
      max: number;
    }
  | {
      kind: 'kpiGuard';
      metric: string;
      maxDegradation: number;
    }
  | {
      kind: 'humanEscalation';
      condition: string;
      actions: string[];
    }
  | {
      kind: 'rateLimit';
      action: string;
      maxCount: number;
      windowSecs: number;
    };

/** Result of a safety check. */
export type SafetyResult =
  | { status: 'allowed' }
  | { status: 'rejected'; reason: string }
  | { status: 'requiresApproval'; reason: string };

// ---------------------------------------------------------------------------
// Embedding & distance types
// ---------------------------------------------------------------------------

/** Configuration for embedding generation. */
export interface EmbeddingConfig {
  /** Name of the embedding model to use. */
  modelName: string;
  /** Dimensionality of the embedding vectors. */
  dimensions: number;
}

/** Distance metric for vector similarity search. */
export enum DistanceMetric {
  Cosine = 'Cosine',
  L2 = 'L2',
  InnerProduct = 'InnerProduct',
  Manhattan = 'Manhattan',
  Wasserstein = 'Wasserstein',
  Poincare = 'Poincare',
  ProductManifold = 'ProductManifold',
}

// ---------------------------------------------------------------------------
// TRM model types
// ---------------------------------------------------------------------------

/** Configuration for a Traditional ML (TRM) model. */
export interface TrmModelConfig {
  /** Model name identifier. */
  name: string;
  /** Path to the model file or directory. */
  path: string;
  /** Total number of trainable parameters. */
  params: number;
  /** Number of layers in the model. */
  layers: number;
  /** Input dimensionality. */
  inputDim: number;
  /** Number of output classes. */
  outputClasses: number;
  /** Maximum reasoning cycles before halting. */
  maxCycles: number;
  /** Confidence threshold to halt early. */
  haltThreshold: number;
}

// ---------------------------------------------------------------------------
// Graph schema types
// ---------------------------------------------------------------------------

/** A node type in the graph schema. */
export interface NodeType {
  /** Name of the node type. */
  name: string;
  /** Properties associated with this node type. */
  properties: Record<string, string>;
}

/** An edge type in the graph schema. */
export interface EdgeType {
  /** Name of the edge type. */
  name: string;
  /** Source node type name. */
  sourceType: string;
  /** Target node type name. */
  targetType: string;
  /** Properties associated with this edge type. */
  properties: Record<string, string>;
}

/** Graph schema for knowledge graph integration. */
export interface GraphSchema {
  /** Node types in the graph. */
  nodeTypes: NodeType[];
  /** Edge types in the graph. */
  edgeTypes: EdgeType[];
}

// ---------------------------------------------------------------------------
// RVF segment types
// ---------------------------------------------------------------------------

/** Definition of a segment type for the RVF (Rich Vector Format). */
export interface SegmentTypeDefinition {
  /** Name of the segment type. */
  name: string;
  /** JSON schema describing the segment's metadata structure. */
  schema: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Context types
// ---------------------------------------------------------------------------

/** Storage tier classification for context segments. */
export enum Tier {
  /** Frequently accessed, kept in fast storage. */
  Hot = 'Hot',
  /** Occasionally accessed, standard storage. */
  Warm = 'Warm',
  /** Rarely accessed, archived storage. */
  Cold = 'Cold',
}

/** A segment of contextual data produced by an ingest adapter. */
export interface ContextSegment {
  /** Unique identifier for this segment. */
  id: string;
  /** Embedding vector for similarity search. */
  embedding: number[];
  /** The textual content of this segment. */
  content: string;
  /** Source identifier (e.g., file path, API endpoint). */
  source: string;
  /** Name of the plugin that produced this segment. */
  plugin: string;
  /** Type of segment (e.g., "pm_counter", "alarm", "config"). */
  segmentType: string;
  /** Timestamp of when this segment was created or ingested (ISO 8601). */
  timestamp: string;
  /** Additional metadata. */
  metadata: Record<string, unknown>;
  /** Storage tier for this segment. */
  tier: Tier;
}

// ---------------------------------------------------------------------------
// Evaluator types
// ---------------------------------------------------------------------------

/** Result of a domain evaluation. */
export interface EvaluationResult {
  /** Overall quality score (0.0 to 1.0). */
  score: number;
  /** Named metrics with their values. */
  metrics: Record<string, number>;
  /** Human-readable feedback string. */
  feedback: string;
}

/** Interface for domain-specific evaluation of query responses. */
export interface DomainEvaluator {
  /** Evaluate a response given the original query and context segments. */
  evaluate(
    query: string,
    response: string,
    context: ContextSegment[],
  ): EvaluationResult;
}

// ---------------------------------------------------------------------------
// Ingest adapter interface
// ---------------------------------------------------------------------------

/** Interface for adapters that ingest data from various sources into ContextSegments. */
export interface IngestAdapter {
  /** Name of this ingest adapter. */
  readonly name: string;
  /** List of supported input formats (e.g., "xml", "csv", "json"). */
  supportedFormats(): string[];
  /** Ingest data from a source path, returning all segments at once. */
  ingest(source: string): Promise<ContextSegment[]>;
  /** Ingest data in batches for large sources. */
  ingestBatch(source: string, batchSize: number): Promise<ContextSegment[]>;
}

// ---------------------------------------------------------------------------
// Plugin manifest & status
// ---------------------------------------------------------------------------

/** Status of a plugin within the registry. */
export enum PluginStatus {
  /** Plugin is registered and active. */
  Active = 'Active',
  /** Plugin is registered but disabled. */
  Disabled = 'Disabled',
  /** Plugin encountered an error during loading. */
  Error = 'Error',
}

/** Manifest describing a plugin's metadata and capabilities. */
export interface PluginManifest {
  /** Unique name of the plugin. */
  name: string;
  /** Semantic version string (e.g., "0.1.0"). */
  version: string;
  /** Human-readable description. */
  description: string;
  /** Optional author name or organization. */
  author?: string;
  /** Optional license identifier. */
  license?: string;
  /** Optional homepage or repository URL. */
  homepage?: string;
  /** List of domain tags for filtering. */
  domains?: string[];
  /** Minimum required host version (semver range). */
  minHostVersion?: string;
}

// ---------------------------------------------------------------------------
// DomainPlugin interface — core extension point
// ---------------------------------------------------------------------------

/** Core interface that all domain plugins must implement. */
export interface DomainPlugin {
  /** Returns the unique name of this plugin. */
  readonly name: string;

  /** Returns the semantic version string. */
  readonly version: string;

  /** Returns a human-readable description. */
  readonly description: string;

  /** Returns the ingest adapters provided by this plugin. */
  ingestAdapters(): IngestAdapter[];

  /** Returns strategy preferences mapping task patterns to strategies. */
  strategyPreferences(): StrategyPreference[];

  /** Returns action definitions that this plugin supports. */
  actionExtensions(): ActionDefinition[];

  /** Returns optional embedding configuration for this domain. */
  embeddingConfig?(): EmbeddingConfig | undefined;

  /** Returns the preferred distance metric for vector similarity. */
  distanceMetric?(): DistanceMetric | undefined;

  /** Returns TRM model configurations used by this plugin. */
  trmModels?(): TrmModelConfig[];

  /** Returns a system prompt extension with domain-specific instructions. */
  systemPromptExtension(): string;

  /** Returns safety constraints that must be enforced for this plugin. */
  safetyConstraints(): SafetyConstraint[];

  /** Returns an optional domain-specific evaluator. */
  evaluator?(): DomainEvaluator | undefined;

  /** Returns RVF segment type definitions for this domain. */
  rvfSegmentTypes?(): SegmentTypeDefinition[];

  /** Returns an optional path to a knowledge base directory. */
  knowledgeBase?(): string | undefined;

  /** Returns an optional graph schema for knowledge graph integration. */
  graphSchema?(): GraphSchema | undefined;
}

// ---------------------------------------------------------------------------
// Plugin info (returned by registry.list())
// ---------------------------------------------------------------------------

/** Information about a loaded plugin, returned by PluginRegistry.list(). */
export interface PluginInfo {
  /** Plugin name. */
  name: string;
  /** Plugin version. */
  version: string;
  /** Plugin description. */
  description: string;
  /** Number of strategy preferences. */
  strategyCount: number;
  /** Number of action definitions. */
  actionCount: number;
  /** Current status of the plugin. */
  status: PluginStatus;
}

// ---------------------------------------------------------------------------
// Plugin lifecycle events
// ---------------------------------------------------------------------------

/** Events emitted during plugin lifecycle changes. */
export type PluginEvent =
  | { type: 'registered'; pluginName: string; version: string }
  | { type: 'unregistered'; pluginName: string }
  | { type: 'enabled'; pluginName: string }
  | { type: 'disabled'; pluginName: string }
  | { type: 'error'; pluginName: string; error: string };

/** Listener callback for plugin lifecycle events. */
export type PluginEventListener = (event: PluginEvent) => void;

// ---------------------------------------------------------------------------
// ActionDefinition builder helper
// ---------------------------------------------------------------------------

/** Builder for creating ActionDefinition objects with a fluent API. */
export class ActionDefinitionBuilder {
  private readonly _name: string;
  private _description: string = '';
  private _params: ActionParam[] = [];

  constructor(name: string) {
    this._name = name;
  }

  /** Set the description. */
  description(desc: string): this {
    this._description = desc;
    return this;
  }

  /** Add a parameter. */
  param(
    name: string,
    paramType: ParamType,
    required: boolean,
    description: string,
  ): this {
    this._params.push({ name, paramType, required, description });
    return this;
  }

  /** Build the ActionDefinition. */
  build(): ActionDefinition {
    return {
      name: this._name,
      description: this._description,
      params: [...this._params],
    };
  }
}

/** Create a new ActionDefinitionBuilder. */
export function actionDef(name: string): ActionDefinitionBuilder {
  return new ActionDefinitionBuilder(name);
}
