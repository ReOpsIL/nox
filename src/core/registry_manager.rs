//! Registry manager module for the Nox agent ecosystem
//! 
//! This module handles the persistence of agents and tasks in the .nox-registry directory.

use crate::core::{config_manager, git_manager};
use crate::types::{Agent, Task};
use anyhow::{anyhow, Result};
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs::{self, create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use toml;

/// Registry manager struct
struct RegistryManager;

/// Agent registry file structure
#[derive(Serialize, Deserialize)]
struct AgentRegistry {
    agents: Vec<Agent>,
}

impl RegistryManager {
    /// Get the registry paths from config
    async fn get_paths() -> Result<(PathBuf, PathBuf, PathBuf)> {
        let registry_path = config_manager::get_registry_path().await?;
        let agents_file = registry_path.join("agents.toml");
        let tasks_dir = registry_path.join("tasks");
        Ok((registry_path, agents_file, tasks_dir))
    }

    /// Initialize the registry
    async fn initialize() -> Result<()> {
        let (registry_path, agents_file, tasks_dir) = Self::get_paths().await?;
        
        // Create registry directory if it doesn't exist
        if !registry_path.exists() {
            info!("Creating registry directory: {:?}", registry_path);
            create_dir_all(&registry_path)?;
        }

        // Create agents file if it doesn't exist
        if !agents_file.exists() {
            info!("Creating agents file: {:?}", agents_file);
            let agent_registry = AgentRegistry { agents: Vec::new() };
            let toml_string = toml::to_string(&agent_registry)?;
            let mut file = File::create(&agents_file)?;
            file.write_all(toml_string.as_bytes())?;
        }

        // Create tasks directory if it doesn't exist
        if !tasks_dir.exists() {
            info!("Creating tasks directory: {:?}", tasks_dir);
            create_dir_all(&tasks_dir)?;
        }

        // Initialize git repository
        git_manager::initialize_repo(&registry_path).await?;

        Ok(())
    }

    /// Load agents from the registry
    async fn load_agents() -> Result<Vec<Agent>> {
        let (_, agents_file, _) = Self::get_paths().await?;
        
        if !agents_file.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(&agents_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let agent_registry: AgentRegistry = toml::from_str(&contents)?;
        Ok(agent_registry.agents)
    }

    /// Save agents to the registry
    async fn save_agents(agents: &[Agent]) -> Result<()> {
        let (registry_path, agents_file, _) = Self::get_paths().await?;
        
        let agent_registry = AgentRegistry { agents: agents.to_vec() };
        let toml_string = toml::to_string(&agent_registry)?;

        let mut file = File::create(&agents_file)?;
        file.write_all(toml_string.as_bytes())?;

        // Commit changes to git
        git_manager::commit_changes(&registry_path, "Updated agents.toml").await?;

        Ok(())
    }

    /// Load a task from the registry
    async fn load_task(task_id: &str) -> Result<Option<Task>> {
        let (_, _, tasks_dir) = Self::get_paths().await?;
        let task_file = tasks_dir.join(format!("{}.json", task_id));

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
    async fn save_task(task: &Task) -> Result<()> {
        let (registry_path, _, tasks_dir) = Self::get_paths().await?;
        let task_file = tasks_dir.join(format!("{}.json", task.id));
        let json_string = serde_json::to_string_pretty(task)?;

        let mut file = File::create(task_file)?;
        file.write_all(json_string.as_bytes())?;

        // Commit changes to git
        git_manager::commit_changes(&registry_path, &format!("Updated task {}", task.id)).await?;

        Ok(())
    }

    /// Delete a task from the registry
    async fn delete_task(task_id: &str) -> Result<()> {
        let (registry_path, _, tasks_dir) = Self::get_paths().await?;
        let task_file = tasks_dir.join(format!("{}.json", task_id));

        if task_file.exists() {
            fs::remove_file(task_file)?;

            // Commit changes to git
            git_manager::commit_changes(&registry_path, &format!("Deleted task {}", task_id)).await?;
        }

        Ok(())
    }

    /// Load all tasks from the registry
    async fn load_all_tasks() -> Result<Vec<Task>> {
        let (_, _, tasks_dir) = Self::get_paths().await?;
        let mut tasks = Vec::new();

        if !tasks_dir.exists() {
            return Ok(tasks);
        }

        for entry in fs::read_dir(&tasks_dir)? {
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
    RegistryManager::initialize().await
}

/// Save an agent to the registry
pub async fn save_agent(agent: &Agent) -> Result<()> {
    // Load existing agents
    let mut agents = RegistryManager::load_agents().await?;

    // Find and update or add the agent
    if let Some(index) = agents.iter().position(|a| a.id == agent.id) {
        agents[index] = agent.clone();
    } else {
        agents.push(agent.clone());
    }

    // Save the updated agents
    RegistryManager::save_agents(&agents).await
}

/// Get all agents from the registry
pub async fn get_all_agents() -> Result<Vec<Agent>> {
    RegistryManager::load_agents().await
}

/// Get an agent by ID
pub async fn get_agent(agent_id: &str) -> Result<Option<Agent>> {
    let agents = RegistryManager::load_agents().await?;
    Ok(agents.into_iter().find(|a| a.id == agent_id))
}

/// Delete an agent from the registry
pub async fn delete_agent(agent_id: &str) -> Result<()> {
    // Load existing agents
    let mut agents = RegistryManager::load_agents().await?;

    // Remove the agent
    let original_len = agents.len();
    agents.retain(|a| a.id != agent_id);

    if agents.len() == original_len {
        return Err(anyhow!("Agent not found: {}", agent_id));
    }

    // Save the updated agents
    RegistryManager::save_agents(&agents).await
}

/// Save a task to the registry
pub async fn save_task(task: &Task) -> Result<()> {
    RegistryManager::save_task(task).await
}

/// Get all tasks from the registry
pub async fn get_all_tasks() -> Result<Vec<Task>> {
    RegistryManager::load_all_tasks().await
}

/// Get a task by ID
pub async fn get_task(task_id: &str) -> Result<Option<Task>> {
    RegistryManager::load_task(task_id).await
}

/// Delete a task from the registry
pub async fn delete_task(task_id: &str) -> Result<()> {
    RegistryManager::delete_task(task_id).await
}

/// Atomically update a task during execution to avoid deadlocks
pub async fn update_task_for_execution<F>(task_id: &str, updater: F) -> Result<Task>
where
    F: FnOnce(&mut Task) -> Result<()>,
{
    // Load the task
    let mut task = RegistryManager::load_task(task_id).await?
        .ok_or_else(|| anyhow!("Task not found: {}", task_id))?;
    
    // Apply the update
    updater(&mut task)?;
    
    // Save the updated task
    RegistryManager::save_task(&task).await?;
    
    Ok(task)
}
