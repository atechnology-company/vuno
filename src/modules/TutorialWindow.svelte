<script lang="ts">
  import { fly, fade } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { appState } from '../stores/appState';

  export let showTutorial: boolean;
  export let onClose: () => void;
</script>

{#if showTutorial}
  <div 
    class="tutorial-overlay" 
    transition:fade={{ duration: 150 }}
    role="dialog"
    aria-modal="true"
    aria-label="Tutorial window"
    tabindex="-1"
    on:keydown={e => e.key === 'Escape' && onClose()}
  >
    <div 
      class="tutorial-content" 
      transition:fly={{ y: 20, duration: 200, easing: elasticOut }}
      role="document"
      tabindex="-1"
    >
      <h2>Welcome to vuno</h2>
      <p>A simple, AI-enhanced editor for your notes and code.</p>
      
      <div class="tutorial-tips">
        <div class="tip">
          <h3>Press <kbd>Esc</kbd></h3>
          <p>To open the command bar at any time</p>
        </div>
        
        <div class="tip">
          <h3>Type commands</h3>
          <p>Like "open", "new markdown", "save as"</p>
        </div>
        
        <div class="tip">
          <h3>Ask questions</h3>
          <p>About your code or to get help</p>
        </div>
      </div>
      
      <div class="tutorial-close">
        <button on:click={onClose}>Start Using vuno</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .tutorial-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.8);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
    font-family: 'Onest', sans-serif;
  }
  
  .tutorial-content {
    background-color: #252526;
    border-radius: 4px;
    padding: 20px;
    width: 80%;
    max-width: 600px;
    max-height: 80vh;
    overflow-y: auto;
    box-shadow: 0 0 20px rgba(0, 0, 0, 0.5);
    font-family: 'Onest', sans-serif;
  }
  
  button {
    background-color: #2d2d2d;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
    margin-top: 16px;
    font-family: 'Onest', sans-serif;
    transition: background-color 0.2s;
  }
  
  button:hover {
    background-color: #3d3d3d;
  }
  
  .tutorial-close {
    text-align: center;
    margin-top: 20px;
  }
  
  .tutorial-tips {
    display: flex;
    flex-wrap: wrap;
    gap: 16px;
    margin: 20px 0;
  }
  
  .tip {
    flex: 1;
    min-width: 160px;
    padding: 12px;
    background-color: #2a2a2a;
    border-radius: 4px;
  }
  
  .tip h3 {
    margin-top: 0;
    margin-bottom: 8px;
    font-size: 16px;
  }
  
  .tip p {
    margin: 0;
    font-size: 14px;
    color: #bbbbbb;
  }

  kbd {
    background-color: #333;
    border-radius: 3px;
    border: 1px solid #666;
    box-shadow: 0 1px 0 rgba(255, 255, 255, 0.2), 0 0 0 2px #333 inset;
    color: #eee;
    display: inline-block;
    font-family: 'JetBrains Mono', monospace;
    font-size: 0.85em;
    font-weight: 700;
    line-height: 1;
    padding: 2px 4px;
    white-space: nowrap;
    margin: 0 2px;
  }
</style>