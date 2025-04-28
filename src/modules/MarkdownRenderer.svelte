<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { marked } from 'marked';
  import * as markedHighlight from 'marked-highlight';
  import { gfmHeadingId } from 'marked-gfm-heading-id';
  import { getHighlighter } from 'shiki';
  import { fly } from 'svelte/transition';
  
  export let content: string = '';
  
  let renderedContent = '';
  let markdownElement: HTMLElement;
  let editorElement: HTMLTextAreaElement;
  let highlighter: any = null;
  let loading = true;
  let editing = false;
  let editingContent = '';
  let commandBarActive = false;
  let commandText = '';
  let commandBarPosition = { x: 0, y: 0 };
  let commandBarInput: HTMLInputElement;
  let commandSuggestions = [
    // Text formatting
    { cmd: 'h1', description: 'Heading 1', icon: 'H1' },
    { cmd: 'h2', description: 'Heading 2', icon: 'H2' },
    { cmd: 'h3', description: 'Heading 3', icon: 'H3' },
    { cmd: 'bold', description: 'Bold text', icon: 'B' },
    { cmd: 'italic', description: 'Italic text', icon: 'I' },
    { cmd: 'code', description: 'Code block', icon: '{ }' },
    { cmd: 'inlinecode', description: 'Inline code', icon: '`' },
    { cmd: 'quote', description: 'Quote block', icon: '"' },
    { cmd: 'divider', description: 'Divider line', icon: '—' },
    
    // Lists
    { cmd: 'list', description: 'Bullet list', icon: '•' },
    { cmd: 'numbered', description: 'Numbered list', icon: '1.' },
    { cmd: 'checklist', description: 'Check list', icon: '☑' },
    
    // Tables and media
    { cmd: 'table', description: 'Insert table', icon: '⊞' },
    { cmd: 'image', description: 'Insert image', icon: '🖼' },
    { cmd: 'link', description: 'Insert link', icon: '🔗' },
  ];

  // Only working commands - for Escape menu (non-shift menu)
  let escapeMenuCommands = ['h1', 'h2', 'h3', 'bold', 'code', 'inlinecode', 'table', 'list', 'numbered', 'checklist', 'quote', 'divider', 'link', 'image'];
  let filteredSuggestions = [...commandSuggestions];
  let shiftPressed = false;
  let shiftKeyDownTime = 0; // Track when shift was pressed down
  let shiftKeyTapped = false; // Track if shift was tapped quickly
  
  const dispatch = createEventDispatcher();
  
  // Add debounce function for performance
  function debounce(func: Function, wait: number) {
    let timeout: ReturnType<typeof setTimeout>;
    return function(...args: any[]) {
      clearTimeout(timeout);
      timeout = setTimeout(() => func.apply(null, args), wait);
    };
  }
  
  // Cached render results for performance
  const renderCache = new Map<string, string>();
  
  async function setupHighlighter() {
    try {
      // Define WASM and theme paths for Shiki
      const wasmPath = '/dist/onig.wasm';  // Will be served from static folder
      const themePath = '/themes/github-dark.json'; // Will be served from static folder
      
      // Only load essential languages initially
      highlighter = await getHighlighter({
        theme: themePath, // Use the path to theme instead of name
        langs: ['javascript', 'markdown'], // Reduced initial languages
        paths: {
          wasm: wasmPath // Tell Shiki where to find the WASM file
        }
      });
      
      // Configure marked with shiki highlighter
      marked.use(
        markedHighlight.markedHighlight({
          langPrefix: 'shiki language-',
          highlight(code, lang) {
            if (!highlighter) return code;
            
            try {
              // Load language on demand if not already loaded
              if (lang && !highlighter.getLoadedLanguages().includes(lang)) {
                // For unknown languages, fallback to text
                return highlighter.codeToHtml(code, { lang: 'text' });
              }
              
              return highlighter.codeToHtml(code, { lang: lang || 'text' });
            } catch (err) {
              console.error('Highlight error:', err);
              return code;
            }
          }
        }),
        gfmHeadingId()
      );
      
      // Set options
      marked.setOptions({
        gfm: true, // GitHub Flavored Markdown
        breaks: true, // Convert \n to <br>
      });
      
      loading = false;
      renderMarkdown();
    } catch (error) {
      console.error('Failed to initialize highlighter:', error);
      
      // Skip syntax highlighting and continue with basic markdown
      marked.setOptions({
        gfm: true,
        breaks: true
      });
      
      loading = false;
      renderMarkdown();
    }
  }
  
  // Optimize markdown rendering with caching
  function renderMarkdown() {
    if (!editingContent) return;
    
    const cacheKey = editingContent;
    
    // Check if we have a cached version
    if (renderCache.has(cacheKey)) {
      renderedContent = renderCache.get(cacheKey) || '';
      return;
    }
    
    try {
      // Render the markdown
      const rendered = marked.parse(editingContent);
      
      // Save to cache (limit cache size to prevent memory issues)
      if (renderCache.size > 50) {
        // Clear oldest entries if cache gets too large
        const keys = Array.from(renderCache.keys());
        renderCache.delete(keys[0]);
      }
      
      renderCache.set(cacheKey, rendered);
      renderedContent = rendered;
    } catch (error: any) {
      console.error('Markdown rendering error:', error);
      // Fallback to plain content if rendering fails
      renderedContent = `<div class="error">Error rendering markdown: ${error?.message || 'Unknown error'}</div><pre>${editingContent}</pre>`;
    }
  }
  
  // Debounced version of renderMarkdown for better performance
  const debouncedRenderMarkdown = debounce(renderMarkdown, 300);
  
  onMount(() => {
    // First setup content and editor state
    editingContent = content || '';
    editing = true;
    
    // Setup the highlighter (async operation that might fail)
    setupHighlighter();
    
    // Set a small delay to ensure DOM is ready
    setTimeout(() => {
      if (editorElement) {
        // Force a state update to ensure value is correct
        editorElement.value = editingContent;
        
        // Make sure element is editable
        editorElement.readOnly = false;
        
        // Try to focus the editor
        try {
          editorElement.focus();
        } catch (error) {
          console.error('Focus error:', error);
        }
      }
    }, 100);
    
    // Add document click listener only
    document.addEventListener('click', handleDocumentClick);
  });
  
  onDestroy(() => {
    highlighter = null;
    document.removeEventListener('click', handleDocumentClick);
  });
  
  // Watch for external content changes
  $: {
    if (content && !editing && content !== editingContent) {
      editingContent = content;
      debouncedRenderMarkdown();
    }
  }
  
  // Handle changes to content from parent
  $: {
    if (content && !loading) {
      if (!editing) {
        // Only update editingContent when we're not in edit mode
        editingContent = content;
        debouncedRenderMarkdown();
      }
    }
  }
  
  // Handle updates when editing
  function handleContentUpdate() {
    if (editing) {
      // When editing, reflect changes to content
      dispatch('update', { content: editingContent });
      debouncedRenderMarkdown();
    }
  }
  
  // Live preview handling
  let livePreviewTimer: any = null;
  function handleMarkdownInput(event: Event) {
    const target = event.target as HTMLTextAreaElement;
    editingContent = target.value;
    
    // Process live markdown formatting
    processLiveMarkdown(target);
    
    // Notify parent but don't update our own content variable
    dispatch('update', { content: editingContent });
    
    // Render the markdown with debounce
    debouncedRenderMarkdown();
  }
  
  function processLiveMarkdown(target: HTMLTextAreaElement) {
    const cursorPos = target.selectionStart;
    const text = target.value;
    const textBeforeCursor = text.substring(0, cursorPos);
    const textAfterCursor = text.substring(cursorPos);
    
    // Detect markdown syntax at current cursor position
    
    // Heading 1: When line starts with # followed by space
    if (/^#\s/.test(getLineAtCursor(text, cursorPos)) && !textBeforeCursor.endsWith("\\#")) {
      // Leave it as is, it will be rendered as H1
    }
    
    // Check for bold with asterisks - if we just completed **text**
    if (textBeforeCursor.endsWith("**") && !textBeforeCursor.endsWith("\\**")) {
      const priorText = textBeforeCursor.substring(0, textBeforeCursor.length - 2);
      const boldMatch = priorText.match(/\*\*([^*]+)$/);
      if (boldMatch) {
        // Don't auto-format while typing, just display it correctly when rendered
      }
    }
    
    // Check for italics with single asterisk
    if (textBeforeCursor.endsWith("*") && !textBeforeCursor.endsWith("\\*")) {
      const priorText = textBeforeCursor.substring(0, textBeforeCursor.length - 1);
      const italicMatch = priorText.match(/(?<!\*)\*([^*]+)$/);
      if (italicMatch) {
        // Don't auto-format while typing, just display it correctly when rendered
      }
    }
    
    // Check for code block
    if (textBeforeCursor.endsWith("```") && !textBeforeCursor.endsWith("\\```")) {
      // Do nothing while typing, will be rendered properly
    }
    
    // Check for inline code
    if (textBeforeCursor.endsWith("`") && !textBeforeCursor.endsWith("\\`")) {
      const priorText = textBeforeCursor.substring(0, textBeforeCursor.length - 1);
      const codeMatch = priorText.match(/`([^`]+)$/);
      if (codeMatch) {
        // Do nothing while typing, will be rendered properly
      }
    }
  }
  
  function getLineAtCursor(text: string, cursorPos: number): string {
    const lines = text.substring(0, cursorPos).split('\n');
    return lines[lines.length - 1];
  }

  function handleKeydown(event: KeyboardEvent) {
    // Handle Tab key to insert spaces instead of changing focus
    if (event.key === 'Tab') {
      event.preventDefault();
      
      const target = event.target as HTMLTextAreaElement;
      const start = target.selectionStart;
      const end = target.selectionEnd;
      
      // Insert 2 spaces at cursor position
      const newValue = target.value.substring(0, start) + '  ' + target.value.substring(end);
      target.value = newValue;
      
      // Move cursor after inserted spaces
      target.selectionStart = target.selectionEnd = start + 2;
      
      // Make sure to update our tracked content
      editingContent = target.value;
      dispatch('update', { content: editingContent });
      return;
    }
    
    // For Enter key - modify approach to force a newline insertion
    if (event.key === 'Enter') {
      // Let default browser behavior handle enter press first
      // But also manually insert a newline to guarantee it works
      const target = event.target as HTMLTextAreaElement;
      const start = target.selectionStart;
      const end = target.selectionEnd;
      
      // Insert newline at cursor position
      const newValue = target.value.substring(0, start) + '\n' + target.value.substring(end);
      
      // We need to delay this slightly to not interfere with browser's default handling
      setTimeout(() => {
        target.value = newValue;
        
        // Move cursor after inserted newline
        target.selectionStart = target.selectionEnd = start + 1;
        
        // Update our tracked content
        editingContent = target.value;
        dispatch('update', { content: editingContent });
        debouncedRenderMarkdown();
      }, 0);
    }
    
    // Allow Ctrl+S for saving
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      saveContent();
    }
  }
  
  // Handle global keydown for command/shortcuts
  function handleGlobalKeydown(event: KeyboardEvent) {
    // Always handle Escape regardless of editing state to improve reliability
    if (event.key === 'Escape') {
      // If command bar is active, close it first
      if (commandBarActive) {
        closeCommandBar();
        event.preventDefault();
        event.stopPropagation();
        return;
      }
      
      // If we're in editing mode, handle confirmation
      if (editing) {
        // Ask for confirmation if content was changed
        if (editingContent !== content) {
          if (confirm('Discard changes?')) {
            cancelEditing();
          }
        } else {
          cancelEditing();
        }
        event.preventDefault();
        event.stopPropagation();
      }
    }
    
    // Track Shift key state and timestamp when it was pressed
    if (event.key === 'Shift' && !shiftPressed) {
      shiftPressed = true;
      shiftKeyDownTime = Date.now();
      shiftKeyTapped = false; // Reset tap state
    }
    
    // Handle Ctrl+` key combination to reset UI
    if (event.ctrlKey && event.key === '`') {
      // Close command bar if open
      if (commandBarActive) {
        closeCommandBar();
      }
      event.preventDefault();
      event.stopPropagation();
      
      // Attempt to call the global UI recovery function
      if (typeof window !== 'undefined' && (window as any).__recoverUI) {
        (window as any).__recoverUI();
      }
    }
  }
  
  // Add new handler for keyup events to detect shift tap
  function handleGlobalKeyup(event: KeyboardEvent) {
    if (event.key === 'Shift') {
      // Calculate how long shift was held
      const shiftHoldDuration = Date.now() - shiftKeyDownTime;
      
      // Only register as tap if held briefly
      if (shiftHoldDuration < 200) {
        shiftKeyTapped = true;
        
        // Toggle command bar state
        if (editing) {
          if (commandBarActive) {
            closeCommandBar();
          } else if (!event.ctrlKey && !event.altKey && !event.metaKey) {
            // Position and show command bar
            if (editorElement) {
              const cursorPos = editorElement.selectionStart;
              const rect = editorElement.getBoundingClientRect();
              const lineHeight = parseInt(window.getComputedStyle(editorElement).lineHeight);
              
              // Calculate approximate cursor position
              const textUntilCursor = editorElement.value.substring(0, cursorPos);
              const lines = textUntilCursor.split('\n');
              const currentLine = lines.length;
              const verticalPos = rect.top + (currentLine * lineHeight) - editorElement.scrollTop;
              
              // Position command bar at cursor
              commandBarPosition = { 
                x: rect.left + 20, 
                y: verticalPos + 20
              };
              
              showCommandBar();
            }
          }
        }
      }
      
      // Reset shift state when key is released
      shiftPressed = false;
    }
  }
  
  function syncTextareaValue() {
    if (editorElement && editorElement.value !== editingContent) {
      editorElement.value = editingContent;
      
      // Ensure the cursor position is preserved
      const selStart = editorElement.selectionStart;
      const selEnd = editorElement.selectionEnd;
      
      // Re-focus and restore selection after updating value
      setTimeout(() => {
        if (editorElement) {
          editorElement.focus();
          editorElement.selectionStart = selStart;
          editorElement.selectionEnd = selEnd;
        }
      }, 0);
    }
  }

  // Watch for changes in editingContent
  $: {
    if (editing && editorElement) {
      syncTextareaValue();
    }
  }

  function handleEditorInput(event: Event) {
    // We're using bind:value so editingContent is already updated
    
    // Process live markdown formatting
    if (event.target) {
      processLiveMarkdown(event.target as HTMLTextAreaElement);
    }
    
    // Notify parent but don't update our own content variable
    dispatch('update', { content: editingContent });
    
    // Render the markdown with debounce
    debouncedRenderMarkdown();
  }
  
  function startEditing() {
    editing = true;
    
    // Update content to ensure correct state 
    editingContent = content || '';
    
    // Force a UI update before focusing
    setTimeout(() => {
      if (editorElement) {
        // Force textarea value to match editingContent
        editorElement.value = editingContent;
        
        // Make sure element is editable
        editorElement.readOnly = false;
        editorElement.setAttribute('contenteditable', 'true');
        
        // Focus the editor with forced re-rendering
        editorElement.blur();
        editorElement.focus();
        
        // Place cursor at the end of text
        editorElement.selectionStart = editorElement.selectionEnd = editingContent.length;
        
        // Simulate an input event to ensure everything is updated
        const inputEvent = new Event('input', { bubbles: true });
        editorElement.dispatchEvent(inputEvent);
      } else {
        console.warn('Editor element not found');
      }
    }, 50);
  }
  
  // Save content to parent without exiting edit mode
  function saveContent() {
    content = editingContent;
    dispatch('update', { content });
    renderMarkdown();
  }
  
  function cancelEditing() {
    content = editingContent;
    editing = false;
    dispatch('update', { content });
  }
  
  // Handle markdown preview click to start editing
  function handlePreviewClick(event: MouseEvent) {
    // Prevent event from bubbling to parent elements
    event.preventDefault();
    event.stopPropagation();
    
    // Start editing mode with a clear focus
    editing = true;
    
    // Update content
    editingContent = content || '';
    
    // Force immediate render update
    renderMarkdown();
    
    // Delay focusing to ensure the DOM has updated
    setTimeout(() => {
      if (editorElement) {
        // Force value update and focus
        editorElement.value = editingContent;
        
        // Make sure element is fully editable
        editorElement.readOnly = false;
        editorElement.setAttribute('contenteditable', 'true');
        
        // Focus with forced re-rendering
        editorElement.blur();
        editorElement.focus();
        
        // Place cursor at end
        editorElement.selectionStart = editorElement.selectionEnd = editingContent.length;
        
        // Simulate an input event to ensure everything is updated
        const inputEvent = new Event('input', { bubbles: true });
        editorElement.dispatchEvent(inputEvent);
      }
    }, 50);
  }
  
  function showCommandBar() {
    commandBarActive = true;
    commandText = '';
    filteredSuggestions = [...commandSuggestions];
    
    // Ensure focus is set after a brief delay
    setTimeout(() => {
      if (commandBarInput) {
        commandBarInput.focus();
      }
    }, 0);
  }

  function closeCommandBar() {
    commandBarActive = false;
    commandText = '';
    filteredSuggestions = [];
  }

  function handleCommandInput(event: Event) {
    const input = event.target as HTMLInputElement;
    const value = input.value.toLowerCase();
    
    // Filter suggestions based on input
    filteredSuggestions = commandSuggestions.filter(s => 
      s.cmd.toLowerCase().includes(value) || 
      s.description.toLowerCase().includes(value)
    );
  }

  function handleSuggestionClick(cmd: string) {
    commandText = cmd;
    executeCommand(cmd);
  }

  function executeCommand(cmd: string) {
    // Execute the command logic here
    dispatch('command', { command: cmd });
    closeCommandBar();
  }
  
  // Filter suggestions as user types
  $: {
    if (commandText) {
      filteredSuggestions = commandSuggestions.filter(s => 
        s.cmd.toLowerCase().includes(commandText.toLowerCase()) || 
        s.description.toLowerCase().includes(commandText.toLowerCase())
      );
    } else {
      filteredSuggestions = [...commandSuggestions];
    }
  }
  
  // Handle clicks outside the editor
  function handleDocumentClick(event: MouseEvent) {
    // Close command bar if it's active when clicking anywhere
    if (commandBarActive) {
      closeCommandBar();
    }
    
    if (editing && editorElement) {
      // Check if click is outside the editor
      const container = editorElement.closest('.markdown-container');
      const commandBar = document.querySelector('.command-bar');
      const textarea = editorElement;
      
      // If we clicked inside container but outside textarea, exit editing mode
      if (container && container.contains(event.target as Node) && 
          !textarea.contains(event.target as Node) && 
          (!commandBar || !commandBar.contains(event.target as Node))) {
        
        event.preventDefault();
        event.stopPropagation();
        
        // Save content and exit edit mode
        content = editingContent;
        editing = false;
        
        // Force preview to update with latest content
        renderMarkdown();
        
        // Notify parent
        dispatch('update', { content });
      }
    }
  }
