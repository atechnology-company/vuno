import { appState } from '../stores/appState';
import { get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/window';
import { register, unregisterAll } from '@tauri-apps/api/globalShortcut';

// This class will use DOM level event listeners for key handling
export class TauriKeyHandler {
  private static instance: TauriKeyHandler;
  private commandBarInput: HTMLInputElement | null = null;
  private isInitialized: boolean = false;
  private unlistenHandlers: Array<(() => void) | undefined> = [];
  
  private constructor() {}
  
  public static getInstance(): TauriKeyHandler {
    if (!TauriKeyHandler.instance) {
      TauriKeyHandler.instance = new TauriKeyHandler();
    }
    return TauriKeyHandler.instance;
  }
  
  public async initialize() {
    if (this.isInitialized) return;
    
    try {
      // Listen for window close event
      const tauriWindow = await getCurrent();
      const unlisten = await listen('window-will-close', () => {
        console.log('Window is closing, cleaning up');
        this.cleanup();
      });
      
      this.unlistenHandlers.push(unlisten);
      
      // Listen for hotkey events from Rust backend
      const hotkeyUnlisten = await listen('hotkey-triggered', (event: any) => {
        const hotkeyId = event.payload as string;
        this.handleHotkeyTriggered(hotkeyId);
      });
      
      this.unlistenHandlers.push(hotkeyUnlisten);
      
      console.log('TauriKeyHandler initialized with Rust-based hotkey system');
      this.isInitialized = true;
    } catch (error) {
      console.error('Failed to initialize TauriKeyHandler:', error);
      this.isInitialized = false;
      throw error;
    }
  }
  
  private async registerGlobalShortcuts() {
    try {
      // Register Command+K / Ctrl+K for command bar
      await register('CommandOrControl+K', () => {
        console.log('Global shortcut triggered: CommandOrControl+K');
        this.toggleCommandBar();
      });
      
      // Register F1 for help
      await register('F1', () => {
        console.log('Global shortcut triggered: F1');
        const state = get(appState);
        if (state.showHelp) {
          // If help is already open, close it
          appState.update(s => ({ ...s, showHelp: false }));
          console.log('Closing help window with F1 global shortcut');
        } else {
          // Otherwise toggle it
          this.toggleHelp();
          console.log('Opening help window with F1 global shortcut');
        }
      });
      
      // Register Escape for closing menus
      await register('Escape', () => {
        console.log('Global shortcut triggered: Escape');
        const state = get(appState);
        if (state.showHelp) {
          // Force close help window if it's open
          appState.update(s => ({ ...s, showHelp: false }));
          console.log('Force closing help window with Escape global shortcut');
        } else {
          this.handleEscape();
        }
      });
      
      console.log('Global shortcuts registered successfully');
    } catch (error) {
      console.error('Failed to register global shortcuts:', error);
      // Don't re-throw the error, just return false to indicate failure
      // This allows the initialize method to continue with fallback DOM handlers
      return false;
    }
    return true;
  }
  
  private handleHotkeyTriggered = (hotkeyId: string) => {
    console.log(`Executing hotkey command for ID: ${hotkeyId}`);
    
    // Get the current app state
    const state = get(appState);
    
    switch (hotkeyId) {
      case 'command-bar':
        this.toggleCommandBar();
        break;
      case 'help':
        if (state.showHelp) {
          appState.update(s => ({ ...s, showHelp: false }));
        } else {
          this.toggleHelp();
        }
        break;
      case 'escape':
        if (state.showHelp) {
          appState.update(s => ({ ...s, showHelp: false }));
        } else if (state.commandMode) {
          this.handleEscape();
        }
        break;
      case 'enter':
        if (state.commandMode && state.commandText) {
          this.handleEnter();
        }
        break;
    }
  }
  
  private toggleCommandBar() {
    const state = get(appState);
    console.log('Toggling command bar, current state:', state.commandMode);
    appState.update(s => ({ ...s, commandMode: !s.commandMode }));
    
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
    console.log('Handling escape key');
    
    if (state.showHelp) {
      console.log('Closing help window with Escape key');
      appState.update(s => ({ ...s, showHelp: false }));
      // Add a small delay before focusing editor
      setTimeout(() => {
        this.focusEditor();
      }, 100);
      return; // Exit early to prevent further handling
    } else if (state.commandMode) {
      console.log('Closing command bar with Escape key');
      appState.update(s => ({ ...s, commandMode: false }));
      this.focusEditor();
    }
  }
  
  private focusEditor() {
    // Focus back on editor after closing menus with increased delay
    setTimeout(() => {
      try {
        // Try focusing editor directly
        const editorElement = document.querySelector('.cm-editor');
        if (editorElement) {
          (editorElement as HTMLElement).focus();
          console.log('Focused editor element directly');
          
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
        
        console.log('Editor focus attempted with multiple methods');
      } catch (error) {
        console.error('Error focusing editor:', error);
      }
    }, 200); // Increased delay
  }
  
  private handleEnter() {
    const state = get(appState);
    console.log('Handling enter key in command mode');
    
    if (state.commandMode && state.commandText) {
      const event = new CustomEvent('command', {
        detail: { command: state.commandText },
        bubbles: true
      });
      document.body.dispatchEvent(event);
      appState.update(s => ({ ...s, commandMode: false, commandText: '' }));
    }
  }
  
  public async cleanup() {
    // Remove all Tauri event listeners
    this.unlistenHandlers.forEach(unlisten => unlisten && unlisten());
    this.unlistenHandlers = [];
    
    this.isInitialized = false;
    console.log('TauriKeyHandler cleaned up');
  }
  
  public setCommandBarInput(element: HTMLInputElement) {
    this.commandBarInput = element;
  }
}