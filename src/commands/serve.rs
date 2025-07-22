//! Implementation of the serve command

use crate::api::server;
use crate::core;
use anyhow::Result;
use log::info;

/// Execute the serve command
pub async fn execute() -> Result<()> {
    // Ensure basic system is initialized (config + registry for API operations)
    core::ensure_basic_init().await?;
    
    // Get server configuration
    let server_config = core::config_manager::get_server_config().await?;
    let port = server_config.port;

    info!("Starting API server on port {}", port);
    info!("Press Ctrl+C to stop the server");

    // Start the API server
    server::start_server().await?;

    Ok(())
}
