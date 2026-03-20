<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import MetricCard from '../components/MetricCard.svelte';
  import { detectCapabilities, type EdgeCapabilities } from '../api/wasm';

  let caps = $state<EdgeCapabilities | null>(null);
  let joined = $state(false);
  let activeCells = $state<boolean[]>(Array(24).fill(false));

  $effect(() => {
    detectCapabilities().then(c => { caps = c; });
  });

  function toggleJoin() {
    joined = !joined;
    if (joined) {
      // Simulate active cells ramping up
      for (let i = 0; i < 24; i++) {
        setTimeout(() => {
          activeCells[i] = Math.random() > 0.4;
          activeCells = [...activeCells];
        }, i * 60);
      }
    } else {
      activeCells = Array(24).fill(false);
    }
  }
</script>

<div class="space-y-6">
  <!-- Capability detection -->
  {#if caps}
    <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
      <MetricCard label="WASM" value={caps.wasm ? 'YES' : 'NO'} color={caps.wasm ? 'mint' : 'bone'} />
      <MetricCard label="SIMD" value={caps.simd ? 'YES' : 'NO'} color={caps.simd ? 'mint' : 'bone'} />
      <MetricCard label="WebGPU" value={caps.webgpu ? 'YES' : 'NO'} color={caps.webgpu ? 'frost' : 'bone'} />
      <MetricCard label="Workers" value={caps.workers ? 'YES' : 'NO'} color={caps.workers ? 'mint' : 'bone'} />
      <MetricCard label="Mode" value={caps.mode} color="ember" />
    </div>
  {/if}

  <ForgePanel title="Browser Compute Pool">
    {#snippet actions()}
      <button
        onclick={toggleJoin}
        class="font-mono text-xs px-4 py-1.5 rounded-none transition-colors
          {joined
            ? 'bg-molten/10 border border-molten/30 text-molten hover:bg-molten/20'
            : 'bg-ember text-void hover:bg-copper'}"
      >
        {joined ? 'Leave Forge' : 'Join Forge'}
      </button>
    {/snippet}

    <div class="grid grid-cols-6 gap-2">
      {#each activeCells as active, i}
        <div
          class="h-10 border transition-all duration-300
            {active
              ? 'bg-ember/20 border-ember/40'
              : 'bg-iron border-steel'}"
          style={active ? 'animation: pulse-ember 2s infinite; animation-delay: ' + (i * 83) + 'ms;' : ''}
        >
          {#if active}
            <div class="w-full h-full flex items-center justify-center">
              <div class="w-1.5 h-1.5 bg-ember rounded-none"></div>
            </div>
          {/if}
        </div>
      {/each}
    </div>
    <p class="font-mono text-[10px] text-smoke mt-3">
      {activeCells.filter(Boolean).length}/24 cells active
    </p>
  </ForgePanel>
</div>
