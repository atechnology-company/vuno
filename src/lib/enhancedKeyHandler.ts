import { appState } from '../stores/appState';
import { get } from 'svelte/store';

// Check if we're in a browser environment
const isBrowser = typeof window !== 'undefined';

// Dynamically import Tauri APIs only in browser
let listen: any = null;
let invoke: any = null;

if (isBrowser) {
  import('@tauri-apps/api/event').then(module => {
    listen = module.listen;
  });
  import('@tauri-apps/api/tauri').then(module => {
    invoke = module.invoke;
  });
}

export interface KeyBinding {
  id: string;
  keys: string[];
  action: string;
  description: string;
  context: string;
}

export interface KeyEvent {
  key: string;
  modifiers: string[];
  action: string;
  timestamp: number;
}

export class EnhancedKeyHandler {
  private static instance: EnhancedKeyHandler;
  private isInitialized: boolean = false;
  private unlistenHandlers: Array<(() => void) | undefined> = [];
  private commandBarInput: HTMLInputElement | null = null;
  private backendForceCloseUnlistener: (() => void) | null = null;
  private domFallbackInstalled: boolean = false;
  
  private constructor() {}
  
  public static getInstance(): EnhancedKeyHandler {
    if (!EnhancedKeyHandler.instance) {
      EnhancedKeyHandler.instance = new EnhancedKeyHandler();
    }
    return EnhancedKeyHandler.instance;
  }
  
  public async initialize() {
    if (this.isInitialized) return;
    
    if (!isBrowser) {
      console.warn('EnhancedKeyHandler: Not in browser environment, skipping initialization');
      return;
    }
    
    // Wait for Tauri APIs to be loaded
    if (!listen || !invoke) {
      console.log('EnhancedKeyHandler: Waiting for Tauri APIs to load...');
      await new Promise(resolve => {
        const checkInterval = setInterval(() => {
          if (listen && invoke) {
            clearInterval(checkInterval);
            resolve(void 0);
          }
        }, 100);
        
        // Timeout after 5 seconds
        setTimeout(() => {
          clearInterval(checkInterval);
          console.warn('EnhancedKeyHandler: Timeout waiting for Tauri APIs');
          resolve(void 0);
        }, 5000);
      });
    }
    
    if (!listen || !invoke) {
      console.warn('EnhancedKeyHandler: Tauri APIs not available, using fallback mode');
      // Minimal DOM fallback for ESC and Cmd/Ctrl+K
      this.installDOMFallback();
      this.isInitialized = true;
      return;
    }
    
    try {
      // Listen for key actions from Rust backend
      const keyActionUnlisten = await listen('key_action', (event: any) => {
        const action = event.payload.action;
        this.handleKeyAction(action);
      });
      
      this.unlistenHandlers.push(keyActionUnlisten);
      
      // Listen for force close command bar events from backend
      this.backendForceCloseUnlistener = await listen('force_close_command_bar', (event: any) => {
        console.log('Received force close command from backend:', event.payload);
        this.forceCloseCommandBar();
      });
      
      // Listen for raw key events (optional, for debugging)
      const keyEventUnlisten = await listen('key_event', (event: any) => {
        const keyEvent = event.payload as KeyEvent;
        this.handleKeyEvent(keyEvent);
      });
      
      this.unlistenHandlers.push(keyEventUnlisten);
      
      // Start key monitoring in Rust backend with fallback awareness
      if (invoke) {
        try {
          await invoke('start_key_monitoring_with_fallback');
        } catch (e) {
          console.warn('start_key_monitoring_with_fallback failed, trying start_key_monitoring', e);
          try { await invoke('start_key_monitoring'); } catch {}
        }
      }
      
      console.log('EnhancedKeyHandler initialized with Rust backend');
      this.isInitialized = true;
      // If backend indicates monitoring issues, enable DOM fallback
      try {
        if (listen) {
          const u1 = await listen('key_monitoring_error', () => this.installDOMFallback());
          const u2 = await listen('key_monitoring_stopped', () => this.installDOMFallback());
          const u3 = await listen('key_monitoring_mode', (e: any) => {
            if (e?.payload?.mode === 'window_focused_only') this.installDOMFallback();
          });
          this.unlistenHandlers.push(u1, u2, u3);
        }
        // Proactively check accessibility and install fallback if needed (macOS)
        if (invoke) {
          try {
            const ok = await invoke('check_accessibility_permissions');
            if (ok === false) {
              this.installDOMFallback();
            }
          } catch {
            // If the check fails, be safe and install fallback
            this.installDOMFallback();
          }
        }
      } catch {}
    } catch (error) {
      console.error('Failed to initialize EnhancedKeyHandler:', error);
      // Even if backend init fails, install DOM fallback so keys still work when focused
      this.installDOMFallback();
      this.isInitialized = false;
      throw error;
    }
  }
  
