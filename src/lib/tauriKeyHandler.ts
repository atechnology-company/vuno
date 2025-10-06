import { appState } from '../stores/appState';
import { get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { getCurrent } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/tauri';

// This class will use DOM level event listeners for key handling
export class TauriKeyHandler {
  private static instance: TauriKeyHandler;
  private commandBarInput: HTMLInputElement | null = null;
  private isInitialized: boolean = false;
  private unlistenHandlers: Array<(() => void) | undefined> = [];
  private hasAccessibilityPermissions: boolean = false;
  private domKeydownHandler?: (e: KeyboardEvent) => void;
  
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
      // Check if we have accessibility permissions on macOS
      try {
        this.hasAccessibilityPermissions = await invoke('check_accessibility_permissions');
        console.log('Accessibility permissions:', this.hasAccessibilityPermissions);
      } catch (error) {
        console.warn('Could not check accessibility permissions:', error);
        this.hasAccessibilityPermissions = false;
      }
      
      // Listen for window close event
      const tauriWindow = await getCurrent();
      const unlisten = await listen('window-will-close', () => {
        console.log('Window is closing, cleaning up');
        this.cleanup();
      });
      
      this.unlistenHandlers.push(unlisten);
      
      // Listen for key events from Rust backend
      const keyEventUnlisten = await listen('key_event', (event: any) => {
        this.handleKeyEvent(event.payload);
      });
      
      this.unlistenHandlers.push(keyEventUnlisten);
      
      // Listen for key action events from Rust backend
      const keyActionUnlisten = await listen('key_action', (event: any) => {
        this.handleKeyAction(event.payload);
      });
      
      this.unlistenHandlers.push(keyActionUnlisten);
      
      // Listen for key monitoring errors
      const errorUnlisten = await listen('key_monitoring_error', (event: any) => {
        console.warn('Key monitoring error:', event.payload);
        this.handleKeyMonitoringError(event.payload);
      });
      
      this.unlistenHandlers.push(errorUnlisten);
      
      // Listen for key monitoring mode changes
      const modeUnlisten = await listen('key_monitoring_mode', (event: any) => {
        console.log('Key monitoring mode:', event.payload);
        this.handleKeyMonitoringMode(event.payload);
      });
      
      this.unlistenHandlers.push(modeUnlisten);
      
  console.log('TauriKeyHandler initialized (Tauri events only, no DOM fallback)');
      this.isInitialized = true;

      // Add a minimal, window-scoped DOM listener for Escape to support ESC-to-open/close
      // We do NOT register global ESC at the OS level for safety.
      if (typeof document !== 'undefined') {
        this.domKeydownHandler = (e: KeyboardEvent) => {
          if (e.key === 'Escape') {
            e.preventDefault();
            this.handleEscape();
          }
        };
        document.addEventListener('keydown', this.domKeydownHandler, { capture: true });
      }
    } catch (error) {
      console.error('Failed to initialize TauriKeyHandler:', error);
      this.isInitialized = false;
      throw error;
    }
  }
  
  private handleKeyEvent(event: any) {
    // Handle raw key events from Rust
    console.log('Key event:', event);
  }
  
  private handleKeyAction(event: any) {
    // Handle key action events from Rust
    const action = event.action;
    console.log('Key action:', action);
    
    switch (action) {
      case 'toggle_command_bar':
        this.toggleCommandBar();
        break;
      case 'toggle_help':
        this.toggleHelp();
        break;
      case 'escape':
        this.handleEscape();
        break;
      case 'save_file':
        this.handleSaveFile();
        break;
      case 'new_file':
        this.handleNewFile();
        break;
      case 'open_file':
        this.handleOpenFile();
        break;
    }
  }
  
  private handleKeyMonitoringError(error: any) {
    console.warn('Key monitoring error:', error);
    if (error.error === 'accessibility_permission_required') {
      // Show user notification about accessibility permissions
      const event = new CustomEvent('accessibility-permission-required', {
        detail: error,
        bubbles: true
      });
      document.body.dispatchEvent(event);
    }
  }
  
  private handleKeyMonitoringMode(mode: any) {
    console.log('Key monitoring mode changed:', mode);
    // Notify user about the mode change
    const event = new CustomEvent('key-monitoring-mode-changed', {
      detail: mode,
      bubbles: true
    });
    document.body.dispatchEvent(event);
  }

  private async registerGlobalShortcuts() {
    // This method is no longer used, but kept for compatibility
    console.log('Global shortcuts registration skipped - using Rust-based key handling');
    return true;
  }

  private handleHotkeyTriggered = (hotkeyId: string) => {
    // Legacy method - redirect to handleKeyAction
    this.handleKeyAction({ action: hotkeyId.replace('-', '_') });
  }
  
  private handleSaveFile() {
    const event = new CustomEvent('save-file', { bubbles: true });
    document.body.dispatchEvent(event);
  }
  
  private handleNewFile() {
    const event = new CustomEvent('new-file', { bubbles: true });
    document.body.dispatchEvent(event);
  }
  
  private handleOpenFile() {
    const event = new CustomEvent('open-file', { bubbles: true });
    document.body.dispatchEvent(event);
  }
  
  private toggleCommandBar() {
    const state = get(appState);
    const nextMode = !state.commandMode;
    console.log('Toggling command bar, next state:', nextMode);
    appState.update(s => ({ ...s, commandMode: nextMode, showTutorial: false }));

    if (nextMode) {
      const el = this.commandBarInput;
      const isConnected = !!(el && typeof el === 'object' && 'isConnected' in el ? (el as any).isConnected : document.contains(el as any));
      if (el && isConnected) {
        setTimeout(() => {
          try { el.focus(); } catch {}
          console.log('Focused command bar input');
        }, 50);
      } else {
        // Ask parent to focus/create the input if we don't have a valid ref
        const focusEvent = new CustomEvent('command-input-focus-requested', { bubbles: true });
        document.body.dispatchEvent(focusEvent);
      }
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
    // Always allow ESC to close any toasts first
    try {
      const evt = new CustomEvent('toast-close-all', { bubbles: true });
      document.body.dispatchEvent(evt);
    } catch {}
    
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
      return;
    } else if (state.showTutorial) {
      console.log('Closing tutorial with Escape key');
      appState.update(s => ({ ...s, showTutorial: false }));
      this.focusEditor();
      return;
    } else {
      // Nothing is open; ESC should OPEN the command bar
      console.log('Opening command bar with Escape key');
      // Use the same toggle path to ensure focus flows correctly
      this.toggleCommandBar();
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
  // Enter is handled by the CommandBar component input; no-op here
  }
  
  public async cleanup() {
    // Remove all Tauri event listeners
    this.unlistenHandlers.forEach(unlisten => unlisten && unlisten());
    this.unlistenHandlers = [];
    if (this.domKeydownHandler && typeof document !== 'undefined') {
      document.removeEventListener('keydown', this.domKeydownHandler, { capture: true } as any);
      this.domKeydownHandler = undefined;
    }
    
    this.isInitialized = false;
    console.log('TauriKeyHandler cleaned up');
  }
  
  public setCommandBarInput(element: HTMLInputElement | null) {
    this.commandBarInput = element;
  }
}