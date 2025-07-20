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
use portable_pty::{CommandBuilder, PtySize};
use lazy_static::lazy_static;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

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
    
    // Spawn the task execution
    let task_handle = tokio::spawn(async move {
        execute_claude_cli_streaming(
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

/// Simple Claude CLI streaming execution
async fn execute_claude_cli_streaming(
    agent: &crate::types::Agent,
    initial_message: &str,
    cancellation_flag: Arc<AtomicBool>,
    mut input_rx: mpsc::Receiver<String>,
    output_tx: mpsc::Sender<String>,
    log_file_path: PathBuf,
    task_id: String,
) -> Result<String> {
    use crate::types::TaskStatus;
    use chrono::Utc;
    
    info!("Starting Claude CLI streaming for task: {}", task_id);
    
    // Create log file
    let mut log_file = tokio::fs::File::create(&log_file_path).await?;
    let log_header = format!("# Task {} Execution Log\n\n**Agent:** {}\n**Started:** {}\n\n", 
        task_id, agent.name, Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    log_file.write_all(log_header.as_bytes()).await?;
    
    // Set up PTY size (standard terminal size)
    let pty_size = PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    
    // Create PTY system and pair
    let pty_system = portable_pty::native_pty_system();
    let pty_pair = pty_system.openpty(pty_size)
        .map_err(|e| anyhow::anyhow!("Failed to create PTY: {}", e))?;
    
    // Create command for Claude CLI using PTY
    let mut cmd = CommandBuilder::new("claude");
    cmd.arg("--dangerously-skip-permissions");
    // Set working directory to Claude CLI location to fix yoga.wasm issue
    cmd.cwd("/Users/dovcaspi/nox_project_debug");
    
    // Spawn the process with PTY
    let mut child = pty_pair.slave.spawn_command(cmd)
        .map_err(|e| anyhow::anyhow!("Failed to spawn Claude CLI with PTY: {}", e))?;
    
    // Get master PTY for I/O
    let master = pty_pair.master;
    
    // Get reader and writer from master PTY
    let master_reader = master.try_clone_reader()
        .map_err(|e| anyhow::anyhow!("Failed to clone PTY reader: {}", e))?;
    let master_writer = master.take_writer()
        .map_err(|e| anyhow::anyhow!("Failed to get PTY writer: {}", e))?;
    let master_writer = Arc::new(std::sync::Mutex::new(master_writer));
    
    // Send initial message to PTY
    let full_message = format!("{}\n", initial_message);
    {
        use std::io::Write;
        let mut writer = master_writer.lock().unwrap();
        writer.write_all(full_message.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to write to PTY: {}", e))?;
        writer.flush()
            .map_err(|e| anyhow::anyhow!("Failed to flush PTY: {}", e))?;
    }
    
    // Log initial input
    log_file.write_all(format!("**Input:** {}\n\n", initial_message).as_bytes()).await?;
    log_file.write_all("**Output:**\n".as_bytes()).await?;

    let mut accumulated_output = String::new();
    
    // Spawn blocking task to handle PTY input (from channel to Claude)
    let writer_handle = master_writer.clone();
    let input_handle = tokio::task::spawn_blocking(move || {
        use std::io::Write;
        loop {
            match input_rx.blocking_recv() {
                Some(input) => {
                    if let Ok(mut writer) = writer_handle.lock() {
                        if let Err(e) = writer.write_all(input.as_bytes()) {
                            error!("Failed to write to PTY: {}", e);
                            break;
                        }
                        if let Err(e) = writer.flush() {
                            error!("Failed to flush PTY writer: {}", e);
                            break;
                        }
                    }
                }
                None => break,
            }
        }
    });
    
    // Spawn blocking task to handle PTY output (from Claude to channel)
    let output_tx_clone = output_tx.clone();
    let log_file_path_clone = log_file_path.clone();
    let cancellation_flag_clone = Arc::clone(&cancellation_flag);
    let pty_output_handle = tokio::task::spawn_blocking(move || {
        use std::io::Read;
        let mut reader = master_reader;
        let mut buffer = [0u8; 8192];
        
        loop {
            if cancellation_flag_clone.load(Ordering::Relaxed) {
                break;
            }
            
            match reader.read(&mut buffer) {
                Ok(0) => {
                    // EOF - process has terminated
                    info!("PTY reader reached EOF");
                    break;
                }
                Ok(n) => {
                    let output = String::from_utf8_lossy(&buffer[..n]).to_string();
                    
                    // Send to output channel
                    if let Err(_) = output_tx_clone.blocking_send(output.clone()) {
                        break; // Channel closed
                    }
                    
                    // Append to log file (async version)
                    let log_path = log_file_path_clone.clone();
                    let data = buffer[..n].to_vec();
                    tokio::spawn(async move {
                        if let Ok(mut file) = tokio::fs::OpenOptions::new()
                            .append(true)
                            .open(&log_path)
                            .await
                        {
                            let _ = file.write_all(&data).await;
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to read from PTY: {}", e);
                    break;
                }
            }
        }
    });
    
    
    // Wait for process completion (portable-pty wait is sync)
    let exit_status = tokio::task::spawn_blocking(move || child.wait()).await
        .map_err(|e| anyhow::anyhow!("Failed to join wait task: {}", e))?
        .map_err(|e| anyhow::anyhow!("Process wait failed: {}", e))?;
    
    // Clean up handles
    pty_output_handle.abort();
    input_handle.abort();
    
    // Read accumulated output from log file
    if let Ok(output) = tokio::fs::read_to_string(&log_file_path).await {
        accumulated_output = output;
    }
    
    // Update task status
    let final_status = if exit_status.success() {
        TaskStatus::Done
    } else {
        TaskStatus::Failed
    };
    
    let _ = registry_manager::update_task_for_execution(&task_id, |task| {
        task.metadata.insert("claude_response".to_string(), accumulated_output.clone());
        task.metadata.insert("execution_timestamp".to_string(), Utc::now().to_rfc3339());
        task.status = final_status;
        task.completed_at = Some(Utc::now());
        task.progress = 100;
        Ok(())
    }).await;
    
    // Remove from running tasks
    {
        let mut running_tasks = RUNNING_TASKS.lock().await;
        running_tasks.remove(&task_id);
    }
    
    if exit_status.success() {
        info!("Task {} completed successfully", task_id);
        Ok(accumulated_output)
    } else {
        error!("Task {} failed", task_id);
        Err(anyhow::anyhow!("Claude CLI process failed"))
    }
}

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