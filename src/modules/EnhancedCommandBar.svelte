<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { fly, fade } from 'svelte/transition';
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
      e.stopPropagation();
      
      // Ensure we have the latest input value
      const target = e.target as HTMLInputElement;
      const currentValue = target?.value || commandText;
      
      if (selectedIndex >= 0 && selectedIndex < allChips.length) {
        executeCommand(allChips[selectedIndex].value);
      } else if (currentValue.trim()) {
        executeCommand(currentValue);
      }
      selectedIndex = -1;
      return;
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
  <!-- Gradient overlay from bottom up behind the command bar - centered with vignette -->
  <div 
    class="fixed inset-0 pointer-events-none z-40"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 150 }}
  >
    <div class="absolute inset-x-0 bottom-0 h-[300px] bg-gradient-to-t from-black via-black/60 to-transparent"></div>
    <div class="absolute inset-0 bg-gradient-radial from-transparent via-transparent to-black/30"></div>
  </div>

  <!-- Added command-content class so outside click detector in +page.svelte can properly close it -->
  <!-- Responsive command bar: avoid fixed 800px width which could constrain layout measurements -->
  <div class="fixed left-8 right-8 w-auto max-w-none pointer-events-auto z-50 command-content" role="dialog" aria-label="Command Bar" aria-modal="false"
    style="bottom: 12px;"
    in:fly={{ y: 40, duration: 280, opacity: 0 }} 
    out:fly={{ y: 20, duration: 180, opacity: 0 }}>
    <div class="flex flex-col gap-3 backdrop-blur-xl bg-black/40 rounded-2xl border border-white/8 shadow-2xl p-4">
      <!-- Title area to ensure exported `title` is used -->
      <div class="text-base text-white/90 font-medium mb-1">{title}</div>
      <!-- Suggestion chips -->
    {#if allChips.length}
  <div class="flex flex-wrap gap-2 mb-2 justify-start" role="listbox" tabindex="0" on:keydown|stopPropagation in:fade={{ duration: 200 }}>
          {#each allChips as chip, i (chip.value)}
            <button
              type="button"
              class={`px-3 py-2 border rounded-lg text-xs font-mono focus:outline-none focus:ring-2 focus:ring-white/30 transition-all duration-200 hover:scale-105 active:scale-95 ${
                selectedIndex === i 
                  ? 'bg-white/15 border-white/30 text-white' 
                  : 'bg-black/30 border-white/12 text-white/90 hover:bg-white/12 hover:border-white/20'
              }`}
              on:click={() => executeCommand(chip.value)}
              title={chip.sub}
              in:fly={{ x: -10, duration: 200, delay: i * 30 }}
            >{chip.label}</button>
          {/each}
        </div>
      {/if}

      <!-- Transparent input box -->
    <div class="bg-black/20 rounded-xl px-5 py-4 border border-white/8">
        <input
          bind:this={inputEl}
      class="w-full bg-transparent text-white placeholder:text-white/40 outline-none text-lg leading-relaxed command-input command-bar-input border-0 focus:outline-none focus:ring-0 focus:border-0"
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
