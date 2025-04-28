<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import { EditorState, Compartment } from '@codemirror/state';
  import { EditorView, keymap, highlightActiveLine, highlightSpecialChars } from '@codemirror/view';
  import { defaultKeymap, indentWithTab, history, historyKeymap } from '@codemirror/commands';
  import { syntaxHighlighting, defaultHighlightStyle, HighlightStyle } from '@codemirror/language';
  import { searchKeymap, highlightSelectionMatches } from '@codemirror/search';
  import { closeBrackets, closeBracketsKeymap } from '@codemirror/autocomplete';
  import { oneDark } from '@codemirror/theme-one-dark';
  import { tags } from '@lezer/highlight';

  // Create a material-inspired theme
  const materialTheme = EditorView.theme({
    "&": {
      backgroundColor: "#1e1e2e",
      color: "#e1e4e8"
    },
    ".cm-content": {
      caretColor: "#6f98e3"
    },
    ".cm-cursor, .cm-dropCursor": { 
      borderLeftColor: "#6f98e3"
    },
    "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection": {
      backgroundColor: "#364a7d"
    },
    ".cm-panels": {
      backgroundColor: "#1a1a27", 
      color: "#e1e4e8"
    },
    ".cm-panels.cm-panels-top": {
      borderBottom: "1px solid #292a3a"
    },
    ".cm-panels.cm-panels-bottom": {
      borderTop: "1px solid #292a3a"
    },
    ".cm-searchMatch": {
      backgroundColor: "#3a5485",
      outline: "1px solid #446699"
    },
    ".cm-searchMatch.cm-searchMatch-selected": {
      backgroundColor: "#4b5f8b"
    },
    ".cm-activeLine": { backgroundColor: "#232332" },
    ".cm-selectionMatch": { backgroundColor: "#2e385f" },
    ".cm-matchingBracket, .cm-nonmatchingBracket": {
      backgroundColor: "#364a7d",
      outline: "1px solid #5278c9"
    },
    ".cm-gutters": {
      backgroundColor: "#1a1a27",
      color: "#637286",
      border: "none"
    },
    ".cm-lineNumbers .cm-gutterElement": { color: "#637286" },
    ".cm-foldPlaceholder": {
      backgroundColor: "#2e3750",
      border: "none",
      color: "#ddd"
    },
    ".cm-tooltip": {
      border: "1px solid #292a3a",
      backgroundColor: "#1e1e2e"
    },
    ".cm-tooltip .cm-tooltip-arrow:before": {
      borderTopColor: "#292a3a",
      borderBottomColor: "#292a3a"
    },
    ".cm-tooltip .cm-tooltip-arrow:after": {
      borderTopColor: "#1e1e2e",
      borderBottomColor: "#1e1e2e"
    },
    ".cm-tooltip-autocomplete": {
      "& > ul > li[aria-selected]": {
        backgroundColor: "#2e3750",
        color: "#e1e4e8"
      }
    }
  }, { dark: true });

  // Material highlighting style
  const materialHighlightStyle = HighlightStyle.define([
    { tag: tags.keyword, color: "#9e78ff" },
    { tag: tags.operator, color: "#78c4ff" },
    { tag: tags.variableName, color: "#e1e4e8" },
    { tag: tags.typeName, color: "#78c4ff" },
    { tag: tags.propertyName, color: "#62aeef" },
    { tag: tags.comment, color: "#637286" },
    { tag: tags.string, color: "#7dd3a4" },
    { tag: tags.number, color: "#ffbc7a" },
    { tag: tags.meta, color: "#78c4ff" },
    { tag: tags.definitionKeyword, color: "#9e78ff" },
    { tag: tags.special(tags.variableName), color: "#e1e4e8" },
    { tag: tags.bracket, color: "#78c4ff" },
    { tag: tags.tagName, color: "#62aeef" },
    { tag: tags.attributeName, color: "#7dd3a4" },
    { tag: tags.heading, color: "#62aeef" },
    { tag: tags.quote, color: "#7dd3a4" },
    { tag: tags.link, color: "#9e78ff" },
    { tag: tags.atom, color: "#ffbc7a" },
    { tag: tags.definition(tags.variableName), color: "#e1e4e8" },
    { tag: tags.className, color: "#ffbc7a" },
    { tag: tags.punctuation, color: "#e1e4e8" },
    { tag: tags.bool, color: "#ffbc7a" },
    { tag: tags.function(tags.variableName), color: "#62aeef" }
  ]);
  
  // Combine them into a single extension
  const materialThemeExtension = [materialTheme, syntaxHighlighting(materialHighlightStyle)];
  
  // Only import common languages initially, others will be lazy-loaded
  import { javascript } from '@codemirror/lang-javascript';
  import { markdown } from '@codemirror/lang-markdown';

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
  export let mode: EditorProps['mode'] = 'code';
  export let content: EditorProps['content'] = '';
  export let language: EditorProps['language'] = 'javascript';
  export let bufferId: EditorProps['bufferId'] = null;
  export let readOnly: EditorProps['readOnly'] = false;
  
  let element: HTMLElement;
  let view: EditorView;
  let languageCompartment = new Compartment();
  let editorReadOnly = new Compartment();
  let themeCompartment = new Compartment(); // Keep this for now
  let statusMessage = '';
  // Track loaded languages to avoid duplicate imports
  let loadedLanguages = new Set(['javascript', 'markdown']);

  const dispatch = createEventDispatcher();
  
  // Lazy load language modules as needed
  async function loadLanguage(lang: string) {
    if (loadedLanguages.has(lang)) return;
    
    try {
      switch (lang.toLowerCase()) {
        case 'rust':
        case 'rs':
          const { rust } = await import('@codemirror/lang-rust');
          loadedLanguages.add('rust');
          return rust();
        case 'python':
        case 'py':
          const { python } = await import('@codemirror/lang-python');
          loadedLanguages.add('python');
          return python();
        case 'html':
        case 'htm':
          const { html } = await import('@codemirror/lang-html');
          loadedLanguages.add('html');
          return html();
        case 'css':
          const { css } = await import('@codemirror/lang-css');
          loadedLanguages.add('css');
          return css();
        case 'json':
          const { json } = await import('@codemirror/lang-json');
          loadedLanguages.add('json');
          return json();
        // Lazy load other languages as needed
        default:
          return null; // Will fallback to javascript
      }
    } catch (error) {
      console.error(`Failed to load language module for ${lang}:`, error);
      return null;
    }
  }

  // Get language extension, with fallback and lazy loading support
  async function getLanguageExtensionAsync(lang: string) {
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
      default:
        // Try to dynamically load the language
        const extension = await loadLanguage(lang);
        return extension || javascript(); // Fallback to javascript
    }
  }
  
  // Sync version for initial setup
  function getLanguageExtension(lang: string) {
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
      default:
        return javascript(); // Default fallback
    }
  }

  onMount(() => {
    try {
      const languageExtension = getLanguageExtension(language || 'javascript');
      
      // Ensure element exists before creating view
      if (!element) {
        return;
      }
      
      // Always use material theme
      const themeExtension = materialThemeExtension;
      
      view = new EditorView({
        state: EditorState.create({
          doc: content,
          extensions: [
            // Other keymaps with lower precedence
            keymap.of([...defaultKeymap, indentWithTab]),
            syntaxHighlighting(defaultHighlightStyle),
            themeCompartment.of(themeExtension),
            highlightActiveLine(),
            closeBrackets(),
            history(),
            languageCompartment.of(languageExtension),
            editorReadOnly.of(EditorView.editable.of(!readOnly)),
            EditorView.updateListener.of(update => {
              if (update.docChanged) {
                content = update.state.doc.toString();
                dispatch('update', { content });
                
                // If we have a buffer ID, update the buffer on the Rust side
                if (bufferId !== null) {
                  // In browser-only mode we don't sync with Rust backend
                }
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

  // Async language loading when language changes
  $: if (view && language) {
    (async () => {
      const languageExtension = await getLanguageExtensionAsync(language);
      view.dispatch({
        effects: languageCompartment.reconfigure(languageExtension)
      });
    })();
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
</script>

<div class="editor-container" bind:this={element}></div>

<style>
  .editor-container {
    height: 100%;
    width: 100%;
    overflow: auto;
  }
  
  :global(.cm-editor) {
    height: 100%;
  }
  
  :global(.cm-scroller) {
    overflow: auto;
  }
</style>