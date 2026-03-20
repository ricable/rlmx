// ---------------------------------------------------------------------------
// @aix/mcp-server — Tool input/output types
//
// Type definitions for all 47 MCP tool schemas, request/response envelopes,
// and shared structures used across tool handlers.
// ---------------------------------------------------------------------------

import type { JsonRecord } from '@aix/core';

// ---------------------------------------------------------------------------
// JSON-RPC 2.0 envelope types
// ---------------------------------------------------------------------------

export interface JsonRpcRequest {
  jsonrpc: '2.0';
  id: string | number | null;
  method: string;
  params?: Record<string, unknown>;
}

export interface JsonRpcSuccessResponse {
  jsonrpc: '2.0';
  id: string | number | null;
  result: unknown;
}

export interface JsonRpcErrorResponse {
  jsonrpc: '2.0';
  id: string | number | null;
  error: {
    code: number;
    message: string;
    data?: unknown;
  };
}

export type JsonRpcResponse = JsonRpcSuccessResponse | JsonRpcErrorResponse;

// ---------------------------------------------------------------------------
// MCP tool definition
// ---------------------------------------------------------------------------

export interface McpToolDefinition {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Tool handler function signature
// ---------------------------------------------------------------------------

export type ToolHandler = (
  args: Record<string, unknown>,
) => Promise<JsonRecord>;

// ---------------------------------------------------------------------------
// Tool input types (by tool name)
// ---------------------------------------------------------------------------

export interface QueryInput {
  query: string;
  context_window?: number;
  strategy?: 'auto' | 'rlm' | 'trm';
}

export interface IngestInput {
  data: string;
  plugin: string;
  metadata?: Record<string, unknown>;
}

export interface GraphQueryInput {
  cypher: string;
  limit?: number;
}

export interface StrategyOverrideInput {
  strategy: string;
  ttl_seconds?: number;
}

export interface PluginActionInput {
  plugin: string;
  action: string;
  params?: Record<string, unknown>;
}

export interface RvfSealInput {
  container_id: string;
  segments?: string[];
}

export interface RvfBranchInput {
  container_id: string;
  branch_name: string;
}

export interface AgentSpawnInput {
  agent_type: string;
  name?: string;
  task?: string;
}

export interface AgentTerminateInput {
  agent_id: string;
  reason?: string;
}

export interface ResearchStartInput {
  topic: string;
  hypotheses?: number;
  nodes?: number;
}

export interface ResearchStatusInput {
  research_id: string;
}

export interface SandboxSpawnInput {
  profile: string;
  zone?: string;
}

export interface SandboxTerminateInput {
  sandbox_id: string;
}

export interface SandboxStatusInput {
  sandbox_id: string;
}

export interface SandboxListInput {
  state?: string;
  profile?: string;
}

export interface FleetDeployInput {
  manifest: Record<string, unknown>;
}

export interface MarketplaceSearchInput {
  domain: string;
  query?: string;
  min_rating?: number;
}

export interface MarketplaceInstallInput {
  agent_id: string;
}

export interface MarketplaceUninstallInput {
  agent_id: string;
}

export interface MarketplaceRateInput {
  agent_id: string;
  rating: number;
  review?: string;
}

export interface MarketplacePublishInput {
  path: string;
  description?: string;
}

export interface VoiceTranscribeInput {
  audio_data?: string;
  text?: string;
  language?: string;
}

export interface VoiceSynthesizeInput {
  text: string;
  persona?: string;
}

export interface VoiceSessionInput {
  action: 'start' | 'end' | 'list';
  session_id?: string;
}

export interface MeshStatusInput {
  verbose?: boolean;
}

export interface MeshDevicesInput {
  zone?: string;
}

export interface FederationContributeInput {
  force?: boolean;
}

export interface BillingUpgradeInput {
  tier: string;
}

export interface BillingFamilyInput {
  action?: string;
}

export interface ForecastInput {
  metric: string;
  horizon_hours?: number;
  model?: string;
}

export interface TrainInput {
  config: Record<string, unknown>;
  node_id?: string;
  backend?: string;
}

export interface MutationHistoryInput {
  research_id?: string;
  limit?: number;
}

// ---------------------------------------------------------------------------
// Tool output types
// ---------------------------------------------------------------------------

export interface QueryResult {
  query: string;
  strategy_used: string;
  segments_returned: number;
  results: Array<{
    segment_id: string;
    content: string;
    relevance_score: number;
    tier: string;
  }>;
  total_segments_scanned: number;
  latency_ms: number;
}

export interface IngestResult {
  segment_id: string;
  source: string;
  size_bytes: number;
  status: string;
}

export interface MemoryStatsResult {
  region: string;
  total_segments: number;
  total_queries: number;
  total_ingestions: number;
  embedding_dim: number;
}

export interface ToolResult {
  status: string;
  [key: string]: unknown;
}

// ---------------------------------------------------------------------------
// Shared tool state
// ---------------------------------------------------------------------------

export interface ToolStateData {
  ingestCount: number;
  queryCount: number;
  swarmNodes: JsonRecord[];
  agents: JsonRecord[];
  experiments: JsonRecord[];
  mutations: JsonRecord[];
  researchTasks: JsonRecord[];
  sandboxProfiles: JsonRecord[];
  sandboxInstances: JsonRecord[];
}
