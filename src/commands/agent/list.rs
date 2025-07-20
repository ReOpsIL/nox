//! Implementation of the agent list command

use crate::core::agent_manager;
use anyhow::Result;
use log::info;

/// Execute the agent list command
pub async fn execute() -> Result<()> {
    info!("Listing all agents");
    
    // Get all agents from the registry
    let agents = agent_manager::get_all_agents().await?;
    
    if agents.is_empty() {
        info!("No agents found");
        return Ok(());
    }
    
    info!("Registered agents:");
    info!("{:<36} {:<20} {:<10}", "ID", "NAME", "STATUS");
    info!("{}", "-".repeat(70));
    
    for agent in agents {
        info!("{:<36} {:<20} {:<10}", agent.id, agent.name, agent.status);
    }
    
    Ok(())
}