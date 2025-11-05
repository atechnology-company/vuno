use std::fs;
use std::path::PathBuf;
use tauri;
use chrono;
use rand::seq::SliceRandom;

pub struct FirstRunStore {
    has_run_path: PathBuf,
}

impl FirstRunStore {
    pub fn new(app_handle: &tauri::AppHandle) -> Self {
        let app_dir = if cfg!(feature = "portable") {
            app_handle
                .path_resolver()
                .resolve_resource(".")
                .expect("Failed to get executable directory")
        } else {
            app_handle
                .path_resolver()
                .app_config_dir()
                .expect("Failed to get app config directory")
        };
        
        fs::create_dir_all(&app_dir).expect("Failed to create app config directory");
        
        let has_run_path = app_dir.join("has_run_before.txt");
        
        Self {
            has_run_path,
        }
    }
    
    pub fn check_has_run_before(&self) -> bool {
        self.has_run_path.exists()
    }
    
    pub fn set_has_run_before(&self) -> Result<(), String> {
        let timestamp = chrono::Local::now().to_string();
        fs::write(&self.has_run_path, timestamp)
            .map_err(|e| format!("Failed to write first run marker: {}", e))
    }
}

#[tauri::command]
pub fn get_command_suggestions(mode: String, editor_language: String, _context: String, current_file: String) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Add file operations commands
    suggestions.push("new markdown".to_string());
    suggestions.push("new javascript".to_string());
    suggestions.push("new python".to_string());
    suggestions.push("new html".to_string());
    suggestions.push("new rust".to_string());
    suggestions.push("new c".to_string());
    suggestions.push("new cpp".to_string());
    suggestions.push("new typescript".to_string());
    suggestions.push("open file".to_string());
    suggestions.push("save".to_string());
    suggestions.push("save as".to_string());
    
    // Add window management commands
    suggestions.push("minimize".to_string());
    suggestions.push("fullscreen".to_string());
    suggestions.push("help".to_string());
    
    // Add terminal commands
    suggestions.push("git status".to_string());
    suggestions.push("ls".to_string());
    suggestions.push("pwd".to_string());
    suggestions.push("npm run dev".to_string());
    suggestions.push("npm start".to_string());
    
    // Add code editing commands
    if mode == "code" {
        match editor_language.as_str() {
            "javascript" | "typescript" => {
                suggestions.push("npm install".to_string());
                suggestions.push("format code".to_string());
                suggestions.push("lint code".to_string());
                suggestions.push("optimize imports".to_string());
                suggestions.push("node index.js".to_string());
            },
            "python" => {
                suggestions.push("pip install".to_string());
                suggestions.push("python main.py".to_string());
                suggestions.push("format with black".to_string());
                suggestions.push("run tests".to_string());
            },
            "rust" => {
                suggestions.push("cargo run".to_string());
                suggestions.push("cargo build".to_string());
                suggestions.push("cargo test".to_string());
                suggestions.push("rustfmt".to_string());
            },
            "c" | "cpp" => {
                suggestions.push("gcc main.c".to_string());
                suggestions.push("g++ main.cpp".to_string());
                suggestions.push("make build".to_string());
                suggestions.push("clang-format".to_string());
            },
            _ => {
                suggestions.push(format!("format {}", editor_language));
                suggestions.push("indent code".to_string());
                suggestions.push("find references".to_string());
            }
        }
    } else if mode == "markdown" {
        suggestions.push("bold text".to_string());
        suggestions.push("italic text".to_string());
        suggestions.push("insert heading".to_string());
        suggestions.push("insert code block".to_string());
        suggestions.push("insert table".to_string());
        suggestions.push("insert link".to_string());
        suggestions.push("insert image".to_string());
        suggestions.push("export to pdf".to_string());
        suggestions.push("check spelling".to_string());
    }
    
    // Add GitHub Copilot commands (high priority)
    suggestions.push("copilot start".to_string());
    suggestions.push("copilot sign in".to_string());
    suggestions.push("copilot sign out".to_string());
    suggestions.push("copilot status".to_string());
    suggestions.push("copilot stop".to_string());
    
    // Add AI commands
    suggestions.push("explain this code".to_string());
    suggestions.push("optimize this code".to_string());
    suggestions.push("generate documentation".to_string());
    suggestions.push("help me debug".to_string());
    
    // Add file-related commands if we have a current file
    if !current_file.is_empty() {
        suggestions.push(format!("rename {} to newname", current_file));
        suggestions.push(format!("delete {}", current_file));
    }
    
    // Return random selection of suggestions
    let mut rng = rand::thread_rng();
    suggestions.shuffle(&mut rng);
    suggestions.truncate(12);
    suggestions
}

#[tauri::command]
pub fn check_has_run_before(first_run_store: tauri::State<'_, FirstRunStore>) -> bool {
    first_run_store.check_has_run_before()
}

#[tauri::command]
pub fn set_has_run_before(first_run_store: tauri::State<'_, FirstRunStore>) -> Result<(), String> {
    first_run_store.set_has_run_before()
} 