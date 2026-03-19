/** RLMX MCP JSON-RPC 2.0 Client */

const MCP_URL = '/mcp';
let reqId = 0;
let initialized = false;

export interface RpcResponse {
  jsonrpc: string;
  id: number;
  result?: { content?: Array<{ text: string }> };
  error?: { code: number; message: string };
}

export interface LogEntry {
  ts: string;
  method: string;
  id: number;
  response?: RpcResponse;
}

export const logs: LogEntry[] = [];

export async function rpc(method: string, params: Record<string, unknown> = {}): Promise<RpcResponse> {
  const id = ++reqId;
  const ts = new Date().toLocaleTimeString();
  const entry: LogEntry = { ts, method, id };
  logs.unshift(entry);
  if (logs.length > 200) logs.length = 200;

  try {
    const res = await fetch(MCP_URL, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id, method, params }),
    });
    const json: RpcResponse = await res.json();
    entry.response = json;
    return json;
  } catch (err) {
    const errMsg = err instanceof Error ? err.message : String(err);
    entry.response = { jsonrpc: '2.0', id, error: { code: -1, message: errMsg } };
    throw err;
  }
}

export async function connect(): Promise<boolean> {
  try {
    await rpc('initialize', {
      protocolVersion: '2025-03-26',
      clientInfo: { name: 'rlmx-forge', version: '2.0' },
      capabilities: {},
    });
    initialized = true;
    return true;
  } catch {
    return false;
  }
}

export async function callTool(name: string, args: Record<string, unknown> = {}): Promise<RpcResponse> {
  if (!initialized) await connect();
  return rpc('tools/call', { name, arguments: args });
}

export function extractText(res: RpcResponse): string {
  return res?.result?.content?.map(c => c.text).join('') ?? '';
}

export function parseToolResult<T>(res: RpcResponse): T | null {
  try {
    return JSON.parse(extractText(res)) as T;
  } catch {
    return null;
  }
}
