//! API module for the Nox agent ecosystem
//! 
//! This module contains the Actix Web API routes and server logic.

pub mod routes;
pub mod server;
pub mod websocket;

/// Start the API server with the given configuration
pub async fn start_server(_port: Option<u16>) -> anyhow::Result<()> {
    // This will be implemented later
    Ok(())
}