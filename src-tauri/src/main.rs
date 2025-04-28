// بسم الله الرحمن الرحيم

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::env;
use std::sync::Mutex;
use tauri::Manager;

mod buffer;
mod api;
mod config;
mod window;
mod commands;
mod hotkeys;

use buffer::BufferManager;
use api::ApiKeyStore;
use config::FirstRunStore;

// Store CLI args for later use
struct CliArgs {
    file_to_open: Mutex<Option<String>>,
}

// Basic AppState for sharing data across the app
pub struct AppState {
    pub version: String,
}

// Initialize the application state
fn init_app_state() -> AppState {
    AppState {
        version: "0.1.0".to_string(),
    }
}

#[tauri::command]
fn get_cli_args(cli_args: tauri::State<'_, CliArgs>) -> Option<String> {
    // Return the file path from CLI args if any
    let mut file_to_open = cli_args.file_to_open.lock().unwrap();
    file_to_open.take()
}

#[tauri::command]
fn ping() -> String {
    "pong".to_string()
}

fn main() {
    env_logger::init();
    
    // Initialize buffer manager
    let buffer_manager = BufferManager::new();
    
    // Initialize hotkey manager
    let hotkey_manager = hotkeys::HotkeyManager::new();
    
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    let file_to_open = if args.len() > 1 {
        Some(args[1].clone())
    } else {
        None
    };
    
    // Store CLI args
    let cli_args = CliArgs {
        file_to_open: Mutex::new(file_to_open.clone()),
    };

    tauri::Builder::default()
        .manage(buffer_manager)
        .manage(init_app_state())
        .manage(cli_args)
        .manage(hotkey_manager.clone())
        .setup(move |app| {
            // Create API key store
            let api_key_store = ApiKeyStore::new(&app.handle());
            app.manage(api_key_store);
            
            // Create first run store
            let first_run_store = FirstRunStore::new(&app.handle());
            app.manage(first_run_store);
            
            // Get main window
            let main_window = app.get_window("main").unwrap();
            
            // Set up hotkey event listener
            let hotkey_manager = app.state::<hotkeys::HotkeyManager>();
            hotkey_manager.setup_event_listener(app.app_handle());
            
            // If we have a file to open from CLI args, emit event to frontend
            if let Some(file_path) = &file_to_open {
                main_window.emit("open-file-from-cli", file_path.clone()).unwrap();
            }
            
            // Set up window-level events for keyboard shortcuts instead of global shortcuts
            // This requires that the frontend will listen for these events
            let win = main_window.clone();
            win.listen("tauri://window-focused", move |_| {
                println!("Window focused");
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Connection check
            ping,
            
            // Buffer commands
            buffer::open_file,
            buffer::get_content,
            buffer::apply_edit,
            buffer::save_file,
            buffer::delete_file,
            
            // API commands
            api::get_api_key,
            api::set_api_key,
            api::send_chat_message,
            
            // Config commands
            config::get_command_suggestions,
            config::check_has_run_before,
            config::set_has_run_before,
            
            // Window commands
            window::minimize_window,
            window::maximize_window,
            window::close_window,
            
            // CLI args command
            get_cli_args,
            
            // Command execution
            commands::execute_command,
            
            // Hotkey commands
            hotkeys::register_hotkey,
            hotkeys::unregister_hotkey,
            hotkeys::list_hotkeys,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
