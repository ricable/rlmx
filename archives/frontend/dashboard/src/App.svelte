<script lang="ts">
  import { connect } from './lib/api/mcp';
  import { connectWS, onEvent } from './lib/api/websocket';
  import { mcpStatus, wsStatus } from './lib/stores/connection';
  import { activeSection, activeView, currentSection } from './lib/stores/navigation';

  import Rail from './lib/components/Rail.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import Header from './lib/components/Header.svelte';

  import SwarmOverview from './lib/views/SwarmOverview.svelte';
  import AgentManager from './lib/views/AgentManager.svelte';
  import PermissionMatrix from './lib/views/PermissionMatrix.svelte';
  import ComputePool from './lib/views/ComputePool.svelte';
  import MemoryQuery from './lib/views/MemoryQuery.svelte';
  import EdgeInference from './lib/views/EdgeInference.svelte';
  import EventStream from './lib/views/EventStream.svelte';
  import ToolRegistry from './lib/views/ToolRegistry.svelte';
  import RpcConsole from './lib/views/RpcConsole.svelte';
  import RequestLog from './lib/views/RequestLog.svelte';
  import SandboxManager from './lib/views/SandboxManager.svelte';

  // Unique key for view transitions
  let viewKey = $derived(`${$activeSection}.${$activeView}`);

  $effect(() => {
    mcpStatus.set('connecting');
    connect().then((ok) => {
      mcpStatus.set(ok ? 'connected' : 'disconnected');
    });

    connectWS();
    const unsub = onEvent((e) => {
      if (e.type === '__connected') wsStatus.set('connected');
      else if (e.type === '__disconnected') wsStatus.set('disconnected');
    });

    return () => { unsub(); };
  });
</script>

