<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { onEvent, isConnected, type SwarmEvent } from '../api/websocket';

  const MAX_EVENTS = 100;
  let events = $state<SwarmEvent[]>([]);
  let container: HTMLDivElement | undefined = $state();
  let wsLive = $state(false);

  const typeColors: Record<string, string> = {
    __connected: 'bg-mint/15 text-mint border border-mint/20',
    __disconnected: 'bg-molten/10 text-molten/60 border border-molten/10',
    NodeJoined: 'bg-frost/15 text-frost border border-frost/20',
    NodeLeft: 'bg-molten/15 text-molten border border-molten/20',
    AgentSpawned: 'bg-ember/15 text-ember border border-ember/20',
    AgentTerminated: 'bg-molten/15 text-molten border border-molten/20',
    HealthUpdate: 'bg-mint/15 text-mint border border-mint/20',
    ExperimentUpdate: 'bg-copper/15 text-copper border border-copper/20',
    MutationFound: 'bg-pink/15 text-pink border border-pink/20',
    ConsensusReached: 'bg-violet/15 text-violet border border-violet/20',
    QueryRouted: 'bg-frost/15 text-frost border border-frost/20',
    SyscallDispatched: 'bg-acid/15 text-acid border border-acid/20',
  };

  function badgeClass(type: string): string {
    return typeColors[type] ?? 'bg-steel/50 text-chalk border border-steel/30';
  }

  // Filter out noisy internal reconnect events, keep only real ones + first connect/disconnect
  let lastWsState = '';
  function shouldShow(event: SwarmEvent): boolean {
    if (event.type === '__connected') {
      if (lastWsState === 'connected') return false;
      lastWsState = 'connected';
      wsLive = true;
      return true;
    }
    if (event.type === '__disconnected') {
      if (lastWsState === 'disconnected') return false;
      lastWsState = 'disconnected';
      wsLive = false;
      return true;
    }
    return true;
  }

  // Demo event generator — produces realistic swarm events when WS is offline
  const demoAgentTypes = ['coordinator', 'researcher', 'worker', 'router', 'monitor', 'embedder', 'experimenter', 'trainer'];
  const demoZones = ['A', 'B', 'C', 'Cloud', 'Browser'];
  const demoStrategies = ['Rlm', 'Trm', 'Edge', 'Auto', 'Swarm'];
  const demoSyscalls = ['VecSearch', 'VecInsert', 'GraphQuery', 'ProcessFork', 'StateMutate', 'AttentionSelect'];

  function randomPick<T>(arr: T[]): T { return arr[Math.floor(Math.random() * arr.length)]; }
  function randomId(): string { return Math.random().toString(36).substring(2, 10); }
  function now(): string { return new Date().toLocaleTimeString(); }

  function generateDemoEvent(): SwarmEvent {
    const types = [
      () => ({
        type: 'AgentSpawned', ts: now(),
        data: { agent_id: randomId(), agent_type: randomPick(demoAgentTypes), zone: randomPick(demoZones) },
      }),
      () => ({
        type: 'HealthUpdate', ts: now(),
        data: { node_id: randomId(), cpu: +(Math.random() * 80 + 5).toFixed(1), mem_mb: Math.floor(Math.random() * 8000 + 500), zone: randomPick(demoZones) },
      }),
      () => ({
        type: 'QueryRouted', ts: now(),
        data: { strategy: randomPick(demoStrategies), latency_ms: Math.floor(Math.random() * 50 + 2), zone: randomPick(demoZones) },
      }),
      () => ({
        type: 'SyscallDispatched', ts: now(),
        data: { syscall: randomPick(demoSyscalls), process_id: Math.floor(Math.random() * 100) },
      }),
      () => ({
        type: 'ConsensusReached', ts: now(),
        data: { protocol: randomPick(['PBFT', 'Raft', 'Gossip']), epoch: Math.floor(Math.random() * 1000), participants: Math.floor(Math.random() * 15 + 3) },
      }),
      () => ({
        type: 'MutationFound', ts: now(),
        data: { fitness: +(Math.random() * 0.4 + 0.5).toFixed(4), generation: Math.floor(Math.random() * 50 + 1) },
      }),
      () => ({
        type: 'ExperimentUpdate', ts: now(),
        data: { experiment_id: randomId(), status: randomPick(['Running', 'Completed', 'CrossPollinated']), val_bpb: +(Math.random() * 2 + 1).toFixed(3) },
      }),
      () => ({
        type: 'NodeJoined', ts: now(),
        data: { node_id: randomId(), zone: randomPick(demoZones), hw: randomPick(['RPi5', 'NUC', 'Mac-M3', 'RPi4', 'Browser']) },
      }),
    ];
    return randomPick(types)() as SwarmEvent;
  }

  let demoInterval: ReturnType<typeof setInterval> | undefined;

  function startDemo() {
    if (demoInterval) return;
    // Add initial burst
    for (let i = 0; i < 5; i++) {
      events = [generateDemoEvent(), ...events].slice(0, MAX_EVENTS);
    }
    // Then stream at random intervals
    demoInterval = setInterval(() => {
      events = [generateDemoEvent(), ...events].slice(0, MAX_EVENTS);
    }, 1500 + Math.random() * 2000);
  }

  function stopDemo() {
    if (demoInterval) { clearInterval(demoInterval); demoInterval = undefined; }
  }

  $effect(() => {
    const unsub = onEvent((event) => {
      if (shouldShow(event)) {
        events = [event, ...events].slice(0, MAX_EVENTS);
      }
    });
    return () => { unsub(); stopDemo(); };
  });
