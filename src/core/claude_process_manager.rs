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
