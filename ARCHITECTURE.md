# Vuno Architecture

Visual overview of the enhanced Vuno architecture with new features.

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         USER INTERFACE                           │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Command      │  │ Git Panel    │  │ AI Assistant Panel   │  │
│  │ Palette      │  │ (Svelte)     │  │ (Svelte)             │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Code Editor  │  │ Diagnostics  │  │ Output Panel         │  │
│  │ (CodeMirror) │  │ Panel        │  │                      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FRONTEND API LAYER                            │
│                    (TypeScript)                                  │
│                                                                   │
│  src/lib/tauriCommands.ts                                        │
│  • Type-safe command wrappers                                    │
│  • Interface definitions                                         │
│  • Helper functions                                              │
└─────────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                       TAURI IPC LAYER                            │
│                                                                   │
│  • Secure inter-process communication                            │
│  • Serialization/Deserialization                                 │
│  • Command routing                                               │
└─────────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    BACKEND COMMAND LAYER                         │
│                    (Rust - src-tauri/src/)                       │
│                                                                   │
│  main.rs - Command registration and app setup                   │
│  • Registers 25 new commands                                     │
│  • Manages app state                                             │
│  • Handles initialization                                        │
└─────────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    FEATURE MODULES (Rust)                        │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  git.rs - Git Integration                              │    │
│  │  • git_status, git_add, git_commit                     │    │
│  │  • git_push, git_pull, git_branch_list                 │    │
│  │  • git_checkout, git_diff, git_log                     │    │
│  │  • git_init, git_clone                                 │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  lsp.rs - Language Server Protocol                     │    │
│  │  • start_lsp_server, get_running_lsp_servers           │    │
│  │  • get_completions, get_diagnostics                    │    │
│  │  • format_document, check_lsp_available                │    │
│  │  • Supports: Rust, JS/TS, Python, Go, Java, C/C++     │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  perplexity.rs - Web Search                            │    │
│  │  • search_web                                          │    │
│  │  • get_perplexity_key, set_perplexity_key             │    │
│  │  • Citation handling                                   │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  command_processor.rs - AI Commands                    │    │
│  │  • analyze_document, suggest_improvements              │    │
│  │  • generate_tests, refactor_code                       │    │
│  │  • explain_selection                                   │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                   │
│  ┌────────────────────────────────────────────────────────┐    │
│  │  api.rs - Gemini AI Integration                        │    │
│  │  • send_chat_message                                   │    │
│  │  • API key management                                  │    │
│  └────────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    EXTERNAL SERVICES                             │
│                                                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐  │
│  │ Git Binary   │  │ LSP Servers  │  │ Gemini API           │  │
│  │ • git        │  │ • rust-      │  │ • AI responses       │  │
│  │   commands   │  │   analyzer   │  │ • Code analysis      │  │
│  │              │  │ • typescript-│  │                      │  │
│  │              │  │   server     │  │                      │  │
│  │              │  │ • pylsp      │  │                      │  │
│  │              │  │ • gopls      │  │                      │  │
│  └──────────────┘  └──────────────┘  └──────────────────────┘  │
│                                                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ Perplexity API                                           │  │
│  │ • Web search                                             │  │
│  │ • Citations                                              │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Data Flow Examples

### 1. Git Status Request

```
User clicks "Git Status" button in GitPanel
    ↓
GitPanel.svelte calls: gitStatus(workingDir)
    ↓
tauriCommands.ts: invoke('git_status', { workingDir })
    ↓
Tauri IPC serializes and sends command
    ↓
main.rs routes to git::git_status()
    ↓
git.rs executes: git status --porcelain
    ↓
Parses output into GitStatus struct
    ↓
Returns Result<GitStatus, String>
    ↓
Tauri IPC serializes response
    ↓
tauriCommands.ts deserializes to TypeScript
    ↓
GitPanel.svelte updates UI with status
```

### 2. AI Document Analysis

```
User types "analyze document" in command palette
    ↓
Command handler calls: analyzeDocument(content, language, apiKey)
    ↓
tauriCommands.ts: invoke('analyze_document', { content, language, apiKey })
    ↓
Tauri IPC sends to backend
    ↓
main.rs routes to command_processor::analyze_document()
    ↓
command_processor.rs formats prompt with context
    ↓
Calls api::send_chat_message() with prompt
    ↓
api.rs makes HTTPS request to Gemini API
    ↓
Receives and parses AI response
    ↓
Returns CommandResult with AI output
    ↓
Frontend displays in output panel
```

### 3. LSP Code Completion

```
User types code in editor
    ↓
CodeMirror triggers completion request
    ↓
getCompletions(filePath, content, position, language)
    ↓
tauriCommands.ts: invoke('get_completions', { ... })
    ↓
Tauri IPC to backend
    ↓
main.rs routes to lsp::get_completions()
    ↓
lsp.rs analyzes code context
    ↓
Returns language-specific completions
    ↓
Frontend displays completion dropdown
```

## Component Interaction

```
┌─────────────────────────────────────────┐
│  +page.svelte (Main App)                │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │  Editor State                      │ │
│  │  • content                         │ │
│  │  • language                        │ │
│  │  • filePath                        │ │
│  │  • selectedText                    │ │
│  └────────────────────────────────────┘ │
│                                          │
│  Components (conditionally rendered):   │
│                                          │
│  ┌────────────────┐                     │
│  │ GitPanel       │─────┐               │
│  └────────────────┘     │               │
│                         ├───> All use   │
│  ┌────────────────┐     │    tauriCommands │
│  │ DiagnosticsPanel│────┤    API        │
│  └────────────────┘     │               │
│                         │               │
│  ┌────────────────┐     │               │
│  │ AIAssistantPanel│────┘               │
│  └────────────────┘                     │
└─────────────────────────────────────────┘
```

