//! Implementation of the task overview command

use anyhow::Result;
use log::info;
use std::collections::HashMap;
use crate::core::{agent_manager, task_manager};
use crate::types::TaskStatus;

/// Execute the task overview command
pub async fn execute() -> Result<()> {
    info!("Showing task overview");

    // Get all agents
    let agents = agent_manager::get_all_agents().await?;

    // Get all tasks
    let tasks = task_manager::get_all_tasks().await?;

    if tasks.is_empty() {
        println!("No tasks found in the system");
        return Ok(());
    }

    // Create a map of agent IDs to names for easier lookup
    let agent_names: HashMap<String, String> = agents
        .into_iter()
        .map(|agent| (agent.id, agent.name))
        .collect();

    // Group tasks by agent
    let mut tasks_by_agent: HashMap<String, Vec<&crate::types::Task>> = HashMap::new();
    for task in &tasks {
        tasks_by_agent
            .entry(task.agent_id.clone())
            .or_default()
            .push(task);
    }

    // Count tasks by status
    let mut total_todo = 0;
    let mut total_in_progress = 0;
    let mut total_done = 0;
    let mut total_cancelled = 0;

    println!("Task Overview by Agent:");
    println!("{:<20} {:<10} {:<15} {:<10} {:<10}", "AGENT", "TODO", "IN PROGRESS", "DONE", "CANCELLED");
    println!("{}", "-".repeat(70));

    for (agent_id, agent_tasks) in &tasks_by_agent {
        let fallback_name = agent_id.clone();
        let agent_name = agent_names.get(agent_id).unwrap_or(&fallback_name);

        // Count tasks by status for this agent
        let todo = agent_tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
        let in_progress = agent_tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        let done = agent_tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        let cancelled = agent_tasks.iter().filter(|t| t.status == TaskStatus::Cancelled).count();

        // Update totals
        total_todo += todo;
        total_in_progress += in_progress;
        total_done += done;
        total_cancelled += cancelled;

        println!("{:<20} {:<10} {:<15} {:<10} {:<10}", 
            truncate(agent_name, 17),
            todo,
            in_progress,
            done,
            cancelled
        );
    }

    println!("{}", "-".repeat(70));
    println!("{:<20} {:<10} {:<15} {:<10} {:<10}", 
        "TOTAL",
        total_todo,
        total_in_progress,
        total_done,
        total_cancelled
    );

    // Print overall statistics
    let total_tasks = tasks.len();
    let completion_rate = if total_tasks > 0 {
        (total_done as f32 / total_tasks as f32) * 100.0
    } else {
        0.0
    };

    println!("\nOverall Statistics:");
    println!("Total Tasks:     {}", total_tasks);
    println!("Completion Rate: {:.1}%", completion_rate);

    Ok(())
}

/// Helper function to truncate a string to a maximum length
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[0..max_len-3])
    }
}
