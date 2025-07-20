//! Task manager module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing tasks.

use crate::core::registry_manager;
use crate::types::Task;
use anyhow::Result;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use std::path::PathBuf;

/// Simple task execution data
pub struct TaskExecution {
    /// Channel for sending input to the Claude process
    pub input_tx: mpsc::Sender<String>,
    /// Channel for receiving output from the Claude process  
    pub output_rx: mpsc::Receiver<String>,
    /// Path to the task's log file
    pub log_file_path: PathBuf,
    /// Task handle for cancellation
    pub task_handle: JoinHandle<Result<String>>,
    /// Cancellation flag
    pub cancellation_flag: Arc<AtomicBool>,
}

// Simple storage for running tasks - no complex singleton
lazy_static! {
    static ref RUNNING_TASKS: Arc<Mutex<HashMap<String, TaskExecution>>> = 
        Arc::new(Mutex::new(HashMap::new()));
}

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
        // Stop running execution if it exists
        {
            let mut running_tasks = RUNNING_TASKS.lock().await;
            if let Some(task_execution) = running_tasks.remove(&task.id) {
                info!("Force stopping running execution for task '{}' ({})", task.title, task.id);
                task_execution.cancellation_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                task_execution.task_handle.abort();
                info!("Task '{}' execution stopped", task.title);
            }
        }
        
        if task.status == crate::types::TaskStatus::InProgress {
            info!("Force stopping task '{}' ({})", task.title, task.id);
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
    
    // First, stop the running execution if it exists
    {
        let mut running_tasks = RUNNING_TASKS.lock().await;
        if let Some(task_execution) = running_tasks.remove(task_id) {
            info!("Stopping running execution for task: {}", task_id);
            task_execution.cancellation_flag.store(true, std::sync::atomic::Ordering::Relaxed);
            task_execution.task_handle.abort();
        }
    }
    
    let mut task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    task.status = TaskStatus::Cancelled;
    task.completed_at = Some(Utc::now());
    
    info!("Cancelling task: {}", task.title);
    registry_manager::save_task(&task).await
}

/// Execute a task using Claude CLI with streaming (non-blocking)
pub async fn execute_task(task_id: &str) -> Result<()> {
    use crate::core::agent_manager;
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    // Get the task first to check it exists and get agent info
    let task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    // Get the agent
    let agent = agent_manager::get_agent(&task.agent_id).await?
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", task.agent_id))?;
    
    info!("Starting streaming execution of task '{}' with agent '{}'", task.title, agent.name);
    
    // Check if task is already running
    {
        let running_tasks = RUNNING_TASKS.lock().await;
        if running_tasks.contains_key(task_id) {
            return Err(anyhow::anyhow!("Task {} is already running", task_id));
        }
    }
    
    // Update task status to InProgress
    registry_manager::update_task_for_execution(task_id, |task| {
        task.status = TaskStatus::InProgress;
        task.started_at = Some(Utc::now());
        Ok(())
    }).await?;
    
    // Create channels for streaming communication
    let (input_tx, input_rx) = mpsc::channel::<String>(100);
    let (output_tx, output_rx) = mpsc::channel::<String>(1000);
    
    // Create log file path
    let log_dir = std::path::Path::new("./logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(log_dir)?;
    }
    let log_file_path = log_dir.join(format!("task_{}.md", task_id));
    
    // Create cancellation flag
    let cancellation_flag = Arc::new(AtomicBool::new(false));
    let cancellation_flag_clone = Arc::clone(&cancellation_flag);
    
    // Clone data for the async task
    let task_id_clone = task_id.to_string();
    let agent_clone = agent.clone();
    let log_file_path_clone = log_file_path.clone();
    
    // Create the initial message for Claude
    let initial_message = format!("Task: {}\n\nDescription: {}", task.title, task.description);
    
    // Spawn the task execution using JSON transport
    let task_handle = tokio::spawn(async move {
        crate::core::claude_json_transport::execute_claude_cli_json(
            &agent_clone,
            &initial_message,
            cancellation_flag_clone,
            input_rx,
            output_tx,
            log_file_path_clone,
            task_id_clone,
        ).await
    });
    
    // Store task execution data
    let task_execution = TaskExecution {
        input_tx,
        output_rx,
        log_file_path,
        task_handle,
        cancellation_flag,
    };
    
    {
        let mut running_tasks = RUNNING_TASKS.lock().await;
        running_tasks.insert(task_id.to_string(), task_execution);
    }
    
    info!("Task '{}' execution started in background", task.title);
    Ok(())
}

// Old PTY-based function removed - now using JSON transport in claude_json_transport.rs

/// Poll for new output from a running task (non-blocking)
pub async fn poll_task_output(task_id: &str) -> Option<Vec<String>> {
    let mut running_tasks = RUNNING_TASKS.lock().await;
    if let Some(task_execution) = running_tasks.get_mut(task_id) {
        let mut output_lines = Vec::new();

        // Try to receive all available output without blocking
        while let Ok(line) = task_execution.output_rx.try_recv() {
            output_lines.push(line);
        }

        if !output_lines.is_empty() {
            Some(output_lines)
        } else {
            None
        }
    } else {
        None
    }
}