  private handleKeyAction(action: string) {
    console.log(`Executing key action: ${action}`);
    
    const state = get(appState);
    
    switch (action) {
      case 'toggle_command_bar':
        this.toggleCommandBar();
        break;
        
      case 'toggle_help':
        if (state.showHelp) {
          appState.update(s => ({ ...s, showHelp: false }));
        } else {
          this.toggleHelp();
        }
        break;
        
      case 'escape':
        this.handleEscape();
        break;
        
      case 'save_file':
        this.saveCurrentFile();
        break;
        
      case 'new_file':
        this.createNewFile();
        break;
        
      case 'open_file':
        this.openFileDialog();
        break;
        
      default:
        // Emit custom event for other actions
        const customEvent = new CustomEvent('key_action', {
          detail: { action },
          bubbles: true
        });
        document.body.dispatchEvent(customEvent);
        break;
    }
  }
  
  private handleKeyEvent(keyEvent: KeyEvent) {
    // Optional: Log key events for debugging
    if (keyEvent.action === 'press') {
      console.log(`Key pressed: ${keyEvent.key} with modifiers: ${keyEvent.modifiers.join('+')}`);
    }
  }
  
  private toggleCommandBar() {
    const state = get(appState);
    console.log('Toggling command bar, current state:', state.commandMode);
    
    appState.update(s => ({ 
      ...s, 
      commandMode: !s.commandMode,
      showTutorial: false 
    }));
    
    if (!state.commandMode && this.commandBarInput) {
      setTimeout(() => {
        this.commandBarInput?.focus();
        console.log('Focused command bar input');
      }, 50);
    }
  }
  
  private toggleHelp() {
    const state = get(appState);
    console.log('Toggling help, current state:', state.showHelp);
    appState.update(s => ({ ...s, showHelp: !s.showHelp }));
  }
  
  private handleEscape() {
    const state = get(appState);
    console.log('Handling escape key, current state:', { 
      commandMode: state.commandMode, 
      showHelp: state.showHelp, 
      showTutorial: state.showTutorial 
    });
    
    if (state.showHelp) {
      console.log('Closing help window with Escape key');
      appState.update(s => ({ ...s, showHelp: false }));
      setTimeout(() => {
        this.focusEditor();
      }, 100);
    } else if (state.commandMode) {
      console.log('Closing command bar with Escape key');
      appState.update(s => ({ 
        ...s, 
        commandMode: false, 
        commandText: '', 
        suggestions: [],
        commandExecutionInProgress: false,
        isAiLoading: false
      }));
      setTimeout(() => {
        this.focusEditor();
      }, 50);
    } else if (state.showTutorial) {
      console.log('Closing tutorial with Escape key');
      appState.update(s => ({ ...s, showTutorial: false }));
    }
  }
  
  private forceCloseCommandBar() {
    console.log('Force closing command bar from backend');
    appState.update(s => ({ 
      ...s, 
      commandMode: false, 
      commandText: '', 
      suggestions: [],
      commandExecutionInProgress: false,
      isAiLoading: false
    }));
    // Force multiple state updates to ensure closure
    setTimeout(() => {
      appState.update(s => ({ 
        ...s, 
        commandMode: false 
      }));
      this.focusEditor();
    }, 25);
    setTimeout(() => {
      appState.update(s => ({ 
        ...s, 
        commandMode: false 
      }));
    }, 50);
  }
  
  private async saveCurrentFile() {
    try {
      if (!invoke) {
        console.warn('Tauri invoke not available for saveCurrentFile');
        return;
      }
      
      // Get current buffer info
      const buffers = await invoke('list_buffers') as any[];
      if (buffers.length > 0) {
        const currentBuffer = buffers[0]; // Assume first buffer is active
        await invoke('save_file', { 
          bufferId: currentBuffer.id,
          path: null // Use existing path
        });
        
        appState.update(s => ({ 
          ...s, 
          statusMessage: 'File saved successfully' 
        }));
        
        // Emit save event
        const saveEvent = new CustomEvent('file_saved', {
          detail: { bufferId: currentBuffer.id },
          bubbles: true
        });
        document.body.dispatchEvent(saveEvent);
      }
    } catch (error) {
      console.error('Error saving file:', error);
      appState.update(s => ({ 
        ...s, 
        statusMessage: `Error saving file: ${error}` 
      }));
    }
  }
  
  private async createNewFile() {
    try {
      if (!invoke) {
        console.warn('Tauri invoke not available for createNewFile');
        return;
      }
      
      const bufferId = await invoke('create_new_buffer') as number;
      
      // Emit new file event
      const newFileEvent = new CustomEvent('new_file_created', {
        detail: { bufferId },
        bubbles: true
      });
      document.body.dispatchEvent(newFileEvent);
      
      appState.update(s => ({ 
        ...s, 
        statusMessage: 'New file created' 
      }));
    } catch (error) {
      console.error('Error creating new file:', error);
      appState.update(s => ({ 
        ...s, 
        statusMessage: `Error creating new file: ${error}` 
      }));
    }
  }
  
  private openFileDialog() {
    // Emit open file dialog event
    const openEvent = new CustomEvent('open_file_dialog', {
      bubbles: true
    });
    document.body.dispatchEvent(openEvent);
  }
  
