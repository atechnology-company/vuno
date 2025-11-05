<script lang="ts">
  import { createEventDispatcher, onMount, onDestroy } from 'svelte';
  const dispatch = createEventDispatcher();

  let show = false;
  let x = 0;
  let y = 0;
  let rootEl: HTMLElement | null = null;
  let lastShift = 0;

  const commands = [
    { label: 'H1', cmd: '# ' },
    { label: 'H2', cmd: '## ' },
    { label: 'Bullet', cmd: '- ' },
    { label: 'Code Block', cmd: '```\n\n```' }
  ];

  function openAtClientRect(rect: DOMRect) {
    x = rect.left + rect.width / 2;
    y = rect.top + 80;
    show = true;
  }

  function close() {
    show = false;
  }

  function tryToggleOnDoubleShift(e: KeyboardEvent) {
    // Disabled to prevent interference with normal app usage
    return;
  }

  function choose(cmd: string) {
    // dispatch a global event so the editor can pick it up and insert
    window.dispatchEvent(new CustomEvent('insert-command', { detail: { command: cmd } }));
    // keep behavior configurable: close by default after choosing
    close();
  }

  function onDocumentClick(e: MouseEvent) {
    if (!show) return;
    const target = e.target as Node | null;
    // If click is inside the root element, do nothing
    if (rootEl && target && (rootEl === target || rootEl.contains(target))) return;
    close();
  }

  function onDocumentKeydown(e: KeyboardEvent) {
    if (!show) return;
    if (e.key === 'Escape') {
      close();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', tryToggleOnDoubleShift);
    document.addEventListener('click', onDocumentClick, true);
    document.addEventListener('keydown', onDocumentKeydown, true);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', tryToggleOnDoubleShift);
    document.removeEventListener('click', onDocumentClick, true);
    document.removeEventListener('keydown', onDocumentKeydown, true);
  });
</script>

{#if show}
  <div bind:this={rootEl} class="fixed z-50 bg-black/95 backdrop-blur-xl text-white p-3 rounded-xl shadow-2xl border border-white/15" style="left:{x}px;top:{y}px;transform:translate(-50%,-50%)">
    {#each commands as c}
      <button class="block w-full text-left px-3 py-2 hover:bg-white/10 rounded-lg transition-all duration-200 text-sm font-medium hover:scale-105 active:scale-95" on:click={() => choose(c.cmd)}>{c.label}</button>
    {/each}
  </div>
{/if}

<style>
  /* Minimal local styles; project Tailwind will apply main styling */
</style>
