<script lang="ts">
  import ForgePanel from '../components/ForgePanel.svelte';
  import { rpc } from '../api/mcp';

  let method = $state('');
  let params = $state('{}');
  let sending = $state(false);
  let result = $state('');

  async function send() {
    if (!method.trim()) return;
    sending = true;
    result = '';
    try {
      let parsed: Record<string, unknown> = {};
      try { parsed = JSON.parse(params); } catch { /* use empty */ }
      const res = await rpc(method.trim(), parsed);
      result = JSON.stringify(res, null, 2);
    } catch (e) {
      result = `Error: ${e instanceof Error ? e.message : String(e)}`;
    }
    sending = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && e.metaKey) send();
  }
</script>

<div class="space-y-6">
  <ForgePanel title="JSON-RPC Console">
    <div class="space-y-4">
      <div>
        <label for="rpc-method" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Method</label>
        <input
          id="rpc-method"
          bind:value={method}
          type="text"
          placeholder="tools/list, tools/call, initialize..."
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none focus:outline-none focus:border-ember placeholder:text-smoke"
        />
      </div>
      <div>
        <label for="rpc-params" class="block font-mono text-[10px] uppercase tracking-wide text-chalk mb-1">Params (JSON)</label>
        <textarea
          id="rpc-params"
          bind:value={params}
          onkeydown={handleKeydown}
          rows="5"
          class="w-full bg-iron border border-steel text-bone font-mono text-sm px-3 py-2 rounded-none resize-y focus:outline-none focus:border-ember"
        ></textarea>
      </div>
      <div class="flex items-center gap-3">
        <button
          onclick={send}
          disabled={sending || !method.trim()}
          class="bg-ember text-void font-display font-bold text-sm px-6 py-2 rounded-none hover:bg-copper transition-colors disabled:opacity-40"
        >
          {sending ? 'Sending...' : 'Send'}
        </button>
        <span class="font-mono text-[10px] text-smoke">Cmd+Enter to send</span>
      </div>
    </div>
  </ForgePanel>

  {#if result}
    <ForgePanel title="Response">
      <pre class="font-mono text-xs text-bone whitespace-pre-wrap leading-relaxed max-h-96 overflow-y-auto">{result}</pre>
    </ForgePanel>
  {/if}
</div>
