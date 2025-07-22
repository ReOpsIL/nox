//! Implementation of the agent delete command

use crate::core::{self, agent_manager};
use crate::types::AgentStatus;
use anyhow::{anyhow, Result};
use log::info;
use std::io::{self, Write};

/// Execute the agent delete command
pub async fn execute(name: String, force: bool) -> Result<()> {
    // Ensure system is initialized for registry access
    core::ensure_basic_init().await?;
    
    info!("Deleting agent: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Check if the agent is active
    if agent.status == AgentStatus::Active {
        info!("Warning: Agent '{}' is currently active", name);
        info!("It will be stopped before deletion");
    }
    
    // Confirm deletion if not forced
    if !force {
        print!("Are you sure you want to delete agent '{}'? [y/N] ", name);
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if !input.trim().eq_ignore_ascii_case("y") {
            info!("Deletion cancelled");
            return Ok(());
        }
    }
    
    // Stop the agent if it's active
    if agent.status == AgentStatus::Active {
        info!("Stopping agent '{}'...", name);
        agent_manager::stop_agent(&agent.id).await?;
    }
    
    // Delete the agent
    agent_manager::delete_agent(&agent.id).await?;
    
    info!("Agent '{}' deleted successfully", name);
    Ok(())
}