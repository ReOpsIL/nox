//! Implementation of the init command

use anyhow::Result;
use log::info;
use crate::core;

/// Execute the init command
pub async fn execute() -> Result<()> {
    info!("Initializing Nox agent ecosystem");
    
    // Call the core initialization function
    core::initialize().await?;
    
    println!("Nox agent ecosystem initialized successfully");
    Ok(())
}