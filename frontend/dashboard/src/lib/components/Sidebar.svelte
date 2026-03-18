<script lang="ts">
  import { currentSection, activeView, navigate } from '../stores/navigation';

  function handleNav(viewId: string) {
    const sec = $currentSection;
    if (sec) navigate(sec.id, viewId);
  }
</script>

<aside class="h-full bg-void/80 border-r border-steel/40 overflow-y-auto">
  <div class="px-5 pt-5 pb-3">
    <h2 class="font-display font-bold text-[11px] uppercase tracking-[0.25em] text-smoke">
      {$currentSection?.title ?? ''}
    </h2>
  </div>

  <nav class="flex flex-col gap-0.5 px-3 pb-4">
    {#each $currentSection?.nav ?? [] as item, i}
      {@const isActive = $activeView === item.id}
      <button
        class="relative w-full text-left px-3 py-2.5 text-[13px] font-mono transition-all duration-200 flex items-center gap-2 group
          {isActive
            ? 'text-ember'
            : 'text-chalk/70 hover:text-chalk'}"
        onclick={() => handleNav(item.id)}
        style="animation: slide-in-right 0.2s ease-out {i * 40}ms both;"
      >
        <!-- Active indicator -->
        <div class="absolute left-0 top-1/2 -translate-y-1/2 w-[2px] h-5 transition-all duration-200
          {isActive ? 'bg-ember opacity-100' : 'bg-transparent opacity-0 group-hover:bg-steel group-hover:opacity-100'}"></div>

        <!-- Active background -->
        {#if isActive}
          <div class="absolute inset-0 bg-ember/[0.04]" style="box-shadow: inset -1px 0 12px rgba(245,158,11,0.03);"></div>
        {/if}

        <span class="relative z-10">{item.label}</span>

        {#if item.badge}
          <span class="relative z-10 ml-auto px-2 py-0.5 text-[8px] font-mono font-semibold bg-ember/15 text-ember tracking-wider">
            {item.badge}
          </span>
        {/if}
      </button>
    {/each}
  </nav>
</aside>

<style>
  @keyframes slide-in-right {
    from { opacity: 0; transform: translateX(-8px); }
    to { opacity: 1; transform: translateX(0); }
  }
</style>
