<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { outputPanel, clearOutput, hidePanel, type OutputItem } from './outputPanelStore';
  import { onDestroy } from 'svelte';

  let state: { visible: boolean; items: OutputItem[] } = { visible: false, items: [] };
  const unsub = outputPanel.subscribe(v => state = v);
  onDestroy(unsub);

  function kindBadge(kind: string) {
    switch (kind) {
      case 'ai': return '🤖';
      case 'shell': return '⌘';
      case 'file': return '📄';
      case 'system': return '🖥';
      case 'error': return '❌';
      default: return 'ℹ️';
    }
  }
</script>

{#if state.visible}
  <div class="fixed bottom-4 right-4 z-[9998] pointer-events-none">
    <!-- Use auto width with a sane max to avoid imposing fixed width constraints -->
    <div class="pointer-events-auto w-auto max-w-[85vw] bg-black/90 text-white border border-gray-800 rounded-lg shadow-xl overflow-hidden"
         in:fly={{ y: 12, duration: 150 }} out:fade={{ duration: 120 }}>
      <div class="flex items-center justify-between px-3 py-2 border-b border-gray-800">
        <div class="text-xs text-gray-400">Output</div>
        <div class="flex items-center gap-2">
          <button class="text-xs text-gray-400 hover:text-white" on:click={() => clearOutput()} title="Clear">Clear</button>
          <button class="text-xs text-gray-400 hover:text-white" on:click={hidePanel} title="Hide">✕</button>
        </div>
      </div>
      <div class="max-h-[40vh] overflow-y-auto">
        {#each state.items as item (item.id)}
          <div class="px-3 py-2 border-b border-gray-900/60">
            <div class="flex items-center gap-2 mb-1 text-xs text-gray-400">
              <span>{kindBadge(item.kind)}</span>
              {#if item.title}<span class="truncate">{item.title}</span>{/if}
              <span class="ml-auto">{new Date(item.timestamp).toLocaleTimeString()}</span>
            </div>
            <pre class="whitespace-pre-wrap text-xs leading-relaxed text-gray-100 font-mono">{item.content}</pre>
          </div>
        {/each}
        {#if state.items.length === 0}
          <div class="px-3 py-4 text-xs text-gray-400">No output</div>
        {/if}
      </div>
    </div>
  </div>
{/if}
