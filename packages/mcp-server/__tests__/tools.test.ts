// ---------------------------------------------------------------------------
// @aix/mcp-server — Tools tests
//
// Validates all 47 tool registrations and individual tool handler behavior.
// ---------------------------------------------------------------------------

import { describe, it, expect } from 'vitest';
import { createAllTools, createToolState, toolNames } from '../src/tools.js';

describe('createAllTools', () => {
  const state = createToolState();
  const tools = createAllTools(state);

  it('should create exactly 47 tools', () => {
    expect(tools).toHaveLength(47);
  });

  it('should have unique tool names', () => {
    const names = toolNames(tools);
    const unique = new Set(names);
    expect(unique.size).toBe(47);
  });

  it('should include all expected tool categories', () => {
    const names = toolNames(tools);

    // Core kernel tools (12)
    expect(names).toContain('rlmx_query');
    expect(names).toContain('rlmx_ingest');
    expect(names).toContain('rlmx_memory_stats');
    expect(names).toContain('rlmx_plugin_list');
    expect(names).toContain('rlmx_plugin_action');
    expect(names).toContain('rlmx_strategy_override');
    expect(names).toContain('rlmx_trm_classify');
    expect(names).toContain('rlmx_rvf_seal');
    expect(names).toContain('rlmx_rvf_branch');
    expect(names).toContain('rlmx_witness_chain');
    expect(names).toContain('rlmx_sona_stats');
    expect(names).toContain('rlmx_graph_query');

    // Swarm tools (2)
    expect(names).toContain('rlmx_swarm_status');
    expect(names).toContain('rlmx_swarm_topology');

    // Agent tools (3)
    expect(names).toContain('rlmx_agent_spawn');
    expect(names).toContain('rlmx_agent_list');
    expect(names).toContain('rlmx_agent_terminate');

    // Research tools (3)
    expect(names).toContain('rlmx_research_start');
    expect(names).toContain('rlmx_research_status');
    expect(names).toContain('rlmx_experiment_list');

    // Evolution (1)
    expect(names).toContain('rlmx_mutation_history');

    // Forecasting (1)
    expect(names).toContain('rlmx_forecast');

    // Training (1)
    expect(names).toContain('rlmx_train');

    // Sandbox tools (5)
    expect(names).toContain('rlmx_sandbox_spawn');
    expect(names).toContain('rlmx_sandbox_terminate');
    expect(names).toContain('rlmx_sandbox_status');
    expect(names).toContain('rlmx_sandbox_list');
    expect(names).toContain('rlmx_fleet_deploy');

    // Marketplace tools (8)
    expect(names).toContain('rlmx_marketplace_search');
    expect(names).toContain('rlmx_marketplace_install');
    expect(names).toContain('rlmx_marketplace_uninstall');
    expect(names).toContain('rlmx_marketplace_rate');
    expect(names).toContain('rlmx_marketplace_list_installed');
    expect(names).toContain('rlmx_marketplace_publish');
    expect(names).toContain('rlmx_marketplace_featured');
    expect(names).toContain('rlmx_marketplace_categories');

    // Voice tools (3)
    expect(names).toContain('rlmx_voice_transcribe');
    expect(names).toContain('rlmx_voice_synthesize');
    expect(names).toContain('rlmx_voice_session');

    // Mesh tools (2)
    expect(names).toContain('rlmx_mesh_status');
    expect(names).toContain('rlmx_mesh_devices');

    // Federation tools (2)
    expect(names).toContain('rlmx_federation_status');
    expect(names).toContain('rlmx_federation_contribute');

    // Billing tools (4)
    expect(names).toContain('rlmx_billing_status');
    expect(names).toContain('rlmx_billing_upgrade');
    expect(names).toContain('rlmx_billing_usage');
    expect(names).toContain('rlmx_billing_family');
  });

  it('should have descriptions for all tools', () => {
    for (const t of tools) {
      expect(t.definition.description.length).toBeGreaterThan(0);
    }
  });

  it('should have input schemas for all tools', () => {
    for (const t of tools) {
      expect(t.definition.inputSchema).toBeDefined();
      expect(t.definition.inputSchema.type).toBe('object');
    }
  });
});

describe('tool handlers', () => {
  it('rlmx_query should return query result', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const queryTool = tools.find((t) => t.definition.name === 'rlmx_query')!;
    const result = await queryTool.handler({ query: 'test', strategy: 'rlm' });
    expect(result).toHaveProperty('query', 'test');
    expect(result).toHaveProperty('strategy_used', 'rlm');
    expect(state.queryCount).toBe(1);
  });

  it('rlmx_ingest should track ingestion count', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const ingestTool = tools.find((t) => t.definition.name === 'rlmx_ingest')!;
    await ingestTool.handler({ data: 'hello', plugin: 'text' });
    await ingestTool.handler({ data: 'world', plugin: 'json' });
    expect(state.ingestCount).toBe(2);
  });

  it('rlmx_agent_spawn should add agent to state', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const spawnTool = tools.find((t) => t.definition.name === 'rlmx_agent_spawn')!;
    const result = await spawnTool.handler({ agent_type: 'worker', name: 'test-agent' });
    expect(result).toHaveProperty('agent_type', 'worker');
    expect(result).toHaveProperty('name', 'test-agent');
    expect(state.agents).toHaveLength(1);
  });

  it('rlmx_agent_terminate should remove agent from state', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const spawnTool = tools.find((t) => t.definition.name === 'rlmx_agent_spawn')!;
    const terminateTool = tools.find((t) => t.definition.name === 'rlmx_agent_terminate')!;

    const spawned = (await spawnTool.handler({ agent_type: 'worker' })) as Record<string, unknown>;
    expect(state.agents).toHaveLength(1);

    await terminateTool.handler({ agent_id: spawned.agent_id as string });
    expect(state.agents).toHaveLength(0);
  });

  it('rlmx_marketplace_categories should return 12 domains', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const catTool = tools.find((t) => t.definition.name === 'rlmx_marketplace_categories')!;
    const result = (await catTool.handler({})) as { categories: string[]; total: number };
    expect(result.categories).toHaveLength(12);
    expect(result.total).toBe(12);
  });

  it('rlmx_voice_session start should return session id', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const sessionTool = tools.find((t) => t.definition.name === 'rlmx_voice_session')!;
    const result = (await sessionTool.handler({ action: 'start' })) as Record<string, unknown>;
    expect(result.status).toBe('started');
    expect(result.session_id).toBeDefined();
  });

  it('rlmx_federation_contribute should include privacy params', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const fedTool = tools.find((t) => t.definition.name === 'rlmx_federation_contribute')!;
    const result = (await fedTool.handler({})) as Record<string, unknown>;
    const privacy = result.privacy as Record<string, unknown>;
    expect(privacy.laplace_epsilon).toBe(1.0);
    expect(privacy.min_aggregation_threshold).toBe(1000);
  });

  it('rlmx_billing_status should show free tier', async () => {
    const state = createToolState();
    const tools = createAllTools(state);
    const billingTool = tools.find((t) => t.definition.name === 'rlmx_billing_status')!;
    const result = (await billingTool.handler({})) as Record<string, unknown>;
    expect(result.tier).toBe('free');
    expect(result.agent_limit).toBe(5);
  });
});
