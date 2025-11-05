// بسم الله الرحمن الرحيم
// GitHub Copilot Language Server Integration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotConfig {
    pub enabled: bool,
    pub telemetry: bool,
    pub auto_trigger: bool,
    pub debounce_ms: u64,
}

impl Default for CopilotConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            telemetry: true,
            auto_trigger: true,
            debounce_ms: 150,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineCompletionItem {
    pub insert_text: String,
    pub range: Option<Range>,
    pub command: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InlineCompletionList {
    pub items: Vec<InlineCompletionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotStatus {
    pub status: String,        // "Normal", "Error", "Warning", "Inactive"
    pub message: String,
    pub signed_in: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInResponse {
    pub user_code: String,
    pub verification_uri: String,
}

// JSON-RPC structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: u64,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcNotification {
    jsonrpc: String,
    method: String,
    params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<serde_json::Value>,
}

pub struct CopilotServer {
    process: Arc<Mutex<Option<Child>>>,
    request_id: Arc<Mutex<u64>>,
    status: Arc<RwLock<CopilotStatus>>,
    config: Arc<RwLock<CopilotConfig>>,
    initialized: Arc<Mutex<bool>>,
}

impl CopilotServer {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
            status: Arc::new(RwLock::new(CopilotStatus {
                status: "Inactive".to_string(),
                message: "Not started".to_string(),
                signed_in: false,
            })),
            config: Arc::new(RwLock::new(CopilotConfig::default())),
            initialized: Arc::new(Mutex::new(false)),
        }
    }

    async fn next_request_id(&self) -> u64 {
        let mut id = self.request_id.lock().await;
        *id += 1;
        *id
    }

    pub async fn start(&self, workspace_path: Option<String>) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;
        
        if process_guard.is_some() {
            return Err("Copilot server already running".to_string());
        }

        // Find the Copilot language server binary
        let copilot_path = self.find_copilot_binary()?;

        // Start the language server process
        let mut child = Command::new(copilot_path)
            .arg("--stdio")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start Copilot server: {}", e))?;

        // Initialize the server
        let stdin = child.stdin.take().ok_or("Failed to get stdin")?;
        let stdout = child.stdout.take().ok_or("Failed to get stdout")?;

        // Store the process
        *process_guard = Some(child);
        drop(process_guard);

        // Send initialize request
        self.initialize(stdin, stdout, workspace_path).await?;

        // Update status
        let mut status = self.status.write().await;
        status.status = "Normal".to_string();
        status.message = "Copilot server started".to_string();

        Ok(())
    }

    async fn initialize(
        &self,
        mut stdin: tokio::process::ChildStdin,
        stdout: tokio::process::ChildStdout,
        workspace_path: Option<String>,
    ) -> Result<(), String> {
        let id = self.next_request_id().await;

        let workspace_folders = if let Some(path) = workspace_path {
            vec![serde_json::json!({ "uri": format!("file://{}", path) })]
        } else {
            vec![]
        };

        let init_params = serde_json::json!({
            "processId": std::process::id(),
            "workspaceFolders": workspace_folders,
            "capabilities": {
                "workspace": { "workspaceFolders": true }
            },
            "initializationOptions": {
                "editorInfo": {
                    "name": "Vuno",
                    "version": "0.1.0"
                },
                "editorPluginInfo": {
                    "name": "Vuno Copilot Integration",
                    "version": "1.0.0"
                }
            }
        });

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "initialize".to_string(),
            params: init_params,
        };

        // Send the request
        self.send_message(&mut stdin, &request).await?;

        // TODO: Read and parse response from stdout
        // For now, we'll assume it succeeded and send initialized notification

        let notification = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: "initialized".to_string(),
            params: serde_json::json!({}),
        };

        self.send_notification(&mut stdin, &notification).await?;

        let mut initialized = self.initialized.lock().await;
        *initialized = true;

        Ok(())
    }

    async fn send_message<T: Serialize>(
        &self,
        stdin: &mut tokio::process::ChildStdin,
        message: &T,
    ) -> Result<(), String> {
        let content = serde_json::to_string(message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        let header = format!("Content-Length: {}\r\n\r\n", content.len());
        let full_message = format!("{}{}", header, content);

        stdin
            .write_all(full_message.as_bytes())
            .await
            .map_err(|e| format!("Failed to write to stdin: {}", e))?;

        stdin
            .flush()
            .await
            .map_err(|e| format!("Failed to flush stdin: {}", e))?;

        Ok(())
    }

    async fn send_notification(
        &self,
        stdin: &mut tokio::process::ChildStdin,
        notification: &JsonRpcNotification,
    ) -> Result<(), String> {
        self.send_message(stdin, notification).await
    }

    pub async fn stop(&self) -> Result<(), String> {
        let mut process_guard = self.process.lock().await;
        
        if let Some(mut child) = process_guard.take() {
            child
                .kill()
                .await
                .map_err(|e| format!("Failed to kill Copilot server: {}", e))?;
        }

        let mut status = self.status.write().await;
        status.status = "Inactive".to_string();
        status.message = "Copilot server stopped".to_string();
        status.signed_in = false;

        let mut initialized = self.initialized.lock().await;
        *initialized = false;

        Ok(())
    }

    pub async fn get_status(&self) -> CopilotStatus {
        self.status.read().await.clone()
    }

    fn find_copilot_binary(&self) -> Result<PathBuf, String> {
        // Try to find the Copilot language server binary
        // Priority:
        // 1. Check node_modules/@github/copilot-language-server-*
        // 2. Check global npm installation
        // 3. Check PATH

        #[cfg(target_os = "macos")]
        {
            #[cfg(target_arch = "aarch64")]
            let platform = "darwin-arm64";
            #[cfg(target_arch = "x86_64")]
            let platform = "darwin-x64";

            // Check node_modules
            let node_modules_path = PathBuf::from("node_modules")
                .join(format!("@github/copilot-language-server-{}", platform))
                .join("copilot-language-server");

            if node_modules_path.exists() {
                return Ok(node_modules_path);
            }
        }

        #[cfg(target_os = "windows")]
        {
            let platform = "win32-x64";
            let node_modules_path = PathBuf::from("node_modules")
                .join(format!("@github/copilot-language-server-{}", platform))
                .join("copilot-language-server.exe");

            if node_modules_path.exists() {
                return Ok(node_modules_path);
            }
        }

        #[cfg(target_os = "linux")]
        {
            #[cfg(target_arch = "x86_64")]
            let platform = "linux-x64";
            #[cfg(target_arch = "aarch64")]
            let platform = "linux-arm64";

            let node_modules_path = PathBuf::from("node_modules")
                .join(format!("@github/copilot-language-server-{}", platform))
                .join("copilot-language-server");

            if node_modules_path.exists() {
                return Ok(node_modules_path);
            }
        }

        // Try using npx as fallback
        Err("Copilot language server not found. Please install @github/copilot-language-server".to_string())
    }

    pub async fn sign_in(&self) -> Result<SignInResponse, String> {
        // TODO: Implement authentication flow
        // This requires sending signIn request and handling the device flow

        Err("Sign in not yet implemented".to_string())
    }

    pub async fn sign_out(&self) -> Result<(), String> {
        // TODO: Implement sign out
        Err("Sign out not yet implemented".to_string())
    }

    pub async fn get_completions(
        &self,
        file_uri: String,
        _content: String,
        position: Position,
        _version: u32,
    ) -> Result<InlineCompletionList, String> {
        let initialized = self.initialized.lock().await;
        if !*initialized {
            return Err("Copilot server not initialized".to_string());
        }
        drop(initialized);

        // TODO: Implement actual completion request
        // For now, return empty list
        println!("Requesting completion for {} at {:?}", file_uri, position);

        Ok(InlineCompletionList { items: vec![] })
    }
}

