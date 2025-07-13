//! Claude process manager module for the Nox agent ecosystem
//! 
//! This module handles the spawning and management of Claude CLI processes.

use anyhow::{Result, anyhow};
use log::{info, error, warn};
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::types::Agent;

// Singleton instance of the Claude process manager
lazy_static::lazy_static! {
    static ref CLAUDE_PROCESS_MANAGER: Arc<Mutex<ClaudeProcessManager>> = Arc::new(Mutex::new(ClaudeProcessManager::new()));
}

/// Claude process manager struct
pub struct ClaudeProcessManager {
    processes: HashMap<String, Child>,
}

impl ClaudeProcessManager {
    /// Create a new Claude process manager
    fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Start a Claude process for an agent
    async fn start_process(&mut self, agent: &Agent) -> Result<()> {
        info!("Starting Claude process for agent: {}", agent.name);
        
        // Check if a process already exists for this agent
        if self.processes.contains_key(&agent.id) {
            warn!("Process already exists for agent: {}", agent.id);
            return Ok(());
        }
        
        // Spawn the Claude CLI process
        let child = spawn_claude_process(agent).await?;
        
        // Store the process
        self.processes.insert(agent.id.clone(), child);
        
        info!("Claude process started for agent: {}", agent.name);
        Ok(())
    }

    /// Stop a Claude process for an agent
    async fn stop_process(&mut self, agent_id: &str) -> Result<()> {
        info!("Stopping Claude process for agent: {}", agent_id);
        
        // Remove the process from the map and kill it
        if let Some(mut child) = self.processes.remove(agent_id) {
            // Try to kill the process gracefully
            match child.kill().await {
                Ok(_) => info!("Claude process killed for agent: {}", agent_id),
                Err(e) => warn!("Failed to kill Claude process for agent {}: {}", agent_id, e),
            }
        } else {
            warn!("No process found for agent: {}", agent_id);
        }
        
        Ok(())
    }

    /// Send a message to a Claude process and get the response
    async fn send_message(&mut self, agent_id: &str, message: &str) -> Result<String> {
        // Get the process for this agent
        if let Some(child) = self.processes.get_mut(agent_id) {
            send_message_to_claude(child, message).await
        } else {
            Err(anyhow!("No process found for agent {}", agent_id))
        }
    }

    /// Check if a process exists for an agent
    async fn has_process(&self, agent_id: &str) -> bool {
        self.processes.contains_key(agent_id)
    }

    /// Get the number of running processes
    async fn process_count(&self) -> usize {
        self.processes.len()
    }
}

/// Spawn a Claude CLI process for an agent
async fn spawn_claude_process(agent: &Agent) -> Result<Child> {
    // Prepare the system prompt
    let system_prompt = &agent.system_prompt;

    // Spawn the Claude CLI process
    let child = Command::new("claude")
        .arg("chat")
        .arg("--system")
        .arg(system_prompt)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

/// Send a message to a Claude CLI process and get the response
async fn send_message_to_claude(child: &mut Child, message: &str) -> Result<String> {
    let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
    let mut stdout = child.stdout.as_mut().ok_or_else(|| anyhow!("Failed to open stdout"))?;

    // Write the message to stdin
    stdin.write_all(message.as_bytes()).await?;
    stdin.write_all(b"\n").await?;
    stdin.flush().await?;

    // Read the response from stdout
    let mut buffer = Vec::new();
    stdout.read_to_end(&mut buffer).await?;

    // Convert the response to a string
    let response = String::from_utf8(buffer)?;

    Ok(response)
}

/// Start a Claude process for an agent
pub async fn start_process(agent: &Agent) -> Result<()> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.start_process(agent).await
}

/// Stop a Claude process for an agent
pub async fn stop_process(agent_id: &str) -> Result<()> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.stop_process(agent_id).await
}

/// Send a message to a Claude process and get the response
pub async fn send_message(agent_id: &str, message: &str) -> Result<String> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.send_message(agent_id, message).await
}

/// Check if a process exists for an agent
pub async fn has_process(agent_id: &str) -> Result<bool> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.has_process(agent_id).await)
}

/// Get the number of running processes
pub async fn process_count() -> Result<usize> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.process_count().await)
}