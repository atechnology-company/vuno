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
mod key_manager;
mod command_processor;
<<<<<<< Updated upstream
=======
mod perplexity;
mod git;
mod lsp;
mod copilot;
>>>>>>> Stashed changes

use buffer::BufferManager;
use api::ApiKeyStore;
use config::FirstRunStore;
use key_manager::KeyManager;

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
    
    // Initialize key manager
    let key_manager = KeyManager::new();
    
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
        .manage(key_manager.clone())
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
            
            // Set up key manager
            let key_manager = app.state::<KeyManager>();
            key_manager.set_app_handle(app.app_handle());
            key_manager.register_default_bindings();
            
            // Use fallback monitoring to avoid accessibility permission issues
            key_manager.start_monitoring_with_fallback();
            
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
            buffer::create_new_buffer,
            buffer::get_content,
            buffer::get_buffer_info,
            buffer::list_buffers,
            buffer::apply_edit,
            buffer::update_cursor_position,
            buffer::update_scroll_position,
            buffer::search_in_buffer,
            buffer::replace_in_buffer,
            buffer::get_edit_history,
            buffer::save_file,
            buffer::update_buffer_content_command,
            buffer::close_buffer,
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
            window::start_drag,
            window::toggle_fullscreen,
            
            // CLI args command
            get_cli_args,
            
            // Command execution
            commands::execute_command,
            command_processor::execute_enhanced_command,
            command_processor::get_enhanced_command_suggestions,
            command_processor::validate_command,
            
<<<<<<< Updated upstream
=======
            // Perplexity search
            perplexity::get_perplexity_key,
            perplexity::set_perplexity_key,
            perplexity::search_web,
            
            // Git commands
            git::git_status,
            git::git_add,
            git::git_commit,
            git::git_push,
            git::git_pull,
            git::git_branch_list,
            git::git_checkout,
            git::git_diff,
            git::git_log,
            git::git_init,
            git::git_clone,
            
            // LSP commands
            lsp::start_lsp_server,
            lsp::get_running_lsp_servers,
            lsp::get_diagnostics,
            lsp::get_completions,
            lsp::format_document,
            lsp::check_lsp_available,
            
            // GitHub Copilot commands
            copilot::copilot_start_server,
            copilot::copilot_stop_server,
            copilot::copilot_get_status,
            copilot::copilot_sign_in,
            copilot::copilot_sign_out,
            copilot::copilot_get_completions,
            copilot::copilot_accept_completion,
            copilot::copilot_reject_completion,
            
>>>>>>> Stashed changes
            // Hotkey commands
            hotkeys::register_hotkey,
            hotkeys::unregister_hotkey,
            hotkeys::list_hotkeys,
            
            // Key manager commands
            key_manager::register_key_binding,
            key_manager::unregister_key_binding,
            key_manager::get_key_bindings,
            key_manager::start_key_monitoring,
            key_manager::stop_key_monitoring,
            key_manager::check_accessibility_permissions,
            key_manager::start_key_monitoring_with_fallback,
            
            // UI State Management
            commands::set_ui_state,
            commands::close_command_bar,
            
            // Window controls
            window::minimize_window,
            window::maximize_window,
            window::close_window,
            window::start_drag,
            window::toggle_fullscreen,
            window::move_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
