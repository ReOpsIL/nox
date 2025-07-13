//! Implementation of the agent start command

use crate::core::agent_manager;
use crate::types::AgentStatus;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent start command
pub async fn execute(name: String) -> Result<()> {
    info!("Starting agent: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Check if the agent is already active
    if agent.status == AgentStatus::Active {
        println!("Agent '{}' is already active", name);
        return Ok(());
    }
    
    // Start the agent
    println!("Starting agent '{}'...", name);
    agent_manager::start_agent(&agent.id).await?;
    
    println!("Agent '{}' started successfully", name);
    Ok(())
}