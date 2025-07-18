//! Claude process manager module for the Nox agent ecosystem
//! 
//! This module handles the spawning and management of Claude CLI processes.

use crate::types::Agent;
use anyhow::{anyhow, Result};
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::Mutex;

// Singleton instance of the Claude process manager
lazy_static::lazy_static! {
    static ref CLAUDE_PROCESS_MANAGER: Arc<Mutex<ClaudeProcessManager>> = Arc::new(Mutex::new(ClaudeProcessManager::new()));
}

/// Claude process manager struct
pub struct ClaudeProcessManager {
    active_agents: HashMap<String, Agent>,
}

impl ClaudeProcessManager {
    /// Create a new Claude process manager
    fn new() -> Self {
        Self {
            active_agents: HashMap::new(),
        }
    }

    /// Start tracking an agent (no persistent process needed with new CLI approach)
    async fn start_process(&mut self, agent: &Agent) -> Result<()> {
        info!("Registering agent for Claude CLI execution: {}", agent.name);
        
        // Store agent for future message execution
        self.active_agents.insert(agent.id.clone(), agent.clone());
        
        info!("Agent registered successfully: {}", agent.name);
        Ok(())
    }

    /// Stop tracking an agent
    async fn stop_process(&mut self, agent_id: &str) -> Result<()> {
        info!("Unregistering agent: {}", agent_id);

        if self.active_agents.remove(agent_id).is_some() {
            info!("Agent unregistered successfully: {}", agent_id);
        } else {
            warn!("No active agent found: {}", agent_id);
        }

        Ok(())
    }

    /// Send a message to Claude via CLI and get the response
    async fn send_message(&mut self, agent_id: &str, message: &str) -> Result<String> {
        // If agent is not in our active list, try to load it from the registry
        if !self.active_agents.contains_key(agent_id) {
            // Try to load the agent from the registry
            use crate::core::agent_manager;
            match agent_manager::get_agent(agent_id).await {
                Ok(Some(agent)) => {
                    info!("Auto-registering agent for Claude CLI execution: {}", agent.name);
                    self.active_agents.insert(agent_id.to_string(), agent);
                },
                Ok(None) => return Err(anyhow!("Agent not found in registry: {}", agent_id)),
                Err(e) => return Err(anyhow!("Failed to load agent from registry: {}", e)),
            }
        }
        
        if let Some(agent) = self.active_agents.get(agent_id) {
            execute_claude_command(agent, message).await
        } else {
            Err(anyhow!("No active agent found: {}", agent_id))
        }
    }

    /// Check if an agent is active
    #[allow(dead_code)]
    async fn has_process(&self, agent_id: &str) -> bool {
        self.active_agents.contains_key(agent_id)
    }

    /// Get the number of active agents
    #[allow(dead_code)]
    async fn process_count(&self) -> usize {
        self.active_agents.len()
    }
}

/// Execute a Claude CLI command for an agent
async fn execute_claude_command(agent: &Agent, user_message: &str) -> Result<String> {
    let system_prompt = &agent.system_prompt;
    
    info!("Executing Claude command for agent: {}", agent.name);
    debug!("User message: {}", 
           if user_message.len() > 200 { &user_message[..200] } else { user_message });

    // Combine system prompt with user message
    let full_message = format!("{}\n\nUser: {}", system_prompt, user_message);

    // Build the command with proper Claude CLI syntax
    let mut cmd = Command::new("-claude-");
    
    cmd.arg("--print")  // Non-interactive mode
       .arg("--dangerously-skip-permissions")
       //.arg("--model").arg("claude-sonnet-4-20250514")  // Current model
       .arg("--output-format").arg("text")  // Text output
       .arg(&full_message)  // The prompt as argument
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    info!("Executing Claude CLI command for agent: {}", agent.name);

    // Execute the command
    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout).to_string();
                info!("Claude command completed successfully for agent: {}", agent.name);
                debug!("Response length: {} characters", response.len());
                Ok(response.trim().to_string())
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                error!("Claude command failed for agent {}: {}", agent.name, error_msg);
                Err(anyhow!("Claude command failed: {}", error_msg))
            }
        },
        Err(e) => {
            error!("Failed to execute Claude command for agent {}: {}", agent.name, e);
            Err(anyhow!("Failed to execute Claude command: {}", e))
        }
    }
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
#[allow(dead_code)]
pub async fn send_message(agent_id: &str, message: &str) -> Result<String> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.send_message(agent_id, message).await
}

/// Check if a process exists for an agent
#[allow(dead_code)]
pub async fn has_process(agent_id: &str) -> Result<bool> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.has_process(agent_id).await)
}

/// Get the number of running processes
#[allow(dead_code)]
pub async fn process_count() -> Result<usize> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.process_count().await)
}
