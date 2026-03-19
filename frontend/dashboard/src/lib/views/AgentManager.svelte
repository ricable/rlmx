<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { callTool, parseToolResult } from '../api/mcp';

  const agentTypes = [
    'coordinator', 'researcher', 'router', 'experimenter', 'worker', 'monitor',
    'reviewer', 'trainer', 'validator', 'replicator', 'embedder', 'analyst',
  ] as const;

  let selectedType = $state<string>('coordinator');
  let agentName = $state('');
  let agentTask = $state('');
  let forging = $state(false);
  let agents = $state<any[]>([]);
  let loading = $state(false);

  async function loadAgents() {
    loading = true;
    try {
      const res = await callTool('rlmx_agent_list');
      const data = parseToolResult<any>(res);
      agents = Array.isArray(data) ? data : (data?.agents ?? []);
    } catch { /* ignore */ }
    loading = false;
  }

  async function forgeAgent() {
    if (!agentName.trim()) return;
    forging = true;
    try {
      await callTool('rlmx_agent_spawn', {
        agent_type: selectedType,
        name: agentName.trim(),
        task: agentTask.trim() || undefined,
      });
      agentName = '';
      agentTask = '';
      await loadAgents();
    } catch { /* ignore */ }
    forging = false;
  }

  async function terminateAgent(id: string) {
    try {
      await callTool('rlmx_agent_terminate', { agent_id: id });
      await loadAgents();
    } catch { /* ignore */ }
  }

  $effect(() => { loadAgents(); });
</script>

<div class="space-y-6">
  <!-- Forge Agent panel -->
  <ForgePanel title="Forge Agent">
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div>
        <label for="agent-type" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Type</label>
        <select
          id="agent-type"
          bind:value={selectedType}
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember"
        >
          {#each agentTypes as t}
            <option value={t}>{t}</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="agent-name" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Name</label>
        <input
          id="agent-name"
          bind:value={agentName}
          type="text"
          placeholder="agent-alpha"
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke"
        />
      </div>
      <div>
        <label for="agent-task" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Task</label>
        <input
          id="agent-task"
          bind:value={agentTask}
          type="text"
          placeholder="optional task description"
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke"
        />
      </div>
      <div class="flex items-end">
        <button
          onclick={forgeAgent}
          disabled={forging || !agentName.trim()}
          class="w-full bg-ember text-void font-display font-bold text-sm px-4 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {forging ? 'Forging...' : 'Forge'}
        </button>
      </div>
    </div>
  </ForgePanel>

  <!-- Agent cards grid -->
  <ForgePanel title="Active Agents">
    {#if loading}
      <p class="font-mono text-sm text-chalk">Loading agents...</p>
    {:else if agents.length === 0}
      <p class="font-mono text-sm text-smoke">No agents active. Forge one above.</p>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {#each agents as agent}
          <div class="bg-iron border border-steel rounded-none p-4 space-y-2">
            <div class="flex items-center justify-between">
              <span class="font-mono text-[10px] uppercase tracking-wide px-2 py-0.5 bg-ember/10 text-ember border border-ember/20">
                {agent.agent_type ?? agent.type ?? 'unknown'}
              </span>
              <div class="flex items-center gap-1.5">
                <div class="w-2 h-2 rounded-full {agent.status === 'running' ? 'bg-mint' : 'bg-smoke'}"></div>
                <span class="font-mono text-[10px] text-chalk">{agent.status ?? 'idle'}</span>
              </div>
            </div>
            <p class="font-display font-semibold text-sm text-bone">{agent.name ?? agent.id ?? '--'}</p>
            {#if agent.zone}
              <p class="font-mono text-[10px] text-smoke">Zone: {agent.zone}</p>
            {/if}
            <button
              onclick={() => terminateAgent(agent.id ?? agent.name)}
              class="w-full mt-2 border border-molten/30 text-molten font-mono text-[10px] uppercase py-1 rounded-none hover:bg-molten/10 transition-colors"
            >
              Terminate
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </ForgePanel>
</div>
