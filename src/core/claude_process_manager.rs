//! Claude process manager module for the Nox agent ecosystem
//! 
//! This module handles the spawning and management of Claude CLI processes.

use anyhow::{Result, anyhow};
use log::{info, error, warn, debug};
use std::collections::HashMap;
use tokio::process::{Command, Child};
use tokio::io::{AsyncWriteExt, AsyncReadExt, AsyncBufReadExt, BufReader};
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};
use std::io::ErrorKind;
use crate::types::Agent;

// Singleton instance of the Claude process manager
lazy_static::lazy_static! {
    static ref CLAUDE_PROCESS_MANAGER: Arc<Mutex<ClaudeProcessManager>> = Arc::new(Mutex::new(ClaudeProcessManager::new()));
}

/// Claude process manager struct
pub struct ClaudeProcessManager {
    processes: HashMap<String, Child>,
}

impl ClaudeProcessManager {
    /// Create a new Claude process manager
    fn new() -> Self {
        Self {
            processes: HashMap::new(),
        }
    }

    /// Start a Claude process for an agent
    async fn start_process(&mut self, agent: &Agent) -> Result<()> {
        info!("Starting Claude process for agent: {}", agent.name);

        // Check if a process already exists for this agent
        if self.processes.contains_key(&agent.id) {
            warn!("Process already exists for agent: {}", agent.id);
            return Ok(());
        }

        // Spawn the Claude CLI process
        let child = spawn_claude_process(agent).await?;

        // Store the process
        self.processes.insert(agent.id.clone(), child);

        info!("Claude process started for agent: {}", agent.name);
        Ok(())
    }

    /// Stop a Claude process for an agent
    async fn stop_process(&mut self, agent_id: &str) -> Result<()> {
        info!("Stopping Claude process for agent: {}", agent_id);

        // Remove the process from the map and kill it
        if let Some(mut child) = self.processes.remove(agent_id) {
            // Try to kill the process gracefully
            match child.kill().await {
                Ok(_) => info!("Claude process killed for agent: {}", agent_id),
                Err(e) => warn!("Failed to kill Claude process for agent {}: {}", agent_id, e),
            }
        } else {
            warn!("No process found for agent: {}", agent_id);
        }

        Ok(())
    }

    /// Send a message to a Claude process and get the response
    async fn send_message(&mut self, agent_id: &str, message: &str) -> Result<String> {
        // Get the process for this agent
        if let Some(child) = self.processes.get_mut(agent_id) {
            send_message_to_claude(child, message).await
        } else {
            Err(anyhow!("No process found for agent {}", agent_id))
        }
    }

    /// Check if a process exists for an agent
    async fn has_process(&self, agent_id: &str) -> bool {
        self.processes.contains_key(agent_id)
    }

    /// Get the number of running processes
    async fn process_count(&self) -> usize {
        self.processes.len()
    }
}

/// Spawn a Claude CLI process for an agent
async fn spawn_claude_process(agent: &Agent) -> Result<Child> {
    // Prepare the system prompt
    let system_prompt = &agent.system_prompt;

    info!("Spawning Claude process for agent: {}", agent.name);
    debug!("System prompt: {}", 
           if system_prompt.len() > 50 { &system_prompt[..50] } else { system_prompt });

    // Build the command with appropriate arguments
    let mut cmd = Command::new("claude");

    // Add common arguments
    cmd.arg("chat")
       .arg("--system")
       .arg(system_prompt)
       .stdin(Stdio::piped())
       .stdout(Stdio::piped())
       .stderr(Stdio::piped());

    // Use default model (claude-3-opus-20240229)
    // This could be enhanced in the future to support different models
    // by adding a model field to the Agent struct
    cmd.arg("--model").arg("claude-3-opus-20240229");

    // Spawn the process
    match cmd.spawn() {
        Ok(child) => {
            info!("Claude process spawned successfully for agent: {}", agent.name);

            // Verify the process is running
            match child.id() {
                Some(pid) => debug!("Claude process ID: {}", pid),
                None => warn!("Could not get process ID for Claude process"),
            }

            Ok(child)
        },
        Err(e) => {
            error!("Failed to spawn Claude process: {}", e);
            Err(anyhow!("Failed to spawn Claude process: {}", e))
        }
    }
}

