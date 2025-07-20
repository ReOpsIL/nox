//! Core module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing agents, tasks, and the registry.

pub mod agent_manager;
pub mod task_manager;
pub mod claude_json_transport;
pub mod registry_manager;
pub mod git_manager;
pub mod config_manager;
pub mod claude_process_manager;
pub mod mcp_manager;
pub mod advanced_agent_features;
pub mod resource_manager;
pub mod security_manager;
pub mod seeding;

use anyhow::Result;

/// Initialize the Nox agent ecosystem
pub async fn initialize() -> Result<()> {
    // Initialize configuration first
    config_manager::initialize().await?;

    // Initialize the registry
    registry_manager::initialize().await?;

    // Initialize security framework
    security_manager::initialize().await?;

    // Initialize resource monitoring
    resource_manager::start_monitoring().await?;

    // Seed default data if registry is empty
    seeding::seed_default_data().await?;

    // Log initialization
    log::info!("Nox agent ecosystem initialized");

    Ok(())
}

/// Start the Nox agent ecosystem
pub async fn start(dev_mode: bool) -> Result<()> {
    // Initialize the system first
    initialize().await?;

    // Set auto-approve mode for testing if in dev mode
    if dev_mode {
        security_manager::set_auto_approve(true).await?;
        advanced_agent_features::set_auto_approve(true).await?;
        log::info!("Development mode enabled: auto-approval activated");
    }

    // Start any active agents
    let agents = agent_manager::get_all_agents().await?;
    for agent in agents {
        if agent.status == crate::types::AgentStatus::Active {
            agent_manager::start_agent(&agent.id).await?;
        }
    }

    log::info!("Nox agent ecosystem started");
    Ok(())
}

/// Stop the Nox agent ecosystem
pub async fn stop() -> Result<()> {
    // Stop all running agents
    let agents = agent_manager::get_all_agents().await?;
    for agent in agents {
        if agent.status == crate::types::AgentStatus::Active || 
           agent.status == crate::types::AgentStatus::Starting {
            agent_manager::stop_agent(&agent.id).await?;
        }
    }

    // Stop resource monitoring
    resource_manager::stop_monitoring().await?;

    // Commit any pending changes to the registry
    let registry_path = std::path::PathBuf::from(".nox-registry");
    git_manager::commit_changes(&registry_path, "System shutdown [SIGNIFICANT]").await?;

    log::info!("Nox agent ecosystem stopped");
    Ok(())
}

/// Get the current status of the Nox agent ecosystem
pub async fn get_status() -> Result<String> {
    // Get agent counts
    let agents = agent_manager::get_all_agents().await?;
    let active_agents = agents.iter().filter(|a| a.status == crate::types::AgentStatus::Active).count();
    let total_agents = agents.len();

    // Get task counts
    let tasks = registry_manager::get_all_tasks().await?;
    let pending_tasks = tasks.iter().filter(|t| t.status == crate::types::TaskStatus::Todo).count();
    let in_progress_tasks = tasks.iter().filter(|t| t.status == crate::types::TaskStatus::InProgress).count();
    let completed_tasks = tasks.iter().filter(|t| t.status == crate::types::TaskStatus::Done).count();

    // Get resource usage
    let (cpu_usage, memory_usage, _disk_usage, _network_usage) = resource_manager::get_system_usage().await?;

    // Get security info
    let pending_approvals = security_manager::get_pending_requests().await?.len();

    // Get MCP service info
    let services = mcp_manager::get_all_services().await?;
    let running_services = services.iter().filter(|s| s.status == mcp_manager::ServiceStatus::Running).count();

    // Format the status message
    let status = format!(
        "System Status: OK\n\
         Agents: {}/{} active\n\
         Tasks: {} pending, {} in progress, {} completed\n\
         Resources: {:.1}% CPU, {} MB memory\n\
         Security: {} pending approvals\n\
         Services: {}/{} running",
        active_agents, total_agents,
        pending_tasks, in_progress_tasks, completed_tasks,
        cpu_usage, memory_usage,
        pending_approvals,
        running_services, services.len()
    );

    Ok(status)
}

/// Check the health of the Nox agent ecosystem
pub async fn check_health() -> Result<bool> {
    // This will be implemented later
    Ok(true)
}
