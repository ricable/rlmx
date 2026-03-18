<script lang="ts">
  import { sections, activeSection, navigate, type SectionId } from '../stores/navigation';

  const groups: SectionId[][] = [
    ['swarm', 'agents', 'compute'],
    ['memory', 'edge'],
    ['research', 'sandbox', 'events'],
    ['tools', 'rpc', 'log'],
  ];

  // Distinctive icons per section (Unicode glyphs for character)
  const iconMap: Record<string, string> = {
    swarm: '⬡', agents: '✦', compute: '⚙', memory: '◈',
    edge: '⚡', research: '❋', sandbox: '▣', events: '⇌', tools: '⚒',
    rpc: '⌘', log: '≡',
  };
</script>

<nav class="h-full bg-void border-r border-steel/60 flex flex-col items-center py-3 gap-1 overflow-y-auto">
  {#each groups as group, gi}
    {#each group as sectionId}
      {@const sec = sections.find(s => s.id === sectionId)}
      {@const isActive = $activeSection === sectionId}
      {#if sec}
        <button
          class="relative w-10 h-10 flex items-center justify-center text-base transition-all duration-200
            {isActive
              ? 'text-ember'
              : 'text-smoke hover:text-chalk'}"
          onclick={() => navigate(sec.id)}
          title={sec.title}
        >
          <!-- Active background glow -->
          {#if isActive}
            <div class="absolute inset-0 bg-ember/8 border-l-2 border-l-ember" style="box-shadow: -6px 0 20px rgba(245,158,11,0.12), inset 0 0 12px rgba(245,158,11,0.04);"></div>
          {:else}
            <div class="absolute inset-0 border-l-2 border-l-transparent hover:bg-iron/50 transition-colors duration-200"></div>
          {/if}

          <span class="relative z-10 select-none" style="font-size: 15px;">
            {iconMap[sec.id] ?? sec.icon}
          </span>
        </button>
      {/if}
    {/each}
    {#if gi < groups.length - 1}
      <div class="w-5 border-t border-steel/40 my-2"></div>
    {/if}
  {/each}
</nav>
