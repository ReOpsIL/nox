//! Task execution command implementation
//! 
//! This module implements the task execution command that runs tasks using Claude CLI.

use anyhow::Result;
use log::{error, info};
use crate::core::task_manager;

/// Execute a task using Claude CLI
pub async fn execute(task_id: String) -> Result<()> {
    info!("Executing task 2: {}", task_id);
    
    match task_manager::execute_task(&task_id).await {
        Ok(response) => {
            info!("\n✅ Task completed successfully!");
            info!("\n📝 Claude's Response:");
            info!("{}", "=".repeat(60));
            info!("{}", response);
            info!("{}", "=".repeat(60));
            info!("\n💾 Task has been marked as completed in the registry.");
        },
        Err(e) => {
            error!("❌ Task execution failed: {}", e);
            error!("💡 The task remains in progress. You can retry with the same command.");
            return Err(e);
        }
    }
    
    Ok(())
}