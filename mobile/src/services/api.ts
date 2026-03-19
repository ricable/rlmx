const MCP_BASE = 'http://localhost:3000';

interface McpResponse {
  jsonrpc: string;
  id: number;
  result?: unknown;
  error?: {code: number; message: string};
}

let requestId = 0;

export async function mcpCall(method: string, params: Record<string, unknown> = {}): Promise<unknown> {
  requestId++;
  const body = {
    jsonrpc: '2.0',
    id: requestId,
    method: 'tools/call',
    params: {
      name: method,
      arguments: params,
    },
  };

  try {
    const res = await fetch(`${MCP_BASE}/mcp`, {
      method: 'POST',
      headers: {'Content-Type': 'application/json'},
      body: JSON.stringify(body),
    });
    const json: McpResponse = await res.json();
    if (json.error) {
      throw new Error(json.error.message);
    }
    return json.result;
  } catch {
    return null;
  }
}

export async function checkConnection(): Promise<boolean> {
  try {
    const res = await fetch(`${MCP_BASE}/health`, {method: 'GET'});
    return res.ok;
  } catch {
    return false;
  }
}
