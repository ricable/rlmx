<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { logs, type LogEntry } from '../api/mcp';

  // Re-read logs reactively on each render
  let displayLogs = $derived(logs.slice(0, 100));
</script>

<ForgePanel title="Request Log">
  <div class="overflow-x-auto">
    <table class="w-full font-mono text-xs">
      <thead>
        <tr class="text-left">
          <th class="p-2 text-chalk border-b border-steel">Time</th>
          <th class="p-2 text-chalk border-b border-steel">Method</th>
          <th class="p-2 text-chalk border-b border-steel">Status</th>
        </tr>
      </thead>
      <tbody>
        {#if displayLogs.length === 0}
          <tr>
            <td colspan="3" class="p-4 text-smoke text-center">No requests yet.</td>
          </tr>
        {:else}
          {#each displayLogs as entry}
            <tr class="hover:bg-iron/50 transition-colors">
              <td class="p-2 text-smoke border-b border-steel/30 whitespace-nowrap">{entry.ts}</td>
              <td class="p-2 text-ember border-b border-steel/30">{entry.method}</td>
              <td class="p-2 border-b border-steel/30">
                {#if entry.response?.error}
                  <span class="text-molten">ERR {entry.response.error.code}</span>
                {:else if entry.response}
                  <span class="text-mint">OK</span>
                {:else}
                  <span class="text-smoke">...</span>
                {/if}
              </td>
            </tr>
          {/each}
        {/if}
      </tbody>
    </table>
  </div>
</ForgePanel>
