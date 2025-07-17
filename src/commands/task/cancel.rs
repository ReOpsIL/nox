//! Implementation of the task cancel command

use crate::core::task_manager;
use crate::types::TaskStatus;
use anyhow::{anyhow, Result};
use log::info;

/// Execute the task cancel command
pub async fn execute(task_id: String) -> Result<()> {
    info!("Cancelling task: {}", task_id);
    
    // Get the task from the registry
    let mut task = task_manager::get_task(&task_id).await?
        .ok_or_else(|| anyhow!("Task with ID '{}' not found", task_id))?;
    
    // Check if the task is already done or cancelled
    match task.status {
        TaskStatus::Done => {
            return Err(anyhow!("Cannot cancel a completed task"));
        },
        TaskStatus::Cancelled => {
            info!("Task '{}' is already cancelled", task_id);
            return Ok(());
        },
        _ => {}
    }
    
    // Update the task status
    let old_status = task.status.clone();
    task.status = TaskStatus::Cancelled;
    
    // Save the updated task
    task_manager::update_task(task).await?;
    
    info!("Task '{}' cancelled successfully (was: {})", task_id, old_status);
    Ok(())
}