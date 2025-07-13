//! Implementation of the start command

use anyhow::Result;
use log::info;
use crate::core;

/// Execute the start command
pub async fn execute(dev: bool) -> Result<()> {
    if dev {
        info!("Starting Nox agent ecosystem in development mode");
        println!("Starting Nox agent ecosystem in development mode...");
    } else {
        info!("Starting Nox agent ecosystem");
        println!("Starting Nox agent ecosystem...");
    }
    
    // Call the core start function
    core::start(dev).await?;
    
    println!("Nox agent ecosystem started successfully");
    Ok(())
}