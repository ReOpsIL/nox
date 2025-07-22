//! Task execution command implementation
//! 
//! This module implements the task execution command that runs tasks using Claude CLI.

use anyhow::Result;
use log::{error, info};
use crate::core::{self, task_manager};

/// Execute a task using Claude CLI
pub async fn execute(task_id: String) -> Result<()> {
    // Ensure system is initialized for registry access
    core::ensure_basic_init().await?;
    
    info!("Executing task: {}", task_id);
    
    match task_manager::execute_task(&task_id).await {
        Ok(()) => {
            info!("\n✅ Task execution started successfully!");
            info!("📝 The task is now running in the background.");
            info!("💾 You can check the task status using the list or show commands.");
            info!("🛑 Use 'cancel' command to stop the task if needed.");
        },
        Err(e) => {
            error!("❌ Task execution failed to start: {}", e);
            error!("💡 Please check the task ID and try again.");
            return Err(e);
        }
    }
    
    Ok(())
}