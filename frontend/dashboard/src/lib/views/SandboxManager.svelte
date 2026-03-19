<script lang="ts">
  import MetricCard from '../components/MetricCard.svelte';
  import ForgePanel from '../components/ForgePanel.svelte';
  import { callTool, parseToolResult } from '../api/mcp';

  // 11 ADR-011 sandbox profiles
  const profiles = [
    { name: 'ran-optimizer', agent_type: 'Experimenter', zone: 'A', gpu: 'Metal', network: 'ClusterOnly', model_tier: 'Medium' },
    { name: 'hypothesis-generator', agent_type: 'Researcher', zone: 'A', gpu: 'None', network: 'EgressOnly', model_tier: 'Medium' },
    { name: 'data-collector', agent_type: 'Worker', zone: 'C', gpu: 'None', network: 'EgressOnly', model_tier: 'Small' },
    { name: 'model-trainer', agent_type: 'Experimenter', zone: 'A', gpu: 'Cuda(24GB)', network: 'ClusterOnly', model_tier: 'Custom' },
    { name: 'result-analyzer', agent_type: 'Analyst', zone: 'B', gpu: 'None', network: 'ClusterOnly', model_tier: 'Medium' },
    { name: 'paper-writer', agent_type: 'Worker', zone: 'C', gpu: 'None', network: 'EgressOnly', model_tier: 'Medium' },
    { name: 'code-generator', agent_type: 'Builder', zone: 'A', gpu: 'Metal', network: 'ClusterOnly', model_tier: 'Medium' },
    { name: 'peer-reviewer', agent_type: 'Validator', zone: 'B', gpu: 'None', network: 'Isolated', model_tier: 'Small' },
    { name: 'knowledge-curator', agent_type: 'Librarian', zone: 'B', gpu: 'None', network: 'ClusterOnly', model_tier: 'Small' },
    { name: 'orchestrator', agent_type: 'Coordinator', zone: 'A', gpu: 'None', network: 'Open', model_tier: 'Medium' },
    { name: 'burst-worker', agent_type: 'Worker', zone: 'D', gpu: 'Cuda(80GB)', network: 'Open', model_tier: 'Custom' },
  ] as const;

  let selectedProfile = $state<string>('ran-optimizer');
  let zoneOverride = $state('');
  let spawning = $state(false);
  let instances = $state<any[]>([]);
  let loading = $state(false);
  let fleetJson = $state('');
  let deploying = $state(false);

  // Computed metrics
  let totalInstances = $derived(instances.length);
  let runningCount = $derived(instances.filter(i => i.state === 'Running').length);
  let provisioningCount = $derived(instances.filter(i => i.state === 'Provisioning').length);

  async function loadInstances() {
    loading = true;
    try {
      const res = await callTool('rlmx_sandbox_list');
      const data = parseToolResult<any>(res);
      if (data && (data.sandbox_count > 0 || (data.sandboxes?.length ?? 0) > 0)) {
        instances = data.sandboxes ?? [];
      } else {
        // Demo data fallback
        instances = [
          { sandbox_id: crypto.randomUUID(), profile: 'ran-optimizer', state: 'Running', zone: 'A', agent_type: 'Experimenter' },
          { sandbox_id: crypto.randomUUID(), profile: 'data-collector', state: 'Running', zone: 'C', agent_type: 'Worker' },
          { sandbox_id: crypto.randomUUID(), profile: 'hypothesis-generator', state: 'Provisioning', zone: 'A', agent_type: 'Researcher' },
          { sandbox_id: crypto.randomUUID(), profile: 'burst-worker', state: 'Running', zone: 'D', agent_type: 'Worker' },
        ];
      }
    } catch {
      instances = [
        { sandbox_id: crypto.randomUUID(), profile: 'ran-optimizer', state: 'Running', zone: 'A', agent_type: 'Experimenter' },
        { sandbox_id: crypto.randomUUID(), profile: 'data-collector', state: 'Running', zone: 'C', agent_type: 'Worker' },
        { sandbox_id: crypto.randomUUID(), profile: 'hypothesis-generator', state: 'Provisioning', zone: 'A', agent_type: 'Researcher' },
      ];
    }
    loading = false;
  }

  async function spawnSandbox() {
    spawning = true;
    try {
      await callTool('rlmx_sandbox_spawn', { profile: selectedProfile });
      await loadInstances();
    } catch { /* ignore */ }
    spawning = false;
  }

  async function terminateInstance(id: string) {
    try {
      await callTool('rlmx_sandbox_terminate', { sandbox_id: id });
      await loadInstances();
    } catch { /* ignore */ }
  }

  async function deployFleet() {
    if (!fleetJson.trim()) return;
    deploying = true;
    try {
      const manifest = JSON.parse(fleetJson);
      await callTool('rlmx_fleet_deploy', manifest);
      fleetJson = '';
      await loadInstances();
    } catch { /* ignore */ }
    deploying = false;
  }

  function stateColor(state: string): string {
    switch (state) {
      case 'Running': return 'text-mint';
      case 'Provisioning': return 'text-ember';
      case 'Starting': return 'text-frost';
      case 'Suspended': return 'text-violet';
      case 'Stopping': return 'text-copper';
      case 'Terminated': return 'text-smoke';
      case 'Failed': return 'text-molten';
      default: return 'text-chalk';
    }
  }

  function networkBadge(policy: string): { text: string; cls: string } {
    switch (policy) {
      case 'Isolated': return { text: 'ISOLATED', cls: 'bg-molten/10 text-molten border-molten/20' };
      case 'EgressOnly': return { text: 'EGRESS', cls: 'bg-ember/10 text-ember border-ember/20' };
      case 'ClusterOnly': return { text: 'CLUSTER', cls: 'bg-frost/10 text-frost border-frost/20' };
      case 'Open': return { text: 'OPEN', cls: 'bg-mint/10 text-mint border-mint/20' };
      default: return { text: policy, cls: 'bg-iron text-chalk border-steel' };
    }
  }

  function gpuBadge(gpu: string): string {
    if (gpu === 'Metal') return 'text-frost';
    if (gpu.startsWith('Cuda')) return 'text-mint';
    if (gpu === 'WebGpu') return 'text-violet';
    return 'text-smoke';
  }

  $effect(() => { loadInstances(); });
