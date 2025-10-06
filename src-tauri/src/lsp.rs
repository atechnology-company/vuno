// بسم الله الرحمن الرحيم

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub severity: String, // "error", "warning", "info", "hint"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: String, // "function", "variable", "class", "method", etc.
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
}

pub struct LspManager {
    // Map of language -> server process info
    servers: Arc<RwLock<HashMap<String, ServerInfo>>>,
}

struct ServerInfo {
    language: String,
    #[allow(dead_code)]
    command: String,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub fn start_server(&self, language: &str) -> Result<(), String> {
        let command = match language {
            "rust" => "rust-analyzer",
            "javascript" | "typescript" => "typescript-language-server",
            "python" => "pylsp", // python-lsp-server
            "go" => "gopls",
            "java" => "jdtls",
            "cpp" | "c" => "clangd",
            _ => return Err(format!("No LSP server configured for language: {}", language)),
        };
        
        let mut servers = self.servers.write();
        
        // Check if server already running
        if servers.contains_key(language) {
            return Ok(());
        }
        
        // Store server info
        servers.insert(
            language.to_string(),
            ServerInfo {
                language: language.to_string(),
                command: command.to_string(),
            }
        );
        
        Ok(())
    }
    
    pub fn get_running_servers(&self) -> Vec<String> {
        self.servers.read().keys().cloned().collect()
    }
}

#[tauri::command]
pub fn start_lsp_server(language: String, lsp_manager: tauri::State<'_, LspManager>) -> Result<String, String> {
    lsp_manager.start_server(&language)?;
    Ok(format!("LSP server started for {}", language))
}

#[tauri::command]
pub fn get_running_lsp_servers(lsp_manager: tauri::State<'_, LspManager>) -> Result<Vec<String>, String> {
    Ok(lsp_manager.get_running_servers())
}

#[tauri::command]
pub fn get_diagnostics(
    file_path: String,
    content: String,
    language: String,
) -> Result<Vec<Diagnostic>, String> {
    // Simplified diagnostics - check for basic syntax issues
    let mut diagnostics = Vec::new();
    
    match language.as_str() {
        "rust" => {
            // Basic Rust syntax checks
            for (line_num, line) in content.lines().enumerate() {
                if line.contains("fn ") && !line.contains("{") && !line.contains(";") {
                    diagnostics.push(Diagnostic {
                        line: line_num as u32,
                        column: 0,
                        message: "Function declaration may be incomplete".to_string(),
                        severity: "warning".to_string(),
                    });
                }
            }
        }
        "javascript" | "typescript" => {
            // Basic JS/TS syntax checks
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.contains("console.log") {
                    diagnostics.push(Diagnostic {
                        line: line_num as u32,
                        column: 0,
                        message: "Consider using a proper logging library".to_string(),
                        severity: "info".to_string(),
                    });
                }
            }
        }
        "python" => {
            // Basic Python syntax checks
            for (line_num, line) in content.lines().enumerate() {
                if line.trim().starts_with("print(") {
                    diagnostics.push(Diagnostic {
                        line: line_num as u32,
                        column: 0,
                        message: "Consider using logging instead of print".to_string(),
                        severity: "info".to_string(),
                    });
                }
            }
        }
        _ => {}
    }
    
    Ok(diagnostics)
}

