//! Implementation of the agent show command

use crate::core::agent_manager;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the agent show command
pub async fn execute(name: String) -> Result<()> {
    info!("Showing agent details: {}", name);
    
    // Get the agent from the registry
    let agent = agent_manager::get_agent_by_name(&name).await?
        .ok_or_else(|| anyhow!("Agent '{}' not found", name))?;
    
    println!("Agent Details:");
    println!("ID:            {}", agent.id);
    println!("Name:          {}", agent.name);
    println!("Status:        {}", agent.status);
    println!("Created:       {}", agent.created_at);
    println!("System Prompt: {}", agent.system_prompt);
    println!("Resource Limits:");
    println!("  Max Memory:  {} MB", agent.resource_limits.max_memory_mb);
    println!("  Max CPU:     {}%", agent.resource_limits.max_cpu_percent);
    
    Ok(())
}