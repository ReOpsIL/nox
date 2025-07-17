//! Implementation of the agent stop command

use crate::core::agent_manager;
use crate::types::AgentStatus;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent stop command
pub async fn execute(name: String) -> Result<()> {
    info!("Stopping agent: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Check if the agent is already inactive
    if agent.status == AgentStatus::Inactive {
        info!("Agent '{}' is already inactive", name);
        return Ok(());
    }
    
    // Stop the agent
    info!("Stopping agent '{}'...", name);
    agent_manager::stop_agent(&agent.id).await?;
    
    info!("Agent '{}' stopped successfully", name);
    Ok(())
}