// Global Copilot server instance
static COPILOT_SERVER: once_cell::sync::Lazy<CopilotServer> =
    once_cell::sync::Lazy::new(|| CopilotServer::new());

// Tauri commands
#[tauri::command]
pub async fn copilot_start_server(workspace_path: Option<String>) -> Result<(), String> {
    COPILOT_SERVER.start(workspace_path).await
}

#[tauri::command]
pub async fn copilot_stop_server() -> Result<(), String> {
    COPILOT_SERVER.stop().await
}

#[tauri::command]
pub async fn copilot_get_status() -> Result<CopilotStatus, String> {
    Ok(COPILOT_SERVER.get_status().await)
}

#[tauri::command]
pub async fn copilot_sign_in() -> Result<SignInResponse, String> {
    COPILOT_SERVER.sign_in().await
}

#[tauri::command]
pub async fn copilot_sign_out() -> Result<(), String> {
    COPILOT_SERVER.sign_out().await
}

#[tauri::command]
pub async fn copilot_get_completions(
    file_uri: String,
    content: String,
    line: u32,
    character: u32,
    version: u32,
) -> Result<InlineCompletionList, String> {
    let position = Position { line, character };
    COPILOT_SERVER
        .get_completions(file_uri, content, position, version)
        .await
}

#[tauri::command]
pub async fn copilot_accept_completion(completion_id: String) -> Result<(), String> {
    // TODO: Send telemetry for accepted completion
    println!("Completion accepted: {}", completion_id);
    Ok(())
}

#[tauri::command]
pub async fn copilot_reject_completion(completion_id: String) -> Result<(), String> {
    // TODO: Send telemetry for rejected completion
    println!("Completion rejected: {}", completion_id);
    Ok(())
}
