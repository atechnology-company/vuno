use parking_lot::RwLock;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri;

#[derive(Clone)]
pub struct Buffer {
    pub content: String,
    pub path: Option<PathBuf>,
}

pub struct BufferManager {
    buffers: RwLock<HashMap<usize, Buffer>>,
    next_id: RwLock<usize>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            buffers: RwLock::new(HashMap::new()),
            next_id: RwLock::new(1),
        }
    }
    
    pub fn create_buffer(&self, content: String, path: Option<PathBuf>) -> usize {
        let mut next_id = self.next_id.write();
        let id = *next_id;
        *next_id += 1;
        
        let buffer = Buffer { content, path };
        self.buffers.write().insert(id, buffer);
        id
    }
    
    pub fn get_buffer(&self, id: usize) -> Option<Buffer> {
        self.buffers.read().get(&id).cloned()
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
            
            // Apply the edit
            let new_content = format!("{}{}{}", 
                &content[..start], 
                text, 
                &content[end.min(content.len())..]);
            
            buffer.content = new_content;
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
pub fn get_content(buffer_id: usize, buffer_manager: tauri::State<'_, BufferManager>) -> Result<String, String> {
    if let Some(buffer) = buffer_manager.get_buffer(buffer_id) {
        Ok(buffer.content)
    } else {
        Err(format!("Buffer {} not found", buffer_id))
    }
}

#[tauri::command]
pub fn apply_edit(buffer_id: usize, start: usize, end: usize, text: String, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    buffer_manager.apply_edit(buffer_id, start, end, &text)
}

#[tauri::command]
pub fn save_file(buffer_id: usize, path: String, buffer_manager: tauri::State<'_, BufferManager>) -> Result<(), String> {
    if let Some(buffer) = buffer_manager.get_buffer(buffer_id) {
        fs::write(path, buffer.content)
            .map_err(|e| format!("Failed to save file: {}", e))
    } else {
        Err(format!("Buffer {} not found", buffer_id))
    }
}

#[tauri::command]
pub fn delete_file(path: String) -> Result<(), String> {
    fs::remove_file(path)
        .map_err(|e| format!("Failed to delete file: {}", e))
} 