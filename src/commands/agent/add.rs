//! Implementation of the agent add command

use crate::core::{self, agent_manager};
use crate::types::Agent;
use anyhow::Result;
use log::info;

/// Execute the agent add command
pub async fn execute(name: String, prompt: String) -> Result<()> {
    // Ensure system is initialized for registry access
    core::ensure_basic_init().await?;
    
    info!("Creating new agent: {}", name);
    
    // Create a new agent
    let agent = Agent::new(name.clone(), prompt);
    
    // Add the agent to the registry
    agent_manager::add_agent(agent).await?;
    
    info!("Agent '{}' created successfully", name);
    Ok(())
}