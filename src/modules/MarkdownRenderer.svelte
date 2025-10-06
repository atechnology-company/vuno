<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  const browser = typeof window !== 'undefined';
  import { marked } from 'marked';
  import { markedHighlight } from 'marked-highlight';
  import { gfmHeadingId } from 'marked-gfm-heading-id';
  import { getHighlighter } from 'shiki';
  import { fly } from 'svelte/transition';
  
  export let content: string = '';
  
  /*****************
   * Types
   *****************/
  type CommandBarMode = 'menu' | 'slash';
  interface CommandSuggestion { cmd: string; description: string; icon: string; }

  /*****************
   * State
   *****************/
  let renderedContent = '';
  let editorElement: HTMLTextAreaElement;
  let previewElement: HTMLElement;
  let highlighter: any = null;

  let editing = true; // start in edit mode
  let editingContent = content || '';
  let loading = false; // reserved for future async states

  let commandBarActive = false;
  let commandBarMode: CommandBarMode = 'menu';
  let commandText = '';
  let commandBarPosition = { x: 0, y: 0 };
  let selectedSuggestion = 0;
    // Custom caret tracking state
    let caretPosition = { x: 0, y: 0, height: 20 };
    let mirrorElement: HTMLDivElement | null = null;

  // Full command suggestion list
  const commandSuggestions: CommandSuggestion[] = [
    { cmd: 'h1', description: 'Heading 1', icon: 'H1' },
    { cmd: 'h2', description: 'Heading 2', icon: 'H2' },
    { cmd: 'h3', description: 'Heading 3', icon: 'H3' },
    { cmd: 'bold', description: 'Bold text', icon: 'B' },
    { cmd: 'italic', description: 'Italic text', icon: 'I' },
    { cmd: 'code', description: 'Code block', icon: '{ }' },
    { cmd: 'inlinecode', description: 'Inline code', icon: '`' },
    { cmd: 'quote', description: 'Quote block', icon: '"' },
    { cmd: 'divider', description: 'Divider line', icon: '—' },
    { cmd: 'list', description: 'Bullet list', icon: '•' },
    { cmd: 'numbered', description: 'Numbered list', icon: '1.' },
    { cmd: 'checklist', description: 'Task list', icon: '☑' },
    { cmd: 'table', description: 'Insert table', icon: '⊞' },
    { cmd: 'image', description: 'Insert image', icon: '🖼' },
    { cmd: 'link', description: 'Insert link', icon: '🔗' }
  ];

  // Only working commands - for Escape menu (non-shift menu)
  const escapeMenuCommands = ['h1', 'h2', 'h3', 'bold', 'code', 'inlinecode', 'table', 'list', 'numbered', 'checklist', 'quote', 'divider', 'link', 'image'];
  let filteredSuggestions: CommandSuggestion[] = [...commandSuggestions];
  let shiftPressed = false;
  let shiftKeyDownTime = 0; // Track when shift was pressed down
  let shiftKeyTapped = false; // Track if shift was tapped quickly
  
  const dispatch = createEventDispatcher();
  
  // (debounce & cache removed for instant rendering)
  
  async function setupHighlighter() {
    try {
      // Try to initialize Shiki in the background; don't block rendering
      highlighter = await getHighlighter({
        theme: 'github-dark',
        langs: ['javascript', 'typescript', 'python', 'html', 'css', 'json', 'markdown', 'rust', 'c', 'cpp', 'java', 'go', 'shell', 'bash', 'sql']
      });
      
      marked.use(
        markedHighlight({
          langPrefix: 'shiki language-',
          highlight(code, lang) {
            if (!highlighter) return `<pre><code>${code}</code></pre>`;
            try {
              const langMap: Record<string, string> = { js: 'javascript', ts: 'typescript', py: 'python', rs: 'rust', sh: 'shell' };
              const actualLang = langMap[lang || ''] || lang || 'text';
              if (!highlighter.getLoadedLanguages().includes(actualLang)) {
                return `<pre><code class="language-${actualLang}">${code}</code></pre>`;
              }
              return highlighter.codeToHtml(code, { lang: actualLang });
            } catch (err) {
              console.error('Highlight error:', err);
              return `<pre><code class="language-${lang || 'text'}">${code}</code></pre>`;
            }
          }
        }),
        gfmHeadingId()
      );
      marked.setOptions({ gfm: true, breaks: true });
      renderMarkdown();
    } catch (error) {
      console.error('Failed to initialize highlighter:', error);
      marked.setOptions({ gfm: true, breaks: true });
      renderMarkdown();
    }
  }
  
  // Instant markdown rendering
  function renderMarkdown() {
    if (editingContent == null) return;
    
    try {
      // Render the markdown
      const rendered = marked.parse(editingContent);
      renderedContent = rendered;
    } catch (error: any) {
      console.error('Markdown rendering error:', error);
      // Fallback to plain content if rendering fails
      renderedContent = `<div class="error">Error rendering markdown: ${error?.message || 'Unknown error'}</div><pre>${editingContent}</pre>`;
    }
  }
  
  onMount(() => {
    // First setup content and editor state
    editingContent = content || '';
    editing = true;
    
    // Setup the highlighter (async operation that might fail)
    // Start async without affecting UI readiness
    setTimeout(() => { setupHighlighter(); }, 0);
    
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
  // Attach global key listeners for Escape and Shift tap
  window.addEventListener('keydown', handleGlobalKeydown, true);
  window.addEventListener('keyup', handleGlobalKeyup, true);
    setTimeout(() => { updateCaretPosition(); }, 150);
  });
  
  onDestroy(() => {
    // Clean up highlighter and event listeners
    highlighter = null;
    if (browser) {
      document.removeEventListener('click', handleDocumentClick);
      window.removeEventListener('keydown', handleGlobalKeydown, true);
      window.removeEventListener('keyup', handleGlobalKeyup, true);
    }
    if (mirrorElement && mirrorElement.parentNode) mirrorElement.parentNode.removeChild(mirrorElement);
  });
  
  // Watch for external content changes
  $: if (content && !editing && content !== editingContent) {
    editingContent = content;
    renderMarkdown();
  }
  
  // Handle changes to content from parent
  $: if (content && !loading && !editing) {
    editingContent = content;
    renderMarkdown();
  }
  
  // Handle updates when editing
  function handleContentUpdate() {
    if (!editing) return;
    dispatch('update', { content: editingContent });
    renderMarkdown();
  }
  
  // Live preview handling
  // (Removed old debounced live preview handler)
  
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
    
  // Auto-wrap selection when typing quick patterns like **text** or *text*
  // This keeps typing natural; rendering already reflects styling.
  // Additional conversion is done on space below for heading markers.
    
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
    // If inline command bar is open, handle navigation keys and ESC here first
    if (commandBarActive) {
      if (event.key === 'Escape') {
        event.preventDefault();
        event.stopPropagation();
        closeCommandBar();
        return;
      }
      if (event.key === 'ArrowDown') {
        event.preventDefault();
        selectedSuggestion = (selectedSuggestion + 1 + filteredSuggestions.length) % (filteredSuggestions.length || 1);
        return;
      }
      if (event.key === 'ArrowUp') {
        event.preventDefault();
        selectedSuggestion = (selectedSuggestion - 1 + filteredSuggestions.length) % (filteredSuggestions.length || 1);
        return;
      }
      if (event.key === 'Enter') {
        // Apply the currently selected suggestion; also remove the '/query' trigger
        event.preventDefault();
        const cmd = filteredSuggestions.length > 0
          ? (typeof filteredSuggestions[selectedSuggestion] === 'string' ? filteredSuggestions[selectedSuggestion] : (filteredSuggestions[selectedSuggestion] as any).cmd)
          : commandText;
        applyInlineCommand(String(cmd || ''));
        // After applying, close the command bar and clear state
        closeCommandBar();
        return;
      }
      // For other keys, update the query from textarea after the key is processed in input handler
      // We'll recompute commandText in handleEditorInput
    }
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

    // Slash command disabled - was causing issues with Enter key
    // if (event.key === '/' && !event.metaKey && !event.ctrlKey && !event.altKey) {
    //   if (!commandBarActive) {
    //     commandBarMode = 'slash';
    //     openCommandBarAtCursor();
    //     event.stopPropagation();
    //     selectedSuggestion = 0;
    //   }
    // }
    // Never treat heading hash (#) as a slash trigger; if active with empty query close
    if (event.key === '#') {
      if (commandBarActive && commandBarMode === 'slash' && commandText === '') {
        closeCommandBar();
      }
      return; // Do not process '#' further for command palette logic
    }

    // Handle space key for live markdown formatting (Notion-like triggers)
    if (event.key === ' ') {
      const target = event.target as HTMLTextAreaElement;
      const cursorPos = target.selectionStart;
      const textBeforeCursor = target.value.substring(0, cursorPos);
      const currentLine = getCurrentLine(target.value, cursorPos);
      
      // Check for markdown patterns at the beginning of lines
      if (currentLine.match(/^#{1,6}$/)) {
        // Convert # to heading after space
        event.preventDefault();
        const headingLevel = currentLine.length;
        const beforeLine = target.value.substring(0, cursorPos - currentLine.length);
        const afterCursor = target.value.substring(cursorPos);
        
        // Replace with heading and add space
        const newText = beforeLine + '#'.repeat(headingLevel) + ' ' + afterCursor;
        target.value = newText;
        target.selectionStart = target.selectionEnd = cursorPos + 1;
        
        editingContent = newText;
        dispatch('update', { content: editingContent });
  renderMarkdown();
        return;
      }

      // Convert -- to em dash on space
      if (/--$/.test(currentLine)) {
        event.preventDefault();
        const before = target.value.substring(0, cursorPos - 2);
        const after = target.value.substring(cursorPos);
        const newText = before + '—' + ' ' + after;
        target.value = newText;
        const newPos = before.length + 2; // em dash plus space
        target.selectionStart = target.selectionEnd = newPos;
        editingContent = newText;
        dispatch('update', { content: editingContent });
  renderMarkdown();
        return;
      }

      // Convert ~~text~~ to strikethrough naturally via rendering; no text mutation needed
      
      // Auto-complete bold/italic patterns
      if (currentLine.endsWith('**')) {
        // Don't auto-complete, let user type normally
      } else if (currentLine.endsWith('*') && !currentLine.endsWith('**')) {
        // Don't auto-complete, let user type normally  
      }
    }
    
    // For Enter key - handle list continuation
    if (event.key === 'Enter') {
      const target = event.target as HTMLTextAreaElement;
      const cursorPos = target.selectionStart;
      const currentLine = getCurrentLine(target.value, cursorPos);
      
      // Continue lists
      const listMatch = currentLine.match(/^(\s*)([-*+]|\d+\.)\s/);
      if (listMatch) {
        event.preventDefault();
        const indent = listMatch[1];
        const bullet = listMatch[2];
        
        const beforeCursor = target.value.substring(0, cursorPos);
        const afterCursor = target.value.substring(cursorPos);
        
        let newBullet = bullet;
        if (bullet.match(/\d+\./)) {
          const num = parseInt(bullet) + 1;
          newBullet = `${num}.`;
        }
        
        const newText = beforeCursor + '\n' + indent + newBullet + ' ' + afterCursor;
        target.value = newText;
        target.selectionStart = target.selectionEnd = cursorPos + indent.length + newBullet.length + 2;
        
        editingContent = newText;
        dispatch('update', { content: editingContent });
  renderMarkdown();
        return;
      }
    }
    
    // ESC anywhere closes inline command bar
    if (event.key === 'Escape' && commandBarActive) {
      event.preventDefault();
      closeCommandBar();
      return;
    }

    // Allow Ctrl+S for saving
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
      event.preventDefault();
      saveContent();
      // Notify parent to perform save flow
      dispatch('save');
    }
    // Schedule caret measurement next frame
    requestAnimationFrame(updateCaretPosition);
  }

  function getCurrentLine(text: string, cursorPos: number): string {
    const lines = text.substring(0, cursorPos).split('\n');
    return lines[lines.length - 1];
  }
  
  // Handle global keydown for command/shortcuts
  function handleGlobalKeydown(event: KeyboardEvent) {
    // Only handle Escape for our inline command bar; let global handlers process others
    if (event.key === 'Escape') {
      if (commandBarActive) {
        closeCommandBar();
        event.preventDefault();
        event.stopPropagation();
        return;
      }
      // Do not intercept ESC for general editing; allow global command bar toggle to work
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
              openCommandBarAtCursor();
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
  $: if (editing && editorElement) {
    syncTextareaValue();
  }

  function handleEditorInput(event: Event) {
    // We're using bind:value so editingContent is already updated
    
    // Process live markdown formatting
    if (event.target) {
      processLiveMarkdown(event.target as HTMLTextAreaElement);
    }

    // If the inline slash menu is open, derive the query from the text after the last '/'
  if (commandBarActive && editorElement) {
      updateCommandQueryFromText();
    }
    
    // Notify parent but don't update our own content variable
    dispatch('update', { content: editingContent });
    
    // Render the markdown with debounce
  renderMarkdown();
    updateCaretPosition();
  }
  
  function startEditing() {
    editing = true;
    editingContent = content || '';
    setTimeout(() => {
      if (!editorElement) return;
      editorElement.value = editingContent;
      editorElement.readOnly = false;
      editorElement.setAttribute('contenteditable', 'true');
      editorElement.focus();
      editorElement.selectionStart = editorElement.selectionEnd = editingContent.length;
      editorElement.dispatchEvent(new Event('input', { bubbles: true }));
    }, 30);
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
  function handlePreviewClick(event: MouseEvent | KeyboardEvent) {
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
        updateCaretPosition();
      }
    }, 50);
  }
  
  function showCommandBar() {
    // Always clear state on open
    commandBarActive = true;
    commandText = '';
    filteredSuggestions = [...commandSuggestions];
    selectedSuggestion = 0;
  }

  // Helper to open inline command bar near the current caret position
  function openCommandBarAtCursor() {
    if (!editorElement) return;
  const cursorPos = editorElement.selectionStart || 0;
    const rect = editorElement.getBoundingClientRect();
    const style = window.getComputedStyle(editorElement);
  const lh = parseInt(style.lineHeight || '0', 10);
  const lineHeight = Number.isFinite(lh) && lh > 0 ? lh : 20;
    const textUntilCursor = editorElement.value.substring(0, cursorPos);
    const lines = textUntilCursor.split('\n');
    const currentLine = Math.max(1, lines.length);
    const verticalPos = rect.top + currentLine * lineHeight - editorElement.scrollTop;
    // Approximate X near left margin; precise glyph measurement is overkill here
    commandBarPosition = {
      x: rect.left + 20,
      y: verticalPos + 10
    };
    showCommandBar();
  }

  function closeCommandBar() {
    commandBarActive = false;
    commandBarMode = 'menu';
    commandText = '';
    filteredSuggestions = [];
    selectedSuggestion = 0;
    // Refocus editor for accessibility
    if (editorElement) {
      editorElement.focus();
      updateCaretPosition();
    }
  }

  // Update commandText by parsing text around the caret, looking for '/<query>' immediately before the caret
  function updateCommandQueryFromText() {
    if (!editorElement) return;
  if (commandBarMode !== 'slash') return;
    const pos = editorElement.selectionStart || 0;
    const text = editingContent;
    // Find the last '/' before the caret on the same line
    const lineStart = text.lastIndexOf('\n', Math.max(0, pos - 1)) + 1;
    const slashIndex = text.lastIndexOf('/', pos - 1);
    if (slashIndex >= lineStart) {
      const raw = text.substring(slashIndex + 1, pos);
      commandText = raw.toLowerCase();
      // Filter based on current query
      filteredSuggestions = commandSuggestions.filter(s =>
        s.cmd.toLowerCase().includes(commandText) || s.description.toLowerCase().includes(commandText)
      );
      if (!filteredSuggestions.length) {
        filteredSuggestions = [...commandSuggestions];
      }
    } else {
      // Slash was removed; close the menu
      closeCommandBar();
    }
  }

  function handleSuggestionClick(cmd: string) {
    commandText = cmd;
    executeCommand(cmd);
  }

  function executeCommand(cmd: string) {
    // Apply markdown edit inline like Notion/Obsidian
    if (!editorElement) return;
    const selStart = editorElement.selectionStart;
    const selEnd = editorElement.selectionEnd;

    const apply = (newText: string, newSelStart: number, newSelEnd: number) => {
      editingContent = newText;
      // Push to textarea and selection
      editorElement.value = newText;
      editorElement.selectionStart = newSelStart;
      editorElement.selectionEnd = newSelEnd;
      dispatch('update', { content: editingContent });
  renderMarkdown();
    };

    const getLineRange = (text: string, index: number) => {
      let start = text.lastIndexOf('\n', Math.max(0, index - 1));
      start = start === -1 ? 0 : start + 1;
      let end = text.indexOf('\n', index);
      end = end === -1 ? text.length : end;
      return { start, end };
    };

    const replaceLinePrefix = (prefix: string) => {
      const { start, end } = getLineRange(editingContent, selStart);
      const line = editingContent.slice(start, end);
      const stripped = line.replace(/^\s*(#{1,6}\s+|>\s+|- \[ \]\s+|-\s+|\d+\.\s+)?/, '');
      const newLine = `${prefix}${stripped}`;
      const newText = editingContent.slice(0, start) + newLine + editingContent.slice(end);
      const delta = newLine.length - line.length;
      apply(newText, selStart + delta, selEnd + delta);
    };

    const wrapSelection = (left: string, right: string = left) => {
      const before = editingContent.slice(0, selStart);
      const selected = editingContent.slice(selStart, selEnd);
      const after = editingContent.slice(selEnd);
      const newSelected = selected || '';
      const newText = before + left + newSelected + right + after;
      const cursorStart = selStart + left.length;
      const cursorEnd = selEnd + left.length;
      apply(newText, cursorStart, cursorEnd);
    };

    const insertBlock = (block: string) => {
      const { start, end } = getLineRange(editingContent, selStart);
      const prefix = editingContent.slice(0, start);
      const suffix = editingContent.slice(end);
      const sepBefore = start > 0 && editingContent[start - 1] !== '\n' ? '\n' : '';
      const sepAfter = end < editingContent.length && editingContent[end] !== '\n' ? '\n' : '';
      const insertion = `${sepBefore}${block}${sepAfter}`;
      const newText = prefix + insertion + suffix;
      const newCaret = (prefix + insertion).length;
      apply(newText, newCaret, newCaret);
    };

    switch (cmd) {
      case 'h1': replaceLinePrefix('# '); break;
      case 'h2': replaceLinePrefix('## '); break;
      case 'h3': replaceLinePrefix('### '); break;
      case 'bold': wrapSelection('**'); break;
      case 'italic': wrapSelection('*'); break;
      case 'inlinecode': wrapSelection('`'); break;
      case 'quote': replaceLinePrefix('> '); break;
      case 'divider': insertBlock('---'); break;
      case 'list': replaceLinePrefix('- '); break;
      case 'numbered': replaceLinePrefix('1. '); break;
      case 'checklist': replaceLinePrefix('- [ ] '); break;
      case 'code': insertBlock('```\n\n```'); break;
      case 'table': insertBlock(`| Column 1 | Column 2 | Column 3 |\n|---|---|---|\n|  |  |  |`); break;
      case 'link': wrapSelection('[', '](url)'); break;
      case 'image': wrapSelection('![', '](url)'); break;
      default:
        // Fallback: emit event for parent or backend if needed
        dispatch('command', { command: cmd });
    }

    closeCommandBar();
  }

  // Remove the "/query" trigger then apply command transformation
  function applyInlineCommand(cmd: string) {
    if (!editorElement) return;
    const pos = editorElement.selectionStart || 0;
    const text = editingContent;
    const lineStart = text.lastIndexOf('\n', Math.max(0, pos - 1)) + 1;
    const slashIndex = text.lastIndexOf('/', pos - 1);
    const validSlash = slashIndex >= lineStart;
    let nextSel = pos;
    if (validSlash) {
      const before = text.slice(0, slashIndex);
      const after = text.slice(pos);
      const newText = before + after;
      // Update textarea and selection: caret at slashIndex
      editingContent = newText;
      editorElement.value = newText;
      editorElement.selectionStart = editorElement.selectionEnd = slashIndex;
      nextSel = slashIndex;
      dispatch('update', { content: editingContent });
  renderMarkdown();
    }
    // Apply transformation at the updated caret
    executeCommand(cmd);
  }
  
  // Filter suggestions as user types
  $: {
    if (commandText) {
      filteredSuggestions = commandSuggestions.filter((s: CommandSuggestion) =>
        s.cmd.toLowerCase().includes(commandText.toLowerCase()) ||
        s.description.toLowerCase().includes(commandText.toLowerCase())
      );
    } else {
      filteredSuggestions = [...commandSuggestions];
    }
  }
  
  // Handle clicks outside the editor
  function handleDocumentClick(event: MouseEvent) {
    const target = event.target as HTMLElement;
    // Ignore clicks inside the editor textarea or command bar
    if (target.closest('textarea') || target.closest('.command-bar')) return;
    if (commandBarActive) closeCommandBar();
  }

  function ensureMirror() {
    if (!mirrorElement) {
      mirrorElement = document.createElement('div');
      mirrorElement.className = 'mdx-caret-mirror';
      Object.assign(mirrorElement.style, {
        position: 'fixed',
        top: '-9999px',
        left: '0',
        visibility: 'hidden',
        whiteSpace: 'pre-wrap',
        wordWrap: 'break-word'
      } as CSSStyleDeclaration);
      document.body.appendChild(mirrorElement);
    }
  }

  function updateCaretPosition() {
    if (!editing || !editorElement) return;
    ensureMirror();
    if (!mirrorElement) return;
    const ta = editorElement;
    const selStart = ta.selectionStart;
    const fullText = ta.value;
    const upto = fullText.substring(0, selStart);
    const lines = upto.split('\n');
    const currentLineIndex = lines.length - 1;
    const currentLine = lines[currentLineIndex];
    
    // Get actual line height by checking if line starts with heading markers
    let lineHeightMultiplier = 1.6; // base line-height from textarea
    let fontSize = 16; // base font size
    
    // Check if current line is a heading
    const trimmedLine = currentLine.trim();
    if (trimmedLine.startsWith('# ')) {
      fontSize = 32; // h1
      lineHeightMultiplier = 1.2;
    } else if (trimmedLine.startsWith('## ')) {
      fontSize = 24; // h2
      lineHeightMultiplier = 1.3;
    } else if (trimmedLine.startsWith('### ')) {
      fontSize = 20; // h3
      lineHeightMultiplier = 1.4;
    }
    
    const lineHeight = fontSize * lineHeightMultiplier;
    
    // Use mirror element for width calculation
    const style = window.getComputedStyle(ta);
    const padLeft = parseFloat(style.paddingLeft) || 32;
    const padTop = parseFloat(style.paddingTop) || 32;
    
    mirrorElement.style.width = ta.clientWidth + 'px';
    mirrorElement.style.fontSize = fontSize + 'px';
    ['font-family','font-weight','font-style','letter-spacing','text-transform','padding','border','box-sizing','tab-size'].forEach(p => {
      const v = style.getPropertyValue(p);
      if (v) mirrorElement!.style.setProperty(p, v);
    });
    
    // Measure width to cursor on current line
    const textUpToCursor = currentLine.replace(/^#{1,6}\s+/, ''); // Remove heading markers for width calc
    const escaped = (textUpToCursor || '\u200b').replace(/</g,'&lt;').replace(/>/g,'&gt;').replace(/ /g,'&nbsp;');
    mirrorElement.innerHTML = `<span>${escaped}</span>`;
    const span = mirrorElement.querySelector('span');
    const spanRect = span ? span.getBoundingClientRect() : mirrorElement.getBoundingClientRect();
    
    const taRect = ta.getBoundingClientRect();
    const scrollLeft = ta.scrollLeft;
    const scrollTop = ta.scrollTop;
    
    // Calculate cumulative height of all previous lines more accurately
    let yOffset = 0;
    for (let i = 0; i < currentLineIndex; i++) {
      const line = lines[i].trim();
      if (line.startsWith('# ')) {
        yOffset += 32 * 1.2; // h1 line height
      } else if (line.startsWith('## ')) {
        yOffset += 24 * 1.3; // h2 line height
      } else if (line.startsWith('### ')) {
        yOffset += 20 * 1.4; // h3 line height
      } else {
        yOffset += 16 * 1.6; // regular line height
      }
    }
    
    // Calculate exact position without offset adjustments
    const x = spanRect.width - scrollLeft + padLeft + taRect.left;
    const y = yOffset - scrollTop + padTop + taRect.top;
    
    // Store caret height based on current line's font size (full line height)
    const caretHeight = lineHeight;
    caretPosition = { x, y, height: caretHeight };
  }
</script>

<style>
  .custom-caret {
    position: fixed;
    width: 2.5px;
    /* height is set dynamically via inline style based on current line */
    background: rgba(255, 255, 255, 0.95);
    pointer-events: none;
    animation: caret-blink 1.0s steps(2, start) infinite;
    z-index: 60;
    border-radius: 0.5px;
    box-shadow: 0 0 8px rgba(255, 255, 255, 0.3);
  }
  @keyframes caret-blink { 
    50% { opacity: 0; } 
  }
</style>

<div class="flex flex-col h-full w-full min-h-0 flex-1 bg-black markdown-container relative">
  <!-- Single editor view with live preview rendering -->
  <div class="flex-1 min-h-0 h-full flex flex-col relative">
  <textarea
      bind:this={editorElement}
    class="flex-1 w-full h-full min-h-0 max-w-none p-8 text-base leading-relaxed text-gray-200 selection:bg-white/20 bg-transparent resize-none outline-none caret-transparent font-sans relative z-10 mdx-editor"
      class:mdx-hidden-text={editing}
      bind:value={editingContent}
      on:input={handleEditorInput}
      on:keydown={handleKeydown}
      on:scroll={(e) => {
        // Keep preview scroll in sync with textarea
        const t = e.target;
        if (previewElement && t) {
          // @ts-ignore - DOM typing in template context
          previewElement.scrollTop = t.scrollTop;
          // @ts-ignore
          previewElement.scrollLeft = t.scrollLeft;
        }
      }}
      placeholder="Start typing... Use # for headings, **bold**, *italic*, `code`"
      spellcheck="false"
      tabindex="0"
    ></textarea>
    
    <!-- Live preview overlay (subtle, under text) -->
  <div bind:this={previewElement} class="absolute inset-0 p-8 overflow-auto text-gray-300 mdx-preview-layer pointer-events-none">
      <div class="prose prose-invert max-w-none mdx-preview" class:mdx-live={editing}>
        {@html renderedContent}
      </div>
    </div>
    <!-- Custom caret with dynamic height -->
    {#if editing}
      <div class="custom-caret" style="transform: translate({caretPosition.x}px, {caretPosition.y}px); height: {caretPosition.height || 20}px;"></div>
    {/if}
  </div>

  <!-- Command bar disabled - using global CommandBar component instead -->
  {#if false}
    <div style="display: none;"></div>
  {/if}
</div>

