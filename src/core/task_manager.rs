//! Task manager module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing tasks.

use crate::core::registry_manager;
use crate::types::Task;
use anyhow::Result;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::process::Child;
use std::process::Stdio;
use tokio::process::Command;
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};

/// Task execution context that tracks a running task process
#[derive(Debug)]
pub struct TaskExecutionContext {
    /// The task ID being executed
    pub task_id: String,
    /// The agent ID executing the task
    pub agent_id: String,
    /// The spawned process handle
    pub process_handle: Option<Child>,
    /// The async task handle
    pub task_handle: JoinHandle<Result<String>>,
    /// Cancellation flag
    pub cancellation_flag: Arc<AtomicBool>,
}

/// Task execution manager that tracks running tasks
pub struct TaskExecutionManager {
    /// Map of task IDs to their execution contexts
    running_tasks: HashMap<String, TaskExecutionContext>,
}

impl TaskExecutionManager {
    /// Create a new task execution manager
    pub fn new() -> Self {
        Self {
            running_tasks: HashMap::new(),
        }
    }

    /// Start executing a task in a non-blocking way
    pub async fn start_task_execution(&mut self, task_id: String, agent_id: String) -> Result<()> {
        // Check if task is already running
        if self.running_tasks.contains_key(&task_id) {
            return Err(anyhow::anyhow!("Task {} is already running", task_id));
        }

        // Create cancellation flag
        let cancellation_flag = Arc::new(AtomicBool::new(false));
        let cancellation_flag_clone = Arc::clone(&cancellation_flag);
        
        // Clone task_id for the async task
        let task_id_clone = task_id.clone();
        let agent_id_clone = agent_id.clone();
        
        // Spawn the task execution
        let task_handle = tokio::spawn(async move {
            execute_task_operation(task_id_clone, agent_id_clone, cancellation_flag_clone).await
        });

        // Store the execution context
        let context = TaskExecutionContext {
            task_id: task_id.clone(),
            agent_id,
            process_handle: None, // Will be set during execution
            task_handle,
            cancellation_flag,
        };

        self.running_tasks.insert(task_id, context);
        Ok(())
    }

    /// Stop a running task execution
    pub async fn stop_task_execution(&mut self, task_id: &str) -> Result<()> {
        if let Some(mut context) = self.running_tasks.remove(task_id) {
            info!("Stopping task execution: {}", task_id);
            
            // Set the cancellation flag
            context.cancellation_flag.store(true, Ordering::Relaxed);
            
            // Kill the process if it exists
            if let Some(mut child) = context.process_handle.take() {
                match child.kill().await {
                    Ok(()) => info!("Successfully killed process for task {}", task_id),
                    Err(e) => warn!("Failed to kill process for task {}: {}", task_id, e),
                }
            }
            
            // Abort the async task
            context.task_handle.abort();
            
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task {} is not running", task_id))
        }
    }

    /// Check if a task is currently running
    pub fn is_task_running(&self, task_id: &str) -> bool {
        self.running_tasks.contains_key(task_id)
    }

    /// Get all running task IDs
    pub fn get_running_task_ids(&self) -> Vec<String> {
        self.running_tasks.keys().cloned().collect()
    }

    /// Clean up finished tasks
    pub async fn cleanup_finished_tasks(&mut self) {
        let mut finished_tasks = Vec::new();
        
        for (task_id, context) in &self.running_tasks {
            if context.task_handle.is_finished() {
                finished_tasks.push(task_id.clone());
            }
        }
        
        for task_id in finished_tasks {
            self.running_tasks.remove(&task_id);
            info!("Cleaned up finished task: {}", task_id);
        }
    }
}

// Global task execution manager
lazy_static! {
    static ref TASK_EXECUTION_MANAGER: Arc<Mutex<TaskExecutionManager>> = 
        Arc::new(Mutex::new(TaskExecutionManager::new()));
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
            let mut manager = TASK_EXECUTION_MANAGER.lock().await;
            if manager.is_task_running(&task.id) {
                info!("Force stopping running execution for task '{}' ({})", task.title, task.id);
                match manager.stop_task_execution(&task.id).await {
                    Ok(_) => info!("Task '{}' execution stopped", task.title),
                    Err(e) => error!("Failed to stop execution for task '{}': {}", task.title, e),
                }
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
        let mut manager = TASK_EXECUTION_MANAGER.lock().await;
        if manager.is_task_running(task_id) {
            info!("Stopping running execution for task: {}", task_id);
            manager.stop_task_execution(task_id).await?;
        }
    }
    
    let mut task = registry_manager::get_task(task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    task.status = TaskStatus::Cancelled;
    task.completed_at = Some(Utc::now());
    
    info!("Cancelling task: {}", task.title);
    registry_manager::save_task(&task).await
}

/// Execute a task using Claude CLI (non-blocking)
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
    
    info!("Starting non-blocking execution of task '{}' with agent '{}'", task.title, agent.name);
    
    // Check if task is already running
    {
        let manager = TASK_EXECUTION_MANAGER.lock().await;
        if manager.is_task_running(task_id) {
            return Err(anyhow::anyhow!("Task {} is already running", task_id));
        }
    }
    
    // Update task status to InProgress
    registry_manager::update_task_for_execution(task_id, |task| {
        task.status = TaskStatus::InProgress;
        task.started_at = Some(Utc::now());
        Ok(())
    }).await?;
    
    // Start the non-blocking execution
    {
        let mut manager = TASK_EXECUTION_MANAGER.lock().await;
        manager.start_task_execution(task_id.to_string(), task.agent_id.clone()).await?;
    }
    
    info!("Task '{}' execution started in background", task.title);
    Ok(())
}

