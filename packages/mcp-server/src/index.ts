// ---------------------------------------------------------------------------
// @aix/mcp-server — Public API
//
// Re-exports all public types and classes for the MCP server package.
// ---------------------------------------------------------------------------

export { McpServer } from './server.js';

export {
  type McpConfig,
  type Transport,
  defaultConfig,
  roleForToken,
} from './config.js';

export {
  McpError,
  ErrorCode,
  type ErrorCodeValue,
} from './errors.js';

export {
  type Role,
  type Operation,
  checkAccess,
  isPrivilegedRole,
  parseUnprivilegedRole,
  toolToOperation,
} from './rbac.js';

export {
  createAllTools,
  createToolState,
  toolNames,
  type RegisteredTool,
} from './tools.js';

export type {
  JsonRpcRequest,
  JsonRpcResponse,
  JsonRpcSuccessResponse,
  JsonRpcErrorResponse,
  McpToolDefinition,
  ToolHandler,
  ToolStateData,
  QueryInput,
  IngestInput,
  GraphQueryInput,
  StrategyOverrideInput,
  PluginActionInput,
  RvfSealInput,
  RvfBranchInput,
  AgentSpawnInput,
  AgentTerminateInput,
  ResearchStartInput,
  ResearchStatusInput,
  SandboxSpawnInput,
  SandboxTerminateInput,
  SandboxStatusInput,
  SandboxListInput,
  FleetDeployInput,
  MarketplaceSearchInput,
  MarketplaceInstallInput,
  MarketplaceUninstallInput,
  MarketplaceRateInput,
  MarketplacePublishInput,
  VoiceTranscribeInput,
  VoiceSynthesizeInput,
  VoiceSessionInput,
  MeshStatusInput,
  MeshDevicesInput,
  FederationContributeInput,
  BillingUpgradeInput,
  BillingFamilyInput,
  ForecastInput,
  TrainInput,
  MutationHistoryInput,
  QueryResult,
  IngestResult,
  MemoryStatsResult,
  ToolResult,
} from './types.js';

export {
  WsServer,
  SwarmEventBus,
  ClientState,
  ALL_EVENT_TYPES,
  MAX_WS_CONNECTIONS,
  HEARTBEAT_INTERVAL_MS,
  MAX_MISSED_PONGS,
  MAX_PENDING_MESSAGES,
  AUTH_TIMEOUT_MS,
  type SwarmEvent,
  type Zone,
  type LeaveReason,
  type TerminationReason,
  type ExpStatus,
  type HapticPattern,
  type CardData,
  type WsSubscription,
  type WsFilters,
  type WsAuthMessage,
} from './websocket.js';