#[tauri::command]
pub fn get_completions(
    file_path: String,
    content: String,
    position: Position,
    language: String,
) -> Result<Vec<CompletionItem>, String> {
    let mut completions = Vec::new();
    
    // Get the current line
    let lines: Vec<&str> = content.lines().collect();
    if position.line as usize >= lines.len() {
        return Ok(completions);
    }
    
    let current_line = lines[position.line as usize];
    let prefix = if position.character as usize <= current_line.len() {
        &current_line[..position.character as usize]
    } else {
        current_line
    };
    
    // Provide language-specific completions
    match language.as_str() {
        "rust" => {
            if prefix.contains("std::") {
                completions.extend(vec![
                    CompletionItem {
                        label: "std::collections".to_string(),
                        kind: "module".to_string(),
                        detail: Some("Collections module".to_string()),
                        documentation: Some("Standard library collections".to_string()),
                    },
                    CompletionItem {
                        label: "std::fs".to_string(),
                        kind: "module".to_string(),
                        detail: Some("File system module".to_string()),
                        documentation: Some("File system operations".to_string()),
                    },
                    CompletionItem {
                        label: "std::io".to_string(),
                        kind: "module".to_string(),
                        detail: Some("IO module".to_string()),
                        documentation: Some("Input/output operations".to_string()),
                    },
                ]);
            }
            
            if prefix.trim().is_empty() || prefix.ends_with(char::is_whitespace) {
                completions.extend(vec![
                    CompletionItem {
                        label: "fn".to_string(),
                        kind: "keyword".to_string(),
                        detail: Some("Function definition".to_string()),
                        documentation: None,
                    },
                    CompletionItem {
                        label: "let".to_string(),
                        kind: "keyword".to_string(),
                        detail: Some("Variable binding".to_string()),
                        documentation: None,
                    },
                    CompletionItem {
                        label: "pub".to_string(),
                        kind: "keyword".to_string(),
                        detail: Some("Public visibility".to_string()),
                        documentation: None,
                    },
                ]);
            }
        }
        "javascript" | "typescript" => {
            if prefix.contains("console.") {
                completions.extend(vec![
                    CompletionItem {
                        label: "console.log".to_string(),
                        kind: "method".to_string(),
                        detail: Some("Log to console".to_string()),
                        documentation: Some("Outputs a message to the console".to_string()),
                    },
                    CompletionItem {
                        label: "console.error".to_string(),
                        kind: "method".to_string(),
                        detail: Some("Log error to console".to_string()),
                        documentation: Some("Outputs an error message to the console".to_string()),
                    },
                    CompletionItem {
                        label: "console.warn".to_string(),
                        kind: "method".to_string(),
                        detail: Some("Log warning to console".to_string()),
                        documentation: Some("Outputs a warning message to the console".to_string()),
                    },
                ]);
            }
            
            completions.extend(vec![
                CompletionItem {
                    label: "function".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Function declaration".to_string()),
                    documentation: None,
                },
                CompletionItem {
                    label: "const".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Constant declaration".to_string()),
                    documentation: None,
                },
                CompletionItem {
                    label: "let".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Variable declaration".to_string()),
                    documentation: None,
                },
            ]);
        }
        "python" => {
            if prefix.contains("import ") {
                completions.extend(vec![
                    CompletionItem {
                        label: "import os".to_string(),
                        kind: "module".to_string(),
                        detail: Some("Operating system interface".to_string()),
                        documentation: None,
                    },
                    CompletionItem {
                        label: "import sys".to_string(),
                        kind: "module".to_string(),
                        detail: Some("System-specific parameters".to_string()),
                        documentation: None,
                    },
                    CompletionItem {
                        label: "import json".to_string(),
                        kind: "module".to_string(),
                        detail: Some("JSON encoder and decoder".to_string()),
                        documentation: None,
                    },
                ]);
            }
            
            completions.extend(vec![
                CompletionItem {
                    label: "def".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Function definition".to_string()),
                    documentation: None,
                },
                CompletionItem {
                    label: "class".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Class definition".to_string()),
                    documentation: None,
                },
                CompletionItem {
                    label: "import".to_string(),
                    kind: "keyword".to_string(),
                    detail: Some("Import module".to_string()),
                    documentation: None,
                },
            ]);
        }
        _ => {}
    }
    
    Ok(completions)
}

#[tauri::command]
pub fn format_document(content: String, language: String) -> Result<String, String> {
    match language.as_str() {
        "rust" => {
            // Try to format with rustfmt
            let output = Command::new("rustfmt")
                .arg("--emit")
                .arg("stdout")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to run rustfmt: {}. Is it installed?", e))?
                .wait_with_output()
                .map_err(|e| format!("Failed to wait for rustfmt: {}", e))?;
            
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err("Failed to format Rust code".to_string())
            }
        }
        "javascript" | "typescript" => {
            // Basic formatting for JS/TS (simplified)
            Ok(content) // In a real implementation, use prettier or similar
        }
        "python" => {
            // Try to format with black
            let output = Command::new("black")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .map_err(|e| format!("Failed to run black: {}. Is it installed?", e))?
                .wait_with_output()
                .map_err(|e| format!("Failed to wait for black: {}", e))?;
            
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).to_string())
            } else {
                Err("Failed to format Python code".to_string())
            }
        }
        _ => Ok(content),
    }
}

#[tauri::command]
pub fn check_lsp_available(language: String) -> Result<bool, String> {
    let command = match language.as_str() {
        "rust" => "rust-analyzer",
        "javascript" | "typescript" => "typescript-language-server",
        "python" => "pylsp",
        "go" => "gopls",
        "java" => "jdtls",
        "cpp" | "c" => "clangd",
        _ => return Ok(false),
    };
    
    // Check if the command exists
    let result = if cfg!(target_os = "windows") {
        Command::new("where")
            .arg(command)
            .output()
    } else {
        Command::new("which")
            .arg(command)
            .output()
    };
    
    match result {
        Ok(output) => Ok(output.status.success()),
        Err(_) => Ok(false),
    }
}
