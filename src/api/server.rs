//! API server module for the Nox agent ecosystem
//! 
//! This module contains the Actix Web server implementation.

use actix_web::{web, App, HttpServer, middleware, HttpResponse};
use actix_cors::Cors;
use log::{info, error};
use std::sync::Arc;
use crate::api::websocket;
use crate::api::routes;
use crate::core::config_manager;

/// Start the API server
pub async fn start_server(port: Option<u16>) -> anyhow::Result<()> {
    // Get server configuration
    let server_config = config_manager::get_server_config().await?;
    
    // Use provided port or default from config
    let port = port.unwrap_or(server_config.port);
    let host = &server_config.host;
    
    info!("Starting API server on {}:{}", host, port);
    
    // Create and start the HTTP server
    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                // If no origins are specified, allow all
                if server_config.cors_origins.is_empty() {
                    return true;
                }
                
                // Check if the origin is in the allowed list
                let origin_str = match origin.to_str() {
                    Ok(s) => s,
                    Err(_) => return false,
                };
                
                server_config.cors_origins.iter().any(|allowed| allowed == origin_str)
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec!["Authorization", "Content-Type"])
            .supports_credentials()
            .max_age(3600);
        
        // Create the application
        let mut app = App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors);
        
        // Add API routes if enabled
        if server_config.api_enabled {
            app = app.service(
                web::scope("/api")
                    .configure(routes::configure)
            );
        }
        
        // Add WebSocket routes if enabled
        if server_config.websocket_enabled {
            app = app.service(
                web::scope("")
                    .configure(websocket::configure)
            );
        }
        
        // Add a health check endpoint
        app = app.route("/health", web::get().to(|| async {
            HttpResponse::Ok().body("OK")
        }));
        
        app
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;
    
    info!("API server stopped");
    Ok(())
}