//! Claude CLI JSON transport implementation
//! 
//! This module implements subprocess communication with Claude CLI using JSON output format
//! to avoid PTY issues, based on the reference implementation from claude-code-sdk-rust.

use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use std::path::PathBuf;
use tokio::io::{AsyncWriteExt, AsyncBufReadExt, BufReader};
use tokio::process::Command;
use std::process::Stdio;
use serde_json;
use chrono::Utc;

/// Execute Claude CLI task using JSON output format (like reference implementation)
pub async fn execute_claude_cli_json(
    agent: &crate::types::Agent,
    initial_message: &str,
    cancellation_flag: Arc<AtomicBool>,
    _input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
    log_file_path: PathBuf,
    task_id: String,
) -> Result<String> {
    use crate::types::TaskStatus;
    
    info!("Starting Claude CLI JSON streaming for task: {}", task_id);
    
    // Create log file
    let mut log_file = tokio::fs::File::create(&log_file_path).await?;
    let log_header = format!("# Task {} Execution Log\n\n**Agent:** {}\n**Started:** {}\n\n", 
        task_id, agent.name, Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    log_file.write_all(log_header.as_bytes()).await?;
    
    // Build Claude CLI command with JSON output format (like the reference implementation)
    let mut cmd = Command::new("claude");
    cmd.args([
        "--dangerously-skip-permissions",
        "--output-format", "stream-json",
        "--verbose"
    ]);
    
    // Set environment variable like reference implementation
    cmd.env("CLAUDE_CODE_ENTRYPOINT", "nox-rust");
    
    // Configure subprocess I/O
    cmd.stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());
    
    // Spawn the process
    let mut child = cmd.spawn()
        .map_err(|e| anyhow::anyhow!("Failed to spawn Claude CLI: {}", e))?;
    
    // Get stdin handle for sending the prompt
    if let Some(mut stdin) = child.stdin.take() {
        // Send the initial prompt with system context
        let prompt = format!("{}\n\nUser: {}", agent.system_prompt, initial_message);
        stdin.write_all(prompt.as_bytes()).await
            .map_err(|e| anyhow::anyhow!("Failed to write to stdin: {}", e))?;
        stdin.shutdown().await
            .map_err(|e| anyhow::anyhow!("Failed to close stdin: {}", e))?;
    }
    
    // Log initial input
    log_file.write_all(format!("**Input:** {}\n\n", initial_message).as_bytes()).await?;
    log_file.write_all("**Output:**\n".as_bytes()).await?;

    let mut accumulated_output = String::new();
    let mut json_buffer = String::new();
    
    // Handle stdout with JSON parsing
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        
        while let Some(line) = lines.next_line().await
            .map_err(|e| anyhow::anyhow!("Failed to read line: {}", e))? {
            
            if cancellation_flag.load(Ordering::Relaxed) {
                break;
            }
            
            // Process JSON line like reference implementation
            if let Some(json_result) = process_json_line(&mut json_buffer, line.clone()) {
                match json_result {
                    Ok(json_data) => {
                        // Extract content from JSON message
                        if let Some(content) = extract_content_from_json(&json_data) {
                            accumulated_output.push_str(&content);
                            
                            // Send to output channel
                            if let Err(_) = output_tx.send(content.clone()).await {
                                break; // Channel closed
                            }
                            
                            // Append to log file
                            log_file.write_all(content.as_bytes()).await?;
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse JSON: {}", e);
                    }
                }
            }
            
            // Also send raw line for debugging
            if let Err(_) = output_tx.send(format!("RAW: {}\n", line)).await {
                break;
            }
        }
    }
    
    // Wait for process completion
    let exit_status = child.wait().await
        .map_err(|e| anyhow::anyhow!("Process wait failed: {}", e))?;
    
    if exit_status.success() {
        info!("Task {} completed successfully", task_id);
        Ok(accumulated_output)
    } else {
        error!("Task {} failed with exit code: {:?}", task_id, exit_status.code());
        Err(anyhow::anyhow!("Claude CLI process failed"))
    }
}

/// Process JSON line similar to reference implementation
fn process_json_line(json_buffer: &mut String, line: String) -> Option<Result<serde_json::Value, serde_json::Error>> {
    // Accumulate JSON across multiple lines like reference implementation
    if line.starts_with('{') && json_buffer.is_empty() {
        *json_buffer = line;
    } else if !json_buffer.is_empty() {
        json_buffer.push('\n');
        json_buffer.push_str(&line);
    } else {
        // Single line JSON or non-JSON line
        return None;
    }
    
    // Try to parse accumulated buffer
    match serde_json::from_str::<serde_json::Value>(json_buffer) {
        Ok(json_data) => {
            json_buffer.clear();
            Some(Ok(json_data))
        }
        Err(e) => {
            // If buffer is too large, clear it to prevent memory issues
            if json_buffer.len() > 1024 * 1024 { // 1MB limit
                json_buffer.clear();
                Some(Err(e))
            } else {
                None // Continue accumulating
            }
        }
    }
}

/// Extract content from Claude CLI JSON message
fn extract_content_from_json(json_data: &serde_json::Value) -> Option<String> {
    // Handle Claude's assistant message format
    if let Some(msg_type) = json_data.get("type").and_then(|t| t.as_str()) {
        match msg_type {
            "assistant" => {
                // Extract content from assistant message
                if let Some(message) = json_data.get("message") {
                    if let Some(content_array) = message.get("content").and_then(|c| c.as_array()) {
                        if let Some(first_content) = content_array.first() {
                            if let Some(text) = first_content.get("text").and_then(|t| t.as_str()) {
                                return Some(text.to_string());
                            }
                        }
                    }
                }
            }
            "result" => {
                // Extract the final result
                if let Some(result) = json_data.get("result").and_then(|r| r.as_str()) {
                    return Some(format!("Final result: {}\n", result));
                }
            }
            "system" => {
                // System messages can be logged but don't need content extraction
                if let Some(subtype) = json_data.get("subtype").and_then(|s| s.as_str()) {
                    return Some(format!("System: {} initialized\n", subtype));
                }
            }
            _ => {
                // Other message types
                return Some(format!("Message type: {}\n", msg_type));
            }
        }
    }
    
    // Fallback: try simple content fields
    if let Some(content) = json_data.get("content").and_then(|c| c.as_str()) {
        return Some(content.to_string());
    }
    
    if let Some(text) = json_data.get("text").and_then(|t| t.as_str()) {
        return Some(text.to_string());
    }
    
    if let Some(message) = json_data.get("message").and_then(|m| m.as_str()) {
        return Some(message.to_string());
    }
    
    None // Don't return debug JSON for cleaner output
}