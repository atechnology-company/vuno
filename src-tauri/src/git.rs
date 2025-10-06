// بسم الله الرحمن الرحيم

use std::process::Command;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitStatus {
    pub branch: String,
    pub modified: Vec<String>,
    pub added: Vec<String>,
    pub deleted: Vec<String>,
    pub untracked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBranch {
    pub name: String,
    pub is_current: bool,
}

fn execute_git_command(args: &[&str], working_dir: Option<&str>) -> Result<String, String> {
    let mut cmd = Command::new("git");
    cmd.args(args);
    
    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }
    
    let output = cmd.output()
        .map_err(|e| format!("Failed to execute git command: {}. Is git installed?", e))?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(format!("Git error: {}", stderr))
    }
}

#[tauri::command]
pub fn git_status(working_dir: Option<String>) -> Result<GitStatus, String> {
    // Get current branch
    let branch_output = execute_git_command(&["branch", "--show-current"], working_dir.as_deref())?;
    let branch = branch_output.trim().to_string();
    
    // Get status
    let status_output = execute_git_command(&["status", "--porcelain"], working_dir.as_deref())?;
    
    let mut modified = Vec::new();
    let mut added = Vec::new();
    let mut deleted = Vec::new();
    let mut untracked = Vec::new();
    
    for line in status_output.lines() {
        if line.len() < 3 {
            continue;
        }
        
        let status_code = &line[0..2];
        let file_path = line[3..].to_string();
        
        match status_code {
            " M" | "M " | "MM" => modified.push(file_path),
            "A " | "AM" => added.push(file_path),
            " D" | "D " => deleted.push(file_path),
            "??" => untracked.push(file_path),
            _ => {}
        }
    }
    
    Ok(GitStatus {
        branch,
        modified,
        added,
        deleted,
        untracked,
    })
}

#[tauri::command]
pub fn git_add(files: Vec<String>, working_dir: Option<String>) -> Result<String, String> {
    if files.is_empty() {
        return Err("No files specified to add".to_string());
    }
    
    let mut args = vec!["add"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    
    execute_git_command(&args, working_dir.as_deref())?;
    Ok(format!("Added {} file(s)", files.len()))
}

#[tauri::command]
pub fn git_commit(message: String, working_dir: Option<String>) -> Result<String, String> {
    if message.is_empty() {
        return Err("Commit message cannot be empty".to_string());
    }
    
    let output = execute_git_command(&["commit", "-m", &message], working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_push(remote: Option<String>, branch: Option<String>, working_dir: Option<String>) -> Result<String, String> {
    let mut args = vec!["push"];
    
    if let Some(r) = &remote {
        args.push(r.as_str());
    }
    
    if let Some(b) = &branch {
        args.push(b.as_str());
    }
    
    let output = execute_git_command(&args, working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_pull(remote: Option<String>, branch: Option<String>, working_dir: Option<String>) -> Result<String, String> {
    let mut args = vec!["pull"];
    
    if let Some(r) = &remote {
        args.push(r.as_str());
    }
    
    if let Some(b) = &branch {
        args.push(b.as_str());
    }
    
    let output = execute_git_command(&args, working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_branch_list(working_dir: Option<String>) -> Result<Vec<GitBranch>, String> {
    let output = execute_git_command(&["branch", "-a"], working_dir.as_deref())?;
    
    let branches: Vec<GitBranch> = output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            
            let is_current = trimmed.starts_with('*');
            let name = if is_current {
                trimmed[2..].to_string()
            } else {
                trimmed.to_string()
            };
            
            Some(GitBranch { name, is_current })
        })
        .collect();
    
    Ok(branches)
}

#[tauri::command]
pub fn git_checkout(branch: String, create_new: bool, working_dir: Option<String>) -> Result<String, String> {
    if branch.is_empty() {
        return Err("Branch name cannot be empty".to_string());
    }
    
    let mut args = vec!["checkout"];
    if create_new {
        args.push("-b");
    }
    args.push(&branch);
    
    let output = execute_git_command(&args, working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_diff(file_path: Option<String>, working_dir: Option<String>) -> Result<String, String> {
    let mut args = vec!["diff"];
    
    if let Some(path) = &file_path {
        args.push(path.as_str());
    }
    
    let output = execute_git_command(&args, working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_log(limit: Option<u32>, working_dir: Option<String>) -> Result<Vec<GitCommit>, String> {
    let limit_str = limit.unwrap_or(20).to_string();
    let output = execute_git_command(
        &["log", &format!("-{}", limit_str), "--pretty=format:%H|%an|%ad|%s", "--date=short"],
        working_dir.as_deref()
    )?;
    
    let commits: Vec<GitCommit> = output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() < 4 {
                return None;
            }
            
            Some(GitCommit {
                hash: parts[0].to_string(),
                author: parts[1].to_string(),
                date: parts[2].to_string(),
                message: parts[3].to_string(),
            })
        })
        .collect();
    
    Ok(commits)
}

#[tauri::command]
pub fn git_init(working_dir: Option<String>) -> Result<String, String> {
    let output = execute_git_command(&["init"], working_dir.as_deref())?;
    Ok(output)
}

#[tauri::command]
pub fn git_clone(url: String, destination: Option<String>, working_dir: Option<String>) -> Result<String, String> {
    if url.is_empty() {
        return Err("Repository URL cannot be empty".to_string());
    }
    
    let mut args = vec!["clone", url.as_str()];
    
    if let Some(dest) = &destination {
        args.push(dest.as_str());
    }
    
    let output = execute_git_command(&args, working_dir.as_deref())?;
    Ok(output)
}
