// بسم الله الرحمن الرحيم

use std::sync::{Arc, Mutex};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use global_hotkey::hotkey::HotKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;


/// Store for managing global hotkeys
#[derive(Clone)]
pub struct HotkeyManager {
    hotkeys: Arc<Mutex<HashMap<String, HotKey>>>,
    manager: Arc<Mutex<Option<GlobalHotKeyManager>>>,
    commands: Arc<Mutex<HashMap<String, String>>>, // id -> action/command
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HotkeyConfig {
    pub id: String,
    pub key: String,
    pub command: String,
}

impl HotkeyManager {
    pub fn new() -> Self {
        let manager = GlobalHotKeyManager::new().ok();
        HotkeyManager {
            hotkeys: Arc::new(Mutex::new(HashMap::new())),
            manager: Arc::new(Mutex::new(manager)),
            commands: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    pub fn register_hotkey_internal(
        &self,
        app_handle: AppHandle,
        id: String,
        key: String,
        command: String,
    ) -> Result<(), String> {
        let hotkey = match HotKey::from_str(&key) {
            Ok(hk) => hk,
            Err(_) => return Err(format!("Invalid hotkey: {}", key)),
        };
        
        let mut manager = self.manager.lock().unwrap();
        if let Some(mgr) = manager.as_mut() {
            if let Err(e) = mgr.register(hotkey) {
                return Err(format!("Failed to register hotkey: {}", e));
            }
        } else {
            // Manager unavailable (e.g., missing accessibility permissions); proceed without OS-level registration
        }
        
        // Store the hotkey
        let mut hotkeys = self.hotkeys.lock().unwrap();
        hotkeys.insert(id.clone(), hotkey);
    // Store id -> command mapping
    let mut commands = self.commands.lock().unwrap();
    commands.insert(id.clone(), command.clone());

        // Store the configuration
        let config = HotkeyConfig {
            id: id.clone(),
            key,
            command,
        };
        
        // Emit an event to confirm registration
        if let Err(e) = app_handle.emit_all("hotkey-registered", config) {
            eprintln!("Failed to emit hotkey-registered event: {}", e);
        }
        
        Ok(())
    }
    
    pub fn unregister_hotkey_internal(&self, app_handle: AppHandle, id: String) -> Result<(), String> {
        let mut hotkeys = self.hotkeys.lock().unwrap();
        
        if let Some(hotkey) = hotkeys.remove(&id) {
            let mut manager = self.manager.lock().unwrap();
            if let Some(mgr) = manager.as_mut() {
                if let Err(e) = mgr.unregister(hotkey) {
                    return Err(format!("Failed to unregister hotkey: {}", e));
                }
            }
            // Remove command mapping
            let mut commands = self.commands.lock().unwrap();
            commands.remove(&id);
            
            // Emit an event to confirm unregistration
            if let Err(e) = app_handle.emit_all("hotkey-unregistered", id) {
                eprintln!("Failed to emit hotkey-unregistered event: {}", e);
            }
            
            Ok(())
        } else {
            Err(format!("Hotkey with ID '{}' not found", id))
        }
    }
    
    pub fn list_hotkeys_internal(&self) -> Vec<String> {
        let hotkeys = self.hotkeys.lock().unwrap();
        hotkeys.keys().cloned().collect()
    }
    
    /// Set up the global hotkey event listener
    pub fn setup_event_listener(&self, app_handle: AppHandle) {
        let app_handle_clone = app_handle.clone();
        let commands_arc = self.commands.clone();

        // Clone the Arc<Mutex<HashMap>> to avoid capturing self in the closure
        let hotkeys_arc = self.hotkeys.clone();
        
        // Listen for global hotkey events
    GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
            let hotkeys = hotkeys_arc.lock().unwrap();
            
            // Find which hotkey was triggered
            for (id, hotkey) in hotkeys.iter() {
                if event.id == hotkey.id() {
                    // Look up mapped command/action and emit key_action directly
                    if let Some(cmd) = commands_arc.lock().unwrap().get(id).cloned() {
                        let _ = app_handle_clone.emit_all("key_action", serde_json::json!({
                            "action": cmd
                        }));
                    } else {
                        // Fallback: emit legacy event with id
                        let _ = app_handle_clone.emit_all("hotkey-triggered", id);
                    }
                    break;
                }
            }
        }));
    }
}   

/// Register a new global hotkey
#[tauri::command]
pub fn register_hotkey(
    hotkey_manager: tauri::State<'_, HotkeyManager>,
    app_handle: AppHandle,
    id: String,
    key: String,
    command: String,
) -> Result<(), String> {
    hotkey_manager.register_hotkey_internal(app_handle, id, key, command)
}

/// Unregister a global hotkey by its ID
#[tauri::command]
pub fn unregister_hotkey(
    hotkey_manager: tauri::State<'_, HotkeyManager>,
    app_handle: AppHandle,
    id: String,
) -> Result<(), String> {
    hotkey_manager.unregister_hotkey_internal(app_handle, id)
}

/// List all registered hotkeys
#[tauri::command]
pub fn list_hotkeys(hotkey_manager: tauri::State<'_, HotkeyManager>) -> Vec<String> {
    hotkey_manager.list_hotkeys_internal()
}