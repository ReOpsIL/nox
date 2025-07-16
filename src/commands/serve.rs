//! Implementation of the serve command

use crate::api::server;
use crate::core::config_manager;
use anyhow::Result;
use log::info;

/// Execute the serve command
pub async fn execute() -> Result<()> {
    // Get server configuration from configuration manager
    let server_config = config_manager::get_server_config().await?;
    let port = server_config.port;

    info!("Starting API server on port {}", port);

    println!("Starting API server on port {}...", port);
    println!("Press Ctrl+C to stop the server");

    // Start the API server
    server::start_server().await?;

    Ok(())
}
