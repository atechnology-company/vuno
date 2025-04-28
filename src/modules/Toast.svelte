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
  
  onDestroy(() => {
    unsubscribe();
  });
</script>

<div class="toast-container">
  {#each toastItems as toast (toast.id)}
    <div 
      class="toast toast-{toast.type} {isCodeOutput(toast.message) ? 'toast-code' : ''}"
      in:fly={{ y: 50, duration: 300 }}
      out:fade={{ duration: 200 }}
    >
      <div class="toast-header">
        <div class="toast-icon">
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
        <button class="toast-close" on:click={() => removeToast(toast.id)}>×</button>
      </div>
      
      {#if isCodeOutput(toast.message)}
        <div class="toast-code-content">
          <pre>{toast.message}</pre>
        </div>
      {:else}
        <div class="toast-message">{toast.message}</div>
      {/if}
    </div>
  {/each}
</div>

<style>
  .toast-container {
    position: fixed;
    bottom: 80px; /* Moved up to avoid command bar */
    right: 20px;
    z-index: 9999;
    display: flex;
    flex-direction: column;
    gap: 10px;
    pointer-events: none;
    max-height: 70vh;
    overflow-y: auto;
  }
  
  .toast {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    padding: 12px 16px;
    border-radius: 6px;
    background-color: rgba(34, 34, 34, 0.9);
    color: #fff;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    max-width: 350px;
    pointer-events: auto;
    font-family: 'Onest', sans-serif;
    backdrop-filter: blur(4px);
  }
  
  .toast-code {
    max-width: 80vw;
    width: 600px;
  }
  
  .toast-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 8px;
  }
  
  .toast-info {
    border-left: 4px solid #646cff;
  }
  
  .toast-success {
    border-left: 4px solid #4caf50;
  }
  
  .toast-warning {
    border-left: 4px solid #ff9800;
  }
  
  .toast-error {
    border-left: 4px solid #f44336;
  }
  
  .toast-icon {
    margin-right: 10px;
    font-size: 18px;
  }
  
  .toast-message {
    font-size: 14px;
    word-break: break-word;
  }
  
  .toast-code-content {
    background-color: rgba(20, 20, 20, 0.8);
    border-radius: 4px;
    padding: 8px;
    max-height: 300px;
    overflow-y: auto;
    font-family: monospace;
  }
  
  .toast-code-content pre {
    margin: 0;
    white-space: pre-wrap;
    font-size: 12px;
    color: #ddd;
  }
  
  .toast-close {
    background: none;
    border: none;
    color: #aaa;
    font-size: 18px;
    cursor: pointer;
    padding: 0;
    height: 20px;
    width: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
  }
  
  .toast-close:hover {
    color: #fff;
  }
  
  /* Scrollbar styling */
  .toast-code-content::-webkit-scrollbar,
  .toast-container::-webkit-scrollbar {
    width: 6px;
    height: 6px;
  }
  
  .toast-code-content::-webkit-scrollbar-track,
  .toast-container::-webkit-scrollbar-track {
    background: transparent;
  }
  
  .toast-code-content::-webkit-scrollbar-thumb,
  .toast-container::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
  }
</style> 