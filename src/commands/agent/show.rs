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

    info!("Agent Details:");
    info!("ID:            {}", agent.id);
    info!("Name:          {}", agent.name);
    info!("Status:        {}", agent.status);
    info!("Created:       {}", agent.created_at);
    info!("System Prompt: {}", agent.system_prompt);
    info!("Resource Limits:");
    info!("  Max Memory:  {} MB", agent.resource_limits.max_memory_mb);
    info!("  Max CPU:     {}%", agent.resource_limits.max_cpu_percent);
    
    Ok(())
}