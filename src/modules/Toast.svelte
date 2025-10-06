<script lang="ts">
  import { onDestroy } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { toasts, removeToast, type ToastItem } from './toastStore';
  
  // Local store for component
  let toastItems: ToastItem[] = [];
  
  // Subscribe to the store
  const unsubscribe = toasts.subscribe(value => {
    toastItems = value;
  });
  
  // Function to check if message should be treated as code output
  function isCodeOutput(message: string): boolean {
    return message.includes('\n') || message.length > 100;
  }
  
  function closeAllToasts() {
    try {
      // Remove all toasts by clearing the store
      // We don't import clearToasts to keep this component decoupled
      toastItems.forEach(t => removeToast(t.id));
    } catch {}
  }

  // ESC to close top-most toast; custom event to close all
  if (typeof window !== 'undefined') {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && toastItems.length > 0) {
        const last = toastItems[toastItems.length - 1];
        removeToast(last.id);
      }
    };
    const onEvt = () => closeAllToasts();
    window.addEventListener('keydown', onKey, true);
    document.body.addEventListener('toast-close-all', onEvt as any);
    onDestroy(() => {
      window.removeEventListener('keydown', onKey, true);
      document.body.removeEventListener('toast-close-all', onEvt as any);
      unsubscribe();
    });
  } else {
    onDestroy(() => {
      unsubscribe();
    });
  }
</script>

<div class="fixed bottom-20 right-5 z-[9999] flex flex-col gap-2 max-h-[70vh] overflow-y-auto pointer-events-none">
  {#each toastItems as toast (toast.id)}
    <div
      class={
        "flex flex-col items-stretch px-4 py-3 rounded bg-black/90 text-white shadow-lg border border-gray-800 pointer-events-auto font-mono transition-all duration-200" +
        (isCodeOutput(toast.message) ? ' w-auto max-w-[80vw]' : ' max-w-[350px]')
      }
      class:border-l-4={toast.type === 'info' || toast.type === 'success' || toast.type === 'warning' || toast.type === 'error'}
      class:border-gray-600={toast.type === 'info'}
      class:border-gray-400={toast.type === 'success' || toast.type === 'warning'}
      class:border-white={toast.type === 'error'}
      in:fly={{ y: 50, duration: 300 }}
      out:fade={{ duration: 200 }}
    >
      <div class="flex justify-between items-center mb-2">
        <div class="mr-2 text-lg">
          {#if toast.type === 'error'}
            ❌
          {:else if toast.type === 'warning'}
            ⚠️
          {:else if toast.type === 'success'}
            ✅
          {:else}
            ℹ️
          {/if}
        </div>
        <button
          type="button"
          class="text-gray-400 hover:text-white text-lg h-5 w-5 flex items-center justify-center rounded focus:outline-none cursor-pointer"
          style="pointer-events: auto; z-index: 10000;"
          on:click|preventDefault|stopPropagation={(e) => {
            e.stopPropagation();
            e.preventDefault();
            console.log('Removing toast:', toast.id);
            removeToast(toast.id);
          }}
        >×</button>
      </div>
      {#if isCodeOutput(toast.message)}
        <div class="bg-black/80 rounded p-2 max-h-[300px] overflow-y-auto border border-gray-800">
          <pre class="m-0 whitespace-pre-wrap text-xs text-white font-mono">{toast.message}</pre>
        </div>
      {:else}
        <div class="text-sm break-words">{toast.message}</div>
      {/if}
    </div>
  {/each}
</div>