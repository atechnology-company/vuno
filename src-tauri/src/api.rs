use parking_lot::RwLock;
use std::fs;
use std::path::PathBuf;
use tauri;
use reqwest;
use serde_json;

pub struct ApiKeyStore {
    key: RwLock<String>,
    config_path: PathBuf,
}

impl ApiKeyStore {
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
        
        let config_path = app_dir.join("api_key.txt");
        
        let key = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(key) => key,
                Err(_) => String::new(),
            }
        } else {
            String::new()
        };
        
        Self {
            key: RwLock::new(key),
            config_path,
        }
    }
    
    pub fn get_key(&self) -> String {
        self.key.read().clone()
    }
    
    pub fn set_key(&self, new_key: &str) -> Result<(), String> {
        let mut key = self.key.write();
        *key = new_key.to_string();
        
        fs::write(&self.config_path, new_key)
            .map_err(|e| format!("Failed to save API key: {}", e))
    }
}

#[tauri::command]
pub fn get_api_key(state: tauri::State<'_, ApiKeyStore>) -> String {
    state.get_key()
}

#[tauri::command]
pub fn set_api_key(state: tauri::State<'_, ApiKeyStore>, api_key: String) -> Result<(), String> {
    state.set_key(&api_key)
}

#[tauri::command]
pub async fn send_chat_message(messages: Vec<serde_json::Value>, api_key: String) -> Result<String, String> {
    if api_key.is_empty() {
        return Err("No API key provided".to_string());
    }
    
    if messages.is_empty() {
        return Err("No messages provided".to_string());
    }
    
    let client = reqwest::Client::new();
    let api_url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent";
    
    let mut gemini_messages = Vec::new();
    for msg in messages {
        let role = msg.get("role")
            .and_then(|r| r.as_str())
            .ok_or_else(|| "Message missing role".to_string())?;
            
        let content = msg.get("content")
            .and_then(|c| c.as_str())
            .ok_or_else(|| "Message missing content".to_string())?;
            
        let gemini_role = match role {
            "user" => "user",
            "assistant" => "model",
            _ => return Err(format!("Invalid role: {}", role)),
        };
        
        gemini_messages.push(serde_json::json!({
            "role": gemini_role,
            "parts": [{
                "text": content
            }]
        }));
    }
    
    let payload = serde_json::json!({
        "contents": gemini_messages,
        "generationConfig": {
            "temperature": 0.7,
            "topP": 0.95,
            "topK": 40,
            "maxOutputTokens": 4096
        }
    });
    
    let response = client.post(format!("{}?key={}", api_url, api_key))
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("API request error: {}", e))?;
        
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("API error ({}): {}", status, error_text));
    }
    
    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
        
    let text = response_json
        .get("candidates")
        .and_then(|candidates| candidates.get(0))
        .and_then(|candidate| candidate.get("content"))
        .and_then(|content| content.get("parts"))
        .and_then(|parts| parts.get(0))
        .and_then(|part| part.get("text"))
        .and_then(|text| text.as_str())
        .unwrap_or("Sorry, I couldn't generate a response.");
        
    Ok(text.to_string())
} 