// Constants for Claude process communication
const MESSAGE_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_RETRIES: usize = 3;
const RETRY_DELAY: Duration = Duration::from_secs(2);

/// Send a message to a Claude CLI process and get the response
async fn send_message_to_claude(child: &mut Child, message: &str) -> Result<String> {
    let mut retries = 0;

    while retries < MAX_RETRIES {
        match send_message_with_timeout(child, message).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                retries += 1;
                if retries >= MAX_RETRIES {
                    return Err(anyhow!("Failed to communicate with Claude after {} retries: {}", MAX_RETRIES, e));
                }

                warn!("Error communicating with Claude (attempt {}/{}): {}. Retrying in {} seconds...", 
                      retries, MAX_RETRIES, e, RETRY_DELAY.as_secs());

                tokio::time::sleep(RETRY_DELAY).await;
            }
        }
    }

    // This should never be reached due to the return in the error case above
    Err(anyhow!("Failed to communicate with Claude"))
}

/// Send a message to Claude with timeout
async fn send_message_with_timeout(child: &mut Child, message: &str) -> Result<String> {
    // Get stdin and stdout handles
    let stdin = child.stdin.as_mut().ok_or_else(|| anyhow!("Failed to open stdin"))?;
    let stdout = child.stdout.as_mut().ok_or_else(|| anyhow!("Failed to open stdout"))?;

    // Create a buffered reader for stdout
    let mut reader = BufReader::new(stdout);

    // Write the message to stdin with timeout
    match timeout(MESSAGE_TIMEOUT, async {
        stdin.write_all(message.as_bytes()).await?;
        stdin.write_all(b"\n").await?;
        stdin.flush().await?;
        Ok::<_, std::io::Error>(())
    }).await {
        Ok(result) => result?,
        Err(_) => return Err(anyhow!("Timeout while sending message to Claude")),
    }

    // Read the response with timeout
    let mut response = String::new();
    let mut buffer = String::new();

    // Use a timeout for the entire read operation
    match timeout(MESSAGE_TIMEOUT, async {
        // Read line by line until we get an empty line or EOF
        loop {
            match reader.read_line(&mut buffer).await {
                Ok(0) => break, // EOF
                Ok(_) => {
                    response.push_str(&buffer);
                    buffer.clear();

                    // Check for response completion marker (this may need adjustment based on Claude's output format)
                    if response.contains("\n\n") && response.trim().len() > 0 {
                        break;
                    }
                },
                Err(e) => {
                    if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                        // Non-blocking I/O would block, wait a bit and try again
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                    return Err(e);
                }
            }
        }
        Ok::<_, std::io::Error>(response)
    }).await {
        Ok(result) => Ok(result?),
        Err(_) => Err(anyhow!("Timeout while reading response from Claude")),
    }
}

/// Start a Claude process for an agent
pub async fn start_process(agent: &Agent) -> Result<()> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.start_process(agent).await
}

/// Stop a Claude process for an agent
pub async fn stop_process(agent_id: &str) -> Result<()> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.stop_process(agent_id).await
}

/// Send a message to a Claude process and get the response
pub async fn send_message(agent_id: &str, message: &str) -> Result<String> {
    let mut manager = CLAUDE_PROCESS_MANAGER.lock().await;
    manager.send_message(agent_id, message).await
}

/// Check if a process exists for an agent
pub async fn has_process(agent_id: &str) -> Result<bool> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.has_process(agent_id).await)
}

/// Get the number of running processes
pub async fn process_count() -> Result<usize> {
    let manager = CLAUDE_PROCESS_MANAGER.lock().await;
    Ok(manager.process_count().await)
}
