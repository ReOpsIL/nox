//! Implementation of the health command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the health command
pub async fn execute() -> Result<()> {
    info!("Checking system health");
    
    // Call the core check_health function
    let is_healthy = core::check_health().await?;
    
    if is_healthy {
        info!("System health: OK");
    } else {
        info!("System health: Issues detected");
        // In a real implementation, we would print details about the issues
    }
    
    Ok(())
}