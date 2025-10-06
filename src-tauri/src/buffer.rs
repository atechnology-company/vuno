use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use ropey::Rope;

#[derive(Clone, Serialize, Deserialize)]
pub struct Buffer {
    pub content: String,
    pub path: Option<PathBuf>,
    pub modified: bool,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub cursor_position: usize,
    pub scroll_position: usize,
    pub language: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BufferEdit {
    pub start: usize,
    pub end: usize,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BufferInfo {
    pub id: usize,
    pub path: Option<String>,
    pub modified: bool,
    pub size: usize,
    pub lines: usize,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

pub struct BufferManager {
    buffers: RwLock<HashMap<usize, Buffer>>,
    next_id: RwLock<usize>,
    edit_history: RwLock<HashMap<usize, Vec<BufferEdit>>>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
            edit_history: RwLock::new(HashMap::new()),
        }
    }
    
    pub fn create_buffer(&self, content: String, path: Option<PathBuf>) -> usize {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        
        let now = Utc::now();
        let language = self.detect_language(&path, &content);
        
        let buffer = Buffer { 
            content: content.clone(),
            path: path.clone(),
            modified: false,
            created_at: now,
            modified_at: now,
            cursor_position: 0,
            scroll_position: 0,
            language,
        };
        
        self.buffers.write().insert(id, buffer);
        self.edit_history.write().insert(id, Vec::new());
        id
    }
    
    fn detect_language(&self, path: &Option<PathBuf>, content: &str) -> Option<String> {
        if let Some(path) = path {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                return match ext.to_lowercase().as_str() {
                    "rs" => Some("rust".to_string()),
                    "js" => Some("javascript".to_string()),
                    "ts" => Some("typescript".to_string()),
                    "py" => Some("python".to_string()),
                    "go" => Some("go".to_string()),
                    "java" => Some("java".to_string()),
                    "cpp" | "cc" | "cxx" => Some("cpp".to_string()),
                    "c" => Some("c".to_string()),
                    "h" | "hpp" => Some("c".to_string()),
                    "html" => Some("html".to_string()),
                    "css" => Some("css".to_string()),
                    "json" => Some("json".to_string()),
                    "xml" => Some("xml".to_string()),
                    "md" => Some("markdown".to_string()),
                    "yml" | "yaml" => Some("yaml".to_string()),
                    "toml" => Some("toml".to_string()),
                    "sql" => Some("sql".to_string()),
                    "sh" | "bash" => Some("bash".to_string()),
                    _ => None,
                };
            }
        }
        
        // Try to detect from content
        if content.starts_with("#!/bin/bash") || content.starts_with("#!/usr/bin/env bash") {
            Some("bash".to_string())
        } else if content.starts_with("#!/usr/bin/env python") || content.starts_with("#!/usr/bin/python") {
            Some("python".to_string())
        } else if content.contains("fn main()") && content.contains("println!") {
            Some("rust".to_string())
        } else if content.contains("function") && content.contains("console.log") {
            Some("javascript".to_string())
        } else {
            None
        }
    }
    
    pub fn get_buffer(&self, id: usize) -> Option<Buffer> {
        self.buffers.read().get(&id).cloned()
    }
    
    pub fn get_buffer_info(&self, id: usize) -> Option<BufferInfo> {
        let buffers = self.buffers.read();
        if let Some(buffer) = buffers.get(&id) {
            let lines = buffer.content.lines().count();
            Some(BufferInfo {
                id,
                path: buffer.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                modified: buffer.modified,
                size: buffer.content.len(),
                lines,
                language: buffer.language.clone(),
                created_at: buffer.created_at,
                modified_at: buffer.modified_at,
            })
        } else {
            None
        }
    }
    
