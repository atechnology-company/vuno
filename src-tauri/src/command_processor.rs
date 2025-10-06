// بسم الله الرحمن الرحيم

use std::collections::HashMap;
use std::process::Command;
use serde::{Deserialize, Serialize};
use crate::api::send_chat_message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
    pub command_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub category: String,
    pub priority: i32,
}

pub struct CommandProcessor {
    commands: HashMap<String, Box<dyn Fn(&[String]) -> CommandResult + Send + Sync>>,
    aliases: HashMap<String, String>,
}

impl CommandProcessor {
    pub fn new() -> Self {
        let mut processor = Self {
            commands: HashMap::new(),
            aliases: HashMap::new(),
        };
        processor.register_default_commands();
        processor
    }

    fn register_default_commands(&mut self) {
        // File operations
        self.register_command("ls", Box::new(|args| {
            let path = args.get(0).map(|s| s.as_str()).unwrap_or(".");
            match std::fs::read_dir(path) {
                Ok(entries) => {
                    let mut output = String::new();
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let name = entry.file_name().to_string_lossy().to_string();
                            output.push_str(&format!("{}\n", name));
                        }
                    }
                    CommandResult {
                        success: true,
                        output,
                        error: None,
                        command_type: "file".to_string(),
                    }
                }
                Err(e) => CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    command_type: "file".to_string(),
                }
            }
        }));

        self.register_command("pwd", Box::new(|_args| {
            match std::env::current_dir() {
                Ok(path) => CommandResult {
                    success: true,
                    output: path.to_string_lossy().to_string(),
                    error: None,
                    command_type: "file".to_string(),
                },
                Err(e) => CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    command_type: "file".to_string(),
                }
            }
        }));

        self.register_command("mkdir", Box::new(|args| {
            if args.is_empty() {
                return CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some("Usage: mkdir <directory>".to_string()),
                    command_type: "file".to_string(),
                };
            }
            
            match std::fs::create_dir_all(&args[0]) {
                Ok(_) => CommandResult {
                    success: true,
                    output: format!("Directory '{}' created", args[0]),
                    error: None,
                    command_type: "file".to_string(),
                },
                Err(e) => CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some(e.to_string()),
                    command_type: "file".to_string(),
                }
            }
        }));

        // Text editing commands
        self.register_command("clear", Box::new(|_args| {
            CommandResult {
                success: true,
                output: "Screen cleared".to_string(),
                error: None,
                command_type: "editor".to_string(),
            }
        }));

        self.register_command("word-count", Box::new(|args| {
            let text = args.join(" ");
            let words = text.split_whitespace().count();
            let chars = text.chars().count();
            let lines = text.lines().count();
            
            CommandResult {
                success: true,
                output: format!("Words: {}, Characters: {}, Lines: {}", words, chars, lines),
                error: None,
                command_type: "editor".to_string(),
            }
        }));

        // System commands
        self.register_command("date", Box::new(|_args| {
            CommandResult {
                success: true,
                output: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                error: None,
                command_type: "system".to_string(),
            }
        }));

        self.register_command("whoami", Box::new(|_args| {
            match std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
                Ok(user) => CommandResult {
                    success: true,
                    output: user,
                    error: None,
                    command_type: "system".to_string(),
                },
                Err(_) => CommandResult {
                    success: false,
                    output: String::new(),
                    error: Some("Could not determine username".to_string()),
                    command_type: "system".to_string(),
                }
            }
        }));

        // Help command
        self.register_command("help", Box::new(|_args| {
            let help_text = r#"Available Commands:

File Operations:
  ls [path]         - List directory contents
  pwd              - Print working directory
  mkdir <dir>      - Create directory

Text Editing:
  clear            - Clear the editor
  word-count <text> - Count words, characters, and lines

System:
  date             - Show current date and time
  whoami           - Show current user

AI:
  ai <prompt>      - Send prompt to AI assistant
  explain <text>   - Get AI explanation of text
  
Special:
  help             - Show this help message
  version          - Show version information

You can also run shell commands by prefixing with '!'
Example: !git status
"#;
            CommandResult {
                success: true,
                output: help_text.to_string(),
                error: None,
                command_type: "help".to_string(),
            }
        }));

        self.register_command("version", Box::new(|_args| {
            CommandResult {
                success: true,
                output: "Vuno v0.1.0 - The simple, unitary text editor".to_string(),
                error: None,
                command_type: "info".to_string(),
            }
        }));

        // Register aliases
        self.aliases.insert("l".to_string(), "ls".to_string());
        self.aliases.insert("dir".to_string(), "ls".to_string());
        self.aliases.insert("md".to_string(), "mkdir".to_string());
        self.aliases.insert("cls".to_string(), "clear".to_string());
        self.aliases.insert("wc".to_string(), "word-count".to_string());
        self.aliases.insert("?".to_string(), "help".to_string());
        self.aliases.insert("h".to_string(), "help".to_string());
    }

    fn register_command(&mut self, name: &str, handler: Box<dyn Fn(&[String]) -> CommandResult + Send + Sync>) {
        self.commands.insert(name.to_string(), handler);
    }

    pub fn execute(&self, input: &str, working_dir: Option<&str>) -> CommandResult {
        let input = input.trim();
        if input.is_empty() {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some("Empty command".to_string()),
                command_type: "error".to_string(),
            };
        }

        // Handle shell commands (prefixed with !)
        if input.starts_with('!') {
            return self.execute_shell_command(&input[1..], working_dir);
        }

        // Parse command and arguments
        let parts: Vec<String> = input.split_whitespace().map(|s| s.to_string()).collect();
        if parts.is_empty() {
            return CommandResult {
                success: false,
                output: String::new(),
                error: Some("Invalid command".to_string()),
                command_type: "error".to_string(),
            };
        }

        let command = &parts[0];
        let args = &parts[1..];

        // Check for alias
        let actual_command = self.aliases.get(command).unwrap_or(command);

        // Execute command
        if let Some(handler) = self.commands.get(actual_command) {
            handler(args)
        } else {
            CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Unknown command: {}. Type 'help' for available commands.", command)),
                command_type: "error".to_string(),
            }
        }
    }

    fn execute_shell_command(&self, command: &str, working_dir: Option<&str>) -> CommandResult {
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(&["/C", command]);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c");
            c.arg(command);
            c
        };
        
        // Set working directory if provided
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        let output = cmd.output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                
                if output.status.success() {
                    CommandResult {
                        success: true,
                        output: stdout,
                        error: if stderr.is_empty() { None } else { Some(stderr) },
                        command_type: "shell".to_string(),
                    }
                } else {
                    CommandResult {
                        success: false,
                        output: stdout,
                        error: Some(stderr),
                        command_type: "shell".to_string(),
                    }
                }
            }
            Err(e) => CommandResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to execute command: {}", e)),
                command_type: "shell".to_string(),
            }
        }
    }

    pub fn get_suggestions(&self, input: &str) -> Vec<CommandSuggestion> {
        let mut suggestions = Vec::new();
        let input_lower = input.to_lowercase();

        // Add command suggestions
        for (command, _) in &self.commands {
            if command.starts_with(&input_lower) {
                let description = match command.as_str() {
                    "ls" => "List directory contents",
                    "pwd" => "Print working directory", 
                    "mkdir" => "Create directory",
                    "clear" => "Clear the editor",
                    "word-count" => "Count words, characters, and lines",
                    "date" => "Show current date and time",
                    "whoami" => "Show current user",
                    "help" => "Show help message",
                    "version" => "Show version information",
                    _ => "Custom command",
                };

                let category = match command.as_str() {
                    "ls" | "pwd" | "mkdir" => "File",
                    "clear" | "word-count" => "Editor",
                    "date" | "whoami" => "System",
                    "help" | "version" => "Info",
                    _ => "Other",
                };

                let priority = if command == &input_lower { 100 } else { 80 };

                suggestions.push(CommandSuggestion {
                    command: command.clone(),
                    description: description.to_string(),
                    category: category.to_string(),
                    priority,
                });
            }
        }

        // Add alias suggestions
        for (alias, _) in &self.aliases {
            if alias.starts_with(&input_lower) {
                suggestions.push(CommandSuggestion {
                    command: alias.clone(),
                    description: format!("Alias for command"),
                    category: "Alias".to_string(),
                    priority: 60,
                });
            }
        }

        // Add AI suggestions for natural language
        if input.len() > 3 && !input.starts_with('!') && !self.commands.contains_key(&input_lower) {
            suggestions.push(CommandSuggestion {
                command: format!("ai {}", input),
                description: "Ask AI assistant".to_string(),
                category: "AI".to_string(),
                priority: 50,
            });

            suggestions.push(CommandSuggestion {
                command: format!("explain {}", input),
                description: "Get AI explanation".to_string(),
                category: "AI".to_string(),
                priority: 45,
            });
        }

        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));
        suggestions
    }
}

