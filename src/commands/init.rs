//! Implementation of the init command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the init command
pub async fn execute() -> Result<()> {
    info!("Initializing Nox agent ecosystem");
    
    // Use the unified creation system
    core::create_system().await?;
    
    info!("Nox agent ecosystem initialized successfully");
    Ok(())
}