</script>

<div class="space-y-4">
  <!-- Connection status banner -->
  <div class="flex items-center justify-between bg-forge border border-steel p-4">
    <div class="flex items-center gap-3">
      <div class="relative">
        <div class="w-2.5 h-2.5 rounded-full {wsLive ? 'bg-mint' : 'bg-molten/50'}"></div>
        {#if wsLive}
          <div class="absolute inset-0 w-2.5 h-2.5 rounded-full bg-mint animate-ping opacity-30"></div>
        {/if}
      </div>
      <span class="font-mono text-xs {wsLive ? 'text-mint' : 'text-smoke'}">
        {wsLive ? 'Connected to ws://127.0.0.1:3001' : 'WebSocket offline — ws://127.0.0.1:3001'}
      </span>
    </div>
    <div class="flex gap-2">
      {#if !wsLive}
        <button
          onclick={startDemo}
          class="px-3 py-1.5 text-[10px] font-mono uppercase tracking-wider border border-ember/30 text-ember hover:bg-ember/10 transition-colors"
        >
          {demoInterval ? '● Streaming Demo' : 'Start Demo Stream'}
        </button>
      {/if}
      {#if demoInterval}
        <button
          onclick={stopDemo}
          class="px-3 py-1.5 text-[10px] font-mono uppercase tracking-wider border border-steel text-smoke hover:text-chalk hover:border-chalk/30 transition-colors"
        >
          Stop
        </button>
      {/if}
    </div>
  </div>

  <ForgePanel title="Live Event Stream">
    {#snippet actions()}
      <span class="font-mono text-[9px] text-smoke">{events.length} events</span>
    {/snippet}

    <div bind:this={container} class="max-h-[calc(100vh-320px)] overflow-y-auto">
      {#if events.length === 0}
        <div class="py-12 text-center">
          <p class="font-mono text-xs text-smoke">No events yet</p>
          <p class="font-mono text-[10px] text-soot mt-1">Start the RLMX server or use Demo Stream</p>
        </div>
      {:else}
        {#each events as event, i}
          <div
            class="flex items-start gap-3 py-2.5 px-3 hover:bg-iron/30 transition-colors border-b border-steel/20 animate-slide-in-right"
            style="animation-delay: {Math.min(i * 20, 200)}ms;"
          >
            <span class="font-mono text-[9px] text-soot whitespace-nowrap mt-1 tabular-nums w-16 shrink-0">{event.ts}</span>
            <span class="font-mono text-[9px] px-2 py-0.5 whitespace-nowrap shrink-0 {badgeClass(event.event_type ?? event.type)}">
              {event.event_type ?? event.type}
            </span>
            <span class="font-mono text-[10px] text-chalk/70 flex-1 truncate">
              {#if event.data}
                {#each Object.entries(event.data) as [k, v], j}
                  <span class="text-smoke">{k}:</span><span class="text-chalk ml-0.5">{v}</span>{j < Object.entries(event.data).length - 1 ? '  ' : ''}
                {/each}
              {:else if event.payload}
                {JSON.stringify(event.payload)}
              {:else}
                <span class="text-soot">--</span>
              {/if}
            </span>
          </div>
        {/each}
      {/if}
    </div>
  </ForgePanel>
</div>
