<script lang="ts">
  import { mcpStatus, wsStatus } from '../stores/connection';
</script>

<header class="h-[52px] bg-void/90 backdrop-blur-sm border-b border-steel/60 flex items-center justify-between px-5">
  <div class="flex items-center gap-3">
    <!-- RX Logo Badge with glow -->
    <div class="relative">
      <div class="bg-ember text-void font-display font-extrabold text-xs px-2.5 py-1 select-none tracking-wider"
        style="box-shadow: 0 0 16px rgba(245,158,11,0.2), 0 0 4px rgba(245,158,11,0.3);">
        RX
      </div>
    </div>
    <span class="font-display font-bold text-bone text-sm tracking-[0.08em]">RLMX</span>
    <div class="h-4 w-px bg-steel/50"></div>
    <span class="font-mono text-[9px] text-smoke uppercase tracking-[0.2em]">
      Obsidian Forge
    </span>
  </div>

  <div class="flex items-center gap-5">
    <!-- WebSocket status -->
    <div class="flex items-center gap-2">
      <div class="relative">
        <div
          class="w-[6px] h-[6px] rounded-full transition-colors duration-300
            {$wsStatus === 'connected' ? 'bg-mint' : 'bg-molten'}"
        ></div>
        {#if $wsStatus === 'connected'}
          <div class="absolute inset-0 w-[6px] h-[6px] rounded-full bg-mint animate-ping opacity-40"></div>
        {/if}
      </div>
      <span class="font-mono text-[9px] text-smoke uppercase tracking-wider">ws</span>
    </div>

    <!-- MCP status chip -->
    <div class="flex items-center gap-2 border border-steel/50 px-3 py-1.5 transition-colors duration-300
      {$mcpStatus === 'connected' ? 'border-mint/20' : ''}">
      <div
        class="w-[5px] h-[5px] rounded-full transition-colors duration-300
          {$mcpStatus === 'connected' ? 'bg-mint' : $mcpStatus === 'connecting' ? 'bg-ember animate-pulse' : 'bg-molten'}"
      ></div>
      <span class="font-mono text-[9px] uppercase tracking-wider
        {$mcpStatus === 'connected' ? 'text-mint/80' : 'text-smoke'}">
        {$mcpStatus}
      </span>
    </div>
  </div>
</header>
