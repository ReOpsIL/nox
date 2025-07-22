//! API server module for the Nox agent ecosystem
//! 
//! This module contains the Actix Web server implementation.

use crate::api::routes;
use crate::api::websocket;
use crate::core::config_manager;
use actix_files::Files;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use log::info;


/// Start the API server
pub async fn start_server() -> anyhow::Result<()> {
    // Server configuration will be available through ensure_basic_init in the serve command
    let server_config = config_manager::get_server_config().await?;
    
    // Use provided port or default from config
    let port  = server_config.port;
    let host = &server_config.host;
    
    info!("Starting API server on {}:{}", host, port);
    
    // Clone data needed in the closure
    let _cors_origins = server_config.cors_origins.clone();
    
    // Create and start the HTTP server
    HttpServer::new(move || {
        // Configure CORS
        // let cors_origins = cors_origins.clone();
        // let cors = Cors::default()
        //     .allowed_origin_fn(move |origin, _req_head| {
        //         // If no origins are specified, allow all
        //         if cors_origins.is_empty() {
        //             return true;
        //         }
        //
        //         // Check if the origin is in the allowed list
        //         let origin_str = match origin.to_str() {
        //             Ok(s) => s,
        //             Err(_) => return false,
        //         };
        //
        //         cors_origins.iter().any(|allowed| allowed == origin_str)
        //     })
        //     .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        //     .allowed_headers(vec!["Authorization", "Content-Type"])
        //     .supports_credentials()
        //     .max_age(3600);
        
        // Create the application
        let mut app = App::new()
            .wrap(middleware::Logger::default());
            //.wrap(cors);
        
        // Health check endpoint
        app = app.route("/health", web::get().to(|| async {
            HttpResponse::Ok().body("OK")
        }));

        // API routes
        if server_config.api_enabled {
            app = app.service(
                web::scope("/api")
                    .configure(routes::configure)
            );
        }
        
        // WebSocket routes
        if server_config.websocket_enabled {
            app = app.service(
                web::scope("/ws")
                    .configure(websocket::configure)
            );
        }
        
        // Serve the entire frontend directory as static files
        app = app.service(
            Files::new("/", "frontend/dist")
                .index_file("index.html")
                .prefer_utf8(true)
                .use_last_modified(true)
        );
        
        app
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await?;
    
    info!("API server stopped");
    Ok(())
}