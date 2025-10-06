import { writable, get } from 'svelte/store';

export interface AppState {
  mode: 'code' | 'markdown';
  content: string;
  currentFile: string;
  commandMode: boolean;
  commandText: string;
  showSuggestions: boolean;
  suggestions: string[];
  chatMessages: Array<{ role: string; content: string; displayContent?: string }>;
  showChat: boolean;
  showHelp: boolean;
  apiKey: string;
  bufferId: number | null;
  showMarkdownPreview: boolean;
  editorLanguage: string;
  searchQuery: string;
  isSearchOpen: boolean;
  searchResults: Array<{ line: number, text: string }>;
  showSettings: boolean;
  editorSettings: {
    fontSize: number;
    tabSize: number;
    lineWrapping: boolean;
    theme: string;
    showLineNumbers: boolean;
    fontFamily: string;
  };
  isLoading: boolean;
  statusMessage: string;
  currentLineCount: number;
  currentCursorPosition: { line: number; column: number };
  isSaving: boolean;
  saveFilename: string;
  commandBarTitle: string;
  commandBarPlaceholder: string;
  isAiLoading: boolean;
  isOnline: boolean;
  commandExecutionInProgress: boolean;
  hasUnsavedChanges: boolean;
  pendingAction: string | null;
  pendingArgs: string | null;
  showSaveConfirmation: boolean;
  showRedPulse: boolean;
  isFirstRun: boolean;
  welcomeDisplayed: boolean;
  showTutorial: boolean;
  isRecovering: boolean;
}

const initialState: AppState = {
  mode: 'code',
  content: '',
  currentFile: '',
  commandMode: false,
  commandText: '',
  showSuggestions: true,
  suggestions: [],
  chatMessages: [],
  showChat: false,
  showHelp: false,
  apiKey: '',
  bufferId: null,
  showMarkdownPreview: false,
  editorLanguage: 'plaintext',
  searchQuery: '',
  isSearchOpen: false,
  searchResults: [],
  showSettings: false,
  editorSettings: {
    fontSize: 14,
    tabSize: 2,
    lineWrapping: true,
    theme: 'dark',
    showLineNumbers: true,
    fontFamily: 'JetBrains Mono'
  },
  isLoading: true,
  statusMessage: '',
  currentLineCount: 0,
  currentCursorPosition: { line: 1, column: 1 },
  isSaving: false,
  saveFilename: '',
  commandBarTitle: 'What would you like to do today?',
  commandBarPlaceholder: 'Type a command or press Enter',
  isAiLoading: false,
  isOnline: navigator.onLine,
  commandExecutionInProgress: false,
  hasUnsavedChanges: false,
  pendingAction: null,
  pendingArgs: null,
  showSaveConfirmation: false,
  showRedPulse: false,
  isFirstRun: false,
  welcomeDisplayed: false,
  showTutorial: false,
  isRecovering: false
};

function createAppStore() {
  const { subscribe, set, update: originalUpdate } = writable<AppState>(initialState);
  
  // Create a custom update function that logs state changes
  const update = (updater: (state: AppState) => AppState) => {
    const before = get({ subscribe });
    const result = originalUpdate(state => {
      const newState = updater(state);
      console.log('State updated:', {
        commandMode: {
          before: before.commandMode,
          after: newState.commandMode
        },
        showHelp: {
          before: before.showHelp,
          after: newState.showHelp
        }
      });
      return newState;
    });
    return result;
  };
  
  return {
    subscribe,
    set,
    update,
    
    // Actions
    setMode: (mode: 'code' | 'markdown') => update(state => ({ ...state, mode })),
    setContent: (content: string) => update(state => ({ ...state, content })),
    setCurrentFile: (currentFile: string) => update(state => ({ ...state, currentFile })),
    toggleCommandMode: () => update(state => ({ ...state, commandMode: !state.commandMode })),
    setCommandText: (commandText: string) => update(state => ({ ...state, commandText })),
    setSuggestions: (suggestions: string[]) => update(state => ({ ...state, suggestions })),
    toggleChat: () => update(state => ({ ...state, showChat: !state.showChat })),
    toggleHelp: () => update(state => ({ ...state, showHelp: !state.showHelp })),
    setApiKey: (apiKey: string) => update(state => ({ ...state, apiKey })),
    setBufferId: (bufferId: number | null) => update(state => ({ ...state, bufferId })),
    setEditorLanguage: (editorLanguage: string) => update(state => ({ ...state, editorLanguage })),
    setStatusMessage: (statusMessage: string) => update(state => ({ ...state, statusMessage })),
    setHasUnsavedChanges: (hasUnsavedChanges: boolean) => update(state => ({ ...state, hasUnsavedChanges })),
    
    // Complex actions
    startSaving: (filename: string) => update(state => ({
      ...state,
      isSaving: true,
      saveFilename: filename,
      commandBarTitle: 'Save as...'
    })),
    
    finishSaving: () => update(state => ({
      ...state,
      isSaving: false,
      saveFilename: '',
      commandBarTitle: 'What would you like to do today?',
      hasUnsavedChanges: false
    })),
    
    showSaveConfirmationDialog: (action: string, args: string) => update(state => ({
      ...state,
      showSaveConfirmation: true,
      pendingAction: action,
      pendingArgs: args,
      commandBarTitle: 'Would you like to save first?'
    })),
    
    clearSaveConfirmation: () => update(state => ({
      ...state,
      showSaveConfirmation: false,
      pendingAction: null,
      pendingArgs: null,
      commandBarTitle: 'What would you like to do today?'
    })),
    
    // Reset state
    reset: () => set(initialState)
  };
}

export const appState = createAppStore(); 