</script>

<div class="space-y-6">
  <!-- Metrics -->
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-5">
    <MetricCard label="Total Sandboxes" value={totalInstances} color="frost" delay={0} />
    <MetricCard label="Running" value={runningCount} color="mint" delay={40} />
    <MetricCard label="Provisioning" value={provisioningCount} color="ember" delay={80} />
    <MetricCard label="Profiles" value={profiles.length} color="bone" delay={120} />
    <MetricCard label="MCP Tools" value="28" color="ember" delay={160} subtitle="5 sandbox" />
  </div>

  <!-- Spawn panel -->
  <ForgePanel title="Forge Sandbox">
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div class="md:col-span-2">
        <label for="sb-profile" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Profile</label>
        <select
          id="sb-profile"
          bind:value={selectedProfile}
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember"
        >
          {#each profiles as p}
            <option value={p.name}>{p.name} — {p.agent_type} (Zone {p.zone})</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="sb-zone" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Zone Override</label>
        <input
          id="sb-zone"
          bind:value={zoneOverride}
          type="text"
          placeholder="(from profile)"
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke"
        />
      </div>
      <div class="flex items-end">
        <button
          onclick={spawnSandbox}
          disabled={spawning}
          class="w-full bg-ember text-void font-display font-bold text-sm px-4 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
        >
          {spawning ? 'Forging...' : 'Forge Sandbox'}
        </button>
      </div>
    </div>
  </ForgePanel>

  <!-- Running instances -->
  <ForgePanel title="Running Instances">
    {#snippet actions()}
      <button
        onclick={loadInstances}
        disabled={loading}
        class="bg-ember text-void font-mono text-xs px-4 py-1.5 rounded-none hover:bg-copper transition-colors disabled:opacity-40"
      >
        {loading ? 'Loading...' : 'Refresh'}
      </button>
    {/snippet}

    {#if instances.length === 0 && !loading}
      <p class="font-mono text-sm text-smoke text-center py-8">No sandbox instances. Forge one above.</p>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-left">
          <thead>
            <tr class="border-b border-steel/60">
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">ID</th>
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Profile</th>
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">State</th>
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Zone</th>
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Agent Type</th>
              <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3 text-right">Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each instances as inst, i}
              <tr class="border-b border-steel/30 animate-slide-up" style="animation-delay: {i * 30}ms;">
                <td class="font-mono text-xs text-chalk py-2.5 px-3">{(inst.sandbox_id ?? '').substring(0, 8)}...</td>
                <td class="font-display text-sm text-bone py-2.5 px-3">{inst.profile ?? '--'}</td>
                <td class="font-mono text-xs py-2.5 px-3">
                  <span class="inline-flex items-center gap-1.5 {stateColor(inst.state)}">
                    <span class="w-1.5 h-1.5 rounded-full {inst.state === 'Running' ? 'bg-mint animate-pulse' : inst.state === 'Provisioning' ? 'bg-ember animate-pulse' : 'bg-smoke'}"></span>
                    {inst.state}
                  </span>
                </td>
                <td class="font-mono text-xs text-chalk py-2.5 px-3">Zone {inst.zone ?? '?'}</td>
                <td class="font-mono text-xs text-chalk py-2.5 px-3">{inst.agent_type ?? 'unknown'}</td>
                <td class="py-2.5 px-3 text-right">
                  {#if inst.state !== 'Terminated' && inst.state !== 'Failed'}
                    <button
                      onclick={() => terminateInstance(inst.sandbox_id)}
                      class="border border-molten/30 text-molten font-mono text-[10px] uppercase px-3 py-1 rounded-none hover:bg-molten/10 transition-colors"
                    >
                      Terminate
                    </button>
                  {:else}
                    <span class="font-mono text-[10px] text-smoke">--</span>
                  {/if}
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </ForgePanel>

  <!-- Registered profiles -->
  <ForgePanel title="Registered Profiles (ADR-011)">
    <div class="overflow-x-auto">
      <table class="w-full text-left">
        <thead>
          <tr class="border-b border-steel/60">
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Profile</th>
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Agent Type</th>
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Zone</th>
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">GPU</th>
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Network</th>
            <th class="font-mono text-[9px] uppercase tracking-wider text-smoke py-2 px-3">Model Tier</th>
          </tr>
        </thead>
        <tbody>
          {#each profiles as p, i}
            <tr class="border-b border-steel/30 animate-slide-up" style="animation-delay: {i * 25}ms;">
              <td class="font-display font-semibold text-sm text-ember py-2.5 px-3">{p.name}</td>
              <td class="font-mono text-xs text-bone py-2.5 px-3">{p.agent_type}</td>
              <td class="font-mono text-xs text-chalk py-2.5 px-3">Zone {p.zone}</td>
              <td class="font-mono text-xs py-2.5 px-3">
                <span class={gpuBadge(p.gpu)}>{p.gpu}</span>
              </td>
              <td class="py-2.5 px-3">
                <span class="inline-block font-mono text-[9px] uppercase tracking-wider px-2 py-0.5 border {networkBadge(p.network).cls}">
                  {networkBadge(p.network).text}
                </span>
              </td>
              <td class="font-mono text-xs text-chalk py-2.5 px-3">{p.model_tier}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  </ForgePanel>

  <!-- Fleet Deploy -->
  <ForgePanel title="Fleet Deploy">
    <div class="space-y-4">
      <div>
        <label for="fleet-json" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Fleet Manifest (JSON)</label>
        <textarea
          id="fleet-json"
          bind:value={fleetJson}
          rows="6"
          placeholder={'{"name": "research-fleet", "sandboxes": [{"profile": "ran-optimizer", "count": 2}]}'}
          class="w-full bg-iron border border-steel text-bone font-mono text-xs px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke resize-y"
        ></textarea>
      </div>
      <button
        onclick={deployFleet}
        disabled={deploying || !fleetJson.trim()}
        class="bg-ember text-void font-display font-bold text-sm px-6 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
      >
        {deploying ? 'Deploying...' : 'Deploy Fleet'}
      </button>
    </div>
  </ForgePanel>
</div>
