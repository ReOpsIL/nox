//! Task manager module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing tasks.

use crate::core::registry_manager;
use crate::types::Task;
use anyhow::Result;
use log::{error, info};

/// Add a new task to the registry
pub async fn add_task(task: Task) -> Result<()> {
    info!("Adding task: {} for agent {}", task.title, task.agent_id);
    registry_manager::save_task(&task).await
}

/// Get all tasks from the registry
pub async fn get_all_tasks() -> Result<Vec<Task>> {
    registry_manager::get_all_tasks().await
}

/// Get a task by ID
pub async fn get_task(task_id: &str) -> Result<Option<Task>> {
    registry_manager::get_task(task_id).await
}

/// Get tasks for a specific agent
pub async fn get_tasks_by_agent(agent_id: &str) -> Result<Vec<Task>> {
    let all_tasks = registry_manager::get_all_tasks().await?;
    Ok(all_tasks.into_iter().filter(|t| t.agent_id == agent_id).collect())
}

/// Update a task in the registry
pub async fn update_task(task: Task) -> Result<()> {
    info!("Updating task: {}", task.id);
    registry_manager::save_task(&task).await
}

/// Delete a task from the registry
pub async fn delete_task(task_id: &str) -> Result<()> {
    info!("Deleting task: {}", task_id);
    registry_manager::delete_task(task_id).await
}

/// Delete all tasks for a specific agent
pub async fn delete_all_tasks_for_agent(agent_id: &str) -> Result<()> {
    info!("Deleting all tasks for agent: {}", agent_id);
    
    // Get all tasks for the agent
    let agent_tasks = get_tasks_by_agent(agent_id).await?;
    
    // Force stop any in-progress tasks and delete them
    for task in agent_tasks {
        if task.status == crate::types::TaskStatus::InProgress {
            info!("Force stopping task '{}' ({})", task.title, task.id);
            // Note: We could add logic here to gracefully stop running tasks
            // For now, we'll just mark them as cancelled before deletion
            match cancel_task(&task.id).await {
                Ok(_) => info!("Task '{}' cancelled before deletion", task.title),
                Err(e) => error!("Failed to cancel task '{}': {}", task.title, e),
            }
        }
        
        info!("Deleting task '{}' ({})", task.title, task.id);
        if let Err(e) = delete_task(&task.id).await {
            error!("Failed to delete task '{}': {}", task.title, e);
        }
    }
    
    info!("Finished deleting tasks for agent: {}", agent_id);
    Ok(())
}

/// Start a task (change status to InProgress)
pub async fn start_task(task_id: &str) -> Result<()> {
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    let mut task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    task.status = TaskStatus::InProgress;
    task.started_at = Some(Utc::now());
    
    info!("Starting task: {}", task.title);
    registry_manager::save_task(&task).await
}

/// Complete a task (change status to Done)
pub async fn complete_task(task_id: &str) -> Result<()> {
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    let mut task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    task.status = TaskStatus::Done;
    task.completed_at = Some(Utc::now());
    task.progress = 100;
    
    info!("Completing task: {}", task.title);
    registry_manager::save_task(&task).await
}

/// Cancel a task (change status to Cancelled)
pub async fn cancel_task(task_id: &str) -> Result<()> {
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    let mut task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    task.status = TaskStatus::Cancelled;
    task.completed_at = Some(Utc::now());
    
    info!("Cancelling task: {}", task.title);
    registry_manager::save_task(&task).await
}

/// Execute a task using Claude CLI
pub async fn execute_task(task_id: &str) -> Result<String> {
    use crate::core::claude_process_manager;
    use crate::core::agent_manager;
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    // Get the task first to check it exists and get agent info
    let task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    // Get the agent
    let agent = agent_manager::get_agent(&task.agent_id).await?
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", task.agent_id))?;
    
    info!("Executing task '{}' with agent '{}'", task.title, agent.name);
    
    // Atomically update task status to InProgress
    let mut task = registry_manager::update_task_for_execution(task_id, |task| {
        task.status = TaskStatus::InProgress;
        task.started_at = Some(Utc::now());
        Ok(())
    }).await?;
    
    // Create the message for Claude
    let message = format!("Task: {}\n\nDescription: {}", task.title, task.description);
    
    // Execute the task using Claude CLI
    match claude_process_manager::send_message(&task.agent_id, &message).await {
        Ok(response) => {
            info!("Task '{}' completed successfully", task.title);
            
            // Atomically update task as completed with response
            registry_manager::update_task_for_execution(task_id, |task| {
                task.metadata.insert("claude_response".to_string(), response.clone());
                task.metadata.insert("execution_timestamp".to_string(), Utc::now().to_rfc3339());
                task.status = TaskStatus::Done;
                task.completed_at = Some(Utc::now());
                task.progress = 100;
                Ok(())
            }).await?;
            
            Ok(response)
        },
        Err(e) => {
            error!("Task '{}' failed: {}", task.title, e);
            
            // Keep task as InProgress so user can retry
            // No need to update since it's already InProgress
            
            Err(e)
        }
    }
}