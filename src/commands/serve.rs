//! Implementation of the serve command

use crate::api::server;
use anyhow::Result;
use log::info;

/// Execute the serve command
pub async fn execute(port: Option<u16>) -> Result<()> {
    let port_str = port.map_or("default".to_string(), |p| p.to_string());
    info!("Starting API server on port {}", port_str);

    println!("Starting API server on port {}...", port_str);
    println!("Press Ctrl+C to stop the server");

    // Start the API server
    server::start_server(port).await?;

    Ok(())
}
