<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { getDiagnostics, checkLspAvailable } from '../lib/tauriCommands';
  import type { Diagnostic } from '../lib/tauriCommands';
  
  export let visible = false;
  export let filePath = '';
  export let content = '';
  export let language = 'text';
  
  let diagnostics: Diagnostic[] = [];
  let loading = false;
  let lspAvailable = false;
  let updateTimer: number | null = null;
  
  async function updateDiagnostics() {
    if (!filePath || !content || !language) return;
    
    loading = true;
    try {
      diagnostics = await getDiagnostics(filePath, content, language);
    } catch (e) {
      console.error('Failed to get diagnostics:', e);
      diagnostics = [];
    } finally {
      loading = false;
    }
  }
  
  async function checkLsp() {
    try {
      lspAvailable = await checkLspAvailable(language);
    } catch (e) {
      lspAvailable = false;
    }
  }
  
  function getSeverityIcon(severity: string): string {
    switch (severity) {
      case 'error': return '❌';
      case 'warning': return '⚠️';
      case 'info': return 'ℹ️';
      case 'hint': return '💡';
      default: return '•';
    }
  }
  
  function getSeverityClass(severity: string): string {
    return `diagnostic-${severity}`;
  }
  
  onMount(() => {
    checkLsp();
  });
  
  onDestroy(() => {
    if (updateTimer) {
      clearTimeout(updateTimer);
    }
  });
  
  // Debounce diagnostics updates
  $: {
    if (content && visible) {
      if (updateTimer) clearTimeout(updateTimer);
      updateTimer = setTimeout(() => {
        updateDiagnostics();
      }, 1000) as any;
    }
  }
  
  $: errorCount = diagnostics.filter(d => d.severity === 'error').length;
  $: warningCount = diagnostics.filter(d => d.severity === 'warning').length;
</script>

{#if visible}
  <div class="diagnostics-panel">
    <div class="panel-header">
      <h2>Problems</h2>
      <div class="counts">
        {#if errorCount > 0}
          <span class="error-count">❌ {errorCount}</span>
        {/if}
        {#if warningCount > 0}
          <span class="warning-count">⚠️ {warningCount}</span>
        {/if}
      </div>
    </div>
    
    <div class="panel-content">
      {#if loading}
        <div class="loading">Analyzing...</div>
      {:else if !lspAvailable}
        <div class="info-message">
          <p>LSP server not available for {language}</p>
          <p class="hint">Install the language server for better diagnostics</p>
        </div>
      {:else if diagnostics.length === 0}
        <div class="no-problems">
          <p>✅ No problems found</p>
        </div>
      {:else}
        <ul class="diagnostics-list">
          {#each diagnostics as diagnostic}
            <li class={getSeverityClass(diagnostic.severity)}>
              <div class="diagnostic-icon">
                {getSeverityIcon(diagnostic.severity)}
              </div>
              <div class="diagnostic-content">
                <div class="diagnostic-location">
                  Line {diagnostic.line + 1}:{diagnostic.column}
                </div>
                <div class="diagnostic-message">
                  {diagnostic.message}
                </div>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}

<style>
  .diagnostics-panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    height: 200px;
    background: #1e1e1e;
    border-top: 1px solid #333;
    display: flex;
    flex-direction: column;
    color: #ccc;
    z-index: 100;
  }
  
  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #333;
    background: #252526;
  }
  
  .panel-header h2 {
    margin: 0;
    font-size: 1rem;
  }
  
  .counts {
    display: flex;
    gap: 1rem;
    font-size: 0.9rem;
  }
  
  .error-count {
    color: #ff6b6b;
  }
  
  .warning-count {
    color: #ffa500;
  }
  
  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }
  
  .diagnostics-list {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  
  .diagnostics-list li {
    display: flex;
    align-items: flex-start;
    padding: 0.75rem 1rem;
    border-bottom: 1px solid #2d2d2d;
    cursor: pointer;
    transition: background 0.2s;
  }
  
  .diagnostics-list li:hover {
    background: #2d2d2d;
  }
  
  .diagnostic-icon {
    flex-shrink: 0;
    margin-right: 0.75rem;
    font-size: 1rem;
  }
  
  .diagnostic-content {
    flex: 1;
  }
  
  .diagnostic-location {
    font-size: 0.85rem;
    color: #888;
    margin-bottom: 0.25rem;
    font-family: monospace;
  }
  
  .diagnostic-message {
    font-size: 0.9rem;
    line-height: 1.4;
  }
  
  .diagnostic-error .diagnostic-message {
    color: #ff6b6b;
  }
  
  .diagnostic-warning .diagnostic-message {
    color: #ffa500;
  }
  
  .diagnostic-info .diagnostic-message {
    color: #4fc3f7;
  }
  
  .diagnostic-hint .diagnostic-message {
    color: #aaa;
  }
  
  .loading, .no-problems, .info-message {
    padding: 2rem;
    text-align: center;
    color: #888;
  }
  
  .info-message .hint {
    margin-top: 0.5rem;
    font-size: 0.85rem;
    color: #666;
  }
</style>
