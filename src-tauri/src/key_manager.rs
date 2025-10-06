// بسم الله الرحمن الرحيم

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::str::FromStr;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use global_hotkey::hotkey::HotKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBinding {
    pub id: String,
    pub keys: Vec<String>,
    pub action: String,
    pub description: String,
    pub context: String, // "global", "editor", "command_bar", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyEvent {
    pub key: String,
    pub modifiers: Vec<String>,
    pub action: String, // "press", "release"
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct KeyManager {
    bindings: Arc<RwLock<HashMap<String, KeyBinding>>>,
    app_handle: Arc<Mutex<Option<AppHandle>>>,
    is_running: Arc<RwLock<bool>>,
    hotkeys: Arc<Mutex<HashMap<String, HotKey>>>,
    hotkey_manager: Arc<Mutex<Option<GlobalHotKeyManager>>>,
}

impl KeyManager {
    pub fn new() -> Self {
        Self {
            bindings: Arc::new(RwLock::new(HashMap::new())),
            app_handle: Arc::new(Mutex::new(None)),
            is_running: Arc::new(RwLock::new(false)),
            hotkeys: Arc::new(Mutex::new(HashMap::new())),
            hotkey_manager: Arc::new(Mutex::new(None)),
        }
    }

    pub fn set_app_handle(&self, app_handle: AppHandle) {
        *self.app_handle.lock().unwrap() = Some(app_handle);
    }

    pub fn start_monitoring(&self) {
        let mut is_running = self.is_running.write();
        if *is_running {
            return; // Already running
        }
        *is_running = true;
        drop(is_running);

        // Initialize global hotkey manager if needed
        {
            let mut mgr_lock = self.hotkey_manager.lock().unwrap();
            if mgr_lock.is_none() {
                *mgr_lock = GlobalHotKeyManager::new().ok();
            }
        }

        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
            // Set a single global event handler (idempotent: setting again overwrites)
            let app = app_handle.clone();
            let hotkeys_arc = self.hotkeys.clone();
            let bindings_arc = self.bindings.clone();
            GlobalHotKeyEvent::set_event_handler(Some(move |event: GlobalHotKeyEvent| {
                // Find matching id by comparing hotkey ids
                if let Ok(hotkeys) = hotkeys_arc.lock() {
                    for (id, hk) in hotkeys.iter() {
                        if event.id == hk.id() {
                            // Find the action for this id
                            let action = {
                                let bs = bindings_arc.read();
                                bs.get(id).map(|b| b.action.clone())
                            };
                            if let Some(act) = action {
                                let _ = app.emit_all("key_action", serde_json::json!({
                                    "action": act
                                }));
                            }
                            break;
                        }
                    }
                }
            }));

            // Build and register hotkeys for all bindings (global shortcuts only)
            let mut to_register: Vec<(String, String)> = Vec::new();
            let bindings = self.bindings.read();
            for binding in bindings.values() {
                if let Some(hk_str) = Self::binding_to_hotkey(&binding.keys) {
                    to_register.push((binding.id.clone(), hk_str));
                }
            }
            drop(bindings);

            // Fallback defaults if none resolved (exclude command bar; ESC toggling handled in frontend)
            if to_register.is_empty() {
                to_register.push(("help".into(), "F1".into()));
                to_register.push(("save_file".into(), Self::platform_cmd_ctrl("S")));
                to_register.push(("new_file".into(), Self::platform_cmd_ctrl("N")));
                to_register.push(("open_file".into(), Self::platform_cmd_ctrl("O")));
            }

            // Unregister previous and clear map
            if let Some(ref mut mgr) = *self.hotkey_manager.lock().unwrap() {
                // Best effort: unregister all previously registered
                if let Ok(mut hotkeys) = self.hotkeys.lock() {
                    for (_id, hk) in hotkeys.drain() {
                        let _ = mgr.unregister(hk);
                    }
                }

                for (id, hk_str) in to_register.iter() {
                    match HotKey::from_str(hk_str) {
                        Ok(hk) => {
                            if let Err(e) = mgr.register(hk) {
                                log::warn!("Failed to register hotkey {} ({}): {}", id, hk_str, e);
                            } else {
                                if let Ok(mut map) = self.hotkeys.lock() {
                                    map.insert(id.clone(), hk);
                                }
                                log::info!("Registered global hotkey: {} -> {}", id, hk_str);
                            }
                        }
                        Err(_) => {
                            log::warn!("Invalid hotkey spec for {}: {}", id, hk_str);
                        }
                    }
                }
            }
        }
    }

    pub fn stop_monitoring(&self) {
        *self.is_running.write() = false;
        if let Some(ref mut mgr) = *self.hotkey_manager.lock().unwrap() {
            if let Ok(mut hotkeys) = self.hotkeys.lock() {
                for (_id, hk) in hotkeys.drain() {
                    let _ = mgr.unregister(hk);
                }
            }
        }
    }

    // With Tauri's global shortcuts, we don't get raw press/release events.

    // No longer needed with GlobalShortcutManager

    // With GlobalShortcutManager we map chords directly when registering

    // Not used with GlobalShortcutManager

    fn execute_action(&self, action: &str) {
        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
            match action {
                "toggle_command_bar" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "toggle_command_bar"
                    }));
                }
                "toggle_help" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "toggle_help"
                    }));
                }
                "escape" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "escape"
                    }));
                }
                "save_file" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "save_file"
                    }));
                }
                "new_file" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "new_file"
                    }));
                }
                "open_file" => {
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": "open_file"
                    }));
                }
                _ => {
                    // Custom action
                    let _ = app_handle.emit_all("key_action", serde_json::json!({
                        "action": action
                    }));
                }
            }
        }
    }

    fn emit_key_event(&self, event: KeyEvent) {
        if let Some(app_handle) = self.app_handle.lock().unwrap().as_ref() {
            let _ = app_handle.emit_all("key_event", event);
        }
    }
    // Keycode to string no longer used

    pub fn register_default_bindings(&self) {
    // Command bar is toggled with Escape in the frontend; skip Cmd/Ctrl+K default

        self.register_binding(KeyBinding {
            id: "help".to_string(),
            keys: vec!["f1".to_string()],
            action: "toggle_help".to_string(),
            description: "Toggle help".to_string(),
            context: "global".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "escape".to_string(),
            keys: vec!["escape".to_string()],
            action: "escape".to_string(),
            description: "Close dialogs/menus".to_string(),
            context: "global".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "save_file".to_string(),
            keys: vec!["cmdorctrl".to_string(), "s".to_string()],
            action: "save_file".to_string(),
            description: "Save file".to_string(),
            context: "editor".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "save_file_ctrl".to_string(),
            keys: vec!["cmdorctrl".to_string(), "s".to_string()],
            action: "save_file".to_string(),
            description: "Save file (Ctrl)".to_string(),
            context: "editor".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "new_file".to_string(),
            keys: vec!["cmdorctrl".to_string(), "n".to_string()],
            action: "new_file".to_string(),
            description: "New file".to_string(),
            context: "global".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "new_file_ctrl".to_string(),
            keys: vec!["cmdorctrl".to_string(), "n".to_string()],
            action: "new_file".to_string(),
            description: "New file (Ctrl)".to_string(),
            context: "global".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "open_file".to_string(),
            keys: vec!["cmdorctrl".to_string(), "o".to_string()],
            action: "open_file".to_string(),
            description: "Open file".to_string(),
            context: "global".to_string(),
        });

        self.register_binding(KeyBinding {
            id: "open_file_ctrl".to_string(),
            keys: vec!["cmdorctrl".to_string(), "o".to_string()],
            action: "open_file".to_string(),
            description: "Open file (Ctrl)".to_string(),
            context: "global".to_string(),
        });
    }

    pub fn register_binding(&self, binding: KeyBinding) {
        let mut bindings = self.bindings.write();
        bindings.insert(binding.id.clone(), binding);
    }

    pub fn unregister_binding(&self, id: &str) {
        let mut bindings = self.bindings.write();
        bindings.remove(id);
    }

    pub fn get_bindings(&self) -> Vec<KeyBinding> {
        let bindings = self.bindings.read();
        bindings.values().cloned().collect()
    }

    pub fn check_accessibility_permissions(&self) -> bool { true }

    pub fn start_monitoring_with_fallback(&self) {
        // Use the same registration path
        self.start_monitoring();
    }

    fn binding_to_hotkey(keys: &Vec<String>) -> Option<String> {
        if keys.is_empty() { return None; }
        let mut mods = Vec::new();
        let mut last = String::new();
        for (i, k) in keys.iter().enumerate() {
            if i == keys.len() - 1 { last = k.clone(); } else { mods.push(k.clone()); }
        }
        if last.eq_ignore_ascii_case("escape") { return None; } // Avoid global ESC
        let mut parts: Vec<String> = Vec::new();
        for m in mods {
            let mm = m.to_lowercase();
            if mm == "cmd" || mm == "command" || mm == "cmdorctrl" { parts.push(Self::platform_cmd()); }
            else if mm == "ctrl" || mm == "control" { parts.push("Control".into()); }
            else if mm == "alt" || mm == "option" { parts.push("Alt".into()); }
            else if mm == "shift" { parts.push("Shift".into()); }
        }
        let key = if last.len() == 1 { last.to_uppercase() } else { last.to_uppercase() };
        parts.push(key);
        Some(parts.join("+"))
    }

    fn platform_cmd() -> String {
        if cfg!(target_os = "macos") { "Command".into() } else { "Control".into() }
    }

    fn platform_cmd_ctrl(key: &str) -> String {
        if cfg!(target_os = "macos") { format!("Command+{}", key) } else { format!("Control+{}", key) }
    }

    fn action_for_id(id: &str) -> Option<&'static str> {
        match id {
            "command_bar" | "command_bar_ctrl" => Some("toggle_command_bar"),
            "help" => Some("toggle_help"),
            "escape" => Some("escape"),
            "save_file" | "save_file_ctrl" => Some("save_file"),
            "new_file" | "new_file_ctrl" => Some("new_file"),
            "open_file" | "open_file_ctrl" => Some("open_file"),
            _ => None,
        }
    }
}

