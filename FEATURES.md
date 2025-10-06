# Vuno Enhanced Features

This document describes the new agentic features added to Vuno for AI-powered editing, web search, IntelliSense, and Git integration.

## Overview

Vuno now includes powerful features that make it a more intelligent and capable code editor:

1. **Document-Aware AI** - AI that understands your code and can analyze, improve, and explain it
2. **Perplexity Web Search** - Search the web directly from the editor
3. **Language Server Protocol (LSP)** - IntelliSense, diagnostics, and code formatting
4. **Git Integration** - Full Git workflow support from within the editor

## Features

### 1. Document-Aware AI Features

These commands analyze and improve your code using AI:

#### Analyze Document
```
analyze document
```
Analyzes the current document and provides insights about structure, purpose, and potential improvements.

**Usage from TypeScript:**
```typescript
import { analyzeDocument } from './lib/tauriCommands';

const result = await analyzeDocument(content, language, apiKey);
console.log(result.output);
```

#### Suggest Improvements
```
suggest improvements
```
Provides specific code quality, performance, security, and maintainability suggestions.

**Usage from TypeScript:**
```typescript
import { suggestImprovements } from './lib/tauriCommands';

const result = await suggestImprovements(content, language, apiKey);
console.log(result.output);
```

#### Generate Tests
```
generate tests
```
Automatically generates comprehensive unit tests for your code.

**Usage from TypeScript:**
```typescript
import { generateTests } from './lib/tauriCommands';

const result = await generateTests(content, language, apiKey);
console.log(result.output);
```

#### Refactor Code
```
refactor code: <instruction>
```
Refactors code according to specific instructions.

**Usage from TypeScript:**
```typescript
import { refactorCode } from './lib/tauriCommands';

const result = await refactorCode(
  content, 
  language, 
  "extract this into a separate function", 
  apiKey
);
console.log(result.output);
```

#### Explain Selection
```
explain selection
```
Explains what the selected code does in detail.

**Usage from TypeScript:**
```typescript
import { explainSelection } from './lib/tauriCommands';

const result = await explainSelection(selectedText, language, apiKey);
console.log(result.output);
```

### 2. Perplexity Web Search

Search the web using Perplexity AI's search API to get accurate, sourced answers.

#### Search Web
```
search web: <query>
```

**Setup:**
1. Get a Perplexity API key from https://www.perplexity.ai/
2. Set the key: `set perplexity_key YOUR_KEY`

**Usage from TypeScript:**
```typescript
import { searchWeb, setPerplexityKey } from './lib/tauriCommands';

// Set API key (one time)
await setPerplexityKey('your-api-key');

// Search the web
const result = await searchWeb('What is Rust ownership?');
console.log(result.answer);
console.log(result.results); // Citations
```

### 3. Language Server Protocol (LSP) Support

Get IntelliSense, diagnostics, and formatting for multiple languages.

**Supported Languages:**
- Rust (rust-analyzer)
- JavaScript/TypeScript (typescript-language-server)
- Python (pylsp)
- Go (gopls)
- Java (jdtls)
- C/C++ (clangd)

#### Start LSP Server
```
start lsp server: <language>
```

**Usage from TypeScript:**
```typescript
import { startLspServer } from './lib/tauriCommands';

await startLspServer('rust');
```

#### Get Completions (IntelliSense)
```typescript
import { getCompletions } from './lib/tauriCommands';

const completions = await getCompletions(
  filePath,
  content,
  { line: 10, character: 5 },
  'rust'
);

completions.forEach(item => {
  console.log(`${item.label} (${item.kind})`);
  if (item.detail) console.log(`  ${item.detail}`);
});
```

#### Get Diagnostics
```typescript
import { getDiagnostics } from './lib/tauriCommands';

const diagnostics = await getDiagnostics(filePath, content, 'rust');

diagnostics.forEach(diag => {
  console.log(`Line ${diag.line}: [${diag.severity}] ${diag.message}`);
});
```

#### Format Document
```
format document
```

**Usage from TypeScript:**
```typescript
import { formatDocument } from './lib/tauriCommands';

const formattedCode = await formatDocument(content, 'rust');
```

#### Check LSP Availability
```typescript
import { checkLspAvailable } from './lib/tauriCommands';

const available = await checkLspAvailable('rust');
if (!available) {
  console.log('Please install rust-analyzer');
}
```

### 4. Git Integration

Full Git workflow support directly from the editor.

#### Git Status
```
git status
```

