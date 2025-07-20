//! Agent manager module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing agents.

use crate::core::claude_process_manager;
use crate::core::registry_manager;
use crate::types::{Agent, AgentStatus};
use anyhow::Result;
use log::{error, info};

// The agent manager handles agent lifecycle operations
// Process management is delegated to the claude_process_manager module

/// Add a new agent to the registry
pub async fn add_agent(agent: Agent) -> Result<()> {
    info!("Adding agent: {}", agent.name);
    registry_manager::save_agent(&agent).await
}

/// Get all agents from the registry
pub async fn get_all_agents() -> Result<Vec<Agent>> {
    registry_manager::get_all_agents().await
}

/// Get an agent by ID
pub async fn get_agent(agent_id: &str) -> Result<Option<Agent>> {
    registry_manager::get_agent(agent_id).await
}

/// Get an agent by name
pub async fn get_agent_by_name(name: &str) -> Result<Option<Agent>> {
    let agents = registry_manager::get_all_agents().await?;
    Ok(agents.into_iter().find(|a| a.name == name))
}

/// Update an agent in the registry
pub async fn update_agent(agent: Agent) -> Result<()> {
    info!("Updating agent: {}", agent.name);
    registry_manager::save_agent(&agent).await
}

/// Delete an agent from the registry
pub async fn delete_agent(agent_id: &str) -> Result<()> {
    info!("Deleting agent: {}", agent_id);

    // First, ensure the agent is stopped
    let agent = registry_manager::get_agent(agent_id).await?;
    if let Some(agent) = agent {
        if agent.status == AgentStatus::Active {
            stop_agent(agent_id).await?;
        }
    }

    // Delete all tasks associated with this agent
    info!("Deleting all tasks for agent: {}", agent_id);
    if let Err(e) = crate::core::task_manager::delete_all_tasks_for_agent(agent_id).await {
        error!("Failed to delete tasks for agent {}: {}", agent_id, e);
        // Continue with agent deletion even if task deletion fails
    }

    // Then delete the agent from registry
    registry_manager::delete_agent(agent_id).await
}

/// Start an agent
pub async fn start_agent(agent_id: &str) -> Result<()> {
    info!("Starting agent: {}", agent_id);

    // Get the agent from the registry
    let mut agent = match registry_manager::get_agent(agent_id).await? {
        Some(a) => a,
        None => {
            error!("Agent not found: {}", agent_id);
            return Err(anyhow::anyhow!("Agent not found: {}", agent_id));
        }
    };

    // Update agent status to Starting
    agent.status = AgentStatus::Starting;
    registry_manager::save_agent(&agent).await?;

    // Start the Claude CLI process
    claude_process_manager::start_process(&agent).await?;

    // Update agent status to Active
    agent.status = AgentStatus::Active;
    registry_manager::save_agent(&agent).await?;

    info!("Agent started: {}", agent_id);
    Ok(())
}

/// Stop an agent
pub async fn stop_agent(agent_id: &str) -> Result<()> {
    info!("Stopping agent: {}", agent_id);

    // Get the agent from the registry
    let mut agent = match registry_manager::get_agent(agent_id).await? {
        Some(a) => a,
        None => {
            error!("Agent not found: {}", agent_id);
            return Err(anyhow::anyhow!("Agent not found: {}", agent_id));
        }
    };

    // Update agent status to Stopping
    agent.status = AgentStatus::Stopping;
    registry_manager::save_agent(&agent).await?;

    // Stop the Claude CLI process
    claude_process_manager::stop_process(agent_id).await?;

    // Update agent status to Inactive
    agent.status = AgentStatus::Inactive;
    registry_manager::save_agent(&agent).await?;

    info!("Agent stopped: {}", agent_id);
    Ok(())
}
