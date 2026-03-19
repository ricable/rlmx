// ---------------------------------------------------------------------------
// @aix/mcp-server — Server tests
//
// Mirrors the Rust tests in rlmx-mcp/src/server.rs:
//   - Tool registration (47 tools)
//   - Initialize handshake
//   - tools/list and tools/call dispatch
//   - Rejection before initialization
//   - Method not found
// ---------------------------------------------------------------------------

import { describe, it, expect, beforeEach } from 'vitest';
import { McpServer } from '../src/server.js';
import { McpError, ErrorCode } from '../src/errors.js';

describe('McpServer', () => {
  let server: McpServer;

  beforeEach(() => {
    server = new McpServer();
  });

  it('should register all 47 tools', () => {
    expect(server.toolCount).toBe(47);
  });

  it('should list all tool names', () => {
    const names = server.toolNames;
    expect(names).toContain('rlmx_query');
    expect(names).toContain('rlmx_graph_query');
    expect(names).toContain('rlmx_sona_stats');
    expect(names).toContain('rlmx_marketplace_search');
    expect(names).toContain('rlmx_voice_transcribe');
    expect(names).toContain('rlmx_mesh_status');
    expect(names).toContain('rlmx_federation_status');
    expect(names).toContain('rlmx_billing_status');
  });

  it('should handle initialize', async () => {
    const result = await server.handleRequest('initialize');
    expect(result).toHaveProperty('serverInfo');
    expect((result as Record<string, unknown>).serverInfo).toEqual({
      name: 'aix-mcp',
      version: '0.1.0',
    });
    expect(server.isInitialized).toBe(true);
  });

  it('should handle tools/list after initialization', async () => {
    await server.handleRequest('initialize');
    const result = (await server.handleRequest('tools/list')) as {
      tools: unknown[];
    };
    expect(result.tools).toHaveLength(47);
  });

  it('should handle tools/call after initialization', async () => {
    await server.handleRequest('initialize');
    const result = await server.handleRequest('tools/call', {
      name: 'rlmx_memory_stats',
      arguments: {},
    });
    expect(result).toHaveProperty('content');
  });

  it('should reject tools/list before initialization', async () => {
    await expect(server.handleRequest('tools/list')).rejects.toThrow(McpError);
    try {
      await server.handleRequest('tools/list');
    } catch (e) {
      expect((e as McpError).code).toBe(ErrorCode.SERVER_NOT_INITIALIZED);
    }
  });

  it('should reject tools/call before initialization', async () => {
    await expect(
      server.handleRequest('tools/call', {
        name: 'rlmx_query',
        arguments: { query: 'test' },
      }),
    ).rejects.toThrow(McpError);
  });

  it('should return error for unknown method', async () => {
    await expect(
      server.handleRequest('nonexistent/method'),
    ).rejects.toThrow(McpError);
    try {
      await server.handleRequest('nonexistent/method');
    } catch (e) {
      expect((e as McpError).code).toBe(ErrorCode.METHOD_NOT_FOUND);
    }
  });

  it('should return error for unknown tool', async () => {
    await server.handleRequest('initialize');
    await expect(
      server.handleRequest('tools/call', {
        name: 'nonexistent_tool',
        arguments: {},
      }),
    ).rejects.toThrow(McpError);
  });

  it('should return error when tools/call params missing', async () => {
    await server.handleRequest('initialize');
    await expect(
      server.handleRequest('tools/call'),
    ).rejects.toThrow(McpError);
  });

  it('should return error when tools/call name missing', async () => {
    await server.handleRequest('initialize');
    await expect(
      server.handleRequest('tools/call', { arguments: {} }),
    ).rejects.toThrow(McpError);
  });

  it('should handle initialized notification', async () => {
    const result = await server.handleRequest('initialized');
    expect(result).toEqual({});
  });

  it('should execute rlmx_query tool', async () => {
    await server.handleRequest('initialize');
    const result = (await server.handleRequest('tools/call', {
      name: 'rlmx_query',
      arguments: { query: 'test query' },
    })) as { content: Array<{ text: string }> };
    const parsed = JSON.parse(result.content[0].text);
    expect(parsed.query).toBe('test query');
    expect(parsed.strategy_used).toBe('rlm');
  });

  it('should execute rlmx_ingest tool', async () => {
    await server.handleRequest('initialize');
    const result = (await server.handleRequest('tools/call', {
      name: 'rlmx_ingest',
      arguments: { data: 'hello world', plugin: 'text' },
    })) as { content: Array<{ text: string }> };
    const parsed = JSON.parse(result.content[0].text);
    expect(parsed.status).toBe('ingested');
    expect(parsed.plugin).toBe('text');
  });

  it('should track state across tool calls', async () => {
    await server.handleRequest('initialize');

    // Ingest twice
    await server.handleRequest('tools/call', {
      name: 'rlmx_ingest',
      arguments: { data: 'data1', plugin: 'text' },
    });
    await server.handleRequest('tools/call', {
      name: 'rlmx_ingest',
      arguments: { data: 'data2', plugin: 'json' },
    });

    // Check memory stats
    const result = (await server.handleRequest('tools/call', {
      name: 'rlmx_memory_stats',
      arguments: {},
    })) as { content: Array<{ text: string }> };
    const parsed = JSON.parse(result.content[0].text);
    expect(parsed.total_ingestions).toBe(2);
  });
});
