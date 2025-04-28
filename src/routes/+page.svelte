<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { fly, fade } from 'svelte/transition';
  import { elasticOut } from 'svelte/easing';
  import { marked } from 'marked';
  import { get } from 'svelte/store';
  import Editor from '../modules/Editor.svelte';
  import MarkdownRenderer from '../modules/MarkdownRenderer.svelte';
  import CommandBar from '../modules/CommandBar.svelte';
  import Toast from '../modules/Toast.svelte';
  import HelpWindow from '../modules/HelpWindow.svelte';
  import TutorialWindow from '../modules/TutorialWindow.svelte';
  import { addToast } from '../modules/toastStore';
  import { KeyboardManager } from '../modules/KeyboardManager';
  import { appState } from '../stores/appState';
  import { eventBus } from '../stores/eventBus';
  
  // Import Tauri API
  import { invoke } from '@tauri-apps/api/tauri';
  import { open, save } from '@tauri-apps/api/dialog';
  import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

  // Import TauriKeyHandler instead of KeyboardManager
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

  onMount(async () => {
    // Set app element reference for easier manipulation
    appElement = document.getElementById('app');
    svelteContainer = document.getElementById('svelte');

    try {
      // Initialize TauriKeyHandler
      await keyHandler.initialize();
      console.log('TauriKeyHandler initialized successfully');

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
    } catch (error) {
      console.error('Error during app initialization:', error);
      addToast('Error initializing app', 'error');
    }
  });

  onDestroy(() => {
    // Cleanup TauriKeyHandler
    keyHandler.cleanup();
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
      document.body.style.backgroundColor = '#1e1e1e';
      document.body.style.overflow = 'auto';
      document.body.style.height = '100%';
      document.body.style.width = '100%';
      
      // Ensure html element has proper height/width
      const htmlElement = document.documentElement;
      htmlElement.style.height = '100%';
      htmlElement.style.width = '100%'; 
      
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
          appState.update(s => ({ ...s, content: fileContent, currentFile: filePath, hasUnsavedChanges: false }));
          
          // Set editor mode based on file extension
          const ext = filePath.split('.').pop()?.toLowerCase();
          const detectedLanguage = await invoke('get_language_mode', { path: filePath })
            .catch((_: any) => ext === 'md' ? 'markdown' : 'plaintext');
            
          appState.update(s => ({ 
            ...s, 
            mode: detectedLanguage === 'markdown' || ext === 'md' ? 'markdown' : 'code',
            editorLanguage: detectedLanguage as string,
            statusMessage: `Opened ${filePath}`
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
        }
      } catch (e) {
        console.error('Error focusing command input:', e);
      }
    }
  }

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

  // Helper function to display toast notifications
  function showToast(message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') {
    addToast(message, type);
  }

  // Function to execute commands
  async function executeCommand(command: string) {
    const [cmd, ...args] = command.toLowerCase().split(' ');
    
    // 1. Check if it's an editor command
    const editorCommands: Record<string, () => Promise<void>> = {
      'new': async () => {
        const fileType = args[0] || 'txt';
        // TODO: Implement new file creation
      },
      'open': async () => {
        // TODO: Implement file opening
      },
      'save': async () => {
        // TODO: Implement save
      },
      'save as': async () => {
        // TODO: Implement save as
      },
      'rename': async () => {
        if (args.length < 2) {
          throw new Error('Rename requires old and new filenames');
        }
        // TODO: Implement rename
      },
      'delete': async () => {
        if (!args[0]) {
          throw new Error('Delete requires filename');
        }
        // TODO: Implement delete
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
        appState.update(s => ({ ...s, commandMode: false }));
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

    // 2. Check if it's a shell command
    const shellCommands = [
      'dir', 'cd', 'ls', 'pwd', 'git', 'npm', 'pip', 'python', 'node', 
      'cargo', 'gcc', 'g++', 'make', 'docker', 'kubectl', 'terraform',
      'aws', 'gcloud', 'az', 'systeminfo', 'tasklist', 'netstat',
      'ipconfig', 'ifconfig', 'ping', 'tracert', 'nslookup', 'curl',
      'wget', 'tar', 'zip', 'unzip', 'chmod', 'chown', 'sudo',
      'apt-get', 'yum', 'brew', 'pacman', 'systemctl', 'journalctl',
      'top', 'htop', 'ps', 'kill', 'killall', 'find', 'grep',
      'sed', 'awk', 'sort', 'uniq', 'wc', 'cat', 'head', 'tail',
      'less', 'more', 'tmux', 'screen', 'ssh', 'scp', 'rsync', 
      'lynx', 'w3m', 'links', 'elinks', 'man', 'info', 'whatis', 
      'apropos', 'locate', 'updatedb', 'which', 'whereis', 'type', 
      'alias', 'history', 'clear', 'reset', 'logout', 'shutdown', 
      'reboot', 'halt', 'poweroff', 'date', 'cal', 'bc', 'dc', 
      'factor', 'seq', 'jot', 'yes', 'no', 'true', 'false', 'echo', 'printf'
    ];

    if (shellCommands.includes(cmd)) {
      try {
        // Show loading indicator
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Executing: ${command}...`,
          commandExecutionInProgress: true 
        }));
        
        const output = await invoke('execute_terminal_command', { command });
        
        // Show the output in a nice format
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Command executed: ${command}`,
          commandMode: true,
          commandBarTitle: `Output: ${command}`,
          commandExecutionInProgress: false
        }));
        
        // Display output as a toast that stays longer
        addToast(output as string, 'info', 10000);
        
        // Focus back on command input
        setTimeout(() => {
          if (commandBarComponent) {
            const inputElement = commandBarComponent.getInputElement();
            if (inputElement) {
              inputElement.focus();
            }
          }
        }, 100);
        
      } catch (e) {
        console.error('Error executing terminal command:', e);
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Error executing command: ${e}`,
          commandMode: false,
          commandExecutionInProgress: false
        }));
        addToast(`Error executing command: ${e}`, 'error');
      }
      return;
    }

    // 3. If not an editor or shell command, try AI
    if ($appState.apiKey) {
      try {
        // Show loading indicator
        appState.update(s => ({ 
          ...s, 
          statusMessage: 'Processing AI command...',
          commandMode: true,
          commandBarTitle: 'AI is thinking...',
          isAiLoading: true
        }));
        
        // TODO: Implement actual AI command handling
        const aiResponse = "This is a placeholder for the AI response. In a real implementation, this would be the response from the AI model.";
        
        // Show AI response
        setTimeout(() => {
          appState.update(s => ({ 
            ...s, 
            statusMessage: 'AI response received',
            commandBarTitle: 'AI Response',
            isAiLoading: false
          }));
          
          // Display AI response
          addToast(aiResponse, 'info', 10000);
          
          // Focus back on command input
          setTimeout(() => {
            if (commandBarComponent) {
              const inputElement = commandBarComponent.getInputElement();
              if (inputElement) {
                inputElement.focus();
              }
            }
          }, 100);
        }, 1000);
        
      } catch (e) {
        console.error('Error processing AI command:', e);
        appState.update(s => ({ 
          ...s, 
          statusMessage: `Error processing AI command: ${e}`,
          commandMode: false,
          isAiLoading: false
        }));
        addToast(`Error processing AI command: ${e}`, 'error');
      }
    } else {
      appState.update(s => ({ 
        ...s, 
        statusMessage: 'Please set your API key to use AI features',
        commandMode: false
      }));
      addToast('Please set your API key to use AI features', 'warning');
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
    appState.update(s => ({ ...s, commandMode: false }));
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

  // Event handlers
  // Remove direct keydown handler to avoid conflict with TauriKeyHandler
  // function handleKeydown(event: KeyboardEvent) {
  //   ...existing code for keydown...
  // }

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

  function focusEditor() {
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
  
  function handleEditorFocusRequest() {
    console.log('Editor focus event received');
    focusEditor();
  }
</script>

<div id="app" class="app" on:click={handleClick}>
  <div class="status-bar">
    <div class="left-section">
      <span class="app-title">Vuno</span>
      {#if currentFile}
        <span class="current-file">{currentFile}</span>
      {/if}
    </div>
    <div class="center-section">
      <span class="status-message">{statusMessage}</span>
    </div>
    <div class="right-section">
      <span class="cursor-position">
        {#if currentCursorPosition}
          Line {currentCursorPosition.line + 1}, Col {currentCursorPosition.column + 1}
        {/if}
      </span>
      <button class="window-control minimize" on:click={minimizeWindow}>_</button>
      <button class="window-control maximize" on:click={maximizeWindow}>□</button>
      <button class="window-control close" on:click={closeWindow}>×</button>
    </div>
  </div>
  
  <div class="editor-container">
    <Editor 
      bind:this={editorComponent}
      content={content}
      language={editorLanguage}
      settings={editorSettings}
      on:change={(e) => {
        appState.update(s => ({
          ...s,
          content: e.detail.content,
          hasUnsavedChanges: true,
          currentLineCount: e.detail.lineCount || 0,
          currentCursorPosition: e.detail.cursorPosition || { line: 0, ch: 0 }
        }));
      }}
      on:save={() => {
        // Handle save event
        console.log('Save requested from editor');
        // TODO: Implement save functionality
      }}
    />
    
    {#if showMarkdownPreview && mode === 'markdown'}
      <div class="markdown-preview" transition:fade={{ duration: 150 }}>
        <MarkdownRenderer content={content} />
      </div>
    {/if}
  </div>
  
  {#if commandMode}
    <div class="command-overlay">
      <CommandBar 
        bind:this={commandBarComponent}
        title={commandBarTitle || 'What would you like to do?'}
        commandText={commandText}
        showSuggestions={showSuggestions}
        suggestions={suggestions}
        on:close={handleCommandClose}
        on:input={handleCommandInput}
        on:execute={(e) => executeCommand(e.detail.command)}
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
  
  <Toast />
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    font-family: 'Onest', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen-Sans, Ubuntu, Cantarell, 'Helvetica Neue', sans-serif;
    background-color: #1e1e1e;
    color: #e0e0e0;
    height: 100vh;
    width: 100vw;
    overflow: hidden;
  }
  
  :global(html) {
    height: 100%;
    width: 100%;
  }
  
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    width: 100vw;
    background-color: #1e1e1e;
    overflow: hidden;
  }
  
  .editor-container {
    flex: 1;
    position: relative;
    overflow: hidden;
  }
  
  .status-bar {
    display: flex;
    justify-content: space-between;
    padding: 4px 8px;
    background-color: #1a1a1a;
    color: #cccccc;
    font-size: 12px;
    user-select: none;
    border-bottom: 1px solid #333;
    font-family: 'Onest', sans-serif;
    cursor: grab;
    -webkit-user-select: none;
    -webkit-app-region: drag;
  }
  
  .status-bar:active {
    cursor: grabbing;
  }

</style>

<MarkdownRenderer
  content={content}
/>
