//! Implementation of the task create command

use anyhow::{Result, anyhow};
use log::info;
use crate::core::{agent_manager, task_manager};
use crate::types::Task;

/// Execute the task create command
pub async fn execute(agent_id: String, title: String, desc: String) -> Result<()> {
    info!("Creating new task for agent: {}", agent_id);
    
    // Check if the agent exists
    let agent = agent_manager::get_agent(&agent_id).await?
        .ok_or_else(|| anyhow!("Agent with ID '{}' not found", agent_id))?;
    
    // Create a new task
    let task = Task::new(agent_id.clone(), title.clone(), desc);
    
    // Add the task to the registry
    task_manager::add_task(task).await?;
    
    println!("Task '{}' created successfully for agent '{}'", title, agent.name);
    Ok(())
}