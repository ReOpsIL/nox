//! Predefined agents loader and manager
//! 
//! This module handles loading predefined agent configurations from YAML
//! and creating agent instances in the Nox system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs};
use log::{info, error};
use uuid::Uuid;

use crate::core::{agent_manager, task_manager};
use crate::types::{Agent, Task, AgentStatus, TaskStatus, TaskPriority};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredefinedAgentConfig {
    pub version: String,
    pub metadata: ConfigMetadata,
    pub categories: Vec<Category>,
    pub agents: Vec<PredefinedAgent>,
    pub task_templates: HashMap<String, TaskTemplate>,
    pub filters: Filters,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConfigMetadata {
    pub created_at: String,
    pub description: String,
    pub author: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PredefinedAgent {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub system_prompt: String,
    pub capabilities: Vec<String>,
    pub default_tasks: Vec<DefaultTask>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefaultTask {
    pub template: String,
    pub title: String,
    pub description: String,
    pub priority: String,
    pub estimated_duration: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TaskTemplate {
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub default_priority: String,
    pub estimated_duration: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Filters {
    pub by_category: Vec<String>,
    pub by_technology: Vec<String>,
    pub by_complexity: Vec<String>,
    pub by_duration: Vec<String>,
}

/// Load predefined agent configuration from YAML file
pub async fn load_predefined_config() -> Result<PredefinedAgentConfig> {
    let config_path = "config/predefined_agents.yaml";
    let config_content = fs::read_to_string(config_path)
        .map_err(|e| anyhow::anyhow!("Failed to read config file {}: {}", config_path, e))?;
    
    let config: PredefinedAgentConfig = serde_yaml::from_str(&config_content)
        .map_err(|e| anyhow::anyhow!("Failed to parse YAML config: {}", e))?;

    info!("Loaded predefined agent config with {} categories and {} agents", 
        config.categories.len(), config.agents.len());

    Ok(config)
}

/// Get all available categories
pub async fn get_categories() -> Result<Vec<Category>> {
    let config = load_predefined_config().await?;
    Ok(config.categories)
}

/// Get agents by category
pub async fn get_agents_by_category(category_id: &str) -> Result<Vec<PredefinedAgent>> {
    let config = load_predefined_config().await?;
    let agents = config.agents
        .into_iter()
        .filter(|agent| agent.category == category_id)
        .collect();
    Ok(agents)
}

/// Convert predefined agent to Nox agent
fn convert_to_nox_agent(predefined: &PredefinedAgent) -> Agent {
    Agent {
        id: Uuid::new_v4().to_string(),
        name: predefined.name.clone(),
        system_prompt: predefined.system_prompt.clone(),
        status: AgentStatus::Inactive,
        created_at: chrono::Utc::now(),
        resource_limits: crate::types::ResourceLimits {
            max_memory_mb: 512,
            max_cpu_percent: 50,
        },
    }
}

/// Convert task priority string to TaskPriority enum
fn parse_task_priority(priority_str: &str) -> TaskPriority {
    match priority_str.to_lowercase().as_str() {
        "high" => TaskPriority::High,
        "medium" => TaskPriority::Medium,
        "low" => TaskPriority::Low,
        _ => TaskPriority::Medium,
    }
}

/// Convert default task to Nox task
fn convert_to_nox_task(
    default_task: &DefaultTask, 
    agent_id: &str, 
    template: &TaskTemplate
) -> Task {
    // Create detailed description with steps
    let mut description = default_task.description.clone();
    if !template.steps.is_empty() {
        description.push_str("\n\nImplementation Steps:\n");
        for (i, step) in template.steps.iter().enumerate() {
            description.push_str(&format!("{}. {}\n", i + 1, step));
        }
    }

    Task {
        id: Uuid::new_v4().to_string(),
        title: default_task.title.clone(),
        description,
        agent_id: agent_id.to_string(),
        status: TaskStatus::Todo,
        priority: parse_task_priority(&default_task.priority),
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        progress: 0,
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("template".to_string(), default_task.template.clone());
            meta.insert("estimated_duration".to_string(), default_task.estimated_duration.clone());
            meta.insert("category".to_string(), template.category.clone());
            meta.insert("tags".to_string(), template.tags.join(","));
            meta
        },
    }
}

/// Load agents from a specific category into Nox
pub async fn load_category_agents(category_id: &str) -> Result<Vec<(String, Vec<String>)>> {
    let config = load_predefined_config().await?;
    let predefined_agents = config.agents
        .into_iter()
        .filter(|agent| agent.category == category_id)
        .collect::<Vec<_>>();

    if predefined_agents.is_empty() {
        return Err(anyhow::anyhow!("No agents found for category: {}", category_id));
    }

    // Get existing agents to check for duplicates
    let existing_agents = agent_manager::get_all_agents().await?;
    let existing_agent_names: std::collections::HashSet<String> = existing_agents
        .iter()
        .map(|agent| agent.name.clone())
        .collect();

    let mut loaded_agents = Vec::new();

    for predefined_agent in &predefined_agents {
        // Check if agent with this name already exists
        if existing_agent_names.contains(&predefined_agent.name) {
            info!("Skipping agent '{}' - already exists", predefined_agent.name);
            continue;
        }

        info!("Loading agent: {} ({})", predefined_agent.name, predefined_agent.id);

        // Convert and create the agent in Nox
        let nox_agent = convert_to_nox_agent(predefined_agent);
        let agent_id = nox_agent.id.clone();

        // Add agent to the system
        agent_manager::add_agent(nox_agent).await?;
        info!("Created Nox agent with ID: {}", agent_id);

        // Get existing tasks to check for duplicates
        let existing_tasks = task_manager::get_tasks_by_agent(&agent_id).await?;
        let existing_task_titles: std::collections::HashSet<String> = existing_tasks
            .iter()
            .map(|task| task.title.clone())
            .collect();

        // Create tasks for this agent
        let mut task_ids = Vec::new();
        for default_task in &predefined_agent.default_tasks {
            // Check if task with this title already exists for this agent
            if existing_task_titles.contains(&default_task.title) {
                info!("Skipping task '{}' for agent '{}' - already exists", 
                    default_task.title, predefined_agent.name);
                continue;
            }

            if let Some(template) = config.task_templates.get(&default_task.template) {
                let nox_task = convert_to_nox_task(default_task, &agent_id, template);
                let task_id = nox_task.id.clone();
                
                task_manager::add_task(nox_task).await?;
                task_ids.push(task_id);
                
                info!("Created task: {} for agent {}", default_task.title, predefined_agent.name);
            } else {
                error!("Template not found: {}", default_task.template);
            }
        }

        loaded_agents.push((agent_id, task_ids));
    }

    info!("Successfully loaded {} new agents from category: {}", loaded_agents.len(), category_id);
    Ok(loaded_agents)
}

/// Load specific agents by their predefined IDs
pub async fn load_specific_agents(predefined_agent_ids: &[String]) -> Result<Vec<(String, Vec<String>)>> {
    let config = load_predefined_config().await?;
    
    // Get existing agents to check for duplicates
    let existing_agents = agent_manager::get_all_agents().await?;
    let existing_agent_names: std::collections::HashSet<String> = existing_agents
        .iter()
        .map(|agent| agent.name.clone())
        .collect();
    
    let mut loaded_agents = Vec::new();
    let mut skipped_count = 0;

    for predefined_id in predefined_agent_ids {
        if let Some(predefined_agent) = config.agents.iter().find(|a| &a.id == predefined_id) {
            // Check if agent with this name already exists
            if existing_agent_names.contains(&predefined_agent.name) {
                info!("Skipping agent '{}' - already exists", predefined_agent.name);
                skipped_count += 1;
                continue;
            }

            info!("Loading specific agent: {} ({})", predefined_agent.name, predefined_agent.id);

            // Convert and create the agent in Nox
            let nox_agent = convert_to_nox_agent(predefined_agent);
            let agent_id = nox_agent.id.clone();

            // Add agent to the system
            agent_manager::add_agent(nox_agent).await?;
            info!("Created Nox agent with ID: {}", agent_id);

            // Get existing tasks to check for duplicates (initially empty for new agent)
            let existing_tasks = task_manager::get_tasks_by_agent(&agent_id).await?;
            let existing_task_titles: std::collections::HashSet<String> = existing_tasks
                .iter()
                .map(|task| task.title.clone())
                .collect();

            // Create tasks for this agent
            let mut task_ids = Vec::new();
            for default_task in &predefined_agent.default_tasks {
                // Check if task with this title already exists for this agent
                if existing_task_titles.contains(&default_task.title) {
                    info!("Skipping task '{}' for agent '{}' - already exists", 
                        default_task.title, predefined_agent.name);
                    continue;
                }

                if let Some(template) = config.task_templates.get(&default_task.template) {
                    let nox_task = convert_to_nox_task(default_task, &agent_id, template);
                    let task_id = nox_task.id.clone();
                    
                    task_manager::add_task(nox_task).await?;
                    task_ids.push(task_id);
                    
                    info!("Created task: {} for agent {}", default_task.title, predefined_agent.name);
                } else {
                    error!("Template not found: {}", default_task.template);
                }
            }

            loaded_agents.push((agent_id, task_ids));
        } else {
            error!("Predefined agent not found: {}", predefined_id);
        }
    }

    info!("Successfully loaded {} new agents, skipped {} duplicates", 
        loaded_agents.len(), skipped_count);
    Ok(loaded_agents)
}

/// Get all predefined agents
pub async fn get_all_predefined_agents() -> Result<Vec<PredefinedAgent>> {
    let config = load_predefined_config().await?;
    Ok(config.agents)
}

/// Get task template by ID
pub async fn get_task_template(template_id: &str) -> Result<Option<TaskTemplate>> {
    let config = load_predefined_config().await?;
    Ok(config.task_templates.get(template_id).cloned())
}

/// Clear all agents and tasks (for testing)
pub async fn clear_all_loaded_agents() -> Result<()> {
    info!("Clearing all loaded agents and tasks");
    
    // Get all agents
    let agents = agent_manager::get_all_agents().await?;
    
    // Delete all tasks for each agent, then delete the agent
    for agent in agents {
        task_manager::delete_all_tasks_for_agent(&agent.id).await?;
        agent_manager::delete_agent(&agent.id).await?;
    }
    
    info!("All agents and tasks cleared");
    Ok(())
}

/// Get summary of loaded agents by category
pub async fn get_loaded_agents_summary() -> Result<HashMap<String, Vec<String>>> {
    let config = load_predefined_config().await?;
    let current_agents = agent_manager::get_all_agents().await?;
    
    let mut summary = HashMap::new();
    
    for agent in &current_agents {
        // Try to match agent with predefined agents to determine category
        if let Some(predefined) = config.agents.iter().find(|pa| pa.name == agent.name) {
            let category_name = config.categories
                .iter()
                .find(|c| c.id == predefined.category)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| predefined.category.clone());
                
            summary.entry(category_name)
                .or_insert_with(Vec::new)
                .push(agent.name.clone());
        }
    }
    
    Ok(summary)
}