**Usage from TypeScript:**
```typescript
import { gitStatus } from './lib/tauriCommands';

const status = await gitStatus();
console.log(`Branch: ${status.branch}`);
console.log(`Modified: ${status.modified.join(', ')}`);
console.log(`Untracked: ${status.untracked.join(', ')}`);
```

#### Git Add
```
git add <files>
```

**Usage from TypeScript:**
```typescript
import { gitAdd } from './lib/tauriCommands';

await gitAdd(['.']); // Add all files
// or
await gitAdd(['file1.rs', 'file2.rs']); // Add specific files
```

#### Git Commit
```
git commit -m "message"
```

**Usage from TypeScript:**
```typescript
import { gitCommit } from './lib/tauriCommands';

await gitCommit('feat: add new feature');
```

#### Git Push/Pull
```
git push
git pull
```

**Usage from TypeScript:**
```typescript
import { gitPush, gitPull } from './lib/tauriCommands';

await gitPush(); // Push to default remote/branch
await gitPush('origin', 'main'); // Push to specific remote/branch

await gitPull(); // Pull from default remote/branch
await gitPull('origin', 'main'); // Pull from specific remote/branch
```

#### Git Branch
```
git branch
```

**Usage from TypeScript:**
```typescript
import { gitBranchList } from './lib/tauriCommands';

const branches = await gitBranchList();
branches.forEach(branch => {
  console.log(`${branch.name}${branch.is_current ? ' (current)' : ''}`);
});
```

#### Git Checkout
```
git checkout <branch>
```

**Usage from TypeScript:**
```typescript
import { gitCheckout } from './lib/tauriCommands';

await gitCheckout('main'); // Switch to existing branch
await gitCheckout('feature-branch', true); // Create and switch to new branch
```

#### Git Diff
```
git diff [file]
```

**Usage from TypeScript:**
```typescript
import { gitDiff } from './lib/tauriCommands';

const diff = await gitDiff(); // Diff all files
// or
const diff = await gitDiff('src/main.rs'); // Diff specific file
console.log(diff);
```

#### Git Log
```
git log
```

**Usage from TypeScript:**
```typescript
import { gitLog } from './lib/tauriCommands';

const commits = await gitLog(10); // Get last 10 commits
commits.forEach(commit => {
  console.log(`${commit.hash.substring(0, 7)} - ${commit.message}`);
  console.log(`  by ${commit.author} on ${commit.date}`);
});
```

#### Git Init
```
git init
```

**Usage from TypeScript:**
```typescript
import { gitInit } from './lib/tauriCommands';

await gitInit('/path/to/directory');
```

#### Git Clone
```
git clone <url> [destination]
```

**Usage from TypeScript:**
```typescript
import { gitClone } from './lib/tauriCommands';

await gitClone('https://github.com/user/repo.git');
// or
await gitClone('https://github.com/user/repo.git', '/path/to/destination');
```

## Command Suggestions

All new commands are integrated into the command palette. The editor will suggest:

- AI commands when you start typing analysis-related terms
- Git commands when you type "git"
- LSP commands when working with supported languages
- Web search when you type "search"

## Configuration

### API Keys

Set your API keys for AI features:

```
set api_key YOUR_GEMINI_API_KEY
set perplexity_key YOUR_PERPLEXITY_API_KEY
```

Keys are stored securely in the app's config directory.

### LSP Servers

Install language servers for IntelliSense support:

- **Rust**: `rustup component add rust-analyzer`
- **JavaScript/TypeScript**: `npm install -g typescript-language-server`
- **Python**: `pip install python-lsp-server`
- **Go**: `go install golang.org/x/tools/gopls@latest`
- **C/C++**: Install clangd from your package manager

## Architecture

All features are implemented in the Tauri Rust backend for:
- **Performance**: Native Rust code is fast and efficient
- **Security**: API keys and Git operations handled securely
- **Portability**: Works consistently across platforms
- **Lightweight**: Minimal memory footprint

The TypeScript frontend provides a clean API through `src/lib/tauriCommands.ts`.

## Future Enhancements

Planned features:
- Real-time LSP integration with editor
- Advanced Git UI with visual diff
- More AI models (Claude, GPT-4, etc.)
- Code navigation (go to definition, find references)
- Inline diagnostics display
- Git blame and history viewer
- Custom AI prompts and templates

## Contributing

When adding new features:
1. Add Rust implementation in `src-tauri/src/`
2. Register commands in `src-tauri/src/main.rs`
3. Add TypeScript types in `src/lib/tauriCommands.ts`
4. Update command suggestions in `src-tauri/src/config.rs`
5. Document in this file
