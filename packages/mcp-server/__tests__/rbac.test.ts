// ---------------------------------------------------------------------------
// @aix/mcp-server — RBAC tests
//
// Mirrors the Rust RBAC tests in rlmx-mcp/src/server.rs:
//   - Viewer cannot ingest
//   - Client cannot self-assign Admin
//   - Client cannot self-assign System
//   - Server-side Admin via token
//   - Operator allowed via param
// ---------------------------------------------------------------------------

import { describe, it, expect, beforeEach } from 'vitest';
import { McpServer } from '../src/server.js';
import { McpError, ErrorCode } from '../src/errors.js';
import type { Role } from '../src/rbac.js';
import {
  checkAccess,
  isPrivilegedRole,
  parseUnprivilegedRole,
  toolToOperation,
} from '../src/rbac.js';

describe('RBAC', () => {
  describe('checkAccess', () => {
    it('should allow admin all operations', () => {
      expect(checkAccess('admin', 'Query')).toBe(true);
      expect(checkAccess('admin', 'Ingest')).toBe(true);
      expect(checkAccess('admin', 'ParameterModify')).toBe(true);
      expect(checkAccess('admin', 'ContainerSeal')).toBe(true);
    });

    it('should allow viewer only Query and WitnessView', () => {
      expect(checkAccess('viewer', 'Query')).toBe(true);
      expect(checkAccess('viewer', 'WitnessView')).toBe(true);
      expect(checkAccess('viewer', 'Ingest')).toBe(false);
      expect(checkAccess('viewer', 'ParameterModify')).toBe(false);
      expect(checkAccess('viewer', 'ContainerSeal')).toBe(false);
    });

    it('should allow operator Ingest but not ParameterModify', () => {
      expect(checkAccess('operator', 'Query')).toBe(true);
      expect(checkAccess('operator', 'Ingest')).toBe(true);
      expect(checkAccess('operator', 'ParameterModify')).toBe(false);
    });

    it('should allow engineer ParameterModify but not ContainerSeal', () => {
      expect(checkAccess('engineer', 'ParameterModify')).toBe(true);
      expect(checkAccess('engineer', 'ContainerSeal')).toBe(false);
    });
  });

  describe('isPrivilegedRole', () => {
    it('should identify admin and system as privileged', () => {
      expect(isPrivilegedRole('admin')).toBe(true);
      expect(isPrivilegedRole('system')).toBe(true);
      expect(isPrivilegedRole('Admin')).toBe(false); // case-sensitive
    });

    it('should not flag non-privileged roles', () => {
      expect(isPrivilegedRole('operator')).toBe(false);
      expect(isPrivilegedRole('viewer')).toBe(false);
      expect(isPrivilegedRole('engineer')).toBe(false);
    });
  });

  describe('parseUnprivilegedRole', () => {
    it('should parse valid non-privileged roles', () => {
      expect(parseUnprivilegedRole('viewer')).toBe('viewer');
      expect(parseUnprivilegedRole('operator')).toBe('operator');
      expect(parseUnprivilegedRole('engineer')).toBe('engineer');
      expect(parseUnprivilegedRole('Viewer')).toBe('viewer');
    });

    it('should reject privileged roles', () => {
      expect(parseUnprivilegedRole('admin')).toBeUndefined();
      expect(parseUnprivilegedRole('system')).toBeUndefined();
    });

    it('should return undefined for unknown roles', () => {
      expect(parseUnprivilegedRole('superuser')).toBeUndefined();
    });
  });

  describe('toolToOperation', () => {
    it('should map core tools correctly', () => {
      expect(toolToOperation('rlmx_query')).toBe('Query');
      expect(toolToOperation('rlmx_ingest')).toBe('Ingest');
      expect(toolToOperation('rlmx_strategy_override')).toBe('ParameterModify');
      expect(toolToOperation('rlmx_rvf_seal')).toBe('ContainerSeal');
    });

    it('should map agent tools correctly', () => {
      expect(toolToOperation('rlmx_agent_spawn')).toBe('ParameterModify');
      expect(toolToOperation('rlmx_agent_list')).toBe('Ingest');
      expect(toolToOperation('rlmx_agent_terminate')).toBe('ContainerSeal');
    });

    it('should map marketplace tools correctly', () => {
      expect(toolToOperation('rlmx_marketplace_search')).toBe('Query');
      expect(toolToOperation('rlmx_marketplace_install')).toBe('Ingest');
      expect(toolToOperation('rlmx_marketplace_publish')).toBe('ParameterModify');
    });

    it('should default unknown tools to Query', () => {
      expect(toolToOperation('unknown_tool')).toBe('Query');
    });
  });
});

describe('McpServer RBAC integration', () => {
  let server: McpServer;

  beforeEach(async () => {
    server = new McpServer();
    await server.handleRequest('initialize');
  });

  it('should deny viewer from ingest', async () => {
    await expect(
      server.handleRequest('tools/call', {
        name: 'rlmx_ingest',
        arguments: { data: 'test', plugin: 'text' },
        _role: 'viewer',
      }),
    ).rejects.toThrow(McpError);

    try {
      await server.handleRequest('tools/call', {
        name: 'rlmx_ingest',
        arguments: { data: 'test', plugin: 'text' },
        _role: 'viewer',
      });
    } catch (e) {
      expect((e as McpError).code).toBe(ErrorCode.ACCESS_DENIED);
    }
  });

  it('should reject client self-assigning admin role', async () => {
    await expect(
      server.handleRequest('tools/call', {
        name: 'rlmx_query',
        arguments: { query: 'test' },
        _role: 'admin',
      }),
    ).rejects.toThrow(McpError);

    try {
      await server.handleRequest('tools/call', {
        name: 'rlmx_query',
        arguments: { query: 'test' },
        _role: 'admin',
      });
    } catch (e) {
      const err = e as McpError;
      expect(err.code).toBe(ErrorCode.ACCESS_DENIED);
      expect(err.message).toContain('privileged roles must be configured server-side');
    }
  });

  it('should reject client self-assigning system role', async () => {
    await expect(
      server.handleRequest('tools/call', {
        name: 'rlmx_query',
        arguments: { query: 'test' },
        _role: 'system',
      }),
    ).rejects.toThrow(McpError);

    try {
      await server.handleRequest('tools/call', {
        name: 'rlmx_query',
        arguments: { query: 'test' },
        _role: 'system',
      });
    } catch (e) {
      const err = e as McpError;
      expect(err.code).toBe(ErrorCode.ACCESS_DENIED);
      expect(err.message).toContain('privileged roles must be configured server-side');
    }
  });

  it('should allow server-side admin role via token', async () => {
    const tokenRoles = new Map<string, Role>();
    tokenRoles.set('admin-token-123', 'admin');

    const authServer = new McpServer({
      authEnabled: true,
      authToken: 'admin-token-123',
      tokenRoles,
    });
    await authServer.handleRequest('initialize');

    const result = await authServer.handleRequest(
      'tools/call',
      {
        name: 'rlmx_query',
        arguments: { query: 'test' },
      },
      'admin-token-123',
    );
    expect(result).toHaveProperty('content');
  });

  it('should allow operator role via _role param', async () => {
    const result = await server.handleRequest('tools/call', {
      name: 'rlmx_query',
      arguments: { query: 'test' },
      _role: 'operator',
    });
    expect(result).toHaveProperty('content');
  });
});
