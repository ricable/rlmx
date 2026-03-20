<script lang="ts">
  import MetricCard from '../components/MetricCard.svelte';
  import ForgePanel from '../components/ForgePanel.svelte';
  import { callTool, parseToolResult } from '../api/mcp';

  let totalNodes = $state(0);
  let healthyNodes = $state(0);
  let activeAgents = $state(0);
  let consensus = $state('--');
  let uptime = $state('--');
  let mcpTools = $state(23);

  let canvas: HTMLCanvasElement | undefined = $state();
  let animFrame = 0;

  interface ZoneDef {
    id: string;
    label: string;
    color: string;
    cx: number;
    cy: number;
    nodes: number;
  }

  const zones: ZoneDef[] = [
    { id: 'A', label: 'Compute · PBFT', color: '#22d3ee', cx: 0.18, cy: 0.32, nodes: 5 },
    { id: 'B', label: 'Inference · Raft', color: '#a78bfa', cx: 0.50, cy: 0.18, nodes: 11 },
    { id: 'C', label: 'Edge · Gossip', color: '#10b981', cx: 0.82, cy: 0.32, nodes: 6 },
    { id: 'D', label: 'Cloud · Burst', color: '#f59e0b', cx: 0.32, cy: 0.72, nodes: 2 },
    { id: 'BR', label: 'Browser · WASM', color: '#ec4899', cx: 0.68, cy: 0.75, nodes: 1 },
  ];

  const connections: [number, number][] = [
    [0, 1], [1, 2], [0, 3], [2, 4], [3, 4], [1, 3], [1, 4],
  ];

  function drawTopology(time: number) {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);
    const cw = rect.width;
    const ch = rect.height;

    ctx.clearRect(0, 0, cw, ch);

    // Draw animated dashed connections
    for (const [a, b] of connections) {
      const za = zones[a];
      const zb = zones[b];
      const ax = za.cx * cw, ay = za.cy * ch;
      const bx = zb.cx * cw, by = zb.cy * ch;

      // Animated dash offset
      ctx.setLineDash([3, 8]);
      ctx.lineDashOffset = -time * 0.015;
      ctx.lineWidth = 1;
      ctx.strokeStyle = 'rgba(34,34,34,0.9)';

      // Curved connection
      const mx = (ax + bx) / 2;
      const my = (ay + by) / 2 - 15;
      ctx.beginPath();
      ctx.moveTo(ax, ay);
      ctx.quadraticCurveTo(mx, my, bx, by);
      ctx.stroke();

      // Data flow particles along the connection
      const t = ((time * 0.0008 + a * 0.3 + b * 0.17) % 1);
      const px = ax + (bx - ax) * t;
      const py = ay + (by - ay) * t - Math.sin(t * Math.PI) * 15;
      ctx.fillStyle = za.color + '40';
      ctx.beginPath();
      ctx.arc(px, py, 1.5, 0, Math.PI * 2);
      ctx.fill();
    }
    ctx.setLineDash([]);

    // Draw zones
    for (const zone of zones) {
      const x = zone.cx * cw;
      const y = zone.cy * ch;

      // Breathing zone glow
      const glowIntensity = 0.06 + Math.sin(time * 0.002 + zones.indexOf(zone)) * 0.03;
      const gradient = ctx.createRadialGradient(x, y, 0, x, y, 45);
      gradient.addColorStop(0, zone.color + Math.round(glowIntensity * 255).toString(16).padStart(2, '0'));
      gradient.addColorStop(1, 'transparent');
      ctx.fillStyle = gradient;
      ctx.beginPath();
      ctx.arc(x, y, 45, 0, Math.PI * 2);
      ctx.fill();

      // Center ring
      ctx.strokeStyle = zone.color + '50';
      ctx.lineWidth = 1;
      ctx.beginPath();
      ctx.arc(x, y, 12, 0, Math.PI * 2);
      ctx.stroke();
      ctx.fillStyle = zone.color + '10';
      ctx.fill();

      // Pulsing node dots
      for (let i = 0; i < Math.min(zone.nodes, 12); i++) {
        const angle = (i / zone.nodes) * Math.PI * 2 - Math.PI / 2;
        const r = 22 + (i % 2) * 6;
        const nx = x + Math.cos(angle) * r;
        const ny = y + Math.sin(angle) * r;
        const pulse = 2.5 + Math.sin(time * 0.003 + i * 0.5) * 0.8;
        ctx.fillStyle = zone.color + 'cc';
        ctx.beginPath();
        ctx.arc(nx, ny, pulse, 0, Math.PI * 2);
        ctx.fill();
        // Outer ring
        ctx.strokeStyle = zone.color + '25';
        ctx.lineWidth = 0.5;
        ctx.beginPath();
        ctx.arc(nx, ny, pulse + 2, 0, Math.PI * 2);
        ctx.stroke();
      }

      // Zone label
      ctx.fillStyle = zone.color;
      ctx.font = "600 11px 'Syne', sans-serif";
      ctx.textAlign = 'center';
      ctx.fillText(zone.id, x, y + 55);

      ctx.fillStyle = '#525252';
      ctx.font = "10px 'IBM Plex Mono', monospace";
      ctx.fillText(zone.label, x, y + 68);

      ctx.fillStyle = '#404040';
      ctx.fillText(`${zone.nodes} nodes`, x, y + 80);
    }

    animFrame = requestAnimationFrame(drawTopology);
  }

  $effect(() => {
    callTool('rlmx_swarm_status').then((res) => {
      const data = parseToolResult<Record<string, any>>(res);
      if (data) {
        totalNodes = data.total_nodes ?? data.node_count ?? data.nodes ?? 0;
        healthyNodes = data.healthy ?? data.healthy_nodes ?? totalNodes;
        consensus = data.consensus ?? 'raft';
        uptime = data.uptime_secs ? data.uptime_secs + 's' : (data.uptime ?? '--');
      }
    }).catch(() => {});

    callTool('rlmx_agent_list').then((res) => {
      const data = parseToolResult<any>(res);
      if (data) {
        activeAgents = Array.isArray(data) ? data.length : (data.agents?.length ?? data.total ?? 0);
      }
    }).catch(() => {});
  });

  $effect(() => {
    if (canvas) {
      animFrame = requestAnimationFrame(drawTopology);
      return () => cancelAnimationFrame(animFrame);
    }
  });
