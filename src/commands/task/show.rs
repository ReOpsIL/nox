//! Task show command implementation
//! 
//! This module implements the task show command that displays detailed task information and results.

use anyhow::Result;
use log::{error, info};
use crate::core::{self, task_manager, agent_manager};

/// Show detailed information about a task
pub async fn execute(task_id: String) -> Result<()> {
    // Ensure system is initialized for registry access
    core::ensure_basic_init().await?;
    
    // Get the task
    let task = match task_manager::get_task(&task_id).await? {
        Some(task) => task,
        None => {
            error!("❌ Task not found: {}", task_id);
            return Err(anyhow::anyhow!("Task not found"));
        }
    };

    // Get the agent
    let agent = match agent_manager::get_agent(&task.agent_id).await? {
        Some(agent) => agent,
        None => {
            error!("⚠️  Agent not found for task: {}", task.agent_id);
            return Err(anyhow::anyhow!("Agent not found"));
        }
    };

    // Display task details
    info!("📋 Task Details");
    info!("{}", "=".repeat(60));
    info!("ID:           {}", task.id);
    info!("Title:        {}", task.title);
    info!("Description:  {}", task.description);
    info!("Status:       {:?}", task.status);
    info!("Priority:     {:?}", task.priority);
    info!("Progress:     {}%", task.progress);
    info!("");

    // Display agent information
    info!("🤖 Assigned Agent");
    info!("{}", "=".repeat(60));
    info!("ID:           {}", agent.id);
    info!("Name:         {}", agent.name);
    info!("Status:       {:?}", agent.status);
    info!("");

    // Display timestamps
    info!("⏰ Timeline");
    info!("{}", "=".repeat(60));
    info!("Created:      {}", task.created_at.format("%Y-%m-%d %H:%M:%S UTC"));
    
    if let Some(started_at) = task.started_at {
        info!("Started:      {}", started_at.format("%Y-%m-%d %H:%M:%S UTC"));
    }
    
    if let Some(completed_at) = task.completed_at {
        info!("Completed:    {}", completed_at.format("%Y-%m-%d %H:%M:%S UTC"));
        
        if let Some(started_at) = task.started_at {
            let duration = completed_at.signed_duration_since(started_at);
            info!("Duration:     {} seconds", duration.num_seconds());
        }
    }
    info!("");

    // Display Claude response if available
    if let Some(claude_response) = task.metadata.get("claude_response") {
        info!("🤖 Claude Response");
        info!("{}", "=".repeat(60));
        info!("{}", claude_response);
        info!("");
        
        if let Some(exec_timestamp) = task.metadata.get("execution_timestamp") {
            info!("Response generated: {}", exec_timestamp);
            info!("");
        }
    } else if task.status == crate::types::TaskStatus::Done {
        info!("⚠️  No Claude response found in task metadata");
        info!("   (This task may have been completed before response storage was implemented)");
        info!("");
    }

    // Display metadata if any additional data exists
    let non_system_metadata: std::collections::HashMap<_, _> = task.metadata
        .iter()
        .filter(|(k, _)| !k.starts_with("claude_") && !k.ends_with("_timestamp"))
        .collect();
    
    if !non_system_metadata.is_empty() {
        info!("📝 Additional Metadata");
        info!("{}", "=".repeat(60));
        for (key, value) in non_system_metadata {
            info!("{}: {}", key, value);
        }
        info!("");
    }

    Ok(())
}