    pub fn list_buffers(&self) -> Vec<BufferInfo> {
        let buffers = self.buffers.read();
        buffers.iter().map(|(id, buffer)| {
            let lines = buffer.content.lines().count();
            BufferInfo {
                id: *id,
                path: buffer.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                modified: buffer.modified,
                size: buffer.content.len(),
                lines,
                language: buffer.language.clone(),
                created_at: buffer.created_at,
                modified_at: buffer.modified_at,
            }
        }).collect()
    }
    
    pub fn update_buffer_content(&self, id: usize, content: String) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.content = content;
            buffer.modified = true;
            buffer.modified_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn update_cursor_position(&self, id: usize, position: usize) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.cursor_position = position;
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn update_scroll_position(&self, id: usize, position: usize) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.scroll_position = position;
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn update_buffer(&self, id: usize, content: String) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.content = content;
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn apply_edit(&self, id: usize, start: usize, end: usize, text: &str) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            let content = &buffer.content;
            
            // Ensure start and end are valid
            if start > content.len() || end > content.len() {
                return Err("Invalid range".to_string());
            }
            
            // Store edit in history
            let edit = BufferEdit {
                start,
                end,
                text: text.to_string(),
                timestamp: Utc::now(),
            };
            
            let mut history = self.edit_history.write();
            if let Some(buffer_history) = history.get_mut(&id) {
                buffer_history.push(edit);
                // Keep only last 100 edits
                if buffer_history.len() > 100 {
                    buffer_history.remove(0);
                }
            }
            
            // Apply the edit
            let new_content = format!("{}{}{}", 
                &content[..start], 
                text, 
                &content[end.min(content.len())..]);
            
            buffer.content = new_content;
            buffer.modified = true;
            buffer.modified_at = Utc::now();
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn get_edit_history(&self, id: usize) -> Vec<BufferEdit> {
        let history = self.edit_history.read();
        history.get(&id).cloned().unwrap_or_default()
    }
    
