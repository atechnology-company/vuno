<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { outputPanel, type OutputItem } from './outputPanelStore';
  import { onDestroy } from 'svelte';
  import { hidePanel, clearOutput } from './outputPanelStore';

  let state: { visible: boolean; items: OutputItem[] } = { visible: false, items: [] };
  const unsub = outputPanel.subscribe(v => state = v);
  onDestroy(unsub);

  // Show latest N items (most recent first)
  $: latest = state.items.slice(0, 3);
  $: show = state.visible && latest.length > 0;
  
  function close() {
    hidePanel();
  }

  function clear() {
    clearOutput();
  }

  function kindBadge(kind: string) {
    switch (kind) {
      case 'ai': return '🤖';
      case 'shell': return '⌘';
      case 'file': return '📄';
      case 'system': return '🖥️';
      case 'error': return '❌';
      default: return 'ℹ️';
    }
  }
</script>

{#if show}
  <div class="w-full bottom-output z-40" in:fly={{ y: 20, duration: 300 }} out:fly={{ y: 10, duration: 200 }}>
    <div class="mx-8 rounded-2xl overflow-hidden bg-black/95 backdrop-blur-2xl border border-white/10 shadow-2xl">
      <div class="flex items-center gap-3 px-5 py-3 border-b border-white/10 bg-white/5">
        <div class="flex-1 text-left text-sm text-white font-medium flex items-center gap-2">
          <span class="text-lg">{kindBadge(latest[0].kind)}</span>
          <span>{latest[0].title || 'Output'}</span>
        </div>
        <div class="flex items-center gap-2">
          <button class="text-sm text-white/60 hover:text-white px-3 py-1.5 rounded-lg hover:bg-white/10 transition-all" on:click={clear}>Clear</button>
          <button class="text-sm text-white/60 hover:text-white px-3 py-1.5 rounded-lg hover:bg-white/10 transition-all" on:click={close}>Close</button>
        </div>
      </div>

      <div class="px-5 py-4 max-h-[240px] overflow-auto text-sm text-white/90 leading-relaxed whitespace-pre-wrap font-sans">
        {#each latest as item (item.id)}
          <div class="mb-4 last:mb-0 p-3 rounded-lg bg-white/5 border border-white/10">
            {#if item.title}
              <div class="text-xs text-white/50 mb-2 flex items-center justify-between font-mono">
                <span>{item.title}</span>
                <span>{new Date(item.timestamp).toLocaleTimeString()}</span>
              </div>
            {/if}
            <div class="whitespace-pre-wrap text-white/95">{item.content}</div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .bottom-output {
    position: fixed;
    left: 0;
    right: 0;
    bottom: 140px; /* Positioned above command bar with spacing */
    pointer-events: auto;
    display: flex;
    justify-content: center;
    padding-bottom: 0;
    z-index: 45;
  }

  /* Refined scrollbar */
  .bottom-output ::-webkit-scrollbar { width: 10px; height: 10px; }
  .bottom-output ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.15); border-radius: 5px; }
  .bottom-output ::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.25); }
  .bottom-output ::-webkit-scrollbar-track { background: transparent; }
</style>
