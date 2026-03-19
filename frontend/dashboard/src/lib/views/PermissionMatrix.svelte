<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';

  const agentTypes = [
    'coordinator', 'researcher', 'router', 'experimenter', 'worker', 'monitor',
    'reviewer', 'trainer', 'validator', 'replicator', 'embedder', 'analyst',
  ];

  const syscalls = [
    'VecInsert', 'VecSearch', 'VecDelete', 'GraphQuery', 'GraphCut', 'GraphDiffuse',
    'ProcessFork', 'ProcessSend', 'ProcessRecv', 'StateMutate', 'AttentionSelect', 'HaltCheck',
  ];

  // Per ADR-005 capability-secured agents permission matrix
  const matrix: Record<string, boolean[]> = {
    coordinator:  [true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true ],
    researcher:   [true,  true,  false, true,  false, true,  false, true,  true,  false, true,  false],
    router:       [false, true,  false, true,  false, false, true,  true,  true,  false, true,  false],
    experimenter: [true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  false],
    worker:       [true,  true,  false, false, false, false, false, true,  true,  true,  false, false],
    monitor:      [false, true,  false, true,  false, false, false, false, true,  false, true,  true ],
    reviewer:     [false, true,  false, true,  false, true,  false, false, true,  false, true,  false],
    trainer:      [true,  true,  true,  true,  false, true,  false, true,  true,  true,  true,  false],
    validator:    [false, true,  false, true,  false, false, false, false, true,  false, true,  true ],
    replicator:   [true,  true,  true,  true,  true,  false, true,  true,  true,  true,  false, false],
    embedder:     [true,  true,  false, false, false, false, false, true,  true,  false, false, false],
    analyst:      [false, true,  false, true,  true,  true,  false, false, true,  false, true,  false],
  };
</script>

<ForgePanel title="Capability Permission Matrix (ADR-005)">
  <div class="overflow-x-auto">
    <table class="w-full font-mono text-xs">
      <thead>
        <tr>
          <th class="text-left p-2 text-chalk border-b border-steel">Agent</th>
          {#each syscalls as syscall}
            <th class="p-2 border-b border-steel text-chalk">
              <div class="writing-vertical-lr rotate-180 whitespace-nowrap text-[10px]" style="writing-mode: vertical-lr; transform: rotate(180deg);">
                {syscall}
              </div>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each agentTypes as agent}
          <tr class="hover:bg-iron/50 transition-colors">
            <td class="p-2 text-bone border-b border-steel/50 font-semibold">{agent}</td>
            {#each matrix[agent] as allowed}
              <td class="p-2 text-center border-b border-steel/50">
                {#if allowed}
                  <span class="text-ember font-bold">&#10003;</span>
                {:else}
                  <span class="text-smoke">&#10005;</span>
                {/if}
              </td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</ForgePanel>
