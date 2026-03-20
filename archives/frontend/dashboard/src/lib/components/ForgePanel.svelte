<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    title: string;
    children: Snippet;
    actions?: Snippet;
  }

  let { title, children, actions }: Props = $props();
</script>

<div class="forge-card bg-forge border border-steel animate-slide-up">
  <!-- Title bar -->
  <div class="flex items-center justify-between px-5 py-3 border-b border-steel/60">
    <div class="flex items-center gap-2.5">
      <div class="w-1 h-3.5 bg-ember/60"></div>
      <h3 class="font-display font-semibold text-[13px] text-bone uppercase tracking-[0.08em]">{title}</h3>
    </div>
    {#if actions}
      <div class="flex items-center gap-2">
        {@render actions()}
      </div>
    {/if}
  </div>

  <!-- Body -->
  <div class="p-5">
    {@render children()}
  </div>
</div>
