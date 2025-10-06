# Vuno Features Integration Guide

This guide shows developers how to integrate the new features into the Vuno UI.

## Quick Start

All new Tauri commands are available through the TypeScript API:

```typescript
import {
  // Git
  gitStatus, gitAdd, gitCommit, gitPush, gitPull,
  gitBranchList, gitCheckout, gitDiff, gitLog,
  
  // AI
  analyzeDocument, suggestImprovements, generateTests,
  refactorCode, explainSelection,
  
  // Search
  searchWeb, setPerplexityKey,
  
  // LSP
  startLspServer, getCompletions, getDiagnostics,
  formatDocument, checkLspAvailable
} from './lib/tauriCommands';
```

## Integration Examples

### 1. Adding Git Status to the Status Bar

```typescript
// In your status bar component
<script lang="ts">
  import { gitStatus } from '../lib/tauriCommands';
  import { onMount } from 'svelte';
  
  let branch = '';
  let hasChanges = false;
  
  async function updateGitStatus() {
    try {
      const status = await gitStatus();
      branch = status.branch;
      hasChanges = status.modified.length > 0 || 
                   status.added.length > 0 ||
                   status.untracked.length > 0;
    } catch (e) {
      // Not a git repository or git not available
      branch = '';
      hasChanges = false;
    }
  }
  
  onMount(() => {
    updateGitStatus();
    // Update every 5 seconds
    setInterval(updateGitStatus, 5000);
  });
</script>

{#if branch}
  <div class="git-status">
    <span class="branch-icon">⎇</span>
    <span class="branch-name">{branch}</span>
    {#if hasChanges}
      <span class="changes-indicator">●</span>
    {/if}
  </div>
{/if}
```

### 2. Adding AI Commands to the Command Bar

```typescript
// In CommandBar.svelte or similar
<script lang="ts">
  import { 
    analyzeDocument, 
    suggestImprovements,
    explainSelection 
  } from '../lib/tauriCommands';
  import { appState } from '../stores/appState';
  
  async function handleCommand(command: string) {
    const lowerCmd = command.toLowerCase();
    const state = get(appState);
    const apiKey = state.apiKey || '';
    
    if (lowerCmd.startsWith('analyze')) {
      const result = await analyzeDocument(
        state.currentContent,
        state.currentLanguage,
        apiKey
      );
      showOutput(result.output);
    }
    else if (lowerCmd.startsWith('suggest')) {
      const result = await suggestImprovements(
        state.currentContent,
        state.currentLanguage,
        apiKey
      );
      showOutput(result.output);
    }
    else if (lowerCmd.startsWith('explain') && state.selectedText) {
      const result = await explainSelection(
        state.selectedText,
        state.currentLanguage,
        apiKey
      );
      showOutput(result.output);
    }
  }
  
  function showOutput(text: string) {
    // Display in output panel
    // Implementation depends on your UI structure
  }
</script>
```

### 3. Adding IntelliSense to CodeMirror

```typescript
// In your CodeMirror setup
import { autocompletion, CompletionContext } from '@codemirror/autocomplete';
import { getCompletions } from '../lib/tauriCommands';

async function vunoCompletions(context: CompletionContext) {
  const { state, pos } = context;
  const line = state.doc.lineAt(pos);
  const lineNumber = line.number - 1;
  const character = pos - line.from;
  
  // Get current document content and language
  const content = state.doc.toString();
  const language = getCurrentLanguage(); // Implement this based on file extension
  
  try {
    const completions = await getCompletions(
      getCurrentFilePath(),
      content,
      { line: lineNumber, character },
      language
    );
    
    return {
      from: context.pos,
      options: completions.map(item => ({
        label: item.label,
        type: item.kind,
        detail: item.detail,
        info: item.documentation
      }))
    };
  } catch (e) {
    console.error('Failed to get completions:', e);
    return null;
  }
}

// Add to CodeMirror extensions
const extensions = [
  // ... other extensions
  autocompletion({
    override: [vunoCompletions]
  })
];
```

### 4. Adding Diagnostics Display

```typescript
// In your editor component
<script lang="ts">
  import { getDiagnostics } from '../lib/tauriCommands';
  import { onMount } from 'svelte';
  
  let diagnostics = [];
  let editorContent = '';
  let language = 'rust';
  
  async function updateDiagnostics() {
    try {
      diagnostics = await getDiagnostics(
        getCurrentFilePath(),
        editorContent,
        language
      );
    } catch (e) {
      console.error('Failed to get diagnostics:', e);
    }
  }
  
  // Debounce diagnostics updates
  let diagnosticsTimer;
  $: {
    if (editorContent) {
      clearTimeout(diagnosticsTimer);
      diagnosticsTimer = setTimeout(updateDiagnostics, 1000);
    }
  }
</script>

{#if diagnostics.length > 0}
  <div class="diagnostics-panel">
    <h3>Problems</h3>
    {#each diagnostics as diag}
      <div class="diagnostic {diag.severity}">
        <span class="line">Line {diag.line + 1}:{diag.column}</span>
        <span class="message">{diag.message}</span>
      </div>
    {/each}
  </div>
{/if}
```

### 5. Adding Web Search

