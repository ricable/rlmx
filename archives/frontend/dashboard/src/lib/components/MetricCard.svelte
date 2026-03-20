<script lang="ts">
  interface Props {
    label: string;
    value: string | number;
    color?: 'ember' | 'frost' | 'mint' | 'bone';
    subtitle?: string;
    delay?: number;
  }

  let { label, value, color = 'ember', subtitle = '', delay = 0 }: Props = $props();

  const colorMap: Record<string, string> = {
    ember: 'text-ember',
    frost: 'text-frost',
    mint: 'text-mint',
    bone: 'text-bone',
  };

  const glowColor: Record<string, string> = {
    ember: '245,158,11',
    frost: '56,189,248',
    mint: '16,185,129',
    bone: '229,229,229',
  };
</script>

<div
  class="forge-card group relative bg-forge border border-steel p-4 animate-slide-up"
  style="animation-delay: {delay}ms;"
>
  <!-- Corner accent -->
  <div class="absolute top-0 left-0 w-8 h-[1px] bg-gradient-to-r transition-opacity duration-500 opacity-0 group-hover:opacity-100"
    style="background: linear-gradient(90deg, rgba({glowColor[color]},0.6), transparent);"></div>
  <div class="absolute top-0 left-0 w-[1px] h-6 bg-gradient-to-b transition-opacity duration-500 opacity-0 group-hover:opacity-100"
    style="background: linear-gradient(180deg, rgba({glowColor[color]},0.4), transparent);"></div>

  <!-- Glow leak -->
  <div
    class="absolute -top-4 -left-4 w-24 h-24 opacity-0 group-hover:opacity-100 transition-opacity duration-500 pointer-events-none blur-xl"
    style="background: rgba({glowColor[color]},0.04);"
  ></div>

  <div class="relative z-10">
    <p class="font-mono text-[9px] uppercase tracking-[0.18em] text-smoke mb-3 font-medium">{label}</p>
    <p class="font-display font-bold text-[1.7rem] leading-none {colorMap[color]} tabular-nums">{value}</p>
    {#if subtitle}
      <p class="font-mono text-[9px] text-soot mt-2">{subtitle}</p>
    {/if}
  </div>
</div>
