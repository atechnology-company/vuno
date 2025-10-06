<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly } from 'svelte/transition';
  import { appState } from '../stores/appState';
  import { get } from 'svelte/store';
  import { invoke } from '@tauri-apps/api/tauri';

  const dispatch = createEventDispatcher();

  export let title: string = 'What would you like to do today?';
  export let commandText: string = '';
  export let isVisible: boolean = true;
  export let placeholder: string = 'Type a command or press Enter';
  // Unused export kept for external reference
  export const suggestions: string[] = [];

  type Chip = { label: string; value: string; sub?: string };
  let enhancedSuggestions: Chip[] = [];
  let selectedIndex = -1; // keyboard selection across combined suggestions
  let inputEl: HTMLInputElement | null = null;

  function executeCommand(cmd: string) {
    const trimmed = (cmd || '').trim();
    if (!trimmed) return;
    // Let the parent handle execution (unifies AI/toast/status flows)
    dispatch('command', { command: trimmed });
    // Clear local UI state
    commandText = '';
    enhancedSuggestions = [];
    selectedIndex = -1;
  }

  async function handleInput(e: Event) {
    const t = e.target as HTMLInputElement;
    commandText = t.value;
    // Notify parent so it can filter its suggestion list
    dispatch('input', { commandText });
    try {
      const res = await invoke('get_enhanced_command_suggestions', { input: commandText }) as any;
      // Normalize backend objects into chips, take top priority 5
      enhancedSuggestions = (Array.isArray(res) ? res : [])
        .map((s: any) => ({ label: String(s?.command ?? ''), value: String(s?.command ?? ''), sub: s?.description }))
        .filter((c: Chip) => c.label.length > 0)
        .slice(0, 5);
    } catch {
      enhancedSuggestions = [];
    }
  }

  // Prefer backend suggestions only to reduce duplication
  $: allChips = [...enhancedSuggestions].slice(0, 5) as Chip[];

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      if (selectedIndex >= 0 && selectedIndex < allChips.length) {
        executeCommand(allChips[selectedIndex].value);
      } else {
        executeCommand(commandText);
      }
      selectedIndex = -1;
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (allChips.length > 0) selectedIndex = (selectedIndex + 1 + allChips.length) % allChips.length;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      if (allChips.length > 0) selectedIndex = (selectedIndex - 1 + allChips.length) % allChips.length;
    } else if (e.key === 'Escape') {
      e.preventDefault();
      e.stopPropagation();
      // Close the command bar cleanly
      dispatch('close');
      // Also clear local state
      commandText = '';
      enhancedSuggestions = [];
      selectedIndex = -1;
      // Focus editor after a short delay
      setTimeout(() => {
        const editorEl = document.querySelector('.cm-editor, textarea');
        if (editorEl) {
          (editorEl as HTMLElement).focus();
        }
      }, 50);
    }
  }

  // Parent may call getInputElement() to focus input; no frontend key handler registration
  onMount(() => {});

  // Expose input element to parent for focusing
  export function getInputElement(): HTMLInputElement | null {
    return inputEl;
  }
</script>

{#if isVisible}
  <!-- Gradient overlay from bottom up behind the command bar -->
  <div class="fixed inset-x-0 bottom-0 h-[200px] bg-gradient-to-t from-black via-black/60 to-transparent pointer-events-none z-40"></div>

  <!-- Added command-content class so outside click detector in +page.svelte can properly close it -->
  <!-- Responsive command bar: avoid fixed 800px width which could constrain layout measurements -->
  <div class="fixed left-8 right-8 w-auto max-w-none pointer-events-auto z-50 command-content" role="dialog" aria-label="Command Bar" aria-modal="false"
    style="bottom: 8px;"
    in:fly={{ y: 20, duration: 200, opacity: 1 }} 
    out:fly={{ y: 20, duration: 150, opacity: 0 }}>
    <div class="flex flex-col gap-3">
      <!-- Title area to ensure exported `title` is used -->
      <div class="text-sm text-gray-300 px-2">{title}</div>
      <!-- Suggestion chips -->
    {#if allChips.length}
  <div class="flex flex-wrap gap-2.5 mb-1 justify-start px-2 py-1" role="listbox" tabindex="0" on:keydown|stopPropagation>
          {#each allChips as chip, i}
            <button
              type="button"
    class="px-[8px] py-[6px] border border-white/15 text-white rounded-full text-[10px] leading-none font-mono hover:bg-white/10 focus:outline-none focus:ring-1 focus:ring-white/25 bg-transparent"
              class:selected={selectedIndex === i}
              on:click={() => executeCommand(chip.value)}
              title={chip.sub}
            >{chip.label}</button>
          {/each}
        </div>
      {/if}

      <!-- Transparent input box -->
    <div class="bg-transparent rounded px-4 py-3">
        <input
          bind:this={inputEl}
      class="w-full bg-transparent text-white placeholder:text-white/50 outline-none text-[16px] leading-6 command-input command-bar-input border-0 focus:outline-none focus:ring-0 focus:border-0"
          bind:value={commandText}
          on:input={handleInput}
          on:keydown={onKeydown}
          placeholder={placeholder}
          aria-label="Enhanced command input"
        />
      </div>
    </div>
  </div>
{/if}
