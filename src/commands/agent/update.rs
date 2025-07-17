//! Implementation of the agent update command

use crate::core::agent_manager;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent update command
pub async fn execute(id: String, name: String, system_prompt: String) -> Result<()> {
    info!("Updating agent: {} (id: {})", name, id);
    
    // Get the agent from the registry by ID
    let mut agent = agent_manager::get_agent(&id).await?
        .ok_or_else(|| anyhow!("Agent with id '{}' not found", id))?;
    
    // Update the agent fields
    agent.name = name.clone();
    agent.system_prompt = system_prompt;
    
    // Save the updated agent
    agent_manager::update_agent(agent).await?;
    
    println!("Agent '{}' updated successfully", name);
    Ok(())
}
