<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { callTool, parseToolResult } from '../api/mcp';

  let query = $state('');
  let searching = $state(false);
  let results = $state<any[]>([]);

  async function search() {
    if (!query.trim()) return;
    searching = true;
    results = [];
    try {
      const res = await callTool('rlmx_query', { query: query.trim() });
      const data = parseToolResult<any>(res);
      if (Array.isArray(data)) {
        results = data;
      } else if (data?.results) {
        results = data.results;
      } else if (data) {
        results = [data];
      }
    } catch { /* ignore */ }
    searching = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') search();
  }
</script>

<div class="space-y-6">
  <ForgePanel title="Memory Search">
    <div class="flex gap-3">
      <input
        bind:value={query}
        onkeydown={handleKeydown}
        type="text"
        placeholder="Search vector memory..."
        class="flex-1 bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke"
      />
      <button
        onclick={search}
        disabled={searching || !query.trim()}
        class="bg-ember text-void font-display font-bold text-sm px-6 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40"
      >
        {searching ? 'Searching...' : 'Search'}
      </button>
    </div>
  </ForgePanel>

  {#if results.length > 0}
    <div class="space-y-3">
      {#each results as result, i}
        <div class="bg-forge border border-steel rounded-none p-4 space-y-2">
          <div class="flex items-center gap-3">
            {#if result.score !== undefined}
              <span class="font-mono text-[10px] text-ember bg-ember/10 border border-ember/20 px-2 py-0.5">
                {(result.score * 100).toFixed(1)}%
              </span>
            {/if}
            {#if result.tier}
              <span class="font-mono text-[10px] text-frost bg-frost/10 border border-frost/20 px-2 py-0.5">
                {result.tier}
              </span>
            {/if}
            {#if result.segment_id}
              <span class="font-mono text-[10px] text-chalk">
                seg:{result.segment_id}
              </span>
            {/if}
          </div>
          <p class="font-mono text-sm text-bone leading-relaxed">
            {result.content ?? result.text ?? JSON.stringify(result)}
          </p>
        </div>
      {/each}
    </div>
  {:else if !searching && query}
    <p class="font-mono text-sm text-smoke text-center py-8">No results found.</p>
  {/if}
</div>
