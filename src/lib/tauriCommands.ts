// بسم الله الرحمن الرحيم
// TypeScript definitions for Tauri backend commands

import { invoke } from '@tauri-apps/api/tauri';

// ============================================================================
// Type Definitions
// ============================================================================

export interface GitStatus {
  branch: string;
  modified: string[];
  added: string[];
  deleted: string[];
  untracked: string[];
}

export interface GitCommit {
  hash: string;
  author: string;
  date: string;
  message: string;
}

export interface GitBranch {
  name: string;
  is_current: boolean;
}

export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
}

export interface PerplexitySearchResponse {
  results: SearchResult[];
  answer: string;
}

export interface Diagnostic {
  line: number;
  column: number;
  message: string;
  severity: 'error' | 'warning' | 'info' | 'hint';
}

export interface CompletionItem {
  label: string;
  kind: string;
  detail?: string;
  documentation?: string;
}

export interface Position {
  line: number;
  character: number;
}

export interface CommandResult {
  success: boolean;
  output: string;
  error?: string;
  command_type: string;
}

// ============================================================================
// Git Commands
// ============================================================================

export async function gitStatus(workingDir?: string): Promise<GitStatus> {
  return invoke('git_status', { workingDir });
}

export async function gitAdd(files: string[], workingDir?: string): Promise<string> {
  return invoke('git_add', { files, workingDir });
}

export async function gitCommit(message: string, workingDir?: string): Promise<string> {
  return invoke('git_commit', { message, workingDir });
}

export async function gitPush(remote?: string, branch?: string, workingDir?: string): Promise<string> {
  return invoke('git_push', { remote, branch, workingDir });
}

export async function gitPull(remote?: string, branch?: string, workingDir?: string): Promise<string> {
  return invoke('git_pull', { remote, branch, workingDir });
}

export async function gitBranchList(workingDir?: string): Promise<GitBranch[]> {
  return invoke('git_branch_list', { workingDir });
}

export async function gitCheckout(branch: string, createNew: boolean = false, workingDir?: string): Promise<string> {
  return invoke('git_checkout', { branch, createNew, workingDir });
}

export async function gitDiff(filePath?: string, workingDir?: string): Promise<string> {
  return invoke('git_diff', { filePath, workingDir });
}

export async function gitLog(limit?: number, workingDir?: string): Promise<GitCommit[]> {
  return invoke('git_log', { limit, workingDir });
}

export async function gitInit(workingDir?: string): Promise<string> {
  return invoke('git_init', { workingDir });
}

export async function gitClone(url: string, destination?: string, workingDir?: string): Promise<string> {
  return invoke('git_clone', { url, destination, workingDir });
}

// ============================================================================
// Perplexity Search Commands
// ============================================================================

export async function getPerplexityKey(): Promise<string> {
  return invoke('get_perplexity_key');
}

export async function setPerplexityKey(apiKey: string): Promise<void> {
  return invoke('set_perplexity_key', { apiKey });
}

export async function searchWeb(query: string, apiKey?: string): Promise<PerplexitySearchResponse> {
  return invoke('search_web', { query, apiKey });
}

// ============================================================================
// LSP (Language Server Protocol) Commands
// ============================================================================

export async function startLspServer(language: string): Promise<string> {
  return invoke('start_lsp_server', { language });
}

export async function getRunningLspServers(): Promise<string[]> {
  return invoke('get_running_lsp_servers');
}

export async function getDiagnostics(
  filePath: string,
  content: string,
  language: string
): Promise<Diagnostic[]> {
  return invoke('get_diagnostics', { filePath, content, language });
}

export async function getCompletions(
  filePath: string,
  content: string,
  position: Position,
  language: string
): Promise<CompletionItem[]> {
  return invoke('get_completions', { filePath, content, position, language });
}

export async function formatDocument(content: string, language: string): Promise<string> {
  return invoke('format_document', { content, language });
}

export async function checkLspAvailable(language: string): Promise<boolean> {
  return invoke('check_lsp_available', { language });
}

// ============================================================================
// Document-Aware AI Commands
// ============================================================================

export async function analyzeDocument(
  content: string,
  language?: string,
  apiKey?: string
): Promise<CommandResult> {
  return invoke('analyze_document', { content, language, apiKey });
}

export async function suggestImprovements(
  content: string,
  language?: string,
  apiKey?: string
): Promise<CommandResult> {
  return invoke('suggest_improvements', { content, language, apiKey });
}

export async function generateTests(
  content: string,
  language?: string,
  apiKey?: string
): Promise<CommandResult> {
  return invoke('generate_tests', { content, language, apiKey });
}

export async function refactorCode(
  content: string,
  language: string | undefined,
  instruction: string,
  apiKey: string
): Promise<CommandResult> {
  return invoke('refactor_code', { content, language, instruction, apiKey });
}

export async function explainSelection(
  selection: string,
  language?: string,
  apiKey?: string
): Promise<CommandResult> {
  return invoke('explain_selection', { selection, language, apiKey });
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get all available Git commands as suggestions
 */
export function getGitCommandSuggestions(): string[] {
  return [
    'git status',
    'git add .',
    'git commit -m "message"',
    'git push',
    'git pull',
    'git log',
    'git branch',
    'git checkout',
    'git diff',
    'git init',
  ];
}

/**
 * Get all available AI commands as suggestions
 */
export function getAICommandSuggestions(): string[] {
  return [
    'analyze document',
    'suggest improvements',
    'generate tests',
    'refactor code',
    'explain selection',
    'search web',
  ];
}

/**
 * Get all available LSP commands as suggestions
 */
export function getLspCommandSuggestions(): string[] {
  return [
    'start lsp server',
    'format document',
    'show diagnostics',
    'get completions',
  ];
}

/**
 * Check if a string is a git command
 */
export function isGitCommand(command: string): boolean {
  return command.trim().toLowerCase().startsWith('git ');
}

/**
 * Check if a string is an AI command
 */
export function isAICommand(command: string): boolean {
  const aiCommands = ['analyze', 'suggest', 'generate', 'refactor', 'explain', 'search web'];
  const lowerCommand = command.trim().toLowerCase();
  return aiCommands.some(cmd => lowerCommand.startsWith(cmd));
}
