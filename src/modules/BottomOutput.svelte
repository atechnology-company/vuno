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
  <div class="w-full bottom-output z-40" in:fly={{ y: 8, duration: 220 }} out:fade={{ duration: 140 }}>
    <div class="mx-8 rounded-t-2xl overflow-hidden bg-gradient-to-t from-black/95 via-black/85 to-transparent border border-white/6 backdrop-blur-sm shadow-xl">
      <div class="flex items-center gap-3 px-4 py-2 border-b border-white/6">
        <div class="flex-1 text-left text-sm text-white/90 font-medium">{latest[0].title || kindBadge(latest[0].kind)}</div>
        <div class="flex items-center gap-2">
          <button class="text-xs text-white/50 hover:text-white px-2 py-1 rounded hover:bg-white/5" on:click={clear}>Clear</button>
          <button class="text-xs text-white/50 hover:text-white px-2 py-1 rounded hover:bg-white/5" on:click={close}>Close</button>
        </div>
      </div>

      <div class="px-4 py-3 max-h-[220px] overflow-auto text-sm text-white/90 leading-relaxed whitespace-pre-wrap font-sans">
        {#each latest as item (item.id)}
          <div class="mb-3 last:mb-0">
            {#if item.title}
              <div class="text-xs text-white/40 mb-1 flex items-center justify-between">
                <span>{item.title}</span>
                <span>{new Date(item.timestamp).toLocaleTimeString()}</span>
              </div>
            {/if}
            <div class="whitespace-pre-wrap">{item.content}</div>
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
    bottom: 130px; /* Positioned directly above command bar with no gap */
    pointer-events: auto;
    display: flex;
    justify-content: center;
    padding-bottom: 0;
    z-index: 45;
  }

  /* Small scrollbar tweaks */
  .bottom-output ::-webkit-scrollbar { width: 8px; height: 8px; }
  .bottom-output ::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.12); border-radius: 4px; }
  .bottom-output ::-webkit-scrollbar-thumb:hover { background: rgba(255,255,255,0.2); }
</style>
