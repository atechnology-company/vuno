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

<div class="fixed bottom-4 right-4 z-[9999] flex flex-col gap-3 max-h-[70vh] overflow-y-auto pointer-events-none">
  {#each toastItems as toast (toast.id)}
    {@const borderColor = toast.type === 'error' ? 'border-red-500/40' : toast.type === 'warning' ? 'border-yellow-500/40' : toast.type === 'success' ? 'border-green-500/40' : 'border-blue-500/40'}
    <div
      class={
        `flex flex-col items-stretch px-5 py-4 rounded-xl bg-black/95 text-white shadow-2xl border ${borderColor} backdrop-blur-xl pointer-events-auto font-sans transition-all duration-200 hover:scale-[1.02]` +
        (isCodeOutput(toast.message) ? ' w-auto max-w-[80vw]' : ' max-w-[420px]')
      }
      in:fly={{ y: 50, duration: 320, opacity: 0 }}
      out:fly={{ y: -20, duration: 220, opacity: 0 }}
    >
      <div class="flex justify-between items-start gap-3 mb-2">
        <div class="text-2xl">
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
          class="text-white/50 hover:text-white text-xl h-6 w-6 flex items-center justify-center rounded-lg hover:bg-white/10 focus:outline-none cursor-pointer transition-all"
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
        <div class="bg-black/60 rounded-lg p-3 max-h-[300px] overflow-y-auto border border-white/10">
          <pre class="m-0 whitespace-pre-wrap text-sm text-white/90 font-mono">{toast.message}</pre>
        </div>
      {:else}
        <div class="text-sm break-words leading-relaxed text-white/90">{toast.message}</div>
      {/if}
    </div>
  {/each}
</div>