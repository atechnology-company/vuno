<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { outputPanel, hidePanel, type OutputItem } from './outputPanelStore';
  import { onDestroy } from 'svelte';

  let state: { visible: boolean; items: OutputItem[] } = { visible: false, items: [] };
  const unsub = outputPanel.subscribe(v => state = v);
  onDestroy(unsub);

  // Filter to only show AI responses
  $: aiItems = state.items.filter(item => item.kind === 'ai');
  $: showAI = state.visible && aiItems.length > 0;

  function close() {
    hidePanel();
  }
</script>

{#if showAI}
  <div class="fixed left-8 right-8 bottom-32 z-45 pointer-events-auto" 
       in:fly={{ y: 20, duration: 200 }} 
       out:fade={{ duration: 150 }}>
    <div class="bg-gradient-to-b from-black/95 to-black/90 backdrop-blur-sm border border-white/10 rounded-2xl shadow-2xl overflow-hidden max-h-[60vh]">
      <!-- Header -->
      <div class="flex items-center justify-between px-4 py-3 border-b border-white/5">
        <div class="flex items-center gap-2">
          <span class="text-lg">🤖</span>
          <span class="text-sm font-medium text-white">AI Assistant</span>
        </div>
        <button 
          class="text-white/50 hover:text-white/90 transition-colors text-sm px-2 py-1 rounded hover:bg-white/5"
          on:click={close}
          title="Close"
        >✕</button>
      </div>
      
      <!-- Content -->
      <div class="overflow-y-auto max-h-[calc(60vh-60px)] px-4 py-3">
        {#each aiItems as item (item.id)}
          <div class="mb-3 last:mb-0">
            {#if item.title}
              <div class="text-xs text-white/40 mb-1 flex items-center justify-between">
                <span>{item.title}</span>
                <span>{new Date(item.timestamp).toLocaleTimeString()}</span>
              </div>
            {/if}
            <div class="text-sm text-white/90 leading-relaxed whitespace-pre-wrap font-sans">
              {item.content}
            </div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  /* Custom scrollbar for OLED black theme */
  div::-webkit-scrollbar {
    width: 6px;
  }
  
  div::-webkit-scrollbar-track {
    background: transparent;
  }
  
  div::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 3px;
  }
  
  div::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.2);
  }
</style>
