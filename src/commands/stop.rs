//! Implementation of the stop command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the stop command
pub async fn execute() -> Result<()> {
    info!("Stopping Nox agent ecosystem");
    println!("Stopping Nox agent ecosystem...");
    
    // Call the core stop function
    core::stop().await?;
    
    println!("Nox agent ecosystem stopped successfully");
    Ok(())
}