// Tauri commands
#[tauri::command]
pub async fn execute_enhanced_command(
    command: String,
    api_key: Option<String>,
    working_dir: Option<String>,
) -> Result<CommandResult, String> {
    let processor = CommandProcessor::new();
    
    // Handle AI commands
    if command.starts_with("ai ") || command.starts_with("explain ") {
        let prompt = if command.starts_with("ai ") {
            &command[3..]
        } else {
            &command[8..]
        };
        
        if let Some(key) = api_key {
            if !key.is_empty() {
                // Create a simple message for AI
                let messages = vec![
                    serde_json::json!({
                        "role": "user",
                        "content": prompt
                    })
                ];
                
                match send_chat_message(messages, key).await {
                    Ok(response) => {
                        return Ok(CommandResult {
                            success: true,
                            output: response,
                            error: None,
                            command_type: "ai".to_string(),
                        });
                    }
                    Err(e) => {
                        return Ok(CommandResult {
                            success: false,
                            output: String::new(),
                            error: Some(format!("AI Error: {}", e)),
                            command_type: "ai".to_string(),
                        });
                    }
                }
            }
        }
        
        return Ok(CommandResult {
            success: false,
            output: String::new(),
            error: Some("No API key provided for AI commands".to_string()),
            command_type: "ai".to_string(),
        });
    }
    
    // Execute regular command with working directory
    Ok(processor.execute(&command, working_dir.as_deref()))
}

#[tauri::command]
pub fn get_enhanced_command_suggestions(input: String) -> Result<Vec<CommandSuggestion>, String> {
    let processor = CommandProcessor::new();
    Ok(processor.get_suggestions(&input))
}

#[tauri::command]
pub fn validate_command(command: String) -> Result<bool, String> {
    let processor = CommandProcessor::new();
    let result = processor.execute(&command, None);
    Ok(result.success)
}
