//! Implementation of the agent start command

use crate::core::{self, agent_manager};
use crate::types::AgentStatus;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent start command
pub async fn execute(name: String) -> Result<()> {
    // Ensure system is initialized for registry access
    core::ensure_basic_init().await?;
    
    info!("Starting agent: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Check if the agent is already active
    if agent.status == AgentStatus::Active {
        info!("Agent '{}' is already active", name);
        return Ok(());
    }
    
    // Start the agent
    info!("Starting agent '{}'...", name);
    agent_manager::start_agent(&agent.id).await?;
    
    info!("Agent '{}' started successfully", name);
    Ok(())
}