<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { appState } from '../stores/appState';
  import { onMount } from 'svelte';

  export let showTutorial: boolean;
  export let onClose: () => void;

  onMount(() => {
    const handleKey = (e: KeyboardEvent) => { if (e.key === 'Escape') onClose(); };
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  });
</script>

{#if showTutorial}
  <div 
    class="fixed inset-0 bg-black/80 flex items-center justify-center z-[1000] font-mono pointer-events-none" 
    transition:fade={{ duration: 150 }}
    role="dialog"
    aria-modal="true"
    aria-label="Tutorial window"
    tabindex="-1"
  >
    <div 
      class="bg-black border border-gray-800 rounded-lg shadow-2xl w-[80vw] max-w-xl max-h-[80vh] overflow-y-auto p-8 text-white relative pointer-events-auto" 
      transition:fly={{ y: 20, duration: 200, easing: elasticOut }}
      role="document"
      tabindex="-1"
    >
      <h2 class="text-2xl font-bold mb-2">Welcome to vuno</h2>
      <p class="text-gray-300 mb-6">A simple, AI-enhanced editor for your notes and code.</p>
      <div class="flex flex-wrap gap-4 my-6">
        <div class="flex-1 min-w-[160px] p-4 bg-gray-800 border border-gray-600 rounded">
          <h3 class="text-lg mb-2">Press <kbd class="bg-gray-700 border border-gray-600 rounded px-2 py-1 font-bold">Esc</kbd></h3>
          <p class="text-gray-300">To open the command bar at any time</p>
        </div>
        <div class="flex-1 min-w-[160px] p-4 bg-gray-800 border border-gray-600 rounded">
          <h3 class="text-lg mb-2">Type commands</h3>
          <p class="text-gray-300">Like "open", "new markdown", "save as"</p>
        </div>
        <div class="flex-1 min-w-[160px] p-4 bg-gray-800 border border-gray-600 rounded">
          <h3 class="text-lg mb-2">Ask questions</h3>
          <p class="text-gray-300">About your code or to get help</p>
        </div>
      </div>
      <div class="text-center mt-8">
        <button class="bg-gray-700 text-white px-6 py-2 rounded hover:bg-gray-600 text-base font-semibold transition-colors" on:click={onClose}>Start Using vuno</button>
      </div>
    </div>
  </div>
{/if}