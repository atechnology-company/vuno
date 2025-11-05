// بسم الله الرحمن الرحيم

use parking_lot::RwLock;
use std::fs;
use std::path::PathBuf;
use tauri;
use reqwest;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerplexitySearchResponse {
    pub results: Vec<SearchResult>,
    pub answer: String,
}

pub struct PerplexityKeyStore {
    key: RwLock<String>,
    config_path: PathBuf,
}

impl PerplexityKeyStore {
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
        
        let config_path = app_dir.join("perplexity_key.txt");
        
        let key = if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(key) => key.trim().to_string(),
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
            .map_err(|e| format!("Failed to save Perplexity API key: {}", e))
    }
}

#[tauri::command]
pub fn get_perplexity_key(state: tauri::State<'_, PerplexityKeyStore>) -> String {
    state.get_key()
}

#[tauri::command]
pub fn set_perplexity_key(state: tauri::State<'_, PerplexityKeyStore>, api_key: String) -> Result<(), String> {
    state.set_key(&api_key)
}

#[tauri::command]
pub async fn search_web(query: String, api_key: Option<String>, perplexity_store: tauri::State<'_, PerplexityKeyStore>) -> Result<PerplexitySearchResponse, String> {
    // Get API key from parameter or store
    let key = if let Some(k) = api_key {
        k
    } else {
        let stored_key = perplexity_store.get_key();
        if stored_key.is_empty() {
            return Err("No Perplexity API key provided".to_string());
        }
        stored_key
    };
    
    if key.is_empty() {
        return Err("Perplexity API key is empty".to_string());
    }
    
    if query.is_empty() {
        return Err("Search query is empty".to_string());
    }
    
    let client = reqwest::Client::new();
    let api_url = "https://api.perplexity.ai/chat/completions";
    
    let payload = serde_json::json!({
        "model": "llama-3.1-sonar-small-128k-online",
        "messages": [
            {
                "role": "system",
                "content": "You are a helpful assistant that searches the web and provides accurate, concise answers with sources."
            },
            {
                "role": "user",
                "content": query
            }
        ],
        "temperature": 0.2,
        "max_tokens": 1024,
        "top_p": 0.9,
        "search_domain_filter": [],
        "return_images": false,
        "return_related_questions": false,
        "search_recency_filter": "month",
        "stream": false
    });
    
    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Perplexity API request error: {}", e))?;
        
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!("Perplexity API error ({}): {}", status, error_text));
    }
    
    let response_json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse Perplexity response: {}", e))?;
    
    // Extract answer from response
    let answer = response_json
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or("No answer received from Perplexity")
        .to_string();
    
    // Extract citations if available
    let citations = response_json
        .get("citations")
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|citation| {
                    let url = citation.as_str()?.to_string();
                    Some(SearchResult {
                        title: url.clone(),
                        url: url.clone(),
                        snippet: String::new(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    
    Ok(PerplexitySearchResponse {
        answer,
        results: citations,
    })
}
