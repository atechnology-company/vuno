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
  });

  // No need for onDestroy cleanup, handled by onMount's return cleanup.

</script>

{#if showHelp}
  <div 
    class="help-overlay" 
    transition:fade={{ duration: 150 }} 
    on:click|self={closeHelp}

    role="dialog"             
    aria-modal="true"         
    aria-label="Help window"
    tabindex="-1"
  >
    <div 
      class="help-content" 
      bind:this={helpContentEl} 
      tabindex="-1" 
      role="document"
      transition:fly={{ y: 20, duration: 200, easing: elasticOut }}
    >
      <div class="help-header">
        <h2>Help & Commands</h2>
        <button class="close-button" on:click={handleCloseClick} aria-label="Close help window">×</button>
      </div>
      <input 
        class="help-filter" 
        placeholder="Quick filter..." 
        bind:this={filterInputEl} 
        bind:value={filter} 
        on:input={() => console.log('Filter changed:', filter)}
        aria-label="Filter help commands" 
      />
      <p>you may notice we do things a little differently around here.<br>the status bar at the bottom is your window bar, use it to move the window around.<br>the command bar is your gateway to all the other functions you need, here's a list of what you can do.</p>      
      <div class="help-section">
        <h3>Global Shortcuts</h3>
        <table>
          {#each [
            {k:'Esc',d:'Open/close command bar'},
            {k:'Ctrl+S',d:'Save current file'},
            {k:'F1',d:'Toggle help screen'},
            {k:'Ctrl+`',d:'Reset UI (if stuck)'},
            {k:'Ctrl+O',d:'Open file'},
            {k:'Ctrl+N',d:'New file'},
            {k:'Ctrl+Q',d:'Quit application'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
          <tr><td><kbd>{row.k}</kbd></td><td>{row.d}</td></tr>
          {/each}
        </table>
      </div>
      <div class="help-section">
        <h3>Available Commands</h3>
        <table>
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
          <tr><td>{row.k}</td><td>{row.d}</td></tr>
          {/each}
        </table>
      </div>
      <div class="help-section">
        <h3>Supported File Types</h3>
        <table>
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
          <tr><td>{row.k}</td><td>{row.d}</td></tr>
          {/each}
        </table>
      </div>
      <div class="help-section">
        <h3>AI Features</h3>
        <p>Command bar supports natural language AI interactions:</p>
        <table>
          {#each [
            {k:'Ask questions about your code'},
            {k:'Get code explanations'},
            {k:'Request code improvements'},
            {k:'Debug assistance'},
            {k:'Code generation'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase())) as row}
          <tr><td>{row.k}</td></tr>
          {/each}
        </table>
      </div>
      <div class="help-section">
        <h3>CLI Commands</h3>
        <p>Command bar can execute <strong>any</strong> terminal commands, here are some examples:</p>
        <table>
          {#each [
            {k:'npm install',d:'Install npm packages'},
            {k:'npm run dev',d:'Start development server'},
            {k:'git status',d:'Check git repository status'},
            {k:'dir',d:'List directory contents'},
            {k:'cd ..',d:'Move up one directory'},
            {k:'pwd',d:'Show current directory'},
            {k:'ls',d:'List files (Unix)'}
          ].filter(row => row.k.toLowerCase().includes(filter.toLowerCase()) || row.d.toLowerCase().includes(filter.toLowerCase())) as row}
          <tr><td>{row.k}</td><td>{row.d}</td></tr>
          {/each}
        </table>
      </div>
    </div>
  </div>
{/if}

<style>
  .help-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  .help-content {
    background-color: #1e1e1e;
    padding: 2rem;
    border-radius: 8px;
    max-width: 800px;
    width: 90%;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    border: 1px solid #333;
  }
  .help-filter {
    width: 100%;
    margin-bottom: 1rem;
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #333;
    background: #222;
    color: #fff;
    font-size: 1rem;
    outline: none;
    box-sizing: border-box;
  }
  .help-filter:focus {
    border-color: #555;
    background: #242424;
  }
  h2 {
    margin-top: 0;
    color: #fff;
  }
  .help-section {
    margin: 1.5rem 0;
  }
  h3 {
    color: #fff;
    margin-bottom: 0.5rem;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 0.5rem 0;
  }
  td {
    padding: 0.5rem;
    border-bottom: 1px solid #333;
  }
  td:first-child {
    width: 30%;
    font-weight: bold;
  }
  kbd {
    background-color: #333;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
    font-family: monospace;
  }
  .help-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }
  .close-button {
    background-color: #333;
    color: #fff;
    border: none;
    border-radius: 50%;
    width: 30px;
    height: 30px;
    font-size: 20px;
    display: flex;
    justify-content: center;
    align-items: center;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  .close-button:hover {
    background-color: #555;
  }
  .close-button:active {
    background-color: #777;
  }
</style>