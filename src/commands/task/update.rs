//! Implementation of the task update command

use crate::core::task_manager;
use crate::types::TaskStatus;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the task update command
pub async fn execute(task_id: String, status: String) -> Result<()> {
    info!("Updating task status: {}", task_id);
    
    // Parse the status string
    let task_status = match status.to_lowercase().as_str() {
        "todo" => TaskStatus::Todo,
        "in_progress" | "inprogress" | "in-progress" => TaskStatus::InProgress,
        "done" | "completed" => TaskStatus::Done,
        "cancelled" | "canceled" => TaskStatus::Cancelled,
        _ => return Err(anyhow!("Invalid status: {}. Valid values are: todo, in_progress, done, cancelled", status)),
    };
    
    // Get the task from the registry
    let mut task = task_manager::get_task(&task_id).await?
        .ok_or_else(|| anyhow!("Task with ID '{}' not found", task_id))?;
    
    // Update the task status
    let old_status = task.status.clone();
    task.status = task_status.clone();
    
    // Update timestamps based on status change
    match task_status {
        TaskStatus::InProgress if old_status == TaskStatus::Todo => {
            // Task is being started
            task.started_at = Some(chrono::Utc::now());
            task.progress = 1; // Set initial progress
        },
        TaskStatus::Done => {
            // Task is being completed
            task.completed_at = Some(chrono::Utc::now());
            task.progress = 100; // Set to 100% complete
        },
        _ => {}
    }
    
    // Save the updated task
    task_manager::update_task(task).await?;
    
    info!("Task '{}' status updated from {} to {}", task_id, old_status, task_status);
    Ok(())
}