use std::process::Stdio;
use tokio::process::Command as TokioCommand;
use tokio::io::{AsyncBufReadExt, BufReader};
use tauri::Manager;
use serde_json;

#[tauri::command]
pub async fn set_ui_state(app_handle: tauri::AppHandle, state: String, value: bool) -> Result<(), String> {
    // Emit state change to frontend
    let _ = app_handle.emit_all("ui_state_change", serde_json::json!({
        "state": state,
        "value": value,
        "timestamp": chrono::Utc::now().timestamp_millis()
    }));
    Ok(())
}

#[tauri::command]
pub async fn close_command_bar(app_handle: tauri::AppHandle) -> Result<(), String> {
    // Force close command bar with multiple state updates for reliability
    let _ = app_handle.emit_all("force_close_command_bar", serde_json::json!({
        "action": "close",
        "timestamp": chrono::Utc::now().timestamp_millis()
    }));
    Ok(())
}

#[tauri::command]
pub async fn execute_command(command: String) -> Result<String, String> {
    let mut parts = command.split_whitespace();
    let cmd = parts.next().ok_or_else(|| "Empty command".to_string())?;
    let args: Vec<&str> = parts.collect();

    // Handle shell-specific commands
    let (cmd, args) = if cfg!(target_os = "windows") {
        if cmd == "ls" {
            ("dir", args)
        } else if cmd == "pwd" {
            ("cd", args)
        } else {
            (cmd, args)
        }
    } else {
        (cmd, args)
    };

    let mut process = TokioCommand::new(cmd)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| {
            if e.to_string().contains("command not found") {
                "AI_FALLBACK".to_string()
            } else {
                format!("Failed to execute command: {}", e)
            }
        })?;

    let stdout = process.stdout.take().ok_or_else(|| "Failed to capture stdout".to_string())?;
    let stderr = process.stderr.take().ok_or_else(|| "Failed to capture stderr".to_string())?;

    let mut stdout_lines = Vec::new();
    let mut stderr_lines = Vec::new();

    // Read stdout
    let mut stdout_reader = BufReader::new(stdout);
    let mut line = String::new();
    while stdout_reader.read_line(&mut line).await.map_err(|e| format!("Failed to read stdout: {}", e))? > 0 {
        stdout_lines.push(line.trim().to_string());
        line.clear();
    }

    // Read stderr
    let mut stderr_reader = BufReader::new(stderr);
    line.clear();
    while stderr_reader.read_line(&mut line).await.map_err(|e| format!("Failed to read stderr: {}", e))? > 0 {
        stderr_lines.push(line.trim().to_string());
        line.clear();
    }
    
    let status = process.wait().await.map_err(|e| format!("Failed to wait for process: {}", e))?;

    let output = if !stdout_lines.is_empty() {
        stdout_lines.join("\n")
    } else if !stderr_lines.is_empty() {
        stderr_lines.join("\n")
    } else {
        format!("Command exited with status: {}", status)
    };

    Ok(output)
}