</script>

<div class="space-y-6">
  <!-- Metric cards grid with staggered entrance -->
  <div class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-6">
    <MetricCard label="Total Nodes" value={totalNodes} color="frost" delay={0} />
    <MetricCard label="Healthy" value={healthyNodes} color="mint" delay={40} />
    <MetricCard label="Active Agents" value={activeAgents} color="ember" delay={80} />
    <MetricCard label="Consensus" value={consensus} color="bone" delay={120} />
    <MetricCard label="Uptime" value={uptime} color="frost" delay={160} />
    <MetricCard label="MCP Tools" value={mcpTools} color="ember" delay={200} />
  </div>

  <!-- Animated topology map -->
  <ForgePanel title="Topology Map">
    <div class="relative">
      <canvas
        bind:this={canvas}
        class="w-full bg-obsidian"
        style="height: 320px;"
      ></canvas>
    </div>

    <!-- Zone legend -->
    <div class="flex items-center gap-5 mt-4 flex-wrap">
      {#each zones as zone, i}
        <div class="flex items-center gap-2 animate-slide-up" style="animation-delay: {i * 40}ms;">
          <div class="w-2 h-2" style="background: {zone.color}; box-shadow: 0 0 6px {zone.color}40;"></div>
          <span class="font-mono text-[9px] text-smoke">{zone.id}/{zone.label.split('·')[0].trim()}</span>
        </div>
      {/each}
    </div>
  </ForgePanel>
</div>
