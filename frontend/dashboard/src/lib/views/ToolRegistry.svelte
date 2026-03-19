<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { rpc } from '../api/mcp';

  interface ToolDef {
    name: string;
    description?: string;
    inputSchema?: Record<string, unknown>;
  }

  let tools = $state<ToolDef[]>([]);
  let loading = $state(false);

  async function loadTools() {
    loading = true;
    try {
      const res = await rpc('tools/list');
      const list = res?.result as any;
      tools = Array.isArray(list?.tools) ? list.tools : (Array.isArray(list) ? list : []);
    } catch { /* ignore */ }
    loading = false;
  }
</script>

<div class="space-y-6">
  <ForgePanel title="MCP Tool Registry">
    {#snippet actions()}
      <button
        onclick={loadTools}
        disabled={loading}
        class="bg-ember text-void font-mono text-xs px-4 py-1.5 rounded-none hover:bg-copper transition-colors disabled:opacity-40"
      >
        {loading ? 'Loading...' : 'Load Tools'}
      </button>
    {/snippet}

    {#if tools.length === 0 && !loading}
      <p class="font-mono text-sm text-smoke text-center py-8">Click "Load Tools" to fetch MCP tool definitions.</p>
    {:else}
      <div class="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4">
        {#each tools as tool}
          <div class="bg-iron border border-steel rounded-none p-4 space-y-2">
            <p class="font-display font-semibold text-sm text-ember">{tool.name}</p>
            <p class="font-mono text-xs text-chalk leading-relaxed">
              {tool.description ?? 'No description available.'}
            </p>
          </div>
        {/each}
      </div>
    {/if}
  </ForgePanel>
</div>
