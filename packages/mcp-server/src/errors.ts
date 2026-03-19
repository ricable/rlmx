// ---------------------------------------------------------------------------
// @aix/mcp-server — Error types
//
// Mirrors the MCP protocol error codes and provides structured error
// classes for tool dispatch, RBAC, and transport failures.
// ---------------------------------------------------------------------------

/** Standard JSON-RPC 2.0 error codes. */
export const ErrorCode = {
  PARSE_ERROR: -32700,
  INVALID_REQUEST: -32600,
  METHOD_NOT_FOUND: -32601,
  INVALID_PARAMS: -32602,
  INTERNAL_ERROR: -32603,
  SERVER_NOT_INITIALIZED: -32002,
  ACCESS_DENIED: -32001,
} as const;

export type ErrorCodeValue = (typeof ErrorCode)[keyof typeof ErrorCode];

/**
 * Structured MCP error suitable for JSON-RPC error responses.
 */
export class McpError extends Error {
  readonly code: ErrorCodeValue | number;
  readonly data?: unknown;

  constructor(code: ErrorCodeValue | number, message: string, data?: unknown) {
    super(message);
    this.name = 'McpError';
    this.code = code;
    this.data = data;
    Object.setPrototypeOf(this, new.target.prototype);
  }

  static methodNotFound(method: string): McpError {
    return new McpError(
      ErrorCode.METHOD_NOT_FOUND,
      `Method not found: ${method}`,
    );
  }

  static invalidParams(detail: string): McpError {
    return new McpError(ErrorCode.INVALID_PARAMS, detail);
  }

  static accessDenied(detail: string): McpError {
    return new McpError(ErrorCode.ACCESS_DENIED, detail);
  }

  static notInitialized(): McpError {
    return new McpError(
      ErrorCode.SERVER_NOT_INITIALIZED,
      'Server not initialized',
    );
  }

  static internal(detail: string): McpError {
    return new McpError(ErrorCode.INTERNAL_ERROR, detail);
  }

  /** Serialize to a JSON-RPC error object. */
  toJSON(): { code: number; message: string; data?: unknown } {
    return {
      code: this.code,
      message: this.message,
      ...(this.data !== undefined ? { data: this.data } : {}),
    };
  }
}