    pub fn search_in_buffer(&self, id: usize, query: &str, case_sensitive: bool) -> Result<Vec<(usize, usize)>, String> {
        let buffers = self.buffers.read();
        if let Some(buffer) = buffers.get(&id) {
            let content = if case_sensitive {
                buffer.content.clone()
            } else {
                buffer.content.to_lowercase()
            };
            
            let search_query = if case_sensitive {
                query.to_string()
            } else {
                query.to_lowercase()
            };
            
            let mut matches = Vec::new();
            let mut start = 0;
            
            while let Some(pos) = content[start..].find(&search_query) {
                let absolute_pos = start + pos;
                matches.push((absolute_pos, absolute_pos + search_query.len()));
                start = absolute_pos + 1;
            }
            
            Ok(matches)
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn replace_in_buffer(&self, id: usize, query: &str, replacement: &str, case_sensitive: bool) -> Result<usize, String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            let original_content = buffer.content.clone();
            
            let new_content = if case_sensitive {
                original_content.replace(query, replacement)
            } else {
                // Case insensitive replacement is more complex
                let lower_content = original_content.to_lowercase();
                let lower_query = query.to_lowercase();
                let mut result = String::new();
                let mut last_end = 0;
                
                while let Some(pos) = lower_content[last_end..].find(&lower_query) {
                    let absolute_pos = last_end + pos;
                    result.push_str(&original_content[last_end..absolute_pos]);
                    result.push_str(replacement);
                    last_end = absolute_pos + query.len();
                }
                result.push_str(&original_content[last_end..]);
                result
            };
            
            let replacements = if case_sensitive {
                original_content.matches(query).count()
            } else {
                original_content.to_lowercase().matches(&query.to_lowercase()).count()
            };
            
            buffer.content = new_content;
            buffer.modified = true;
            buffer.modified_at = Utc::now();
            
            Ok(replacements)
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn mark_as_saved(&self, id: usize) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        if let Some(buffer) = buffers.get_mut(&id) {
            buffer.modified = false;
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
    
    pub fn close_buffer(&self, id: usize) -> Result<(), String> {
        let mut buffers = self.buffers.write();
        let mut history = self.edit_history.write();
        
        if buffers.remove(&id).is_some() {
            history.remove(&id);
            Ok(())
        } else {
            Err(format!("Buffer {} not found", id))
        }
    }
}

#[tauri::command]
pub fn open_file(path: String, buffer_manager: tauri::State<'_, BufferManager>) -> Result<usize, String> {
    let path_obj = PathBuf::from(path);
    let content = fs::read_to_string(&path_obj)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    Ok(buffer_manager.create_buffer(content, Some(path_obj)))
}

#[tauri::command]
pub fn create_new_buffer(buffer_manager: tauri::State<'_, BufferManager>) -> Result<usize, String> {
    Ok(buffer_manager.create_buffer(String::new(), None))
}

#[tauri::command]
pub fn get_content(buffer_id: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<String, String> {
    if let Some(buffer) = buffer_manager.get_buffer(buffer_id) {
        Ok(buffer.content)
    } else {
        Err(format!("Buffer {} not found", buffer_id))
    }
}

#[tauri::command]
pub fn get_buffer_info(buffer_id: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<BufferInfo, String> {
    if let Some(info) = buffer_manager.get_buffer_info(buffer_id) {
        Ok(info)
    } else {
        Err(format!("Buffer {} not found", buffer_id))
    }
}

#[tauri::command]
pub fn list_buffers(buffer_manager: tauri::State<'_, BufferManager>) -> Result<Vec<BufferInfo>, String> {
    Ok(buffer_manager.list_buffers())
}

#[tauri::command]
pub fn apply_edit(buffer_id: usize, start: usize, end: usize, text: String, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.apply_edit(buffer_id, start, end, &text)
}

#[tauri::command]
pub fn update_cursor_position(buffer_id: usize, position: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.update_cursor_position(buffer_id, position)
}

#[tauri::command]
pub fn update_scroll_position(buffer_id: usize, position: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.update_scroll_position(buffer_id, position)
}

#[tauri::command]
pub fn search_in_buffer(buffer_id: usize, query: String, case_sensitive: bool, buffer_manager: tauri::State<'_, BufferManager>) -> Result<Vec<(usize, usize)>, String> {
    buffer_manager.search_in_buffer(buffer_id, &query, case_sensitive)
}

#[tauri::command]
pub fn replace_in_buffer(buffer_id: usize, query: String, replacement: String, case_sensitive: bool, buffer_manager: tauri::State<'_, BufferManager>) -> Result<usize, String> {
    buffer_manager.replace_in_buffer(buffer_id, &query, &replacement, case_sensitive)
}

#[tauri::command]
pub fn get_edit_history(buffer_id: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<Vec<BufferEdit>, String> {
    Ok(buffer_manager.get_edit_history(buffer_id))
}

#[tauri::command]
pub fn update_buffer_content_command(buffer_id: usize, content: String, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.update_buffer_content(buffer_id, content)
}

#[tauri::command]
pub fn save_file(buffer_id: usize, path: Option<String>, buffer_manager: tauri::State<'_, BufferManager>) -> Result<String, String> {
    if let Some(buffer) = buffer_manager.get_buffer(buffer_id) {
        let file_path = if let Some(path) = path {
            PathBuf::from(path)
        } else if let Some(existing_path) = buffer.path {
            existing_path
        } else {
            return Err("No file path provided and buffer has no associated file".to_string());
        };
        
        fs::write(&file_path, buffer.content)
            .map_err(|e| format!("Failed to save file: {}", e))?;
        
        buffer_manager.mark_as_saved(buffer_id)?;
        Ok(file_path.to_string_lossy().to_string())
    } else {
        Err(format!("Buffer {} not found", buffer_id))
    }
}

#[tauri::command]
pub fn close_buffer(buffer_id: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.close_buffer(buffer_id)
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    fs::remove_file(path)
        .map_err(|e| format!("Failed to delete file: {}", e))
}