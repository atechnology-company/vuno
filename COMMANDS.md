# Vuno Command Reference

Quick reference for all available commands in Vuno.

## Basic Commands

| Command | Description |
|---------|-------------|
| `new markdown` | Create new markdown file |
| `new javascript` | Create new JavaScript file |
| `new python` | Create new Python file |
| `open [path]` | Open a file |
| `save` | Save current file |
| `save as [name]` | Save with new name |
| `quit` | Quit the editor |
| `close` | Close current file |

## AI Commands

| Command | Description | Requires API Key |
|---------|-------------|------------------|
| `ai [prompt]` | Send prompt to AI | ✓ |
| `explain [text]` | Get AI explanation | ✓ |
| `analyze document` | Analyze current document | ✓ |
| `suggest improvements` | Get improvement suggestions | ✓ |
| `generate tests` | Generate unit tests | ✓ |
| `refactor code: [instruction]` | Refactor with instruction | ✓ |
| `explain selection` | Explain selected code | ✓ |
| `search web: [query]` | Search web with Perplexity | ✓ |

## Git Commands

| Command | Description |
|---------|-------------|
| `git status` | Show repository status |
| `git add [files]` | Stage files (use `.` for all) |
| `git commit -m "[message]"` | Commit with message |
| `git push` | Push to remote |
| `git pull` | Pull from remote |
| `git branch` | List branches |
| `git checkout [branch]` | Switch to branch |
| `git diff [file]` | Show changes |
| `git log` | Show commit history |
| `git init` | Initialize repository |
| `git clone [url]` | Clone repository |

## LSP Commands

| Command | Description |
|---------|-------------|
| `start lsp server: [language]` | Start language server |
| `format document` | Format current file |
| `show diagnostics` | Display errors/warnings |
| `get completions` | Get code completions |

## File Operations

| Command | Description |
|---------|-------------|
| `ls [path]` | List directory contents |
| `pwd` | Print working directory |
| `mkdir [name]` | Create directory |
| `clear` | Clear editor |

## System Commands

| Command | Description |
|---------|-------------|
| `date` | Show current date/time |
| `whoami` | Show current user |
| `help` | Show help message |
| `version` | Show version info |

## Shell Commands

Prefix any shell command with `!`:

```
!npm install
!cargo build
!python script.py
!git status
```

## Configuration Commands

| Command | Description |
|---------|-------------|
| `set api_key [key]` | Set Gemini API key |
| `set perplexity_key [key]` | Set Perplexity API key |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl/Cmd + P` | Open command palette |
| `Ctrl/Cmd + S` | Save file |
| `Ctrl/Cmd + O` | Open file |
| `Ctrl/Cmd + N` | New file |
| `Ctrl/Cmd + W` | Close file |
| `Ctrl/Cmd + Q` | Quit |
| `Esc` | Close command palette |

## Command Palette

Press `Ctrl/Cmd + P` to open the command palette. Start typing to see suggestions based on:
- Current file type
- Available commands
- Recent commands
- Contextual suggestions

## Tips

1. **Natural Language**: Many commands accept natural language variations
2. **Auto-completion**: Press Tab to complete commands
3. **History**: Use arrow keys to navigate command history
4. **Context**: Commands adapt based on current file type
5. **API Keys**: Store securely in app config directory

## Examples

### AI Workflow
```
# Analyze code
analyze document

# Get suggestions
suggest improvements

# Generate tests
generate tests

# Refactor specific part
refactor code: extract this into a separate function
```

### Git Workflow
```
# Check status
git status

# Stage and commit
git add .
git commit -m "feat: add new feature"

# Push changes
git push

# Switch branch
git checkout feature-branch
```

### LSP Workflow
```
# Start language server
start lsp server: rust

# Format code
format document

# Check for errors
show diagnostics
```

### Search Workflow
```
# Search for programming concepts
search web: What is Rust ownership model?

# Search for solutions
search web: How to implement async/await in Python?
```

## Command Aliases

| Alias | Full Command |
|-------|--------------|
| `l` | `ls` |
| `dir` | `ls` |
| `md` | `mkdir` |
| `cls` | `clear` |
| `wc` | `word-count` |
| `?` | `help` |
| `h` | `help` |

## Error Messages

Common errors and solutions:

| Error | Solution |
|-------|----------|
| "No API key provided" | Set API key with `set api_key YOUR_KEY` |
| "Git not found" | Install git and add to PATH |
| "LSP server not available" | Install language server for your language |
| "Command not found" | Check spelling or type `help` for available commands |

## Advanced Usage

### Chaining Commands

Execute multiple commands by separating with `&&`:
```
!npm install && npm run build && npm test
```

### Working Directory

Commands respect the current working directory. Use `pwd` to check and navigate as needed.

### Command History

Recent commands are saved and can be accessed with arrow keys in the command palette.

## Support

For detailed documentation:
- [FEATURES.md](FEATURES.md) - Complete feature documentation
- [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md) - Developer guide
- [README.md](README.md) - General information

Report issues: https://github.com/atechnology-company/vuno/issues