// Tauri commands
#[tauri::command]
pub fn register_key_binding(
    key_manager: tauri::State<'_, KeyManager>,
    binding: KeyBinding,
) -> Result<(), String> {
    key_manager.register_binding(binding);
    Ok(())
}

#[tauri::command]
pub fn unregister_key_binding(
    key_manager: tauri::State<'_, KeyManager>,
    id: String,
) -> Result<(), String> {
    key_manager.unregister_binding(&id);
    Ok(())
}

#[tauri::command]
pub fn get_key_bindings(
    key_manager: tauri::State<'_, KeyManager>,
) -> Result<Vec<KeyBinding>, String> {
    Ok(key_manager.get_bindings())
}

#[tauri::command]
pub fn start_key_monitoring(
    key_manager: tauri::State<'_, KeyManager>,
) -> Result<(), String> {
    key_manager.start_monitoring();
    Ok(())
}

#[tauri::command]
pub fn stop_key_monitoring(
    key_manager: tauri::State<'_, KeyManager>,
) -> Result<(), String> {
    key_manager.stop_monitoring();
    Ok(())
}

#[tauri::command]
pub fn check_accessibility_permissions(
    key_manager: tauri::State<'_, KeyManager>,
) -> Result<bool, String> {
    Ok(key_manager.check_accessibility_permissions())
}

#[tauri::command]
pub fn start_key_monitoring_with_fallback(
    key_manager: tauri::State<'_, KeyManager>,
) -> Result<(), String> {
    key_manager.start_monitoring_with_fallback();
    Ok(())
}
