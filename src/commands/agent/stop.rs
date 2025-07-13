//! Implementation of the agent stop command

use anyhow::{Result, anyhow};
use log::info;
use crate::core::agent_manager;
use crate::types::AgentStatus;

/// Execute the agent stop command
pub async fn execute(name: String) -> Result<()> {
    info!("Stopping agent: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Check if the agent is already inactive
    if agent.status == AgentStatus::Inactive {
        println!("Agent '{}' is already inactive", name);
        return Ok(());
    }
    
    // Stop the agent
    println!("Stopping agent '{}'...", name);
    agent_manager::stop_agent(&agent.id).await?;
    
    println!("Agent '{}' stopped successfully", name);
    Ok(())
}