## State Management

```
┌──────────────────────────────────────┐
│  Frontend State                      │
│                                       │
│  • Svelte stores (appState.ts)      │
│  • Component-local state             │
│  • React to user input               │
└──────────────────────────────────────┘
         ▲                    │
         │                    ▼
┌────────────────────────────────────────┐
│  Backend State                          │
│                                         │
│  • API keys (RwLock<String>)           │
│  • LSP servers (HashMap)               │
│  • Buffer manager                      │
│  • Configuration                       │
└────────────────────────────────────────┘
```

## File Organization

```
vuno/
├── src/                          # Frontend
│   ├── lib/
│   │   └── tauriCommands.ts     # Type-safe API
│   ├── modules/
│   │   ├── GitPanel.svelte       # Git UI
│   │   ├── DiagnosticsPanel.svelte # LSP UI
│   │   └── AIAssistantPanel.svelte # AI UI
│   └── routes/
│       └── +page.svelte          # Main app
│
├── src-tauri/src/               # Backend
│   ├── main.rs                   # Entry point
│   ├── git.rs                    # Git integration
│   ├── lsp.rs                    # LSP support
│   ├── perplexity.rs             # Web search
│   ├── command_processor.rs      # AI commands
│   ├── api.rs                    # Gemini API
│   ├── config.rs                 # Configuration
│   └── buffer.rs                 # Buffer management
│
├── FEATURES.md                   # User documentation
├── INTEGRATION_GUIDE.md          # Developer guide
├── COMMANDS.md                   # Command reference
├── IMPLEMENTATION_SUMMARY.md     # Technical summary
└── README.md                     # Overview
```

## Key Design Decisions

### 1. Backend-Heavy Architecture
**Decision:** Implement all business logic in Rust
**Rationale:**
- Performance: Native Rust code is fast
- Security: API keys handled securely
- Portability: Single codebase for all platforms
- Reliability: Strong typing catches errors at compile time

### 2. Type-Safe IPC
**Decision:** Create TypeScript wrapper for all Tauri commands
**Rationale:**
- Type safety: Catch errors at compile time
- Developer experience: IntelliSense for all commands
- Maintainability: Single source of truth for types
- Documentation: Types serve as documentation

### 3. Modular Design
**Decision:** Separate features into distinct modules
**Rationale:**
- Maintainability: Easy to update individual features
- Testability: Each module can be tested independently
- Scalability: New features can be added easily
- Organization: Clear separation of concerns

### 4. Component-Based UI
**Decision:** Create reusable Svelte components
**Rationale:**
- Reusability: Components can be used anywhere
- Flexibility: Can be shown/hidden dynamically
- Maintainability: Self-contained logic
- User experience: Consistent UI patterns

## Performance Considerations

### 1. Debouncing
- Diagnostics updates debounced (1 second)
- Prevents excessive LSP calls
- Reduces backend load

### 2. Caching
- Git status cached for 10 seconds
- LSP completions reused when possible
- API responses cached where appropriate

### 3. Lazy Loading
- LSP servers started only when needed
- Components rendered conditionally
- Minimize initial load time

### 4. Efficient IPC
- Binary serialization for large data
- Minimal data transfer
- Async operations don't block UI

## Security Features

### 1. API Key Storage
- Keys stored in app-specific directory
- File permissions restrict access
- Keys never logged or displayed

### 2. Input Validation
- All user input validated in Rust
- Shell command escaping
- Path traversal prevention

### 3. Sandboxing
- Git operations in controlled environment
- LSP servers isolated
- Network requests limited

## Error Handling

```
Frontend
    ↓ User action
Validation
    ↓ If invalid: Show error toast
Backend Call
    ↓ Try operation
Success → Return result → Display to user
    ↓
Error → Format error message → Display error toast
```

## Extensibility

New features can be added by:

1. **Create Rust module** (`src-tauri/src/feature.rs`)
2. **Add commands** with `#[tauri::command]`
3. **Register in main.rs** via `invoke_handler!`
4. **Add TypeScript types** in `tauriCommands.ts`
5. **Create UI component** (optional)
6. **Document** in FEATURES.md

## Testing Strategy

### Unit Tests
- Test Rust functions in isolation
- Test TypeScript utility functions
- Mock external services

### Integration Tests
- Test full command flow
- Test component interactions
- Test error handling

### Manual Tests
- User acceptance testing
- Performance testing
- Security testing

## Deployment

```
Development
    ↓
npm run tauri:dev
    ↓
Test features
    ↓
Build: npm run tauri:build
    ↓
Package for platform
    ↓
Distribute binary
```

## Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| UI | Svelte | Reactive components |
| Editor | CodeMirror | Code editing |
| Frontend API | TypeScript | Type-safe wrappers |
| IPC | Tauri | Secure communication |
| Backend | Rust | Business logic |
| Git | Native git | Version control |
| LSP | Various servers | Language support |
| AI | Gemini API | AI features |
| Search | Perplexity API | Web search |

## Conclusion

The architecture is designed for:
- ⚡ **Performance** - Fast native Rust backend
- 🔒 **Security** - Secure handling of sensitive data
- 🪶 **Lightweight** - Minimal resource usage
- 🔧 **Maintainability** - Clean separation of concerns
- 📈 **Scalability** - Easy to add new features
- 🎨 **Usability** - Intuitive component-based UI
