//! Implementation of the task list command

use anyhow::Result;
use log::info;
use crate::core::{agent_manager, task_manager};

/// Execute the task list command
pub async fn execute(agent_id: Option<String>) -> Result<()> {
    if let Some(id) = &agent_id {
        info!("Listing tasks for agent: {}", id);
        
        // Check if the agent exists
        let agent = agent_manager::get_agent(id).await?;
        if agent.is_none() {
            println!("Agent with ID '{}' not found", id);
            return Ok(());
        }
        
        // Get tasks for the specific agent
        let tasks = task_manager::get_tasks_by_agent(id).await?;
        
        if tasks.is_empty() {
            println!("No tasks found for agent '{}'", id);
            return Ok(());
        }
        
        println!("Tasks for agent '{}':", agent.unwrap().name);
        print_tasks(tasks);
    } else {
        info!("Listing all tasks");
        
        // Get all tasks
        let tasks = task_manager::get_all_tasks().await?;
        
        if tasks.is_empty() {
            println!("No tasks found");
            return Ok(());
        }
        
        println!("All tasks:");
        print_tasks(tasks);
    }
    
    Ok(())
}

/// Helper function to print tasks in a formatted table
fn print_tasks(tasks: Vec<crate::types::Task>) {
    println!("{:<36} {:<30} {:<15} {:<10}", "ID", "TITLE", "STATUS", "PROGRESS");
    println!("{}", "-".repeat(95));
    
    for task in tasks {
        println!("{:<36} {:<30} {:<15} {:<10}%", 
            task.id, 
            truncate(&task.title, 27), 
            task.status,
            task.progress
        );
    }
}

/// Helper function to truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len-3])
    }
}