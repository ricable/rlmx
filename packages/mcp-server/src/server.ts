// ---------------------------------------------------------------------------
// @aix/mcp-server — McpServer class
//
// Ports the Rust McpServer to TypeScript using @modelcontextprotocol/sdk.
// Handles:
//   - MCP protocol initialization handshake
//   - tools/list and tools/call dispatch
//   - RBAC enforcement with 6 roles
//   - Anti-escalation: clients cannot self-assign Admin or System roles
//   - WebSocket event bus integration
// ---------------------------------------------------------------------------

import { McpError, ErrorCode } from './errors.js';
import {
  type McpConfig,
  defaultConfig,
  roleForToken,
} from './config.js';
import {
  type Role,
  checkAccess,
  isPrivilegedRole,
  parseUnprivilegedRole,
  toolToOperation,
} from './rbac.js';
import {
  createAllTools,
  createToolState,
  type RegisteredTool,
} from './tools.js';
import type { ToolStateData, McpToolDefinition } from './types.js';
import { WsServer, type SwarmEvent } from './websocket.js';

// ---------------------------------------------------------------------------
// McpServer
// ---------------------------------------------------------------------------

export class McpServer {
  private readonly tools: RegisteredTool[];
  private readonly toolIndex: Map<string, RegisteredTool>;
  private readonly config: McpConfig;
  private initialized = false;
  private readonly toolState: ToolStateData;
  private readonly wsServer: WsServer;

  constructor(config?: Partial<McpConfig>) {
    const fullConfig = { ...defaultConfig(), ...config };
    this.config = fullConfig;
    this.toolState = createToolState();
    this.tools = createAllTools(this.toolState);
    this.toolIndex = new Map(this.tools.map(t => [t.definition.name, t]));
    this.wsServer = new WsServer(fullConfig.authToken);
  }

  // -----------------------------------------------------------------------
  // Accessors
  // -----------------------------------------------------------------------

  /** Number of registered tools. */
  get toolCount(): number {
    return this.tools.length;
  }

  /** List of registered tool names. */
  get toolNames(): string[] {
    return this.tools.map((t) => t.definition.name);
  }

  /** Tool definitions for tools/list. */
  get toolDefinitions(): McpToolDefinition[] {
    return this.tools.map((t) => t.definition);
  }

  /** Whether auth is enabled. */
  get authEnabled(): boolean {
    return this.config.authEnabled;
  }

  /** Server configuration (read-only). */
  get serverConfig(): Readonly<McpConfig> {
    return this.config;
  }

  /** Is the server initialized (received initialize handshake)? */
  get isInitialized(): boolean {
    return this.initialized;
  }

  /** The WebSocket event server. */
  get ws(): WsServer {
    return this.wsServer;
  }

  /** The underlying shared tool state. */
  get state(): ToolStateData {
    return this.toolState;
  }

  // -----------------------------------------------------------------------
  // Broadcast convenience
  // -----------------------------------------------------------------------

  /** Broadcast a SwarmEvent to all WebSocket clients. */
  broadcast(event: SwarmEvent): void {
    this.wsServer.broadcast(event);
  }

  // -----------------------------------------------------------------------
  // Request handling
  // -----------------------------------------------------------------------

  /**
   * Handle an incoming MCP JSON-RPC request.
   *
   * @param method   JSON-RPC method name
   * @param params   Request parameters (may be undefined)
   * @param callerToken  Bearer token presented by the caller (if any)
   * @returns The JSON-RPC result object (or throws McpError)
   */
  async handleRequest(
    method: string,
    params?: Record<string, unknown>,
    callerToken?: string,
  ): Promise<unknown> {
    switch (method) {
      case 'initialize':
        return this.handleInitialize();

      case 'initialized':
        return {};

      case 'tools/list':
        this.ensureInitialized();
        return this.handleToolsList();

      case 'tools/call':
        this.ensureInitialized();
        return this.handleToolsCall(params, callerToken);

      default:
        throw McpError.methodNotFound(method);
    }
  }

  // -----------------------------------------------------------------------
  // Handlers
  // -----------------------------------------------------------------------

  private handleInitialize(): Record<string, unknown> {
    this.initialized = true;
    return {
      protocolVersion: '2024-11-05',
      capabilities: {
        tools: { listChanged: false },
      },
      serverInfo: {
        name: 'aix-mcp',
        version: '0.1.0',
      },
    };
  }

  private handleToolsList(): Record<string, unknown> {
    const tools = this.tools.map((t) => ({
      name: t.definition.name,
      description: t.definition.description,
      inputSchema: t.definition.inputSchema,
    }));
    return { tools };
  }

  private async handleToolsCall(
    params?: Record<string, unknown>,
    callerToken?: string,
  ): Promise<Record<string, unknown>> {
    if (!params) {
      throw McpError.invalidParams('Missing params for tools/call');
    }

    const toolName = params.name as string | undefined;
    if (!toolName) {
      throw McpError.invalidParams("Missing 'name' in tools/call params");
    }

    const args = (params.arguments ?? {}) as Record<string, unknown>;

    // Find the tool via index (O(1) lookup)
    const registeredTool = this.toolIndex.get(toolName);
    if (!registeredTool) {
      throw new McpError(
        ErrorCode.METHOD_NOT_FOUND,
        `Tool not found: ${toolName}`,
      );
    }

    // --- RBAC enforcement ---
    const callerRole = this.resolveCallerRole(params, callerToken);
    const requiredOp = toolToOperation(toolName);

    if (!checkAccess(callerRole, requiredOp)) {
      throw McpError.accessDenied(
        `Access denied: role '${callerRole}' cannot perform '${requiredOp}'`,
      );
    }

    // Execute the tool handler
    const result = await registeredTool.handler(args);

    return {
      content: [
        {
          type: 'text',
          text: JSON.stringify(result, null, 2),
        },
      ],
    };
  }

  // -----------------------------------------------------------------------
  // RBAC resolution
  // -----------------------------------------------------------------------

  /**
   * Resolve the caller's RBAC role.
   *
   * SECURITY: Privileged roles (Admin, System) CANNOT be assigned via
   * the client-supplied `_role` parameter. They can only be granted
   * server-side through the `tokenRoles` mapping in McpConfig.
   */
  private resolveCallerRole(
    params: Record<string, unknown>,
    callerToken?: string,
  ): Role {
    // First, reject any attempt to claim a privileged role via _role param
    const requestedRole = params._role as string | undefined;
    if (requestedRole && isPrivilegedRole(requestedRole)) {
      throw McpError.accessDenied(
        `Access denied: role '${requestedRole}' cannot be assigned via request parameters; ` +
          'privileged roles must be configured server-side',
      );
    }

    if (this.config.authEnabled) {
      // Check server-side token role mapping first
      if (callerToken) {
        const tokenRole = roleForToken(this.config, callerToken);
        if (tokenRole) return tokenRole;
      }
      // Fall back to non-privileged role from params, default to viewer
      if (requestedRole) {
        return parseUnprivilegedRole(requestedRole) ?? 'viewer';
      }
      return 'viewer';
    }

    // Auth disabled: allow non-privileged role from params, default to operator
    if (requestedRole) {
      return parseUnprivilegedRole(requestedRole) ?? 'operator';
    }
    return 'operator';
  }

  // -----------------------------------------------------------------------
  // Guards
  // -----------------------------------------------------------------------

  private ensureInitialized(): void {
    if (!this.initialized) {
      throw McpError.notInitialized();
    }
  }
}