  private focusEditor() {
    setTimeout(() => {
      try {
        // Try focusing editor directly
        const editorElement = document.querySelector('.cm-editor');
        if (editorElement) {
          (editorElement as HTMLElement).focus();
          console.log('Focused editor element');
          
          // Dispatch synthetic keypress to force cursor to appear
          const fakeEvent = new KeyboardEvent('keydown', {
            key: 'ArrowRight',
            bubbles: true
          });
          editorElement.dispatchEvent(fakeEvent);
          
          // Try to find cursor element and make it visible
          const cursor = document.querySelector('.cm-cursor');
          if (cursor) {
            (cursor as HTMLElement).style.visibility = 'visible';
            (cursor as HTMLElement).style.display = 'block';
          }
        }
        
        // Also dispatch global focus event
        const focusEvent = new CustomEvent('editor-focus-requested', {
          bubbles: true
        });
        document.body.dispatchEvent(focusEvent);
        
        console.log('Editor focus attempted');
      } catch (error) {
        console.error('Error focusing editor:', error);
      }
    }, 200);
  }
  
  public async registerKeyBinding(binding: KeyBinding): Promise<void> {
    try {
      if (!invoke) {
        console.warn('Tauri invoke not available for registerKeyBinding');
        return;
      }
      await invoke('register_key_binding', { binding });
      console.log('Key binding registered:', binding);
    } catch (error) {
      console.error('Error registering key binding:', error);
      throw error;
    }
  }
  
  public async unregisterKeyBinding(id: string): Promise<void> {
    try {
      if (!invoke) {
        console.warn('Tauri invoke not available for unregisterKeyBinding');
        return;
      }
      await invoke('unregister_key_binding', { id });
      console.log('Key binding unregistered:', id);
    } catch (error) {
      console.error('Error unregistering key binding:', error);
      throw error;
    }
  }
  
  public async getKeyBindings(): Promise<KeyBinding[]> {
    try {
      if (!invoke) {
        console.warn('Tauri invoke not available for getKeyBindings');
        return [];
      }
      return await invoke('get_key_bindings') as KeyBinding[];
    } catch (error) {
      console.error('Error getting key bindings:', error);
      return [];
    }
  }
  
  public async cleanup() {
    try {
      // Stop key monitoring
      if (invoke) {
        await invoke('stop_key_monitoring');
      }
      
      // Remove all event listeners
      this.unlistenHandlers.forEach(unlisten => unlisten && unlisten());
      this.unlistenHandlers = [];
      
      // Clean up backend force close listener
      if (this.backendForceCloseUnlistener) {
        this.backendForceCloseUnlistener();
        this.backendForceCloseUnlistener = null;
      }
      
      this.isInitialized = false;
      console.log('EnhancedKeyHandler cleaned up');
    } catch (error) {
      console.error('Error during cleanup:', error);
    }
  }
  
  public setCommandBarInput(element: HTMLInputElement) {
    this.commandBarInput = element;
  }
  
  // Method to handle special cases where we need DOM-level key handling
  public setupDOMKeyHandlers() {
    // Add minimal DOM key handlers for cases where Rust can't capture
    if (typeof document !== 'undefined') {
      document.addEventListener('keydown', this.handleDOMKeyDown.bind(this));
    }
  }

  private installDOMFallback() {
    if (this.domFallbackInstalled) return;
    this.domFallbackInstalled = true;
    this.setupDOMKeyHandlers();
    console.log('[EnhancedKeyHandler] DOM fallback key handling installed');
  }
  
  private handleDOMKeyDown(event: KeyboardEvent) {
    const state = get(appState);
    // Fallback: toggle command bar with Cmd/Ctrl+K and Escape when Rust can't toggle
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      // Only prevent default if not focused on an input/textarea
      const target = event.target as HTMLElement;
      if (!target.matches('input, textarea, [contenteditable]')) {
        event.preventDefault();
        this.toggleCommandBar();
        return;
      }
    }
    if (event.key === 'Escape') {
      // Only prevent default if not focused on an input/textarea
      const target = event.target as HTMLElement;
      if (!target.matches('input, textarea, [contenteditable]') || state.commandMode || state.showHelp) {
        event.preventDefault();
        this.handleEscape();
        return;
      }
    }

    // Only handle specific cases that Rust might miss
    if (state.commandMode && event.target === this.commandBarInput) {
      if (event.key === 'Enter') {
        const commandText = this.commandBarInput?.value || '';
        if (commandText.trim()) {
          // Emit command event
          const commandEvent = new CustomEvent('command', {
            detail: { command: commandText.trim() },
            bubbles: true
          });
          document.body.dispatchEvent(commandEvent);
          
          appState.update(s => ({ 
            ...s, 
            commandMode: false,
            commandText: '' 
          }));
          
          if (this.commandBarInput) {
            this.commandBarInput.value = '';
          }
        }
        event.preventDefault();
      }
    }
  }
}
