//! Implementation of the init command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the init command
pub async fn execute() -> Result<()> {
    info!("Initializing Nox agent ecosystem");
    
    // Create default configuration file if it doesn't exist
    core::config_manager::create_default_config().await?;
    
    // Call the core initialization function
    core::initialize().await?;
    
    println!("Nox agent ecosystem initialized successfully");
    Ok(())
}