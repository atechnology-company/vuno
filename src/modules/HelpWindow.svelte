<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { appState } from '../stores/appState';
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';

  export let showHelp: boolean = false;
  let helpContentEl: HTMLElement;

  let filter = "";
  let filterInputEl: HTMLInputElement;
  
  // Direct close function for reliability
  function closeHelp() {
    appState.update(s => ({ ...s, showHelp: false }));
  }
  
  // Help window is now controlled by TauriKeyHandler
  // No need for local keyboard event handling
  
  function handleCloseClick() {
    closeHelp();
  }
  
  onMount(() => {
    // Focus logic for accessibility
    setTimeout(() => {
      if (helpContentEl) helpContentEl.focus();
      if (filterInputEl) {
        filterInputEl.value = "";
        filterInputEl.focus();
      }
    }, 0);

    // Add document-level keydown handler instead of inline on:keydown on non-interactive div
    const handleKey = (e: KeyboardEvent) => { if (e.key === 'Escape') closeHelp(); };
    document.addEventListener('keydown', handleKey);
    return () => document.removeEventListener('keydown', handleKey);
  });

  // No need for onDestroy cleanup, handled by onMount's return cleanup.

</script>

{#if showHelp}
  <div 
    class="fixed inset-0 bg-black/60 flex items-center justify-center z-[1000] pointer-events-none" 
    transition:fade={{ duration: 150 }} 
    role="dialog" aria-modal="true" aria-label="Help window" tabindex="-1"
  >
    <div 
      class="bg-black border border-gray-800 rounded-lg shadow-xl max-w-2xl w-[90vw] max-h-[80vh] overflow-y-auto p-8 relative focus:outline-none pointer-events-auto" 
      bind:this={helpContentEl} tabindex="-1" role="document"
      transition:fly={{ y: 20, duration: 200, easing: elasticOut }}
    >
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-2xl font-mono text-white m-0">Help & Commands</h2>
        <button class="w-8 h-8 flex items-center justify-center rounded bg-gray-800 text-white text-xl hover:bg-gray-700 focus:outline-none" on:click={handleCloseClick} aria-label="Close help window">×</button>
      </div>
      <input 
        class="w-full mb-4 px-3 py-2 rounded border border-gray-700 bg-black text-white text-base font-mono focus:border-gray-500 focus:bg-gray-900 outline-none" 
        placeholder="Quick filter..." 
        bind:this={filterInputEl} 
        bind:value={filter} 
        aria-label="Filter help commands" 
      />
      <p class="text-gray-300 mb-4">you may notice we do things a little differently around here.<br>the status bar at the bottom is your window bar, use it to move the window around.<br>the command bar is your gateway to all the other functions you need, here's a list of what you can do.</p>
      <div class="mb-6">
        <h3 class="text-lg font-mono text-white mb-2">Global Shortcuts</h3>
        <table class="w-full text-sm font-mono mb-2">
          <tbody>
          {#each [
            {k:'Esc',d:'Open/close command bar'},
            {k:'Ctrl+S',d:'Save current file'},
            {k:'F1',d:'Toggle help screen'},
            {k:'Ctrl+`',d:'Reset UI (if stuck)'},
            {k:'Ctrl+O',d:'Open file'},
            {k:'Ctrl+N',d:'New file'},
            {k:'Ctrl+Q',d:'Quit application'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
            <tr><td><kbd class="bg-gray-800 px-2 py-1 rounded text-white">{row.k}</kbd></td><td class="text-gray-200">{row.d}</td></tr>
          {/each}
          </tbody>
        </table>
      </div>
      <div class="mb-6">
        <h3 class="text-lg font-mono text-white mb-2">Available Commands</h3>
        <table class="w-full text-sm font-mono mb-2">
          <tbody>
          {#each [
            {k:'new [type]',d:'Create new file (markdown/javascript/python/etc)'},
            {k:'open [file]',d:'Open existing file'},
            {k:'save',d:'Save current file'},
            {k:'save as',d:'Save file with new name'},
            {k:'rename [old] [new]',d:'Rename file'},
            {k:'delete [file]',d:'Delete file'},
            {k:'exit/quit',d:'Exit application'},
            {k:'help',d:'Show this help screen'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
            <tr><td class="font-bold text-gray-100">{row.k}</td><td class="text-gray-200">{row.d}</td></tr>
          {/each}
          </tbody>
        </table>
      </div>
      <div class="mb-6">
        <h3 class="text-lg font-mono text-white mb-2">Supported File Types</h3>
        <table class="w-full text-sm font-mono mb-2">
          <tbody>
          {#each [
            {k:'Markdown',d:'.md, .markdown'},
            {k:'JavaScript',d:'.js, .jsx, .mjs'},
            {k:'TypeScript',d:'.ts, .tsx'},
            {k:'Python',d:'.py, .pyw, .pyi'},
            {k:'HTML',d:'.html, .htm'},
            {k:'CSS',d:'.css, .scss, .sass'},
            {k:'Rust',d:'.rs'},
            {k:'C/C++',d:'.c, .cpp, .h, .hpp'},
            {k:'Java',d:'.java'},
            {k:'Go',d:'.go'},
            {k:'Shell',d:'.sh, .bash, .zsh'},
            {k:'JSON',d:'.json'},
            {k:'YAML',d:'.yml, .yaml'},
            {k:'XML',d:'.xml'},
            {k:'SQL',d:'.sql'},
            {k:'Plain Text',d:'.txt'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
            <tr><td class="font-bold text-gray-100">{row.k}</td><td class="text-gray-200">{row.d}</td></tr>
          {/each}
          </tbody>
        </table>
      </div>
      <div class="mb-6">
        <h3 class="text-lg font-mono text-white mb-2">AI Features</h3>
        <p class="text-gray-300 mb-2">Command bar supports natural language AI interactions:</p>
        <table class="w-full text-sm font-mono mb-2">
          <tbody>
          {#each [
            {k:'Ask questions about your code'},
            {k:'Get code explanations'},
            {k:'Request code improvements'},
            {k:'Debug assistance'},
            {k:'Code generation'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase())) as row}
            <tr><td class="text-gray-200">{row.k}</td></tr>
          {/each}
          </tbody>
        </table>
      </div>
      <div class="mb-6">
        <h3 class="text-lg font-mono text-white mb-2">CLI Commands</h3>
        <p class="text-gray-300 mb-2">Command bar can execute <strong>any</strong> terminal commands, here are some examples:</p>
        <table class="w-full text-sm font-mono mb-2">
          <tbody>
          {#each [
            {k:'npm install',d:'Install npm packages'},
            {k:'npm run dev',d:'Start development server'},
            {k:'git status',d:'Check git repository status'},
            {k:'dir',d:'List directory contents'},
            {k:'cd ..',d:'Move up one directory'},
            {k:'pwd',d:'Show current directory'},
            {k:'ls',d:'List files (Unix)'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
            <tr><td class="font-bold text-gray-100">{row.k}</td><td class="text-gray-200">{row.d}</td></tr>
          {/each}
          </tbody>
        </table>
      </div>
    </div>
  </div>
{/if}