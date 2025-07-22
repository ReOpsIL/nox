//! Core module for the Nox agent ecosystem
//! 
//! This module contains the core business logic for managing agents, tasks, and the registry.

pub mod agent_manager;
pub mod task_manager;
pub mod claude_json_transport;
pub mod registry_manager;
pub mod config_manager;
pub mod claude_process_manager;
pub mod advanced_agent_features;
pub mod resource_manager;
pub mod seeding;
pub mod predefined_agents;

use anyhow::Result;
use std::sync::atomic::{AtomicU8, Ordering};

/// Global flag to track the initialization level (0 = none, 1 = config, 2 = basic, 3 = full)
static INIT_LEVEL: AtomicU8 = AtomicU8::new(0);

/// Initialization level enum to control what gets initialized
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InitLevel {
    /// Only configuration - for commands that just need config access
    ConfigOnly,
    /// Basic system - config, registry, security (no resource monitoring or seeding)
    Basic,
    /// Full system - everything including resource monitoring and seeding
    Full,
}

/// Initialize the Nox system with the specified level
/// This is safe to call multiple times - subsequent calls are no-ops
pub async fn initialize_with_level(level: InitLevel) -> Result<()> {
    let required_level = match level {
        InitLevel::ConfigOnly => 1,
        InitLevel::Basic => 2, 
        InitLevel::Full => 3,
    };
    
    // Check if already initialized at this level or higher
    let current_level = INIT_LEVEL.load(Ordering::SeqCst);
    if current_level >= required_level {
        return Ok(());
    }

    // Initialize incrementally based on what we need
    match level {
        InitLevel::ConfigOnly => {
            if current_level < 1 {
                config_manager::ensure_initialized().await?;
                INIT_LEVEL.store(1, Ordering::SeqCst);
                log::debug!("Nox system initialized (config only)");
            }
        },
        
        InitLevel::Basic => {
            if current_level < 1 {
                config_manager::ensure_initialized().await?;
                INIT_LEVEL.store(1, Ordering::SeqCst);
            }
            if current_level < 2 {
                registry_manager::initialize().await?;
                INIT_LEVEL.store(2, Ordering::SeqCst);
                log::info!("Nox system initialized (basic)");
            }
        },
        
        InitLevel::Full => {
            if current_level < 1 {
                config_manager::ensure_initialized().await?;
                INIT_LEVEL.store(1, Ordering::SeqCst);
            }
            if current_level < 2 {
                registry_manager::initialize().await?;
                INIT_LEVEL.store(2, Ordering::SeqCst);
            }
            if current_level < 3 {
                resource_manager::start_monitoring().await?;
                seeding::seed_default_data().await?;
                INIT_LEVEL.store(3, Ordering::SeqCst);
                log::info!("Nox system initialized (full)");
            }
        },
    }

    Ok(())
}

/// Initialize the Nox agent ecosystem with full setup (legacy compatibility)
pub async fn initialize() -> Result<()> {
    initialize_with_level(InitLevel::Full).await
}

/// Ensure system is initialized for basic operations (config + registry access)
/// This is the most common initialization level needed by commands
pub async fn ensure_basic_init() -> Result<()> {
    initialize_with_level(InitLevel::Basic).await
}

/// Ensure configuration is available (lightest initialization)
pub async fn ensure_config_init() -> Result<()> {
    initialize_with_level(InitLevel::ConfigOnly).await
}

/// Start the Nox agent ecosystem
pub async fn start(dev_mode: bool) -> Result<()> {
    // Initialize the system with full setup
    initialize_with_level(InitLevel::Full).await?;

    // Set auto-approve mode for testing if in dev mode
    if dev_mode {
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

/// Create the system (init command) - ensures config file exists and does full initialization
pub async fn create_system() -> Result<()> {
    // Explicitly create config file if needed
    config_manager::create_default_config().await?;
    
    // Do full initialization
    initialize_with_level(InitLevel::Full).await?;
    
    log::info!("Nox system created and initialized");
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



    // Format the status message
    let status = format!(
        "System Status: OK\n\
         Agents: {}/{} active\n\
         Tasks: {} pending, {} in progress, {} completed\n\
         Resources: {:.1}% CPU, {} MB memory\n\
",
        active_agents, total_agents,
        pending_tasks, in_progress_tasks, completed_tasks,
        cpu_usage, memory_usage
    );

    Ok(status)
}

/// Check the health of the Nox agent ecosystem
pub async fn check_health() -> Result<bool> {
    // This will be implemented later
    Ok(true)
}