<div class="fixed inset-0 bg-void overflow-hidden">
  <!-- Animated dot-grid background -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="
      background-image: radial-gradient(circle, rgba(245,158,11,0.12) 0.5px, transparent 0.5px);
      background-size: 24px 24px;
      opacity: 0.06;
      animation: grid-drift 8s linear infinite;
    "
  ></div>

  <!-- Forge heat — bottom glow -->
  <div
    class="absolute bottom-0 left-0 right-0 h-80 pointer-events-none"
    style="
      background: radial-gradient(ellipse at 50% 100%, rgba(245,158,11,0.06) 0%, transparent 70%);
      animation: ember-breathe 6s ease-in-out infinite;
    "
  ></div>

  <!-- Scan line effect -->
  <div
    class="absolute left-0 right-0 h-px pointer-events-none opacity-[0.03]"
    style="
      background: linear-gradient(90deg, transparent, rgba(245,158,11,0.6), transparent);
      animation: scan-line 12s linear infinite;
    "
  ></div>

  <div class="relative z-10 grid h-screen" style="grid-template-columns: 56px 220px 1fr; grid-template-rows: 52px 1fr;">
    <!-- Header spans full width -->
    <div class="col-span-3">
      <Header />
    </div>

    <!-- Icon Rail -->
    <Rail />

    <!-- Sidebar -->
    <Sidebar />

    <!-- Content Area -->
    <div class="flex flex-col overflow-hidden">
      <!-- Content header with view title -->
      <div class="shrink-0 px-6 pt-5 pb-4 border-b border-steel/50 bg-obsidian">
        <div class="flex items-center gap-3">
          <div class="w-1.5 h-6 bg-ember rounded-none"></div>
          <h1 class="font-display font-bold text-lg text-bone tracking-wide">
            {$currentSection?.nav.find(n => n.id === $activeView)?.label ?? $currentSection?.title ?? ''}
          </h1>
        </div>
        <div class="mt-1.5 ml-5 flex items-center gap-1.5 font-mono text-[10px] text-smoke">
          <span>rlmx</span>
          <span class="text-ember">&#x203A;</span>
          <span>{$activeSection}</span>
          <span class="text-ember">&#x203A;</span>
          <span class="text-chalk">{$activeView}</span>
        </div>
      </div>

      <!-- Scrollable content with view transition -->
      <main class="flex-1 overflow-y-auto bg-obsidian p-6">
        {#key viewKey}
          <div class="view-enter">
            {#if $activeSection === 'swarm'}
              <SwarmOverview />
            {:else if $activeSection === 'agents' && $activeView === 'matrix'}
              <PermissionMatrix />
            {:else if $activeSection === 'agents' && $activeView === 'hierarchy'}
              <div class="animate-slide-up">
                <div class="bg-forge border border-steel p-6">
                  <h3 class="font-display font-semibold text-sm text-bone uppercase tracking-wide mb-4">Spawn Hierarchy (DDD-003)</h3>
                  <pre class="font-mono text-xs text-chalk leading-7 whitespace-pre"><span class="text-ember font-bold">Coordinator</span> (PID 0, All permissions, Raft leader)
├── <span class="text-violet">Router</span> (1/zone, spawns Workers)
│   └── <span class="text-bone">Worker</span> (pool 1-N, auto-terminate 5min idle)
├── <span class="text-mint">Monitor</span> (1/zone, watchdog, never terminates)
├── <span class="text-copper">Replicator</span> (1/zone-pair, continuous sync)
├── <span class="text-bone">Embedder</span> (1/compute-zone, always running)
├── <span class="text-smoke">Validator</span> (periodic or on-demand)
├── <span class="text-smoke">Reviewer</span> (on-demand, max 2)
├── <span class="text-smoke">Analyst</span> (on-demand, max 2)
├── <span class="text-pink">Researcher</span> (on objective, max 3)
│   └── <span class="text-pink">Experimenter</span> (per hypothesis, max 8 total)
│       └── <span class="text-smoke">sub-Experimenter</span> (grid search depth ≤2)
└── <span class="text-copper">Trainer</span> (on job, max 1/GPU-zone)</pre>
                </div>
              </div>
            {:else if $activeSection === 'agents'}
              <AgentManager />
            {:else if $activeSection === 'compute'}
              <ComputePool />
            {:else if $activeSection === 'memory'}
              <MemoryQuery />
            {:else if $activeSection === 'edge'}
              <EdgeInference />
            {:else if $activeSection === 'research'}
              <div class="space-y-6 animate-slide-up">
                <div class="bg-forge border border-steel p-6">
                  <h3 class="font-display font-semibold text-sm text-bone uppercase tracking-wide mb-4">Evolutionary Pipeline (ADR-006)</h3>
                  <pre class="font-mono text-xs text-chalk leading-7 whitespace-pre">
1. <span class="text-ember">Coordinator</span> receives research objective → spawns <span class="text-pink">Researcher</span>
2. <span class="text-pink">Researcher</span> queries prior patterns (VecSearch + SONA) → N hypotheses
3. Spawns N <span class="text-violet">Experimenters</span> (COW-branched state via rlmx_rvf_branch)
4. Experimenters mutate using local ruvltra model (MutationStrategy genome)
5. Request <span class="text-copper">Trainer</span> spawn (MLX on Mac, CUDA on NUC-GPU)
6. Trainer runs 5-min experiment → reports val_bpb
7. <span class="text-mint">Cross-pollination</span> via Gossip: successful mutations propagate
8. Cloud escalation if swarm stalls → SkyPilot GPU VMs
9. Researcher synthesizes → <span class="text-bone">Reviewer</span> validates
10. Coordinator integrates winner → <span class="text-copper">Replicator</span> syncs</pre>
                </div>
              </div>
            {:else if $activeSection === 'sandbox'}
              <SandboxManager />
            {:else if $activeSection === 'events'}
              <EventStream />
            {:else if $activeSection === 'tools'}
              <ToolRegistry />
            {:else if $activeSection === 'rpc'}
              <RpcConsole />
            {:else if $activeSection === 'log'}
              <RequestLog />
            {/if}
          </div>
        {/key}
      </main>
    </div>
  </div>
</div>
