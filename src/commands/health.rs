//! Implementation of the health command

use anyhow::Result;
use log::info;
use crate::core;

/// Execute the health command
pub async fn execute() -> Result<()> {
    info!("Checking system health");
    
    // Call the core check_health function
    let is_healthy = core::check_health().await?;
    
    if is_healthy {
        println!("System health: OK");
    } else {
        println!("System health: Issues detected");
        // In a real implementation, we would print details about the issues
    }
    
    Ok(())
}