/// The actual task execution operation (runs in background)
async fn execute_task_operation(
    task_id: String,
    agent_id: String,
    cancellation_flag: Arc<AtomicBool>,
) -> Result<String> {
    use crate::core::agent_manager;
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    // Get the task and agent
    let task = registry_manager::get_task(&task_id).await?
        .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;
    
    let agent = agent_manager::get_agent(&agent_id).await?
        .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;
    
    info!("Executing task '{}' with agent '{}'", task.title, agent.name);
    
    // Check for cancellation before starting
    if cancellation_flag.load(Ordering::Relaxed) {
        info!("Task '{}' was cancelled before execution", task.title);
        return Err(anyhow::anyhow!("Task execution was cancelled"));
    }
    
    // Create the message for Claude
    let message = format!("Task: {}\n\nDescription: {}", task.title, task.description);
    
    // Execute the Claude CLI command
    let response = execute_claude_cli_command(&agent, &message, cancellation_flag).await?;
    
    info!("Task '{}' completed successfully", task.title);
    
    // Update task as completed with response
    registry_manager::update_task_for_execution(&task_id, |task| {
        task.metadata.insert("claude_response".to_string(), response.clone());
        task.metadata.insert("execution_timestamp".to_string(), Utc::now().to_rfc3339());
        task.status = TaskStatus::Done;
        task.completed_at = Some(Utc::now());
        task.progress = 100;
        Ok(())
    }).await?;
    
    Ok(response)
}

/// Execute a Claude CLI command with cancellation support
async fn execute_claude_cli_command(
    agent: &crate::types::Agent,
    message: &str,
    cancellation_flag: Arc<AtomicBool>,
) -> Result<String> {
    
    let system_prompt = &agent.system_prompt;
    
    info!("Executing Claude command for agent: {}", agent.name);
    
    // Check for cancellation before starting
    if cancellation_flag.load(Ordering::Relaxed) {
        return Err(anyhow::anyhow!("Task execution was cancelled"));
    }
    
    // Combine system prompt with user message
    let full_message = format!("{}\n\nUser: {}", system_prompt, message);
    
    // Build the command with proper Claude CLI syntax
    let mut cmd = Command::new("-claude-");
    
    cmd.arg("--print")  // Non-interactive mode
       .arg("--dangerously-skip-permissions")
       .arg("--output-format").arg("text")  // Text output
       .arg(&full_message)  // The prompt as argument
       .stdout(Stdio::piped())
       .stderr(Stdio::piped())
       .kill_on_drop(true);  // Ensure process is killed when dropped
    
    info!("Spawning Claude CLI command for agent: {}", agent.name);
    
    // Spawn the process
    let mut child = cmd.spawn()?;
    
    // Wait for the process to complete, checking for cancellation
    loop {
        // Check for cancellation
        if cancellation_flag.load(Ordering::Relaxed) {
            info!("Task execution cancelled, killing Claude process for agent: {}", agent.name);
            child.kill().await?;
            return Err(anyhow::anyhow!("Task execution was cancelled"));
        }
        
        // Check if process has finished
        match child.try_wait() {
            Ok(Some(status)) => {
                // Process finished
                let output = child.wait_with_output().await?;
                
                if status.success() {
                    let response = String::from_utf8_lossy(&output.stdout).to_string();
                    info!("Claude command completed successfully for agent: {}", agent.name);
                    return Ok(response.trim().to_string());
                } else {
                    let error_msg = String::from_utf8_lossy(&output.stderr);
                    error!("Claude command failed for agent {}: {}", agent.name, error_msg);
                    return Err(anyhow::anyhow!("Claude command failed: {}", error_msg));
                }
            }
            Ok(None) => {
                // Process still running, wait a bit and check again
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
            Err(e) => {
                error!("Error checking process status for agent {}: {}", agent.name, e);
                return Err(anyhow::anyhow!("Error checking process status: {}", e));
            }
        }
    }
}

/// Check if a task is currently running
pub async fn is_task_running(task_id: &str) -> bool {
    let manager = TASK_EXECUTION_MANAGER.lock().await;
    manager.is_task_running(task_id)
}

/// Get all running task IDs
pub async fn get_running_task_ids() -> Vec<String> {
    let manager = TASK_EXECUTION_MANAGER.lock().await;
    manager.get_running_task_ids()
}

/// Clean up finished tasks
pub async fn cleanup_finished_tasks() -> Result<()> {
    let mut manager = TASK_EXECUTION_MANAGER.lock().await;
    manager.cleanup_finished_tasks().await;
    Ok(())
}

/// Stop a running task execution
pub async fn stop_task_execution(task_id: &str) -> Result<()> {
    let mut manager = TASK_EXECUTION_MANAGER.lock().await;
    manager.stop_task_execution(task_id).await
}