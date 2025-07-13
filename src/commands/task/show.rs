//! Task show command implementation
//! 
//! This module implements the task show command that displays detailed task information and results.

use anyhow::Result;
use crate::core::task_manager;
use crate::core::agent_manager;

/// Show detailed information about a task
pub async fn execute(task_id: String) -> Result<()> {
    // Get the task
    let task = match task_manager::get_task(&task_id).await? {
        Some(task) => task,
        None => {
            eprintln!("❌ Task not found: {}", task_id);
            return Err(anyhow::anyhow!("Task not found"));
        }
    };

    // Get the agent
    let agent = match agent_manager::get_agent(&task.agent_id).await? {
        Some(agent) => agent,
        None => {
            eprintln!("⚠️  Agent not found for task: {}", task.agent_id);
            return Err(anyhow::anyhow!("Agent not found"));
        }
    };

    // Display task details
    println!("📋 Task Details");
    println!("{}", "=".repeat(60));
    println!("ID:           {}", task.id);
    println!("Title:        {}", task.title);
    println!("Description:  {}", task.description);
    println!("Status:       {:?}", task.status);
    println!("Priority:     {:?}", task.priority);
    println!("Progress:     {}%", task.progress);
    println!();

    // Display agent information
    println!("🤖 Assigned Agent");
    println!("{}", "=".repeat(60));
    println!("ID:           {}", agent.id);
    println!("Name:         {}", agent.name);
    println!("Status:       {:?}", agent.status);
    println!();

    // Display timestamps
    println!("⏰ Timeline");
    println!("{}", "=".repeat(60));
    println!("Created:      {}", task.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    if let Some(started_at) = task.started_at {
        println!("Started:      {}", started_at.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    if let Some(completed_at) = task.completed_at {
        println!("Completed:    {}", completed_at.format("%Y-%m-%d %H:%M:%S UTC"));
        
        if let Some(started_at) = task.started_at {
            let duration = completed_at.signed_duration_since(started_at);
            println!("Duration:     {} seconds", duration.num_seconds());
        }
    }
    println!();

    // Display Claude response if available
    if let Some(claude_response) = task.metadata.get("claude_response") {
        println!("🤖 Claude Response");
        println!("{}", "=".repeat(60));
        println!("{}", claude_response);
        println!();
        
        if let Some(exec_timestamp) = task.metadata.get("execution_timestamp") {
            println!("Response generated: {}", exec_timestamp);
            println!();
        }
    } else if task.status == crate::types::TaskStatus::Done {
        println!("⚠️  No Claude response found in task metadata");
        println!("   (This task may have been completed before response storage was implemented)");
        println!();
    }

    // Display metadata if any additional data exists
    let non_system_metadata: std::collections::HashMap<_, _> = task.metadata
        .iter()
        .filter(|(k, _)| !k.starts_with("claude_") && !k.ends_with("_timestamp"))
        .collect();
    
    if !non_system_metadata.is_empty() {
        println!("📝 Additional Metadata");
        println!("{}", "=".repeat(60));
        for (key, value) in non_system_metadata {
            println!("{}: {}", key, value);
        }
        println!();
    }

    Ok(())
}