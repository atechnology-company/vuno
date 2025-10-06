# vuno revision two

*v - vimified<br>
u - nu<br>
no - nano*<br>
it's like neovim, but easy

vuno is a terminal-style text editor, with people in mind.<br>

vuno is made with [sveltekit](https://svelte.dev/) and [rust](https://rust-lang.org/) through [tauri](https://tauri.app/) with <br>
codemirror, tokio, shiki, tree sitter, gemini api and tower lsp <br>

vuno is experimental right now and is missing many of the features that will be in the final release! stick around for that :))

## ✨ New Features

vuno now includes powerful agentic AI features and developer tools:

### 🤖 AI-Powered Editing
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

fonts used are:
- onest
- jetbrains mono

you can sign up for a gemini api key [here](https://aistudio.google.com/) (limited free part though)

## 🛠️ Setup

### API Keys

For AI features, configure your API keys:

```
set api_key YOUR_GEMINI_API_KEY
set perplexity_key YOUR_PERPLEXITY_API_KEY
```

- Get Gemini API key: https://aistudio.google.com/
- Get Perplexity API key: https://www.perplexity.ai/

### Language Servers (Optional)

For IntelliSense and diagnostics, install language servers:

- **Rust**: `rustup component add rust-analyzer`
- **JavaScript/TypeScript**: `npm install -g typescript-language-server`
- **Python**: `pip install python-lsp-server`
- **Go**: `go install golang.org/x/tools/gopls@latest`
- **C/C++**: Install clangd from your package manager

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

## AI commands
analyze document - analyze code structure and purpose<br>
suggest improvements - get AI suggestions for improvements<br>
generate tests - automatically generate unit tests<br>
refactor code: [instruction] - refactor with AI guidance<br>
explain selection - explain selected code<br>
search web: [query] - search the web with Perplexity<br>

## Git commands
git status - show current git status<br>
git add . - stage all changes<br>
git commit -m "message" - commit with message<br>
git push - push to remote<br>
git pull - pull from remote<br>
git branch - list branches<br>
git checkout [branch] - switch branches<br>

## LSP commands
format document - format code with language formatter<br>
show diagnostics - display code errors and warnings<br>

For more details, see [FEATURES.md](FEATURES.md) and [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)