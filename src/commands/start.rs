//! Implementation of the start command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the start command
pub async fn execute(dev: bool) -> Result<()> {
    if dev {
        info!("Starting Nox agent ecosystem in development mode");
        info!("Starting Nox agent ecosystem in development mode...");
    } else {
        info!("Starting Nox agent ecosystem");
        info!("Starting Nox agent ecosystem...");
    }
    
    // Call the core start function
    core::start(dev).await?;
    
    info!("Nox agent ecosystem started successfully");
    Ok(())
}