// بسم الله الرحمن الرحيم
// GitHub Copilot TypeScript Integration

import { invoke } from '@tauri-apps/api/tauri';

// Types
export interface CopilotPosition {
  line: number;
  character: number;
}

export interface CopilotRange {
  start: CopilotPosition;
  end: CopilotPosition;
}

export interface InlineCompletionItem {
  insertText: string;
  range?: CopilotRange;
  command?: any;
}

export interface InlineCompletionList {
  items: InlineCompletionItem[];
}

export interface CopilotStatus {
  status: 'Normal' | 'Error' | 'Warning' | 'Inactive';
  message: string;
  signedIn: boolean;
}

export interface SignInResponse {
  userCode: string;
  verificationUri: string;
}

// Commands

/**
 * Start the GitHub Copilot language server
 * @param workspacePath Optional workspace path
 */
export async function copilotStartServer(workspacePath?: string): Promise<void> {
  return await invoke('copilot_start_server', { workspacePath });
}

/**
 * Stop the GitHub Copilot language server
 */
export async function copilotStopServer(): Promise<void> {
  return await invoke('copilot_stop_server');
}

/**
 * Get the current Copilot server status
 */
export async function copilotGetStatus(): Promise<CopilotStatus> {
  return await invoke('copilot_get_status');
}

/**
 * Initiate the Copilot sign-in flow
 * Returns a user code that should be entered at the verification URI
 */
export async function copilotSignIn(): Promise<SignInResponse> {
  return await invoke('copilot_sign_in');
}

/**
 * Sign out of Copilot
 */
export async function copilotSignOut(): Promise<void> {
  return await invoke('copilot_sign_out');
}

/**
 * Get inline code completions from Copilot
 * @param fileUri The file URI (e.g., "file:///path/to/file.ts")
 * @param content The current file content
 * @param line The cursor line (0-indexed)
 * @param character The cursor character position (0-indexed)
 * @param version The document version number
 */
export async function copilotGetCompletions(
  fileUri: string,
  content: string,
  line: number,
  character: number,
  version: number
): Promise<InlineCompletionList> {
  return await invoke('copilot_get_completions', {
    fileUri,
    content,
    line,
    character,
    version,
  });
}

/**
 * Notify Copilot that a completion was accepted (for telemetry)
 * @param completionId The ID of the accepted completion
 */
export async function copilotAcceptCompletion(completionId: string): Promise<void> {
  return await invoke('copilot_accept_completion', { completionId });
}

/**
 * Notify Copilot that a completion was rejected (for telemetry)
 * @param completionId The ID of the rejected completion
 */
export async function copilotRejectCompletion(completionId: string): Promise<void> {
  return await invoke('copilot_reject_completion', { completionId });
}

// Helper functions

/**
 * Check if Copilot is currently running and signed in
 */
export async function isCopilotReady(): Promise<boolean> {
  try {
    const status = await copilotGetStatus();
    return status.status === 'Normal' && status.signedIn;
  } catch {
    return false;
  }
}

/**
 * Initialize Copilot with the current workspace
 * @param workspacePath Path to the workspace directory
 */
export async function initializeCopilot(workspacePath?: string): Promise<void> {
  try {
    await copilotStartServer(workspacePath);
    const status = await copilotGetStatus();
    
    if (!status.signedIn) {
      console.warn('Copilot started but user is not signed in');
    }
  } catch (error) {
    console.error('Failed to initialize Copilot:', error);
    throw error;
  }
}

/**
 * Open the browser for Copilot authentication
 * @returns The user code that needs to be entered
 */
export async function authenticateCopilot(): Promise<string> {
  try {
    const response = await copilotSignIn();
    
    // Open the verification URI in the browser
    const { open } = await import('@tauri-apps/api/shell');
    await open(response.verificationUri);
    
    return response.userCode;
  } catch (error) {
    console.error('Failed to authenticate with Copilot:', error);
    throw error;
  }
}
