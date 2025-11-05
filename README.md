# vuno revision two

*v - vimified<br>
u - nu<br>
no - nano*<br>
it's like neovim, but easy

vuno is a terminal-style text editor, with people in mind.<br>

vuno is made with [sveltekit](https://svelte.dev/) and [rust](https://rust-lang.org/) through [tauri](https://tauri.app/) with <br>
codemirror, tokio, shiki, tree sitter, gemini api and tower lsp <br>

vuno is experimental right now and is missing many of the features that will be in the final release! stick around for that :))

=======
## ✨ New Features

vuno now includes powerful agentic AI features and developer tools:

### 🤖 AI-Powered Editing
- **GitHub Copilot** - AI pair programmer with inline code completions (NEW!)
- **Document Analysis** - AI analyzes your code structure and suggests improvements
- **Code Generation** - Generate tests, documentation, and boilerplate
- **Refactoring** - AI-guided code refactoring with custom instructions
- **Code Explanation** - Understand complex code with AI explanations
- **Web Search** - Search the web with Perplexity AI directly from the editor

### 🔧 Developer Tools
- **Git Integration** - Full Git workflow (status, commit, push, pull, branches)
- **IntelliSense** - LSP-powered code completions for multiple languages
- **Diagnostics** - Real-time error and warning detection
- **Code Formatting** - Format code with language-specific formatters

### 🚀 Supported Languages
- Rust, JavaScript, TypeScript, Python, Go, Java, C/C++
- Syntax highlighting and language-specific features

See [FEATURES.md](FEATURES.md) for detailed documentation.

>>>>>>> Stashed changes
fonts used are:
- onest
- jetbrains mono

you can sign up for a gemini api key [here](https://aistudio.google.com/) (limited free part though)

<<<<<<< Updated upstream
=======
## 🛠️ Setup

### API Keys & Authentication

For AI features, configure your API keys:

```
set api_key YOUR_GEMINI_API_KEY
set perplexity_key YOUR_PERPLEXITY_API_KEY
```

- Get Gemini API key: https://aistudio.google.com/
- Get Perplexity API key: https://www.perplexity.ai/
- **GitHub Copilot**: Sign up for [GitHub Copilot Free](https://github.com/settings/copilot) - authenticate via the editor

### Language Servers (Optional)

For IntelliSense and diagnostics, install language servers:

- **Rust**: `rustup component add rust-analyzer`
- **JavaScript/TypeScript**: `npm install -g typescript-language-server`
- **Python**: `pip install python-lsp-server`
- **Go**: `go install golang.org/x/tools/gopls@latest`
- **C/C++**: Install clangd from your package manager

### GitHub Copilot Setup

To use GitHub Copilot in Vuno:

1. **Install the language server**:
   ```bash
   pnpm add @github/copilot-language-server
   ```

2. **Sign in**: Use the Copilot panel in the editor or run the sign-in command

3. **Start coding**: Copilot will automatically suggest completions as you type

Note: GitHub Copilot requires a GitHub account with Copilot access. [Sign up for free](https://github.com/settings/copilot)!

>>>>>>> Stashed changes
# usage guide
new file
"new (file type | markdown)

open a file
"open (path to file)"

save file
"save (as whatever)"

quit
"quit" | "close"

run a command - simply type the command it will automatically recognise it