<!-- CommandBar.svelte -->
<script lang="ts">
  import { fade, fly } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  import { appState } from '../stores/appState';
  import { get } from 'svelte/store';

  // Props
  export let title: string = 'What would you like to do today?';
  export let commandText: string = '';
  export let showSuggestions: boolean = true;
  export let suggestions: string[] = [];

  // Internal state
  let commandInput: HTMLInputElement;
  let selectedIndex = -1;
  let isActive = false;
  
  // Event dispatcher
  const dispatch = createEventDispatcher();
  
  // Function to get command input element (for focusing)
  export function getInputElement() {
    return commandInput;
  }
  
  // Function to handle key events
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'ArrowDown') {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, suggestions.length - 1);
    } else if (event.key === 'ArrowUp') {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, -1);
    } else if (event.key === 'Enter') {
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
        // Use suggestion
        const command = suggestions[selectedIndex];
        appState.update(s => ({ ...s, commandText: command }));
        dispatch('command', { command });
      } else if (commandText.trim()) {
        // Use entered command
        dispatch('command', { command: commandText });
      }
    } else if (event.key === 'Tab') {
      event.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < suggestions.length) {
        // Complete with suggestion
        appState.update(s => ({ ...s, commandText: suggestions[selectedIndex] }));
        commandInput.focus();
      }
    } else if (event.key === '?') {
      if (event.ctrlKey || event.metaKey) {
        event.preventDefault();
        dispatch('toggle-help');
      }
    }
  }
  
  // Handle input changes
  function handleInput(event: Event) {
    const target = event.target as HTMLInputElement;
    dispatch('input', { commandText: target.value });
    selectedIndex = -1; // Reset selection when typing
  }
  
  // Click on suggestion
  function selectSuggestion(suggestion: string) {
    appState.update(s => ({ ...s, commandText: suggestion }));
    dispatch('suggestion-click', { suggestion });
    commandInput.focus();
  }
  
  // Lifecycle
  onMount(() => {
    if (commandInput) {
      commandInput.focus();
    }
    
    // Mark as active
    isActive = true;
  });
  
  onDestroy(() => {
    isActive = false;
  });

  // Limit number of suggestions shown
  $: limitedSuggestions = suggestions.slice(0, 5); // Show max 5 suggestions
</script>

<div class="command-modal">
  <div class="command-content">
    <div class="top-bar">
      <div class="command-title">{title}</div>
      <div class="suggestions-container">
        {#if showSuggestions && suggestions.length > 0}
          <div class="command-results">
            {#each limitedSuggestions as suggestion, i}
              <div 
                class="command-item {i === selectedIndex ? 'selected' : ''}" 
                on:click={() => selectSuggestion(suggestion)}
              >
                {suggestion}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    </div>
    <input
      type="text"
      class="command-input"
      placeholder="Type a command or ask a question..."
      bind:value={commandText}
      bind:this={commandInput}
      on:input={handleInput}
      on:keydown={handleKeydown}
      autocomplete="off"
      spellcheck="false"
    />
  </div>
</div>

<style>
  .command-modal {
    position: fixed;
    bottom: 0;
    left: 0;
    width: 100%;
    height: 25vh; /* Increased from 15vh to 25vh for bigger gradient */
    display: flex;
    align-items: flex-end;
    justify-content: center;
    z-index: 1000;
    background: linear-gradient(to top, rgba(0, 0, 0, 1) 0%, rgba(0, 0, 0, 0) 100%);
    pointer-events: all;
    padding: 0;
    margin: 0;
  }

  .command-content {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: flex-end;
    padding: 0.5rem 1rem 1rem 1rem;
    box-sizing: border-box;
  }
  
  .top-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .command-title {
    font-size: 0.9rem;
    color: #aaa;
    font-family: 'Onest', sans-serif;
  }
  
  .suggestions-container {
    flex: 1;
    display: flex;
    justify-content: flex-end;
    max-width: 70%;
  }

  .command-input {
    width: 100%;
    background-color: rgba(30, 30, 30, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 4px;
    padding: 0.75rem;
    color: white;
    font-size: 1rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .command-input:focus {
    border-color: rgba(255, 255, 255, 0.3);
    background-color: rgba(40, 40, 40, 0.8);
  }

  .command-results {
    display: flex;
    gap: 0.5rem;
    max-width: 100%;
    white-space: nowrap;
    overflow: hidden; /* Prevent scrolling */
  }

  .command-item {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    transition: background-color 0.2s;
    color: #ddd;
    font-size: 0.9rem;
    background-color: rgba(40, 40, 40, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .command-item:hover {
    background-color: rgba(60, 60, 60, 0.8);
    border-color: rgba(255, 255, 255, 0.2);
  }

  .command-item.selected {
    background-color: rgba(80, 80, 80, 0.8);
    border-color: rgba(255, 255, 255, 0.3);
  }
</style>