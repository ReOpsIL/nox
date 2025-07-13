//! Task execution command implementation
//! 
//! This module implements the task execution command that runs tasks using Claude CLI.

use anyhow::Result;
use crate::core::task_manager;

/// Execute a task using Claude CLI
pub async fn execute(task_id: String) -> Result<()> {
    println!("Executing task 2: {}", task_id);
    
    match task_manager::execute_task(&task_id).await {
        Ok(response) => {
            println!("\n✅ Task completed successfully!");
            println!("\n📝 Claude's Response:");
            println!("{}", "=".repeat(60));
            println!("{}", response);
            println!("{}", "=".repeat(60));
            println!("\n💾 Task has been marked as completed in the registry.");
        },
        Err(e) => {
            eprintln!("❌ Task execution failed: {}", e);
            eprintln!("💡 The task remains in progress. You can retry with the same command.");
            return Err(e);
        }
    }
    
    Ok(())
}