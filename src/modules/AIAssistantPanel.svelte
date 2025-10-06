<script lang="ts">
  import { 
    analyzeDocument, 
    suggestImprovements, 
    generateTests,
    refactorCode,
    explainSelection,
    searchWeb 
  } from '../lib/tauriCommands';
  import type { CommandResult } from '../lib/tauriCommands';
  import { addToast } from './toastStore';
  import { addOutput } from './outputPanelStore';
  
  export let visible = false;
  export let content = '';
  export let selectedText = '';
  export let language = 'text';
  export let apiKey = '';
  
  let loading = false;
  let activeCommand = '';
  let refactorInstruction = '';
  let searchQuery = '';
  
  async function handleAnalyze() {
    if (!apiKey) {
      addToast('Please set your API key first', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'analyze';
    try {
      const result = await analyzeDocument(content, language, apiKey);
      if (result.success) {
        addOutput(result.output, 'ai', 'Document Analysis');
        addToast('Analysis complete', 'success');
      } else {
        addToast(`Analysis failed: ${result.error}`, 'error');
      }
    } catch (e) {
      addToast(`Error: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
  
  async function handleSuggestImprovements() {
    if (!apiKey) {
      addToast('Please set your API key first', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'suggest';
    try {
      const result = await suggestImprovements(content, language, apiKey);
      if (result.success) {
        addOutput(result.output, 'ai', 'Improvement Suggestions');
        addToast('Suggestions ready', 'success');
      } else {
        addToast(`Failed: ${result.error}`, 'error');
      }
    } catch (e) {
      addToast(`Error: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
  
  async function handleGenerateTests() {
    if (!apiKey) {
      addToast('Please set your API key first', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'tests';
    try {
      const result = await generateTests(content, language, apiKey);
      if (result.success) {
        addOutput(result.output, 'ai', 'Generated Tests');
        addToast('Tests generated', 'success');
      } else {
        addToast(`Failed: ${result.error}`, 'error');
      }
    } catch (e) {
      addToast(`Error: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
  
  async function handleRefactor() {
    if (!apiKey) {
      addToast('Please set your API key first', 'warning');
      return;
    }
    
    if (!refactorInstruction.trim()) {
      addToast('Please enter refactoring instructions', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'refactor';
    try {
      const result = await refactorCode(content, language, refactorInstruction, apiKey);
      if (result.success) {
        addOutput(result.output, 'ai', 'Refactored Code');
        addToast('Refactoring complete', 'success');
      } else {
        addToast(`Failed: ${result.error}`, 'error');
      }
    } catch (e) {
      addToast(`Error: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
  
  async function handleExplain() {
    if (!apiKey) {
      addToast('Please set your API key first', 'warning');
      return;
    }
    
    const textToExplain = selectedText || content;
    if (!textToExplain) {
      addToast('No text to explain', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'explain';
    try {
      const result = await explainSelection(textToExplain, language, apiKey);
      if (result.success) {
        addOutput(result.output, 'ai', 'Code Explanation');
        addToast('Explanation ready', 'success');
      } else {
        addToast(`Failed: ${result.error}`, 'error');
      }
    } catch (e) {
      addToast(`Error: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
  
  async function handleSearch() {
    if (!searchQuery.trim()) {
      addToast('Please enter a search query', 'warning');
      return;
    }
    
    loading = true;
    activeCommand = 'search';
    try {
      const result = await searchWeb(searchQuery);
      addOutput(result.answer, 'ai', `Search: ${searchQuery}`);
      
      if (result.results.length > 0) {
        const sources = result.results
          .map(r => `- ${r.title}: ${r.url}`)
          .join('\n');
        addOutput(sources, 'info', 'Sources');
      }
      
      addToast('Search complete', 'success');
    } catch (e) {
      addToast(`Search failed: ${e}`, 'error');
    } finally {
      loading = false;
      activeCommand = '';
    }
  }
</script>

{#if visible}
  <div class="ai-panel">
    <div class="panel-header">
      <h2>🤖 AI Assistant</h2>
    </div>
    
    <div class="panel-content">
      {#if !apiKey}
        <div class="warning-message">
          <p>⚠️ No API key configured</p>
          <p class="hint">Set your Gemini API key to use AI features</p>
          <p class="hint">Use command: <code>set api_key YOUR_KEY</code></p>
        </div>
      {/if}
      
      <div class="commands-section">
        <h3>Document Commands</h3>
        
        <button 
          on:click={handleAnalyze} 
          disabled={loading || !apiKey || !content}
          class="ai-button"
        >
          {activeCommand === 'analyze' ? '⏳ Analyzing...' : '🔍 Analyze Document'}
        </button>
        
        <button 
          on:click={handleSuggestImprovements} 
          disabled={loading || !apiKey || !content}
          class="ai-button"
        >
          {activeCommand === 'suggest' ? '⏳ Suggesting...' : '💡 Suggest Improvements'}
        </button>
        
        <button 
          on:click={handleGenerateTests} 
          disabled={loading || !apiKey || !content}
          class="ai-button"
        >
          {activeCommand === 'tests' ? '⏳ Generating...' : '🧪 Generate Tests'}
        </button>
      </div>
      
      <div class="commands-section">
        <h3>Selection Commands</h3>
        
        <button 
          on:click={handleExplain} 
          disabled={loading || !apiKey || (!selectedText && !content)}
          class="ai-button"
        >
          {activeCommand === 'explain' ? '⏳ Explaining...' : '📖 Explain Code'}
        </button>
        
        {#if selectedText}
          <p class="selection-info">Selection: {selectedText.length} characters</p>
        {/if}
      </div>
      
      <div class="commands-section">
        <h3>Refactor</h3>
        
        <input
          type="text"
          bind:value={refactorInstruction}
          placeholder="Enter refactoring instruction..."
          disabled={loading}
          class="refactor-input"
        />
        
        <button 
          on:click={handleRefactor} 
          disabled={loading || !apiKey || !content || !refactorInstruction.trim()}
          class="ai-button"
        >
          {activeCommand === 'refactor' ? '⏳ Refactoring...' : '🔧 Refactor Code'}
        </button>
      </div>
      
      <div class="commands-section">
        <h3>Web Search</h3>
        
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search the web..."
          disabled={loading}
          class="search-input"
          on:keydown={(e) => e.key === 'Enter' && handleSearch()}
        />
        
        <button 
          on:click={handleSearch} 
          disabled={loading || !searchQuery.trim()}
          class="ai-button"
        >
          {activeCommand === 'search' ? '⏳ Searching...' : '🔎 Search Web'}
        </button>
      </div>
      
      <div class="info-section">
        <p class="hint">💡 Results will appear in the output panel</p>
      </div>
    </div>
  </div>
{/if}

<style>
  .ai-panel {
    position: fixed;
    right: 0;
    top: 0;
    bottom: 0;
    width: 320px;
    background: #1e1e1e;
    border-left: 1px solid #333;
    display: flex;
    flex-direction: column;
    color: #ccc;
    z-index: 100;
  }
  
  .panel-header {
    padding: 1rem;
    border-bottom: 1px solid #333;
    background: #252526;
  }
  
  .panel-header h2 {
    margin: 0;
    font-size: 1.2rem;
  }
  
  .panel-content {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .commands-section {
    margin-bottom: 1.5rem;
  }
  
  .commands-section h3 {
    margin: 0 0 0.75rem 0;
    font-size: 0.95rem;
    color: #aaa;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  
  .ai-button {
    width: 100%;
    padding: 0.75rem;
    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    color: white;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    font-weight: 500;
    margin-bottom: 0.5rem;
    transition: all 0.2s;
  }
  
  .ai-button:hover:not(:disabled) {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
  }
  
  .ai-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
    transform: none;
  }
  
  .refactor-input,
  .search-input {
    width: 100%;
    padding: 0.5rem;
    background: #2d2d2d;
    border: 1px solid #444;
    color: #ccc;
    border-radius: 4px;
    margin-bottom: 0.5rem;
    font-family: inherit;
  }
  
  .refactor-input:focus,
  .search-input:focus {
    outline: none;
    border-color: #667eea;
  }
  
  .selection-info {
    font-size: 0.85rem;
    color: #888;
    margin: 0.5rem 0;
    font-style: italic;
  }
  
  .warning-message {
    padding: 1rem;
    background: rgba(255, 165, 0, 0.1);
    border: 1px solid rgba(255, 165, 0, 0.3);
    border-radius: 6px;
    margin-bottom: 1rem;
  }
  
  .warning-message p {
    margin: 0.5rem 0;
  }
  
  .warning-message p:first-child {
    font-weight: 500;
    color: #ffa500;
  }
  
  .hint {
    font-size: 0.85rem;
    color: #888;
  }
  
  code {
    background: #2d2d2d;
    padding: 0.2rem 0.4rem;
    border-radius: 3px;
    font-family: monospace;
    font-size: 0.85rem;
  }
  
  .info-section {
    padding-top: 1rem;
    border-top: 1px solid #333;
  }
</style>
