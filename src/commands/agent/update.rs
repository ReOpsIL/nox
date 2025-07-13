//! Implementation of the agent update command

use crate::core::agent_manager;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent update command
pub async fn execute(name: String, prompt: String) -> Result<()> {
    info!("Updating agent: {}", name);
    
    // Get the agent from the registry
    let mut agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    // Update the system prompt
    agent.system_prompt = prompt;
    
    // Save the updated agent
    agent_manager::update_agent(agent).await?;
    
    println!("Agent '{}' updated successfully", name);
    Ok(())
}