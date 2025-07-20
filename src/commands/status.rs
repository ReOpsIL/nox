//! Implementation of the status command

use crate::core;
use anyhow::Result;
use log::info;

/// Execute the status command
pub async fn execute() -> Result<()> {
    info!("Getting system status");
    
    // Call the core get_status function
    let status = core::get_status().await?;
    
    info!("{}", status);
    Ok(())
}