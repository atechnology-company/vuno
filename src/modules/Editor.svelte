<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, highlightActiveLine } from '@codemirror/view';
  import { defaultKeymap, indentWithTab, history } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, HighlightStyle } from '@codemirror/language';
  import { closeBrackets, autocompletion, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { tags } from '@lezer/highlight';
  import { autoCloseTags } from '@codemirror/lang-html';

  // Create a terminal-inspired theme (will be overridden by oneDark if present)
  const terminalTheme = EditorView.theme({
    "&": {
      backgroundColor: "#000000",
      color: "#ffffff"
    },
    ".cm-content": {
      caretColor: "#ffffff"
    },
    ".cm-cursor, .cm-dropCursor": { 
      borderLeftColor: "#ffffff"
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
      backgroundColor: "#333333"
    },
    ".cm-panels": {
      backgroundColor: "#000000", 
      color: "#ffffff"
    },
    ".cm-panels.cm-panels-top": {
      borderBottom: "1px solid #333333"
    },
    ".cm-panels.cm-panels-bottom": {
      borderTop: "1px solid #333333"
    },
    ".cm-searchMatch": {
      backgroundColor: "#333333",
      outline: "1px solid #666666"
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "#666666"
    },
    ".cm-activeLine": { backgroundColor: "#111111" },
    ".cm-selectionMatch": { backgroundColor: "#333333" },
    ".cm-matchingBracket, .cm-nonmatchingBracket": {
      backgroundColor: "#333333",
      outline: "1px solid #666666"
    },
    ".cm-gutters": {
      backgroundColor: "#000000",
      color: "#999999",
      border: "none"
    },
    ".cm-lineNumbers .cm-gutterElement": { color: "#999999" },
    ".cm-foldPlaceholder": {
      backgroundColor: "#333333",
      border: "none",
      color: "#ffffff"
    },
    ".cm-tooltip": {
      border: "1px solid #444444",
      backgroundColor: "#0a0a0a",
      boxShadow: "0 4px 16px rgba(0, 0, 0, 0.8)"
    },
    ".cm-tooltip .cm-tooltip-arrow:before": {
      borderTopColor: "#444444",
      borderBottomColor: "#444444"
    },
    ".cm-tooltip .cm-tooltip-arrow:after": {
      borderTopColor: "#0a0a0a",
      borderBottomColor: "#0a0a0a"
    },
    ".cm-tooltip-autocomplete": {
      backgroundColor: "rgba(10, 10, 10, 0.95)",
      border: "1px solid #444444",
      boxShadow: "0 8px 24px rgba(0, 0, 0, 0.9)",
      backdropFilter: "blur(16px)",
      borderRadius: "12px",
      overflow: "hidden",
      "& > ul": {
        maxHeight: "180px",
        overflowY: "auto",
        overflowX: "hidden",
        backgroundColor: "transparent",
        margin: "8px 0",
        scrollBehavior: "smooth",
        "&::-webkit-scrollbar": {
          width: "6px"
        },
        "&::-webkit-scrollbar-track": {
          background: "transparent"
        },
        "&::-webkit-scrollbar-thumb": {
          background: "rgba(255, 255, 255, 0.2)",
          borderRadius: "3px"
        }
      },
      "& > ul > li": {
        padding: "12px 16px",
        color: "#888888",
        backgroundColor: "transparent",
        transition: "all 0.2s cubic-bezier(0.4, 0, 0.2, 1)",
        fontSize: "13px",
        fontFamily: "monospace",
        position: "relative",
        "&:first-child": {
          opacity: "0.4",
          transform: "scale(0.92) perspective(100px) rotateX(8deg)",
          filter: "blur(0.5px)"
        },
        "&:last-child": {
          opacity: "0.4",
          transform: "scale(0.92) perspective(100px) rotateX(-8deg)",
          filter: "blur(0.5px)"
        }
      },
      "& > ul > li[aria-selected]": {
        backgroundColor: "rgba(255, 255, 255, 0.1)",
        color: "#ffffff",
        outline: "none",
        border: "1px solid rgba(255, 255, 255, 0.15)",
        borderRadius: "8px",
        transform: "scale(1.02)",
        boxShadow: "0 4px 12px rgba(255, 255, 255, 0.1), inset 0 1px 0 rgba(255, 255, 255, 0.1)",
        fontWeight: "500"
      }
    },
    ".cm-tooltip-autocomplete ul li:hover": {
      backgroundColor: "rgba(255, 255, 255, 0.05) !important",
      transform: "scale(0.98) !important"
    }
  }, { dark: true });

  // Terminal highlighting style
  const terminalHighlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: "#ffffff" },
    { tag: tags.operator, color: "#cccccc" },
    { tag: tags.variableName, color: "#ffffff" },
    { tag: tags.typeName, color: "#cccccc" },
    { tag: tags.propertyName, color: "#cccccc" },
    { tag: tags.comment, color: "#999999" },
    { tag: tags.string, color: "#cccccc" },
    { tag: tags.number, color: "#ffffff" },
    { tag: tags.meta, color: "#cccccc" },
    { tag: tags.definitionKeyword, color: "#ffffff" },
    { tag: tags.special(tags.variableName), color: "#ffffff" },
    { tag: tags.bracket, color: "#cccccc" },
    { tag: tags.tagName, color: "#cccccc" },
    { tag: tags.attributeName, color: "#cccccc" },
    { tag: tags.heading, color: "#ffffff" },
    { tag: tags.quote, color: "#cccccc" },
    { tag: tags.link, color: "#ffffff" },
    { tag: tags.atom, color: "#ffffff" },
    { tag: tags.definition(tags.variableName), color: "#ffffff" },
    { tag: tags.className, color: "#ffffff" },
    { tag: tags.punctuation, color: "#ffffff" },
    { tag: tags.bool, color: "#ffbc7a" },
    { tag: tags.function(tags.variableName), color: "#62aeef" }
  ]);
  
  // Combine theme and highlight style
  const terminalThemeExtension = [terminalTheme, syntaxHighlighting(terminalHighlightStyle)];

  // Optional: one dark theme for better defaults
  import { oneDark } from '@codemirror/theme-one-dark';
  
  // Import all language extensions
  import { javascript } from '@codemirror/lang-javascript';
  import { markdown } from '@codemirror/lang-markdown';
  import { python } from '@codemirror/lang-python';
  import { rust } from '@codemirror/lang-rust';
  import { html } from '@codemirror/lang-html';
  import { css } from '@codemirror/lang-css';
  import { json } from '@codemirror/lang-json';
  import { cpp } from '@codemirror/lang-cpp';
  import { java } from '@codemirror/lang-java';
  import { sql } from '@codemirror/lang-sql';

  // Props and methods interface
  interface EditorProps {
    mode?: 'code' | 'markdown';
    content?: string;
    language?: string;
    bufferId?: number | null;
    readOnly?: boolean;
    focus?: () => boolean;
    getContent?: () => string;
    searchText?: (query: string) => void;
    editor?: any;
    markdown?: boolean;
    ch?: number;
  }

  // Props with defaults
  export const mode: EditorProps['mode'] = 'code';
  export let content: EditorProps['content'] = '';
  export let language: EditorProps['language'] = 'javascript';
  export let bufferId: EditorProps['bufferId'] = null;
  export let readOnly: EditorProps['readOnly'] = false;
  
  let element: HTMLElement;
  let view: EditorView;
  let languageCompartment = new Compartment();
  let editorReadOnly = new Compartment();
  let themeCompartment = new Compartment();
  let statusMessage = '';

  const dispatch = createEventDispatcher();

  // Get language extension synchronously with all loaded languages
  function getLanguageExtensionSync(lang: string) {
    switch (lang.toLowerCase()) {
      case 'javascript':
      case 'js':
        return javascript();
      case 'typescript':
      case 'ts':
        return javascript({ typescript: true });
      case 'jsx':
        return javascript({ jsx: true });
      case 'tsx':
        return javascript({ jsx: true, typescript: true });
      case 'markdown':
      case 'md':
        return markdown();
      case 'python':
      case 'py':
        return python();
      case 'rust':
      case 'rs':
        return rust();
      case 'html':
      case 'htm':
        return html({ autoCloseTags: true });
      case 'css':
        return css();
      case 'json':
        return json();
      case 'cpp':
      case 'c++':
        return cpp();
      case 'java':
        return java();
      case 'sql':
        return sql();
      default:
        return javascript(); // Fallback to javascript
    }
  }
  
  // Alias for backwards compatibility
  function getLanguageExtension(lang: string) {
    return getLanguageExtensionSync(lang);
  }

  onMount(() => {
    try {
      // If language is markdown, we'll use a Tailwind textarea/content editor instead of CodeMirror
      if (language && language.toLowerCase() === 'markdown') {
        // No CodeMirror initialization for markdown mode. The DOM for markdown editor is rendered
        // and we rely on textarea event handlers below to dispatch changes.
        dispatch('mount', { editor: null });
        return;
      }

      const languageExtension = getLanguageExtension(language || 'javascript');

      // Ensure element exists before creating view
      if (!element) {
        return;
      }

  // Use oneDark theme for proper syntax highlighting
  const themeExtension = [oneDark];

      view = new EditorView({
        state: EditorState.create({
          doc: content,
          extensions: [
            // Language must come first for proper highlighting
            languageCompartment.of(languageExtension),
            // Other keymaps with lower precedence
            keymap.of([...defaultKeymap, ...closeBracketsKeymap, indentWithTab]),
            syntaxHighlighting(defaultHighlightStyle, { fallback: true }),
            themeCompartment.of(themeExtension),
            highlightActiveLine(),
            closeBrackets(),
            autoCloseTags,
            autocompletion(),
            history(),
            editorReadOnly.of(EditorView.editable.of(!readOnly)),
            EditorView.updateListener.of(update => {
              if (update.docChanged) {
                content = update.state.doc.toString();

                // Get cursor position
                const selection = update.state.selection.main;
                const doc = update.state.doc;
                const line = doc.lineAt(selection.head);
                const lineNumber = line.number;
                const column = selection.head - line.from;

                dispatch('change', {
                  content,
                  lineCount: doc.lines,
                  cursorPosition: { line: lineNumber - 1, column: column }
                });

                // If we have a buffer ID, update the buffer on the Rust side
                if (bufferId !== null) {
                  // In browser-only mode we don't sync with Rust backend here; parent can call invoke as needed
                }
              }

              // Track selection changes even without content changes
              if (update.selectionSet) {
                const selection = update.state.selection.main;
                const doc = update.state.doc;
                const line = doc.lineAt(selection.head);
                const lineNumber = line.number;
                const column = selection.head - line.from;

                dispatch('change', {
                  content,
                  lineCount: doc.lines,
                  cursorPosition: { line: lineNumber - 1, column: column }
                });
              }
            }),
          ]
        }),
        parent: element
      });

      // Add a click handler to ensure editor gets focus when clicked
      element.addEventListener('click', () => {
        if (view) view.focus();
      });

      // Initial focus after creation
      setTimeout(() => {
        if (view) view.focus();
      }, 100);

      dispatch('mount', { editor: view });
    } catch (err) {
      console.error('Failed to initialize editor:', err);
      statusMessage = 'Error initializing editor';
    }
  });

  // --- Markdown-mode helpers (Tailwind textarea + inline command bar) ---
  // We'll create a lightweight inline editor when language is 'markdown'.
  import { tick } from 'svelte';
  let textareaEl: HTMLTextAreaElement | null = null;
  let mdContent = content || '';
  let commandBarActive = false;
  let commandText = '';
  let filteredSuggestions: any[] = [];
  const mdCommands = [
    { cmd: 'h1', icon: 'H1' },
    { cmd: 'h2', icon: 'H2' },
    { cmd: 'h3', icon: 'H3' },
    { cmd: 'bold', icon: 'B' },
    { cmd: 'italic', icon: 'I' },
    { cmd: 'code', icon: '{ }' },
    { cmd: 'inlinecode', icon: '`' },
    { cmd: 'quote', icon: '"' },
    { cmd: 'divider', icon: '\u2014' },
    { cmd: 'list', icon: '\u2022' },
    { cmd: 'numbered', icon: '1.' },
    { cmd: 'checklist', icon: '\u2611' },
    { cmd: 'table', icon: '\u229e' },
    { cmd: 'image', icon: '\ud83d\uddbc' },
    { cmd: 'link', icon: '\ud83d\udd17' }
  ];

  let shiftPressed = false;
  let shiftKeyDownTime = 0;

  function onMdInput(e: Event) {
    const t = e.target as HTMLTextAreaElement;
    mdContent = t.value;
    dispatch('change', { content: mdContent, lineCount: mdContent.split('\n').length, cursorPosition: { line: 0, column: 0 } });
    // If bufferId is set, sync to backend in a non-blocking way
  // Parent component handles buffer syncing (debounced autosave)
  }

  function openCommandBarAtCursor() {
    commandBarActive = true;
    commandText = '';
    filteredSuggestions = [...mdCommands];
    tick().then(() => {
      const el = document.querySelector('.inline-md-command input') as HTMLInputElement | null;
      if (el) el.focus();
    });
  }

  function closeCommandBar() {
    commandBarActive = false;
    commandText = '';
    filteredSuggestions = [];
  }

  function handleMdKeydown(e: KeyboardEvent) {
    if (e.key === 'Shift') {
      shiftPressed = true;
      shiftKeyDownTime = Date.now();
    }

    if (e.key === 'Tab') {
      e.preventDefault();
      // Insert two spaces
      if (!textareaEl) return;
      const start = textareaEl.selectionStart;
      const end = textareaEl.selectionEnd;
      const newValue = mdContent.slice(0, start) + '  ' + mdContent.slice(end);
      mdContent = newValue;
      textareaEl.value = mdContent;
      textareaEl.selectionStart = textareaEl.selectionEnd = start + 2;
      onMdInput(new Event('input') as Event);
    }
    // Save shortcut
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 's') {
      e.preventDefault();
      // Dispatch a save event; parent can call Tauri save
      dispatch('save');
    }
  }

  function handleMdKeyup(e: KeyboardEvent) {
    if (e.key === 'Shift') {
      const duration = Date.now() - shiftKeyDownTime;
      shiftPressed = false;
      if (duration < 200) {
        // treat as tap -> toggle inline command bar
        if (commandBarActive) closeCommandBar(); else openCommandBarAtCursor();
      }
    }
  }

  function executeMdCommand(cmd: string) {
    if (!textareaEl) return;
    const start = textareaEl.selectionStart;
    const end = textareaEl.selectionEnd;
    const before = mdContent.slice(0, start);
    const selected = mdContent.slice(start, end);
    const after = mdContent.slice(end);

    const apply = (newText: string, cursorPos: number) => {
      mdContent = newText;
      if (!textareaEl) return;
      textareaEl.value = mdContent;
      textareaEl.selectionStart = textareaEl.selectionEnd = cursorPos;
      dispatch('change', { content: mdContent });
      closeCommandBar();
    };

    const getLineRange = (text: string, index: number) => {
      let s = text.lastIndexOf('\n', Math.max(0, index - 1));
      s = s === -1 ? 0 : s + 1;
      let e = text.indexOf('\n', index);
      e = e === -1 ? text.length : e;
      return { start: s, end: e };
    };

    const replaceLinePrefix = (prefix: string) => {
      const { start: lineStart, end: lineEnd } = getLineRange(mdContent, start);
      const line = mdContent.slice(lineStart, lineEnd);
      const stripped = line.replace(/^\s*(#{1,6}\s+|>\s+|- \[ \]\s+|-\s+|\d+\.\s+)?/, '');
      
      // Only transform if there's actual visible text (not just whitespace)
      const hasVisibleContent = stripped.trim().length > 0;
      const hasSelection = selected.length > 0;
      
      if (!hasVisibleContent && !hasSelection) {
        // Don't add prefix on empty lines - user needs to type first
        return;
      }
      
      const newLine = `${prefix}${stripped}`;
      const newText = mdContent.slice(0, lineStart) + newLine + mdContent.slice(lineEnd);
      const delta = newLine.length - line.length;
      apply(newText, lineStart + prefix.length + (selected.length ? selected.length : 0));
    };

    const wrapSelection = (left: string, right: string = left) => {
      const newText = before + left + (selected || '') + right + after;
      const cursor = start + left.length + (selected ? selected.length : 0);
      apply(newText, cursor);
    };

    const insertBlock = (block: string) => {
      const { start: lineStart, end: lineEnd } = getLineRange(mdContent, start);
      const prefix = mdContent.slice(0, lineStart);
      const suffix = mdContent.slice(lineEnd);
      const sepBefore = lineStart > 0 && mdContent[lineStart - 1] !== '\n' ? '\n' : '';
      const sepAfter = lineEnd < mdContent.length && mdContent[lineEnd] !== '\n' ? '\n' : '';
      const insertion = `${sepBefore}${block}${sepAfter}`;
      const newText = prefix + insertion + suffix;
      const newCaret = (prefix + insertion).length;
      apply(newText, newCaret);
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
        break;
    }
  }

  // Language loading when language changes
  $: if (view && language) {
    const languageExtension = getLanguageExtensionSync(language);
    view.dispatch({
      effects: languageCompartment.reconfigure(languageExtension)
    });
  }

  $: if (view && readOnly !== undefined) {
    view.dispatch({
      effects: editorReadOnly.reconfigure(EditorView.editable.of(!readOnly))
    });
  }
  
  // Update content if it changes externally - optimize with debouncing if needed
  $: if (view && content !== view.state.doc.toString()) {
    view.dispatch({
      changes: { from: 0, to: view.state.doc.length, insert: content }
    });
  }

  onDestroy(() => {
    if (view) {
      view.destroy();
    }
  });

  // Public methods that can be called from the parent component
  export function focus() {
    if (view) {
      console.log('Focusing editor view');
      try {
        view.focus();
        return true;
      } catch (e) {
        console.error('Error focusing editor:', e);
        return false;
      }
    }
    return false;
  }

  export function getContent() {
    return view ? view.state.doc.toString() : content;
  }

  export function searchText(query: string) {
    if (!view) return;
    
    // In a real implementation, use the search addon to highlight matches
    console.log('Searching for:', query);
  }

  export function insertAtCursor(text: string) {
    if (view) {
      try {
        const sel = view.state.selection.main;
        view.dispatch({ changes: { from: sel.from, to: sel.to, insert: text } });
        // Move cursor to the end of inserted text
        const pos = sel.from + text.length;
        view.dispatch({ selection: { anchor: pos } });
        view.focus();
        return true;
      } catch (e) {
        console.error('Error inserting text into editor:', e);
        return false;
      }
    }
    return false;
  }
</script>

{#if !language || language.toLowerCase() !== 'markdown'}
  <!-- Code editor container: use flex layout to fill available vertical space -->
  <div class="flex-1 flex w-full h-full min-h-0 overflow-hidden font-mono" bind:this={element}></div>
{/if}

<style>
  /* Tailwind CSS migration: All custom CSS removed */
</style>

{#if language && language.toLowerCase() === 'markdown'}
  <div class="markdown-container flex-1 flex flex-col w-full h-full">
    <textarea
      bind:this={textareaEl}
      class="flex-1 w-full min-h-0 px-8 py-7 bg-black text-white resize-none outline-none caret-white font-sans text-[14px] leading-[1.6] tracking-[0.005em]"
      spellcheck="false"
      value={mdContent}
      on:input={onMdInput}
      on:keydown={handleMdKeydown}
      on:keyup={handleMdKeyup}
      placeholder="Start writing in markdown..."
    ></textarea>

    {#if commandBarActive}
      <div class="inline-md-command absolute z-50 bg-[#0b0f14] border border-gray-800 rounded shadow-lg p-2 w-72" style="left: 40px; top: 40px;">
        <input class="w-full p-2 bg-transparent outline-none text-white" bind:value={commandText} on:input={(e) => { filteredSuggestions = mdCommands.filter(c => c.cmd.includes(commandText) || c.icon.includes(commandText)); }} />
            <div class="mt-2 max-h-40 overflow-y-auto">
              {#each filteredSuggestions as s}
                <button
                  type="button"
                  class="w-full text-left flex items-center gap-2 p-2 hover:bg-gray-800 focus:outline-none focus:ring-2 focus:ring-indigo-500 rounded"
                  on:click={() => executeMdCommand(s.cmd)}
                  on:keydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); executeMdCommand(s.cmd); } }}
                >
                  <div class="w-6 h-6 flex items-center justify-center bg-gray-700 rounded text-xs">{s.icon}</div>
                  <div class="text-sm text-white">{s.cmd}</div>
                </button>
              {/each}
            </div>
      </div>
    {/if}
  </div>
{/if}