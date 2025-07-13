//! Implementation of the agent list command

use anyhow::Result;
use log::info;
use crate::core::agent_manager;

/// Execute the agent list command
pub async fn execute() -> Result<()> {
    info!("Listing all agents");
    
    // Get all agents from the registry
    let agents = agent_manager::get_all_agents().await?;
    
    if agents.is_empty() {
        println!("No agents found");
        return Ok(());
    }
    
    println!("Registered agents:");
    println!("{:<36} {:<20} {:<10}", "ID", "NAME", "STATUS");
    println!("{}", "-".repeat(70));
    
    for agent in agents {
        println!("{:<36} {:<20} {:<10}", agent.id, agent.name, agent.status);
    }
    
    Ok(())
}