</script>

<div class="markdown-container">
  {#if editing}
    <!-- Seamless Editor Mode -->
    <textarea 
      bind:this={editorElement}
      class="markdown-editor"
      bind:value={editingContent}
      on:input={(event) => {
        handleEditorInput(event);
      }}
      on:keydown={handleKeydown}
      placeholder="Type / for commands or just start writing..."
      spellcheck="false"
      tabindex="0"
    ></textarea>
  {:else}
    <!-- Preview Mode - clickable and directly editable -->
    <div 
      class="markdown-preview" 
      bind:this={markdownElement}
      on:click|stopPropagation={handlePreviewClick}
    >
      {#if loading}
        <div class="loading">Loading renderer...</div>
      {:else}
        {@html renderedContent}
      {/if}
    </div>
  {/if}
</div>

{#if commandBarActive}
  <div 
    class="command-bar"
    style="left: {commandBarPosition.x}px; top: {commandBarPosition.y}px;"
    in:fly={{ y: -10, duration: 150 }}
  >
    <input 
      type="text"
      class="command-input" 
      bind:this={commandBarInput}
      bind:value={commandText} 
      on:input={handleCommandInput}
      on:keydown={(e) => {
        // Stop propagation for all keys to prevent double handling
        e.stopPropagation();
        e.stopImmediatePropagation();
        
        if (e.key === 'Escape') {
          e.preventDefault();
          closeCommandBar();
        } else if (e.key === 'Enter') {
          e.preventDefault();
          executeCommand(commandText);
          // Close command bar after command execution
          closeCommandBar();
        }
      }}
      placeholder="Type a command..." 
    />
    {#if filteredSuggestions.length > 0}
      <div class="command-suggestions">
        {#each filteredSuggestions as suggestion}
          <div 
            class="command-suggestion" 
            on:click={(e) => {
              e.preventDefault();
              e.stopPropagation();
              handleSuggestionClick(suggestion.cmd);
              // Close command bar after selecting a suggestion
              closeCommandBar();
            }}
          >
            <div class="suggestion-icon">{suggestion.icon}</div>
            <div class="suggestion-content">
              <span class="command">{suggestion.cmd}</span>
              <span class="description">{suggestion.description}</span>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
{/if}

<style>
  @import url('https://fonts.googleapis.com/css2?family=Onest:wght@300;400;500;600;700&display=swap');
  
  .markdown-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: #000;
    align-items: center; /* Center horizontally */
    width: 100%;
    position: relative;
  }
  
  .markdown-editor {
    flex: 1;
    padding: 2rem;
    line-height: 1.6;
    color: #e1e4e8;
    background-color: #000;
    border: none;
    outline: none;
    resize: none;
    font-family: 'Onest', sans-serif;
    font-size: 16px;
    white-space: pre-wrap;
    overflow: auto;
    width: 80%;
    max-width: 900px;
    margin: 0 auto;
    caret-color: #646cff; /* Colored cursor */
    min-height: 300px;
    display: block;
    cursor: text;
    z-index: 10;
    position: relative;
    box-sizing: border-box;
  }
  
  .markdown-editor:focus {
    outline: none;
    border: none;
    caret-color: #646cff; /* Ensure cursor is visible when focused */
  }
  
  .markdown-preview {
    flex: 1;
    padding: 2rem;
    line-height: 1.6;
    color: #e1e4e8;
    overflow: auto;
    font-family: 'Onest', sans-serif;
    width: 80%;
    max-width: 900px;
    margin: 0 auto;
    background-color: #000;
    cursor: text; /* Show text cursor on hover to indicate editability */
    min-height: 300px;
  }

  .markdown-preview:hover {
    background-color: #0a0a0a; /* Subtle highlight on hover */
  }
  
  .loading {
    color: #8b949e;
    padding: 1rem;
    font-style: italic;
    font-family: 'Onest', sans-serif;
  }
  
  /* Command bar styling */
  .command-bar {
    position: absolute;
    z-index: 1000;
    background-color: #161b22;
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    width: 300px;
    font-family: 'Onest', sans-serif;
    overflow: hidden;
  }
  
  .command-input {
    width: 100%;
    padding: 12px 16px;
    background-color: #0d1117;
    color: #c9d1d9;
    border: none;
    outline: none;
    font-size: 14px;
    border-bottom: 1px solid #30363d;
  }
  
  .command-suggestions {
    max-height: 320px;
    overflow-y: auto;
  }
  
  .command-suggestion {
    padding: 8px 12px;
    cursor: pointer;
    display: flex;
    align-items: center;
    transition: background-color 0.15s ease;
  }
  
  .command-suggestion:hover {
    background-color: #1f2937;
  }
  
  .suggestion-icon {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    margin-right: 12px;
    background-color: #21262d;
    border-radius: 4px;
    font-size: 14px;
    color: #8b949e;
  }
  
  .suggestion-content {
    display: flex;
    flex-direction: column;
    flex: 1;
  }
  
  .command {
    font-weight: 500;
    color: #e1e4e8;
    font-size: 14px;
    margin-bottom: 2px;
  }
  
  .description {
    font-size: 12px;
    color: #8b949e;
  }
  
  /* Special styling for editing elements */
  .editing-heading-1 {
    font-size: 24px;
    font-weight: bold;
    color: #fff;
  }
  
  .editing-bold {
    font-weight: bold;
  }
  
  .editing-italic {
    font-style: italic;
  }
  
  .editing-code {
    font-family: 'JetBrains Mono', monospace;
    background-color: rgba(110, 118, 129, 0.2);
    padding: 2px 4px;
    border-radius: 3px;
  }
  
  /* Styling for rendered markdown */
  :global(.markdown-preview h1) {
    font-size: 2em;
    margin-top: 0.67em;
    margin-bottom: 0.67em;
    font-weight: 600;
    color: #fff;
  }
  
  :global(.markdown-preview h2) {
    font-size: 1.5em;
    margin-top: 0.83em;
    margin-bottom: 0.83em;
    font-weight: 600;
    color: #fff;
  }
  
  :global(.markdown-preview h3) {
    font-size: 1.17em;
    margin-top: 1em;
    margin-bottom: 1em;
    font-weight: 600;
    color: #fff;
  }
  
  :global(.markdown-preview p) {
    margin-top: 1em;
    margin-bottom: 1em;
  }
  
  :global(.markdown-preview ul),
  :global(.markdown-preview ol) {
    margin-top: 1em;
    margin-bottom: 1em;
    padding-left: 2em;
  }
  
  :global(.markdown-preview li) {
    margin: 0.5em 0;
  }
  
  :global(.markdown-preview code) {
    font-family: 'JetBrains Mono', monospace;
    background-color: #000;
    padding: 0.2em 0.4em;
    border-radius: 3px;
    font-size: 85%;
    border: 1px solid #333;
  }
  
  :global(.markdown-preview pre) {
    background-color: #000;
    border-radius: 6px;
    padding: 16px;
    overflow: auto;
    font-family: 'JetBrains Mono', monospace;
    margin: 1em 0;
    border: 1px solid #333;
  }
  
  :global(.markdown-preview pre code) {
    background-color: transparent;
    padding: 0;
    border-radius: 0;
    font-size: 100%;
    color: #e1e4e8;
  }
  
  :global(.markdown-preview blockquote) {
    padding: 0 1em;
    color: #8b949e;
    border-left: 3px solid #30363d;
    margin: 1em 0;
  }
  
  :global(.markdown-preview table) {
    border-collapse: collapse;
    width: 100%;
    margin: 1em 0;
  }
  
  :global(.markdown-preview th),
  :global(.markdown-preview td) {
    padding: 8px;
    border: 1px solid #30363d;
    text-align: left;
  }
  
  :global(.markdown-preview th) {
    background-color: #161b22;
  }
</style> 