```typescript
// In your command bar or search component
<script lang="ts">
  import { searchWeb } from '../lib/tauriCommands';
  import { addOutput } from '../modules/outputPanelStore';
  
  async function handleSearch(query: string) {
    try {
      addOutput('Searching web...', 'info');
      const result = await searchWeb(query);
      
      // Display answer
      addOutput(result.answer, 'ai', 'Web Search Result');
      
      // Display sources
      if (result.results.length > 0) {
        const sources = result.results
          .map(r => `- ${r.title}: ${r.url}`)
          .join('\n');
        addOutput(sources, 'info', 'Sources');
      }
    } catch (e) {
      addOutput(`Search failed: ${e}`, 'error');
    }
  }
</script>
```

### 6. Adding Format Document Command

```typescript
// In your command handler
import { formatDocument } from '../lib/tauriCommands';

async function handleFormatDocument() {
  try {
    const currentContent = getEditorContent();
    const language = getCurrentLanguage();
    
    const formatted = await formatDocument(currentContent, language);
    setEditorContent(formatted);
    
    showNotification('Document formatted successfully');
  } catch (e) {
    showNotification(`Format failed: ${e}`, 'error');
  }
}
```

### 7. Adding Git Commit Dialog

```typescript
// GitCommitDialog.svelte
<script lang="ts">
  import { gitStatus, gitAdd, gitCommit, gitPush } from '../lib/tauriCommands';
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let commitMessage = '';
  let status = null;
  let loading = false;
  
  async function loadStatus() {
    status = await gitStatus();
  }
  
  async function commitChanges() {
    if (!commitMessage.trim()) {
      alert('Please enter a commit message');
      return;
    }
    
    loading = true;
    try {
      // Add all changes
      await gitAdd(['.']);
      
      // Commit
      await gitCommit(commitMessage);
      
      // Optionally push
      // await gitPush();
      
      dispatch('success');
    } catch (e) {
      alert(`Commit failed: ${e}`);
    } finally {
      loading = false;
    }
  }
  
  onMount(loadStatus);
</script>

<div class="git-dialog">
  <h2>Git Commit</h2>
  
  {#if status}
    <div class="changes">
      <h3>Changes:</h3>
      <ul>
        {#each status.modified as file}
          <li class="modified">M {file}</li>
        {/each}
        {#each status.added as file}
          <li class="added">A {file}</li>
        {/each}
        {#each status.untracked as file}
          <li class="untracked">? {file}</li>
        {/each}
      </ul>
    </div>
  {/if}
  
  <textarea
    bind:value={commitMessage}
    placeholder="Commit message..."
    rows="4"
  />
  
  <button on:click={commitChanges} disabled={loading}>
    {loading ? 'Committing...' : 'Commit'}
  </button>
</div>
```

## Command Processing

The enhanced command processor (`execute_enhanced_command`) handles natural language commands. You can extend it in `+page.svelte` or your main component:

```typescript
import { invoke } from '@tauri-apps/api/tauri';

async function processCommand(command: string) {
  const apiKey = getApiKey();
  const workingDir = getCurrentWorkingDir();
  
  try {
    const result = await invoke('execute_enhanced_command', {
      command,
      apiKey,
      workingDir
    });
    
    return result;
  } catch (e) {
    console.error('Command failed:', e);
    throw e;
  }
}
```

## Best Practices

1. **Error Handling**: Always wrap Tauri invocations in try-catch blocks
2. **Loading States**: Show loading indicators for async operations
3. **Debouncing**: Debounce expensive operations like diagnostics
4. **Caching**: Cache results when appropriate (e.g., LSP completions)
5. **User Feedback**: Always provide feedback for long-running operations
6. **API Keys**: Check for API keys before making AI/search requests

## Testing

Test your integrations with:

```typescript
// Test Git integration
await gitStatus();

// Test AI features (requires API key)
await analyzeDocument('console.log("test")', 'javascript', 'your-key');

// Test LSP features
await checkLspAvailable('rust');

// Test search (requires Perplexity key)
await searchWeb('What is Rust?');
```

## Troubleshooting

### Git commands fail
- Ensure git is installed and in PATH
- Check that the current directory is a git repository

### LSP features not working
- Install the required language server
- Use `checkLspAvailable()` to verify installation

### AI commands fail
- Verify API key is set correctly
- Check internet connection
- Ensure API quota is not exceeded

### Search fails
- Verify Perplexity API key is set
- Check Perplexity API status

## Performance Tips

1. **Lazy Loading**: Only start LSP servers when needed
2. **Debouncing**: Debounce diagnostics and completions
3. **Caching**: Cache git status for a few seconds
4. **Background Tasks**: Run git operations in background when possible
5. **Rate Limiting**: Implement rate limiting for AI requests

## Architecture Overview

```
Frontend (TypeScript/Svelte)
    ↓
tauriCommands.ts (Type-safe API)
    ↓
Tauri IPC
    ↓
Rust Backend
    ├── git.rs (Git operations)
    ├── lsp.rs (Language server integration)
    ├── perplexity.rs (Web search)
    └── command_processor.rs (AI commands)
```

All heavy lifting is done in Rust for performance and security.
