<script lang="ts">
  import { onMount, onDestroy, SvelteComponent } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { marked } from 'marked';
  import { get } from 'svelte/store';
  import Editor from '../modules/Editor.svelte';
  import MarkdownRenderer from '../modules/MarkdownRenderer.svelte';
  import CommandBar from '../modules/CommandBar.svelte';
  import Toast from '../modules/Toast.svelte';
  import BottomOutput from '../modules/BottomOutput.svelte';
  import { addOutput, showPanel } from '../modules/outputPanelStore';
  import HelpWindow from '../modules/HelpWindow.svelte';
  import TutorialWindow from '../modules/TutorialWindow.svelte';
  import { addToast } from '../modules/toastStore';
  import { appState } from '../stores/appState';
  import { eventBus } from '../stores/eventBus';
  
  // Import Tauri API
  import { invoke } from '@tauri-apps/api/tauri';
  import { readTextFile, renameFile } from '@tauri-apps/api/fs';
  import { basename, dirname, join } from '@tauri-apps/api/path';

  // Use Tauri-only key handler
  import { TauriKeyHandler } from '../lib/tauriKeyHandler';

  // Initialize variables and state
  let keyHandler = TauriKeyHandler.getInstance();
  let editor: any;
  let fileInput: HTMLInputElement;
  let isTauri = true; // Set to true since we're using Tauri
  let appElement: HTMLElement | null = null;
  let svelteContainer: HTMLElement | null = null;
  let commandBarComponent: any;
  let showCommandBar = false;
  let selectedIndex = 0;
  let filteredCommands: string[] = [];
  let editorComponent: any;
  let allGeneratedSuggestions: string[] = [];
  let isFirstRun = false;
  let showTutorial = false;
  let currentFile: string = '';
  let bufferId: number | null = null;
  let welcomeDisplayed = false;
  let content: string = '';
  let renaming: boolean = false;
  let renameValue: string = '';
  let renameInput: HTMLInputElement | null = null;

  function openGlobalCommandBar() {
    // Ensure suggestions are available and input is clean each time it opens
    if (!allGeneratedSuggestions || allGeneratedSuggestions.length === 0) {
      generateSuggestions();
    }
    appState.update(s => ({
      ...s,
      commandMode: true,
      commandText: '',
      commandBarPlaceholder: 'Type a command or press Enter',
      showSuggestions: true,
      suggestions: allGeneratedSuggestions
    }));
    // Focus the input shortly after
    setTimeout(() => {
      try { focusCommandInput(); } catch {}
    }, 0);
  }

  onMount(async () => {
    // Check if we're in browser environment
    if (typeof window === 'undefined') return;
    
    // Set app element reference for easier manipulation
    appElement = document.getElementById('app');
    svelteContainer = document.getElementById('svelte');

    try {
  // Initialize TauriKeyHandler (no DOM fallback)
      await keyHandler.initialize();
  console.log('TauriKeyHandler initialized successfully');

    // Add command event listener
      if (typeof document !== 'undefined') {
        document.body.addEventListener('command', (event: any) => {
          const { command } = event.detail;
      console.log('Received command:', command);
          executeCommand(command);
        });
        document.body.addEventListener('command-input-focus-requested', () => {
          focusCommandInput();
        });
        // Listen for inline insert-command events from InlineCommandBar
        window.addEventListener('insert-command', (e: any) => {
          const cmd = e.detail?.command;
          if (!cmd) return;
          handleInsertCommand(cmd);
        });
      }

      // Setup CLI file opening
  await setupCliFileOpening();

      // Check for API key
      await checkApiKey();

      // Generate initial command suggestions
      generateSuggestions();

      // Hide loading screen
      hideLoadingScreen();

      // Show tutorial for first run
      if (isFirstRun && !welcomeDisplayed) {
        appState.update(s => ({ ...s, showTutorial: true }));
      }

      // Add global mouse event listeners for window dragging
      if (typeof document !== 'undefined') {
        document.addEventListener('mousemove', handleMouseMove);
        document.addEventListener('mouseup', handleMouseUp);
        
  // Rely on backend key_action events
      }
    } catch (error) {
      console.error('Error during app initialization:', error);
      addToast('Error initializing app', 'error');
    }
  });

  function handleInsertCommand(cmd: string) {
    // If the currently mounted editor exposes insertAtCursor, call it
    try {
      if (editorComponent && typeof editorComponent.insertAtCursor === 'function') {
        editorComponent.insertAtCursor(cmd);
        return;
      }
      // Fallback: if editor is a plain textarea inside MarkdownRenderer, find it and insert
      const activeTextarea = document.querySelector('textarea');
      if (activeTextarea instanceof HTMLTextAreaElement) {
        const ta = activeTextarea as HTMLTextAreaElement;
        const s = ta.selectionStart || 0;
        const e = ta.selectionEnd || 0;
        ta.value = ta.value.slice(0, s) + cmd + ta.value.slice(e);
        ta.focus();
        ta.selectionStart = ta.selectionEnd = s + cmd.length;
        // Also update appState content so the UI stays in sync
        appState.update(s => ({ ...s, content: ta.value, hasUnsavedChanges: true }));
      }
    } catch (err) {
      console.warn('Failed to insert command into editor:', err);
    }
  }

  // Inline rename helpers
  async function startRename() {
    try {
      renaming = true;
      if (currentFile && isTauri) {
        renameValue = await basename(currentFile);
      } else {
        renameValue = currentFile || 'untitled.md';
      }
  // Focus the input after it mounts
  setTimeout(() => { try { renameInput?.focus(); renameInput?.select(); } catch {} }, 0);
    } catch (e) {
      renameValue = currentFile || 'untitled.md';
    }
  }

  function cancelRename() {
    renaming = false;
  }

  async function commitRename() {
    const newName = (renameValue || '').trim();
    if (!newName) {
      renaming = false;
      return;
    }

    // If there's no current file path yet, just set the display name
    if (!currentFile) {
      appState.update(s => ({ ...s, currentFile: newName, statusMessage: `Named file: ${newName}` }));
      addToast(`Named file: ${newName}`, 'success');
      renaming = false;
      return;
    }

    if (!isTauri) {
      appState.update(s => ({ ...s, currentFile: newName }));
      renaming = false;
      return;
    }

    try {
      const dir = await dirname(currentFile);
      const newPath = await join(dir, newName);
      if (newPath === currentFile) {
        renaming = false;
        return;
      }
      await renameFile(currentFile, newPath);
      appState.update(s => ({ ...s, currentFile: newPath, statusMessage: `Renamed to ${newName}` }));
      addToast(`Renamed to ${newName}`, 'success');
    } catch (e) {
      console.error('Rename failed:', e);
      addToast(`Rename failed: ${e}`, 'error');
    } finally {
      renaming = false;
    }
  }

  onDestroy(() => {
    // Check if we're in browser environment
    if (typeof window === 'undefined') return;
    
    // Cleanup TauriKeyHandler
    keyHandler.cleanup();
    
    // Remove global mouse event listeners
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  // No DOM keydown fallback
  });
  // Subscribe to app state
  $: ({
    mode,
    content,
    currentFile,
    commandMode,
    commandText,
    showSuggestions,
    suggestions,
    chatMessages,
    showChat,
    showHelp,
    apiKey,
    bufferId,
    showMarkdownPreview,
    editorLanguage,
    searchQuery,
    isSearchOpen,
    searchResults,
    showSettings,
    editorSettings,
    isLoading,
    statusMessage,
    currentLineCount,
    currentCursorPosition,
    isSaving,
    saveFilename,
    commandBarTitle,
    commandBarPlaceholder,
    isAiLoading,
    isOnline,
    commandExecutionInProgress,
    hasUnsavedChanges,
    pendingAction,
    pendingArgs,
    showSaveConfirmation,
    showRedPulse,
    isFirstRun,
    welcomeDisplayed,
    showTutorial,
    isRecovering
  } = $appState);

  // Function to properly hide and remove loading screen
  function hideLoadingScreen() {
    if (typeof window === 'undefined' || typeof document === 'undefined') return;
    
    try {
      // Get the loading container
      const loadingContainer = document.querySelector('.loading-container');
      if (loadingContainer) {
        // Just remove it immediately
        (loadingContainer as HTMLElement).style.display = 'none';
        
        if (loadingContainer.parentNode) {
          loadingContainer.parentNode.removeChild(loadingContainer);
        }
      }
      
      // Also remove the loading container from the index.html
      const loadingEl = document.getElementById('loading-container');
      if (loadingEl) {
        loadingEl.style.display = 'none';
        if (loadingEl.parentNode) {
          loadingEl.parentNode.removeChild(loadingEl);
        }
      }
      
      // Make app visible
      if (appElement) {
        Object.assign(appElement.style, {
          display: 'flex',
          flexDirection: 'column',
          visibility: 'visible',
          opacity: '1',
          backgroundColor: '#1e1e1e',
          width: '100%',
          height: '100%'
        });
      }
      
      if (svelteContainer) {
        Object.assign(svelteContainer.style, {
          display: 'flex',
          visibility: 'visible',
          opacity: '1',
          width: '100%',
          height: '100%'
        });
      }
      
      // Fix body styles
      if (typeof document !== 'undefined') {
        document.body.style.backgroundColor = '#1e1e1e';
        document.body.style.overflow = 'auto';
        document.body.style.height = '100%';
        document.body.style.width = '100%';
        
        // Ensure html element has proper height/width
        const htmlElement = document.documentElement;
        htmlElement.style.height = '100%';
        htmlElement.style.width = '100%'; 
      } 
      
      console.log('Loading screen removed, app should be fully visible');
    } catch (e) {
      console.error('Error hiding loading screen:', e);
    }
  }

  // Function to recover the UI if it gets stuck
  function recoverUI() {
    console.log('Attempting UI recovery');
    // Reset all UI state to default values
    appState.update(s => ({
      ...s,
      commandMode: false,
      commandText: '',
      showSuggestions: true,
      showChat: false,
      showHelp: false,
      isSaving: false,
      showSaveConfirmation: false,
      showTutorial: false,
      pendingAction: null,
      pendingArgs: null,
      isRecovering: false,
      statusMessage: 'UI has been reset'
    }));
    
    // Display toast notification to inform user
    addToast('UI has been reset', 'success');
    
    // If editor exists, try to focus it
    setTimeout(() => {
      if (editor && typeof editor.focus === 'function') {
        try {
          editor.focus();
        } catch (e) {
          console.error('Failed to focus editor:', e);
        }
      }
    }, 100);
  }

  // Function to setup CLI file opening functionality
  async function setupCliFileOpening() {
    if (!isTauri) return;
    
    try {
      // Check if we have a file from CLI args
      const fileFromCli = await invoke('get_cli_args');
      
      if (fileFromCli) {
        // Try to open the file specified in CLI arguments
        const filePath = fileFromCli as string;
        
        try {
          const fileContent = await readTextFile(filePath);
          // Create a buffer and set bufferId
          const createdBufferId = await invoke('create_new_buffer').catch(() => null) as number | null;
          bufferId = createdBufferId;
          appState.update(s => ({ ...s, content: fileContent, currentFile: filePath, hasUnsavedChanges: false, bufferId: createdBufferId }));
          
          // Set editor mode based on file extension
          const ext = filePath.split('.').pop()?.toLowerCase();
          const detectedLanguage = await invoke('get_language_mode', { path: filePath })
            .catch((_: any) => ext === 'md' ? 'markdown' : 'plaintext');
            
          appState.update(s => ({ 
            ...s, 
            mode: detectedLanguage === 'markdown' || ext === 'md' ? 'markdown' : 'code',
            editorLanguage: detectedLanguage as string,
            statusMessage: `Opened ${filePath}", bufferId: ${createdBufferId}`
          }));
          
          console.log(`Opened file from CLI: ${filePath}`);
        } catch (e) {
          console.error('Error opening file from CLI:', e);
          appState.update(s => ({ ...s, statusMessage: `Error opening file: ${e}` }));
        }
      }
    } catch (e) {
      console.error('Error checking CLI args:', e);
    }
  }

  // Helper function to focus command input
  function focusCommandInput() {
    if (commandBarComponent) {
      try {
        const inputElement = commandBarComponent.getInputElement();
        if (inputElement) {
          inputElement.focus();
          // Let Tauri key handler know about the input for focusing on toggles
          try { keyHandler.setCommandBarInput(inputElement); } catch {}
        }
      } catch (e) {
        console.error('Error focusing command input:', e);
      }
    }
  }

  // Simple debounce helper
  function debounce<T extends (...args: any[]) => any>(fn: T, wait = 800) {
    let timer: ReturnType<typeof setTimeout> | null = null;
    return (...args: Parameters<T>) => {
      if (timer) clearTimeout(timer);
      timer = setTimeout(() => fn(...args), wait);
    };
  }

  // Debounced buffer update to reduce IPC frequency
  const debouncedUpdateBuffer = debounce(async (id: number | null, txt: string) => {
    if (!isTauri || id == null) return;
    try {
      await invoke('update_buffer_content_command', { buffer_id: id, content: txt });
    } catch (err) {
      console.warn('Failed to debounced update buffer:', err);
    }
  }, 800);

  // Function to check for API key and prompt if missing
  async function checkApiKey() {
    if (!isTauri) return;
    
    try {
      const key = await invoke('get_api_key');
      appState.update(s => ({ ...s, apiKey: key as string }));
      
      // If no API key found, prompt the user to set one
      if (!$appState.apiKey || $appState.apiKey.trim() === '') {
        setTimeout(() => {
          appState.update(s => ({ 
            ...s, 
            commandMode: true, 
            commandBarTitle: 'Enter your Gemini API key to enable AI features',
            commandText: 'set api_key ',
            showSuggestions: false
          }));
          
          // Focus the command input when ready
          setTimeout(() => {
            if (commandBarComponent) {
              const inputElement = commandBarComponent.getInputElement();
              if (inputElement) {
                inputElement.focus();
              }
            }
          }, 100);
        }, 1000); // Give a short delay before showing the prompt
      }
    } catch (e) {
      console.error('Error checking API key:', e);
    }
  }

  // Helper to route outputs to panel
  function showOutput(content: string, kind: 'ai' | 'shell' | 'file' | 'system' | 'info' | 'error' = 'info', title?: string) {
    addOutput(content, kind, title);
    showPanel();
    // If it's an error, trigger a short red pulse visual feedback
    if (kind === 'error') {
      try {
        appState.update(s => ({ ...s, showRedPulse: true }));
        setTimeout(() => appState.update(s => ({ ...s, showRedPulse: false })), 420);
      } catch (e) {
        console.warn('Failed to trigger red pulse', e);
      }
    }
  }

  // Function to execute commands
  async function executeCommand(command: string) {
    // Clear command text but keep commandMode state intact until we know the result
    const [cmd, ...args] = command.toLowerCase().split(' ');
    
    // Get working directory from current file if available
    let workingDir: string | null = null;
    if (currentFile && isTauri) {
      try {
        workingDir = await dirname(currentFile);
      } catch (e) {
        console.warn('Failed to get file directory:', e);
      }
    }
    
    // Handle special API key setting commands
    if (cmd === 'set' && args[0] === 'api_key') {
      const apiKey = args.slice(1).join(' ');
      if (apiKey && isTauri) {
        try {
          await invoke('set_api_key', { apiKey });
          appState.update(s => ({ 
            ...s, 
            apiKey,
            statusMessage: 'API key saved successfully',
            commandMode: false
          }));
          addToast('API key saved successfully', 'success');
          return;
        } catch (e) {
          addToast(`Error saving API key: ${e}`, 'error');
          return;
        }
      } else {
        addToast('Please provide a valid API key', 'error');
        return;
      }
    }
    
    // 1. Check if it's an editor command
    const editorCommands: Record<string, () => Promise<void>> = {
      'new': async () => {
        const fileType = args[0] || 'txt';
        // Create new file with appropriate language mode
  const languageMap: Record<string, string> = {
          'markdown': 'markdown',
          'md': 'markdown',
          'javascript': 'javascript',
          'js': 'javascript',
          'typescript': 'typescript',
          'ts': 'typescript',
          'python': 'python',
          'py': 'python',
          'html': 'html',
          'css': 'css',
          'json': 'json',
          'rust': 'rust',
          'rs': 'rust',
          'c': 'c',
          'cpp': 'cpp'
        };
        
  const language = languageMap[fileType] || 'plaintext';
        const mode = language === 'markdown' ? 'markdown' : 'code';
        
        // create a new buffer and assign bufferId
        const newBufferId = isTauri ? await invoke('create_new_buffer').catch(() => null) as number | null : null;
        bufferId = newBufferId;
        appState.update(s => ({ 
          ...s, 
          content: '', 
          currentFile: '', 
          editorLanguage: language,
          mode: mode,
          hasUnsavedChanges: false,
          bufferId: newBufferId,
          statusMessage: `Created new ${fileType} file`
        }));
        
        addToast(`Created new ${fileType} file`, 'success');
      },
      'open': async () => {
        // Trigger file dialog
        if (isTauri) {
          try {
            const { open } = await import('@tauri-apps/api/dialog');
            const selected = await open({
              multiple: false,
              filters: [{
                name: 'All Files',
                extensions: ['*']
              }]
            });
            
            if (selected && typeof selected === 'string') {
              const fileContent = await readTextFile(selected);
              const ext = selected.split('.').pop()?.toLowerCase();
              const detectedLanguage = await invoke('get_language_mode', { path: selected })
                .catch((_: any) => ext === 'md' ? 'markdown' : 'plaintext');
              // create a buffer and set bufferId
              const openedBufferId = isTauri ? await invoke('create_new_buffer').catch(() => null) as number | null : null;
              bufferId = openedBufferId;
              appState.update(s => ({ 
                ...s, 
                content: fileContent, 
                currentFile: selected,
                editorLanguage: detectedLanguage as string,
                mode: detectedLanguage === 'markdown' || ext === 'md' ? 'markdown' : 'code',
                hasUnsavedChanges: false,
                bufferId: openedBufferId,
                statusMessage: `Opened ${selected}`
              }));
              
              addToast(`Opened ${selected}`, 'success');
            }
          } catch (e) {
            addToast(`Error opening file: ${e}`, 'error');
          }
        }
      },
      'save': async () => {
        if (isTauri && content) {
          try {
            // First update the buffer content (immediate)
            if (bufferId == null) {
              // create buffer if not present
              bufferId = await invoke('create_new_buffer').catch(() => null) as number | null;
              appState.update(s => ({ ...s, bufferId }));
            }
            await invoke('update_buffer_content_command', { buffer_id: bufferId || 0, content: content });

            if (currentFile) {
              // Save to existing file
              await invoke('save_file', { 
                buffer_id: bufferId || 0, 
                path: currentFile 
              });
              appState.update(s => ({ 
                ...s, 
                hasUnsavedChanges: false,
                statusMessage: `Saved ${currentFile}`
              }));
              addToast(`Saved ${currentFile}`, 'success');
            } else {
              // Save as new file
              const { save } = await import('@tauri-apps/api/dialog');
              const filePath = await save({
                filters: [{
                  name: 'All Files',
                  extensions: ['*']
                }]
              });
              
              if (filePath) {
                await invoke('save_file', { buffer_id: bufferId || 0, path: filePath });
                appState.update(s => ({ 
                  ...s, 
                  currentFile: filePath,
                  hasUnsavedChanges: false,
                  statusMessage: `Saved ${filePath}`
                }));
                addToast(`Saved ${filePath}`, 'success');
              }
            }
          } catch (e) {
            addToast(`Error saving file: ${e}`, 'error');
          }
        }
      },
      'save as': async () => {
        if (isTauri && content) {
          try {
            if (bufferId == null) {
              bufferId = await invoke('create_new_buffer').catch(() => null) as number | null;
              appState.update(s => ({ ...s, bufferId }));
            }
            await invoke('update_buffer_content_command', { buffer_id: bufferId || 0, content: content });

            const { save } = await import('@tauri-apps/api/dialog');
            const filePath = await save({
              filters: [{
                name: 'All Files',
                extensions: ['*']
              }]
            });
            
            if (filePath) {
              await invoke('save_file', { 
                buffer_id: bufferId || 0, 
                path: filePath 
              });
              appState.update(s => ({ 
                ...s, 
                currentFile: filePath,
                hasUnsavedChanges: false,
                statusMessage: `Saved as ${filePath}`
              }));
              addToast(`Saved as ${filePath}`, 'success');
            }
          } catch (e) {
            addToast(`Error saving file: ${e}`, 'error');
          }
        }
      },
      'exit': async () => {
        if (isTauri) {
          await invoke('close_window');
        }
      },
      'quit': async () => {
        if (isTauri) {
          await invoke('close_window');
        }
      },
      'close': async () => {
        if (isTauri) {
          await invoke('close_window');
        }
      },
      'minimize': async () => {
        if (isTauri) {
          await invoke('minimize_window');
        }
      },
      'maximize': async () => {
        if (isTauri) {
          await invoke('maximize_window');
        }
      },
      'help': async () => {
        appState.update(s => ({ ...s, showHelp: true }));
      }
    };

  if (cmd in editorCommands) {
      try {
        await editorCommands[cmd]();
    appState.update(s => ({ ...s, commandMode: false, commandText: '' }));
      } catch (e) {
        console.error('Error executing editor command:', e);
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Error executing command: ${e}`,
          commandMode: false
        }));
        addToast(`Error executing command: ${e}`, 'error');
      }
      return;
    }

  // 2. Check if it's a terminal command (starts with !)
  if (command.trim().startsWith('!')) {
    if (isTauri) {
      try {
        // Show loading indicator
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Executing: ${command}...`,
          commandExecutionInProgress: true,
          isAiLoading: false
        }));
        
        const result = await invoke('execute_enhanced_command', { 
          command: command.trim(),
          apiKey: null,  // Don't pass API key for terminal commands
          workingDir: workingDir
        }) as any;
        
  if (result.success) {
          // Handle different command types
          let displayMessage = result.output;
          let toastType: 'success' | 'info' | 'warning' | 'error' = 'info';
          
          if (result.command_type === 'ai') {
            // AI response - show output panel and KEEP command bar open
            appState.update(s => ({ 
              ...s, 
              statusMessage: 'AI response received',
              commandBarTitle: 'Ask another question or run a command',
              commandBarPlaceholder: 'Ask AI or run a command (ESC for general commands)',
              commandExecutionInProgress: false,
              isAiLoading: false,
              commandMode: true,
              commandText: '',
              suggestions: []
            }));
            showOutput(displayMessage || 'AI responded', 'ai', 'AI Response');
            
          } else if (result.command_type === 'shell' || result.command_type === 'file') {
            // Show output in panel and KEEP command bar open for more terminal commands
            appState.update(s => ({ 
              ...s, 
              statusMessage: `Command executed: ${command}`,
              commandBarTitle: 'Run another command',
              commandBarPlaceholder: 'Type a terminal command (ESC for general commands)',
              commandExecutionInProgress: false,
              isAiLoading: false,
              commandMode: true,
              commandText: '',
              suggestions: []
            }));
            showOutput((displayMessage && displayMessage.trim()) ? displayMessage : 'Command completed successfully', result.command_type === 'shell' ? 'shell' : 'file', `Output: ${command}`);
            
          } else if (result.command_type === 'system') {
            // System operations like minimize, maximize, close
            const lower = command.trim().toLowerCase();
            if (lower === 'minimize') await minimizeWindow();
            if (lower === 'maximize' || lower === 'fullscreen') await maximizeWindow();
            if (lower === 'close' || lower === 'quit' || lower === 'exit') await closeWindow();
            appState.update(s => ({
              ...s,
              statusMessage: result.output || 'System command executed',
              commandMode: false,
              commandExecutionInProgress: false,
              isAiLoading: false,
              commandText: '',
              suggestions: []
            }));
            if (displayMessage && displayMessage.trim()) {
              showOutput(displayMessage, 'system', 'System');
            }
          } else {
            // Other successful commands
            appState.update(s => ({ 
              ...s, 
              statusMessage: result.output || 'Command executed successfully',
              commandMode: false,
              commandExecutionInProgress: false,
              isAiLoading: false,
              commandText: '',
              suggestions: []
            }));
            
            if (displayMessage && displayMessage.trim()) {
              showOutput(displayMessage, 'info', 'Command');
            }
          }
          
        } else {
          // Terminal command failed - show error
          const errorMsg = result.error || 'Command failed';
          appState.update(s => ({ 
            ...s, 
            statusMessage: errorMsg,
            commandExecutionInProgress: false,
            isAiLoading: false
          }));
          showOutput(errorMsg, 'error', 'Error');
        }
        
      } catch (e) {
        console.error('Error executing terminal command:', e);
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Error executing command: ${e}`,
          commandExecutionInProgress: false,
          isAiLoading: false
        }));
        showOutput(`Error executing command: ${e}`, 'error', 'Error');
      } finally {
        console.log('Terminal command execution cleanup');
        appState.update(s => ({
          ...s,
          commandExecutionInProgress: false,
          isAiLoading: false
        }));
      }
    } else {
      addToast('Terminal commands not available in this environment', 'warning');
    }
    return;
  }

  // 3. Everything else goes to AI
  if (isTauri && $appState.apiKey) {
    try {
      appState.update(s => ({ 
        ...s, 
        statusMessage: 'Asking AI...',
        commandExecutionInProgress: true,
        isAiLoading: true
      }));
      
      const aiResult = await invoke('execute_enhanced_command', { 
        command: `ai ${command}`,
        apiKey: $appState.apiKey,
        workingDir: null
      }) as any;
      
      if (aiResult.success) {
        appState.update(s => ({ 
          ...s, 
          statusMessage: 'AI response received',
          commandBarTitle: 'Ask another question or run a command',
          commandBarPlaceholder: 'Ask AI or run a command (ESC for general commands)',
          commandExecutionInProgress: false,
          isAiLoading: false,
          commandMode: true,
          commandText: '',
          suggestions: []
        }));
        showOutput(aiResult.output || '(no response)', 'ai', 'AI Response');
      } else {
        throw new Error(aiResult.error || 'AI failed');
      }
    } catch (aiErr) {
      console.error('Error with AI:', aiErr);
      appState.update(s => ({ 
        ...s, 
        statusMessage: `AI error: ${aiErr}`,
        commandExecutionInProgress: false,
        isAiLoading: false
      }));
      showOutput(`AI error: ${aiErr}`, 'error', 'AI Error');
    } finally {
      appState.update(s => ({
        ...s,
        commandExecutionInProgress: false,
        isAiLoading: false
      }));
    }
  } else {
    // No AI key available
    appState.update(s => ({ 
      ...s, 
      statusMessage: 'No AI key configured. Set one with: set api_key YOUR_KEY',
      commandMode: false
    }));
    addToast('No AI key configured', 'warning');
  }
  }

  // Function to generate command suggestions based on context
  function generateSuggestions() {
    const suggestions: string[] = [];
    
    // Add file-specific commands based on current file
    if (currentFile) {
      const ext = currentFile.split('.').pop()?.toLowerCase();
      switch (ext) {
        case 'js':
        case 'jsx':
        case 'ts':
        case 'tsx':
          suggestions.push('npm install', 'npm run dev', 'npm start', 'npm test');
          break;
        case 'py':
          suggestions.push('pip install', 'python main.py', 'python3 script.py');
          break;
        case 'rs':
          suggestions.push('cargo run', 'cargo build', 'cargo test');
          break;
        case 'c':
          suggestions.push('gcc main.c');
          break;
        case 'cpp':
          suggestions.push('g++ main.cpp');
          break;
        case 'go':
          suggestions.push('go run', 'go build', 'go test');
          break;
        case 'java':
          suggestions.push('javac', 'java');
          break;
      }
    }
    
    // Add common editor commands
    suggestions.push(
      'new markdown',
      'new javascript',
      'new python',
      'save',
      'save as',
      'open README.md'
    );
    
    // Add window control commands
    suggestions.push(
      'minimize',
      'maximize',
      'close',
      'quit',
      'exit'
    );
    
    // Add common shell commands
    suggestions.push(
      'dir',
      'cd ..',
      'git status',
      'npm install',
      'python main.py'
    );
    
    // Add help command
    suggestions.push('help');
    
    // Store all generated suggestions locally
    allGeneratedSuggestions = [...suggestions];

    // Set suggestions in the app state
    appState.update(s => ({
      ...s,
      suggestions: allGeneratedSuggestions, // Initialize with all suggestions
      showSuggestions: true
    }));
    
    console.log('Generated suggestions:', allGeneratedSuggestions);
  }

  // Handle command bar events
  function handleCommandClose() {
    appState.update(s => ({ ...s, commandMode: false, commandText: '', suggestions: [], commandBarPlaceholder: 'Type a command or press Enter' }));
    // Refocus editor after closing
    setTimeout(() => {
      focusEditor();
    }, 100);
  }

  function handleCommandInput(event: CustomEvent) {
    const text = event.detail.commandText;
    
    // Filter suggestions based on input
    if (text) {
      const lowerText = text.toLowerCase();
      // Filter from the master list of suggestions
      const filtered = allGeneratedSuggestions.filter(s => 
        s.toLowerCase().includes(lowerText)
      );
      console.log('Filtered suggestions:', filtered, 'from input:', lowerText);
      appState.update(s => ({
        ...s,
        commandText: text,
        suggestions: filtered, // Update state with filtered list
        showSuggestions: filtered.length > 0 // Only show if we have suggestions
      }));
    } else {
      // Reset to default suggestions if input is empty
      console.log('Resetting to all suggestions');
      appState.update(s => ({
        ...s,
        commandText: '',
        suggestions: allGeneratedSuggestions, // Reset to the full list
        showSuggestions: true
      }));
    }
  }

  // Handle command execution from command bar
  function handleCommandExecution(event: CustomEvent) {
    const { command } = event.detail;
    console.log('Executing command from CommandBar:', command);
    // Don't close command bar here - let executeCommand handle it based on success/failure
    executeCommand(command);
  }

  // Handle command execution results from enhanced command bar
  function handleCommandExecuted(event: CustomEvent) {
    const { command, result } = event.detail;
    console.log('Command executed:', command, 'Result:', result);
    
    // Handle specific command types
    if (result.command_type === 'file') {
      // Handle file operations
      if (command.startsWith('ls') || command.startsWith('pwd')) {
        appState.update(s => ({ ...s, statusMessage: 'File operation completed' }));
      }
    } else if (result.command_type === 'editor') {
      // Handle editor operations
      if (command === 'clear') {
        appState.update(s => ({ ...s, content: '', hasUnsavedChanges: false }));
      }
    } else if (result.command_type === 'system') {
      // Handle system operations like minimize, maximize, close
      if (command === 'minimize') {
        minimizeWindow();
      } else if (command === 'maximize' || command === 'fullscreen') {
        maximizeWindow();
      } else if (command === 'close') {
        closeWindow();
      }
    }
    
    // Update status
    if (result.success) {
      appState.update(s => ({ 
        ...s, 
        statusMessage: 'Command executed successfully',
        commandMode: false 
      }));
    } else {
      appState.update(s => ({ 
        ...s, 
        statusMessage: `Command failed: ${result.error}`,
        commandMode: false 
      }));
    }
  }

  // Function to close tutorial
  function closeTutorial() {
    appState.update(s => ({ 
      ...s,
      showTutorial: false,
      commandMode: true
    }));
    
    generateSuggestions();
    
    // Give time for state to update before focusing
    setTimeout(() => {
      if (commandBarComponent) {
        const inputElement = commandBarComponent.getInputElement();
        if (inputElement) {
          inputElement.focus();
        }
      }
    }, 100);
  }

  // Function to toggle help screen
  function toggleHelp() {
    appState.update(s => ({ ...s, showHelp: !s.showHelp }));
  }

  // Event handlers: rely on Tauri layer for ESC and Cmd+K

  function handleClick(event: MouseEvent) {
    const target = event.target as Element;
    const state = get(appState);
    
    // Check if click is outside command bar and close it
    if (state.commandMode && !target.closest('.command-content')) {
      appState.update(s => ({ ...s, commandMode: false }));
    }

    // Check if click is outside help window and close it
    if (state.showHelp && !target.closest('.help-content')) {
      appState.update(s => ({ ...s, showHelp: false }));
    }
  }

  // Removed duplicate handleKeydown fallback

  // Window control functions
  async function minimizeWindow() {
    if (isTauri) {
      try {
        await invoke('minimize_window');
      } catch (e) {
        console.error('Failed to minimize window:', e);
      }
    }
  }
  
  async function maximizeWindow() {
    if (isTauri) {
      try {
        await invoke('maximize_window');
      } catch (e) {
        console.error('Failed to maximize window:', e);
      }
    }
  }
  
  async function closeWindow() {
    if (isTauri) {
      try {
        await invoke('close_window');
      } catch (e) {
        console.error('Failed to close window:', e);
      }
    }
  }

  // Window dragging uses CSS -webkit-app-region: drag on top bar
  let isDragging = false;
  let dragStartX = 0;
  let dragStartY = 0;

  onMount(() => {
    // Check if first run and show tutorial
    invoke('check_has_run_before')
      .then((hasRunBefore) => {
        isFirstRun = !hasRunBefore;
        if (isFirstRun) {
          showTutorial = true;
        }
      })
      .catch((e) => console.error('Error checking first run:', e));
      
  // No extra F1 handler; handled in Rust layer
  return () => {};
  });

  function handleMouseDown(_: MouseEvent) {}

  function handleMouseMove(event: MouseEvent) {
    if (isDragging && isTauri) {
      // The CSS -webkit-app-region: drag should handle the actual dragging
      // We just track the state here for visual feedback
      event.preventDefault();
    }
  }

  function handleMouseUp() {
    if (isDragging) {
      isDragging = false;
    }
  }

  async function handleDoubleClickFullscreen() {
    if (isTauri) {
      try {
        await invoke('toggle_fullscreen');
      } catch (e) {
        console.error('Error toggling fullscreen:', e);
        addToast('Error toggling fullscreen', 'error');
      }
    }
  }

  function focusEditor() {
    if (typeof window === 'undefined' || typeof document === 'undefined') return;
    
    try {
      // Try focusing editor directly
      const editorElement = document.querySelector('.cm-editor');
      if (editorElement) {
        (editorElement as HTMLElement).focus();
      }
    } catch (error) {
      console.error('Error focusing editor from main page:', error);
    }
  }
  
  // When command mode opens, ensure input is focused and registered with key handler
  $: if (commandMode) {
    setTimeout(() => {
      focusCommandInput();
    }, 0);
  }
  
  function handleEditorFocusRequest() {
    console.log('Editor focus event received');
    focusEditor();
  }
</script>

<main 
  id="app" 
  class="flex flex-col h-screen w-screen bg-black overflow-hidden" 
  role="application" 
  aria-label="Vuno Editor Application"
  tabindex="-1"
>
  <div class="flex justify-between items-center px-2 py-1 bg-[#191919] text-gray-400 text-xs border-b border-gray-800 font-sans select-none relative z-10 h-7 min-h-7" role="banner" aria-label="Title bar">
    <div class="flex items-center gap-2" data-tauri-drag-region>
      {#if currentFile}
        {#if renaming}
          <input
            class="bg-transparent border border-gray-700 text-gray-100 rounded px-1 focus:outline-none focus:ring-1 focus:ring-gray-500 w-64 text-xs"
            bind:value={renameValue}
            on:keydown={(e) => {
              if (e.key === 'Enter') commitRename();
              if (e.key === 'Escape') cancelRename();
            }}
            on:blur={commitRename}
            bind:this={renameInput}
          />
        {:else}
          <button class="text-gray-100 hover:text-white hover:opacity-80 cursor-pointer transition-opacity relative z-20" style="-webkit-app-region: no-drag;" on:click={(e) => { e.stopPropagation(); startRename(); }}>{currentFile}</button>
        {/if}
      {:else}
        {#if renaming}
          <input
            class="bg-transparent border border-gray-700 text-gray-100 rounded px-1 focus:outline-none focus:ring-1 focus:ring-gray-500 w-64 text-xs"
            bind:value={renameValue}
            on:keydown={(e) => {
              if (e.key === 'Enter') commitRename();
              if (e.key === 'Escape') cancelRename();
            }}
            on:blur={commitRename}
            bind:this={renameInput}
          />
        {:else}
          <button class="text-gray-500 hover:text-gray-300 hover:opacity-80 cursor-pointer transition-opacity relative z-20" style="-webkit-app-region: no-drag;" on:click={(e) => { e.stopPropagation(); startRename(); }}>untitled</button>
        {/if}
      {/if}
    </div>
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <span class="text-gray-400">{statusMessage}</span>
    </div>
    <div class="flex items-center gap-2" data-tauri-drag-region>
      <span class="text-gray-500 font-mono">
        {#if currentCursorPosition}
          Line {currentCursorPosition.line + 1}, Col {currentCursorPosition.column + 1}
        {/if}
      </span>
    </div>
  </div>
  
  <div class="flex-1 flex flex-col min-h-0 h-full relative overflow-hidden" on:dblclick={(e) => { e.stopPropagation(); openGlobalCommandBar(); }} role="button" aria-label="Double-click to open command menu" tabindex="0">
  {#if mode === 'code'}
    <div class="flex-1 flex w-full h-full min-h-0">
      <Editor 
        bind:this={editorComponent}
        content={content}
        language={editorLanguage}
        bufferId={bufferId}
        on:change={(e) => {
          const newContent = e.detail.content;
          appState.update(s => ({
            ...s,
            content: newContent,
            hasUnsavedChanges: true,
            currentLineCount: e.detail.lineCount || 0,
            currentCursorPosition: e.detail.cursorPosition || { line: 0, ch: 0 }
          }));

          // Sync buffer to backend (non-blocking)
          if (isTauri) {
            debouncedUpdateBuffer(bufferId, newContent);
          }
        }}
        on:save={async () => {
          // Trigger save via existing command flow
          await executeCommand('save');
        }}
      />
    </div>
  {/if}
    
  {#if mode === 'markdown'}
    <div class="flex-1 flex w-full h-full min-h-0 overflow-hidden">
      <MarkdownRenderer 
        bind:this={editorComponent}
        content={content}
        on:change={(e) => {
          const newContent = e.detail.content;
          appState.update(s => ({
            ...s,
            content: newContent,
            hasUnsavedChanges: true,
            currentLineCount: (newContent || '').split('\n').length,
            currentCursorPosition: { line: 0, column: 0 }
          }));
          if (isTauri) {
            debouncedUpdateBuffer(bufferId, newContent);
          }
        }}
        on:save={async () => {
          await executeCommand('save');
        }}
      />
    </div>
  {/if}
  </div>
  
  {#if commandMode}
    <div class="command-overlay command-content" role="dialog" aria-modal="false" aria-label="Global Command Bar">
      <CommandBar 
        bind:this={commandBarComponent}
        title={commandBarTitle || 'What would you like to do today?'}
        placeholder={commandBarPlaceholder || 'Type a command or press Enter'}
        commandText={commandText}
        showSuggestions={showSuggestions}
        suggestions={suggestions}
        isVisible={commandMode}
        on:close={handleCommandClose}
        on:input={handleCommandInput}
        on:command={handleCommandExecution}
        on:command-executed={handleCommandExecuted}
      />
    </div>
  {/if}
  
  {#if showHelp}
    <div class="command-overlay">
      <HelpWindow on:close={toggleHelp} />
    </div>
  {/if}
  
  {#if showTutorial}
    <div class="command-overlay">
      <TutorialWindow
        showTutorial={$appState.showTutorial}
        onClose={closeTutorial}
      />
    </div>
  {/if}
  
  <BottomOutput />
  <Toast />

  {#if $appState.showRedPulse}
    <div 
      class="fixed inset-0 pointer-events-none z-50" 
      style="box-shadow: inset 0 0 120px 40px rgba(255, 0, 50, 0.35);"
      in:fade={{ duration: 80 }}
      out:fade={{ duration: 340 }}
    ></div>
  {/if}
</main>

<style>
  /* Global html/body styles are defined in src/global.css */
</style>
