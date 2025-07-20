//! Registry manager module for the Nox agent ecosystem
//! 
//! This module handles the persistence of agents and tasks in the .nox-registry directory.

use crate::core::git_manager;
use crate::types::{Agent, Task};
use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use toml;

// Singleton instance of the registry manager
lazy_static::lazy_static! {
    static ref REGISTRY_MANAGER: Arc<Mutex<RegistryManager>> = Arc::new(Mutex::new(RegistryManager::new()));
}

/// Registry manager struct
struct RegistryManager {
    registry_path: PathBuf,
    agents_file: PathBuf,
    tasks_dir: PathBuf,
}

/// Agent registry file structure
#[derive(Serialize, Deserialize)]
struct AgentRegistry {
    agents: Vec<Agent>,
}

impl RegistryManager {
    /// Create a new registry manager
    fn new() -> Self {
        let registry_path = PathBuf::from(".nox-registry");
        let agents_file = registry_path.join("agents.toml");
        let tasks_dir = registry_path.join("tasks");

        Self {
            registry_path,
            agents_file,
            tasks_dir,
        }
    }

    /// Initialize the registry
    async fn initialize(&self) -> Result<()> {
        // Create registry directory if it doesn't exist
        if !self.registry_path.exists() {
            info!("Creating registry directory: {:?}", self.registry_path);
            create_dir_all(&self.registry_path)?;
        }

        // Create agents file if it doesn't exist
        if !self.agents_file.exists() {
            info!("Creating agents file: {:?}", self.agents_file);
            let agent_registry = AgentRegistry { agents: Vec::new() };
            let toml_string = toml::to_string(&agent_registry)?;
            let mut file = File::create(&self.agents_file)?;
            file.write_all(toml_string.as_bytes())?;
        }

        // Create tasks directory if it doesn't exist
        if !self.tasks_dir.exists() {
            info!("Creating tasks directory: {:?}", self.tasks_dir);
            create_dir_all(&self.tasks_dir)?;
        }

        // Initialize git repository
        git_manager::initialize_repo(&self.registry_path).await?;

        Ok(())
    }

    /// Load agents from the registry
    fn load_agents(&self) -> Result<Vec<Agent>> {
        if !self.agents_file.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(&self.agents_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let agent_registry: AgentRegistry = toml::from_str(&contents)?;
        Ok(agent_registry.agents)
    }

    /// Save agents to the registry
    async fn save_agents(&self, agents: &[Agent]) -> Result<()> {
        let agent_registry = AgentRegistry { agents: agents.to_vec() };
        let toml_string = toml::to_string(&agent_registry)?;

        let mut file = File::create(&self.agents_file)?;
        file.write_all(toml_string.as_bytes())?;

        // Commit changes to git
        git_manager::commit_changes(&self.registry_path, "Updated agents.toml").await?;

        Ok(())
    }

    /// Load a task from the registry
    fn load_task(&self, task_id: &str) -> Result<Option<Task>> {
        let task_file = self.tasks_dir.join(format!("{}.json", task_id));

        if !task_file.exists() {
            return Ok(None);
        }

        let mut file = File::open(task_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let task: Task = serde_json::from_str(&contents)?;
        Ok(Some(task))
    }

    /// Save a task to the registry
    async fn save_task(&self, task: &Task) -> Result<()> {
        let task_file = self.tasks_dir.join(format!("{}.json", task.id));
        let json_string = serde_json::to_string_pretty(task)?;

        let mut file = File::create(task_file)?;
        file.write_all(json_string.as_bytes())?;

        // Commit changes to git
        git_manager::commit_changes(&self.registry_path, &format!("Updated task {}", task.id)).await?;

        Ok(())
    }

    /// Delete a task from the registry
    async fn delete_task(&self, task_id: &str) -> Result<()> {
        let task_file = self.tasks_dir.join(format!("{}.json", task_id));

        if task_file.exists() {
            fs::remove_file(task_file)?;

            // Commit changes to git
            git_manager::commit_changes(&self.registry_path, &format!("Deleted task {}", task_id)).await?;
        }

        Ok(())
    }

    /// Load all tasks from the registry
    fn load_all_tasks(&self) -> Result<Vec<Task>> {
        let mut tasks = Vec::new();

        if !self.tasks_dir.exists() {
            return Ok(tasks);
        }

        for entry in fs::read_dir(&self.tasks_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                let mut file = File::open(&path)?;
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;

                match serde_json::from_str::<Task>(&contents) {
                    Ok(task) => tasks.push(task),
                    Err(e) => error!("Failed to parse task file {:?}: {}", path, e),
                }
            }
        }

        Ok(tasks)
    }
}

/// Initialize the registry
pub async fn initialize() -> Result<()> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.initialize().await
}

/// Save an agent to the registry
pub async fn save_agent(agent: &Agent) -> Result<()> {
    let manager = REGISTRY_MANAGER.lock().await;

    // Load existing agents
    let mut agents = manager.load_agents()?;

    // Find and update or add the agent
    if let Some(index) = agents.iter().position(|a| a.id == agent.id) {
        agents[index] = agent.clone();
    } else {
        agents.push(agent.clone());
    }

    // Save the updated agents
    manager.save_agents(&agents).await
}

/// Get all agents from the registry
pub async fn get_all_agents() -> Result<Vec<Agent>> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.load_agents()
}

/// Get an agent by ID
pub async fn get_agent(agent_id: &str) -> Result<Option<Agent>> {
    let manager = REGISTRY_MANAGER.lock().await;
    let agents = manager.load_agents()?;
    Ok(agents.into_iter().find(|a| a.id == agent_id))
}

/// Delete an agent from the registry
pub async fn delete_agent(agent_id: &str) -> Result<()> {
    let manager = REGISTRY_MANAGER.lock().await;

    // Load existing agents
    let mut agents = manager.load_agents()?;

    // Remove the agent
    let original_len = agents.len();
    agents.retain(|a| a.id != agent_id);

    if agents.len() == original_len {
        return Err(anyhow!("Agent not found: {}", agent_id));
    }

    // Save the updated agents
    manager.save_agents(&agents).await
}

/// Save a task to the registry
pub async fn save_task(task: &Task) -> Result<()> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.save_task(task).await
}

/// Get all tasks from the registry
pub async fn get_all_tasks() -> Result<Vec<Task>> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.load_all_tasks()
}

/// Get a task by ID
pub async fn get_task(task_id: &str) -> Result<Option<Task>> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.load_task(task_id)
}

/// Delete a task from the registry
pub async fn delete_task(task_id: &str) -> Result<()> {
    let manager = REGISTRY_MANAGER.lock().await;
    manager.delete_task(task_id).await
}

/// Atomically update a task during execution to avoid deadlocks
pub async fn update_task_for_execution<F>(task_id: &str, updater: F) -> Result<Task>
where
    F: FnOnce(&mut Task) -> Result<()>,
{
    let manager = REGISTRY_MANAGER.lock().await;
    
    // Load the task
    let mut task = manager.load_task(task_id)?
        .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;
    
    // Apply the update
    updater(&mut task)?;
    
    // Save the updated task
    manager.save_task(&task).await?;
    
    Ok(task)
}
