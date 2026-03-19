<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import MetricCard from '../components/MetricCard.svelte';
  import { detectCapabilities, type EdgeCapabilities } from '../api/wasm';
  import { callTool, extractText } from '../api/mcp';

  let caps = $state<EdgeCapabilities | null>(null);
  let prompt = $state('');
  let maxTokens = $state(128);
  let generating = $state(false);
  let output = $state('');

  $effect(() => {
    detectCapabilities().then(c => { caps = c; });
  });

  async function generate() {
    if (!prompt.trim()) return;
    generating = true;
    output = '';
    try {
      const res = await callTool('rlmx_edge_generate', {
        prompt: prompt.trim(),
        max_tokens: maxTokens,
      });
      output = extractText(res);
    } catch (e) {
      output = `Error: ${e instanceof Error ? e.message : String(e)}`;
    }
    generating = false;
  }
</script>

<div class="space-y-6">
  <!-- Capability cards -->
  {#if caps}
    <div class="grid grid-cols-2 md:grid-cols-5 gap-4">
      <MetricCard label="WASM" value={caps.wasm ? 'Ready' : 'N/A'} color={caps.wasm ? 'mint' : 'bone'} />
      <MetricCard label="SIMD" value={caps.simd ? 'Ready' : 'N/A'} color={caps.simd ? 'mint' : 'bone'} />
      <MetricCard label="WebGPU" value={caps.webgpu ? 'Ready' : 'N/A'} color={caps.webgpu ? 'frost' : 'bone'} />
      <MetricCard label="Workers" value={caps.workers ? 'Ready' : 'N/A'} color={caps.workers ? 'mint' : 'bone'} />
      <MetricCard label="Mode" value={caps.mode} color="ember" />
    </div>
  {/if}

  <!-- Generate panel -->
  <ForgePanel title="Edge Generate">
    <div class="space-y-4">
      <div>
        <label for="edge-prompt" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Prompt</label>
        <textarea
          id="edge-prompt"
          bind:value={prompt}
          rows="4"
          placeholder="Enter prompt for edge inference..."
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none resize-y focus:outline-none focus:border-ember placeholder:text-smoke"
        ></textarea>
      </div>
      <div class="flex items-end gap-4">
        <div>
          <label for="edge-max-tokens" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Max Tokens</label>
          <input
            id="edge-max-tokens"
            bind:value={maxTokens}
            type="number"
            min="1"
            max="2048"
            class="w-28 bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember"
          />
        </div>
        <button
          onclick={generate}
          disabled={generating || !prompt.trim()}
          class="bg-ember text-void font-display font-bold text-sm px-6 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40"
        >
          {generating ? 'Generating...' : 'Generate'}
        </button>
      </div>
    </div>
  </ForgePanel>

  <!-- Output -->
  {#if output}
    <ForgePanel title="Output">
      <pre class="font-mono text-sm text-bone whitespace-pre-wrap leading-relaxed">{output}</pre>
    </ForgePanel>
  {/if}
</div>
