//! Implementation of the agent add command

use anyhow::Result;
use log::info;
use crate::types::Agent;
use crate::core::agent_manager;

/// Execute the agent add command
pub async fn execute(name: String, prompt: String) -> Result<()> {
    info!("Creating new agent: {}", name);
    
    // Create a new agent
    let agent = Agent::new(name.clone(), prompt);
    
    // Add the agent to the registry
    agent_manager::add_agent(agent).await?;
    
    println!("Agent '{}' created successfully", name);
    Ok(())
}