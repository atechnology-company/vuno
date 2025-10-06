<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import InlineCommandBar from './InlineCommandBar.svelte';
  import EnhancedCommandBar from './EnhancedCommandBar.svelte';

  export let title: string = 'What would you like to do today?';
  export let commandText: string = '';
  export let isVisible: boolean = true;
  export let suggestions: string[] = [];
  export let showSuggestions: boolean = false;
  export let placeholder: string = 'Type a command or press Enter';

  const dispatch = createEventDispatcher();

  function handleCommandExecuted(e: CustomEvent) {
    dispatch('command-executed', e.detail);
  }

  function handleCommand(e: CustomEvent) {
    dispatch('command', e.detail);
  }

  function handleInput(e: CustomEvent) {
    dispatch('input', e.detail);
  }

  // Expose child input to parent for focus management, and register with key handler
  import { TauriKeyHandler } from '../lib/tauriKeyHandler';
  const keyHandler = TauriKeyHandler.getInstance();

  // Expose child input and register it so ESC/open can focus reliably
  export function getInputElement() {
    const el = (enhancedRef as any)?.getInputElement?.();
    try { keyHandler.setCommandBarInput(el || null); } catch {}
    return el;
  }

  // Keep the reference current when component is visible
  $: if (isVisible) {
    const el = (enhancedRef as any)?.getInputElement?.();
    try { keyHandler.setCommandBarInput(el || null); } catch {}
  }

  // Clear on unmount
  import { onDestroy } from 'svelte';
  onDestroy(() => {
    try { keyHandler.setCommandBarInput(null); } catch {}
  });

  let enhancedRef: any;

  // Reference exported prop to avoid Svelte unused-export warning
  $: _showSuggestions_flag = showSuggestions;
</script>

<InlineCommandBar />
<EnhancedCommandBar
  bind:this={enhancedRef}
  {title}
  bind:commandText
  {isVisible}
  {suggestions}
  {placeholder}
  on:command-executed={handleCommandExecuted}
  on:command={handleCommand}
  on:input={handleInput}
  on:close={() => dispatch('close')}
/>