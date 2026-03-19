// ---------------------------------------------------------------------------
// @aix/mcp-server — Configuration
//
// McpConfig mirrors the Rust McpConfig struct: transport mode, auth,
// rate limiting, token-to-role mapping, and WebSocket port.
// ---------------------------------------------------------------------------

import type { Role } from './rbac.js';

/** Transport mode for the MCP server. */
export type Transport =
  | { kind: 'stdio' }
  | { kind: 'streamable-http'; host: string; port: number };

/** Configuration for the MCP server. */
export interface McpConfig {
  /** Transport mode (stdio or HTTP). */
  transport: Transport;
  /** Whether bearer-token authentication is enabled. */
  authEnabled: boolean;
  /** Bearer token required when authEnabled is true. */
  authToken?: string;
  /** Maximum requests per minute (0 = unlimited). */
  rateLimitPerMinute: number;
  /**
   * Server-side mapping of bearer tokens to RBAC roles.
   *
   * Privileged roles (Admin, System) can only be assigned via this
   * mapping -- clients cannot self-escalate through request parameters.
   */
  tokenRoles: Map<string, Role>;
  /** WebSocket event server port. */
  wsPort: number;
}

/** Create a default configuration (stdio, no auth). */
export function defaultConfig(): McpConfig {
  return {
    transport: { kind: 'stdio' },
    authEnabled: false,
    rateLimitPerMinute: 0,
    tokenRoles: new Map(),
    wsPort: 3001,
  };
}

/**
 * Look up the RBAC role assigned to a given bearer token.
 * Returns undefined if the token has no explicit role mapping.
 */
export function roleForToken(
  config: McpConfig,
  token: string,
): Role | undefined {
  return config.tokenRoles.get(token);
}
