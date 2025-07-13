//! API routes module for the Nox agent ecosystem
//! 
//! This module contains the API route handlers.

use actix_web::{web, HttpResponse, Responder};
use log::{info, error};
use serde::{Serialize, Deserialize};
use serde_json::json;
use crate::core::{agent_manager, task_manager, claude_process_manager};
use crate::types::{Agent, Task, AgentStatus, TaskStatus};
use crate::api::websocket;

/// API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
}

/// Create a success response
fn success<T: serde::Serialize>(data: T, message: Option<String>) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse {
        success: true,
        message,
        error: None,
        data: Some(data),
    })
}

/// Create an error response
fn error(status_code: actix_web::http::StatusCode, error_message: &str) -> HttpResponse {
    HttpResponse::build(status_code).json(ApiResponse::<()> {
        success: false,
        message: None,
        error: Some(error_message.to_string()),
        data: None,
    })
}

/// Agent creation request
#[derive(Deserialize)]
struct CreateAgentRequest {
    name: String,
    system_prompt: String,
}

/// Agent update request
#[derive(Deserialize)]
struct UpdateAgentRequest {
    system_prompt: Option<String>,
}

/// Task creation request
#[derive(Deserialize)]
struct CreateTaskRequest {
    agent_id: String,
    title: String,
    description: String,
}

/// Task update request
#[derive(Deserialize)]
struct UpdateTaskRequest {
    status: Option<String>,
}

/// Get all agents
async fn get_agents() -> impl Responder {
    match agent_manager::get_all_agents().await {
        Ok(agents) => success(agents, None),
        Err(e) => {
            error!("Failed to get agents: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Create a new agent
async fn create_agent(req: web::Json<CreateAgentRequest>) -> impl Responder {
    // Create a new agent
    let agent = Agent::new(req.name.clone(), req.system_prompt.clone());

    // Add the agent to the registry
    match agent_manager::add_agent(agent.clone()).await {
        Ok(_) => {
            // Broadcast the agent creation event
            let _ = websocket::broadcast_system_event(
                "AgentCreated",
                json!({
                    "agent_id": agent.id,
                    "name": agent.name
                })
            ).await;

            success(agent, Some(format!("Agent '{}' created successfully", req.name)))
        },
        Err(e) => {
            error!("Failed to create agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Get an agent by ID
async fn get_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => success(agent, None),
        Ok(None) => error(HttpResponse::NotFound().status(), &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Update an agent
async fn update_agent(path: web::Path<String>, req: web::Json<UpdateAgentRequest>) -> impl Responder {
    let agent_id = path.into_inner();

    // Get the agent from the registry
    match agent_manager::get_agent(&agent_id).await {
        Ok(Some(mut agent)) => {
            // Update the agent
            if let Some(system_prompt) = &req.system_prompt {
                agent.system_prompt = system_prompt.clone();
            }

            // Save the updated agent
            match agent_manager::update_agent(agent.clone()).await {
                Ok(_) => {
                    // Broadcast the agent update event
                    let _ = websocket::broadcast_system_event(
                        "AgentUpdated",
                        json!({
                            "agent_id": agent.id,
                            "name": agent.name
                        })
                    ).await;

                    success(agent, Some(format!("Agent '{}' updated successfully", agent.name)))
                },
                Err(e) => {
                    error!("Failed to update agent: {}", e);
                    error(HttpResponse::InternalServerError().status(), &e.to_string())
                }
            }
        },
        Ok(None) => error(HttpResponse::NotFound().status(), &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Delete an agent
async fn delete_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    // Get the agent name for the response message
    let agent_name = match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => agent.name,
        Ok(None) => return error(HttpResponse::NotFound().status(), &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            return error(HttpResponse::InternalServerError().status(), &e.to_string());
        }
    };

    // Delete the agent
    match agent_manager::delete_agent(&agent_id).await {
        Ok(_) => {
            // Broadcast the agent deletion event
            let _ = websocket::broadcast_system_event(
                "AgentDeleted",
                json!({
                    "agent_id": agent_id,
                    "name": agent_name
                })
            ).await;

            success(json!({"agent_id": agent_id}), Some(format!("Agent '{}' deleted successfully", agent_name)))
        },
        Err(e) => {
            error!("Failed to delete agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Start an agent
async fn start_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    // Get the agent to check if it exists
    match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => {
            // Check if the agent is already active
            if agent.status == AgentStatus::Active {
                return error(HttpResponse::BadRequest().status(), "Agent is already active");
            }

            // Start the agent
            match agent_manager::start_agent(&agent_id).await {
                Ok(_) => {
                    // Get the updated agent
                    match agent_manager::get_agent(&agent_id).await {
                        Ok(Some(updated_agent)) => {
                            success(
                                updated_agent,
                                Some(format!("Agent '{}' started successfully", agent.name))
                            )
                        },
                        Ok(None) => error(HttpResponse::NotFound().status(), "Agent not found after starting"),
                        Err(e) => {
                            error!("Failed to get agent after starting: {}", e);
                            error(HttpResponse::InternalServerError().status(), &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to start agent: {}", e);
                    error(HttpResponse::InternalServerError().status(), &e.to_string())
                }
            }
        },
        Ok(None) => error(HttpResponse::NotFound().status(), &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Stop an agent
async fn stop_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    // Get the agent to check if it exists
    match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => {
            // Check if the agent is already inactive
            if agent.status == AgentStatus::Inactive {
                return error(HttpResponse::BadRequest().status(), "Agent is already inactive");
            }

            // Stop the agent
            match agent_manager::stop_agent(&agent_id).await {
                Ok(_) => {
                    // Get the updated agent
                    match agent_manager::get_agent(&agent_id).await {
                        Ok(Some(updated_agent)) => {
                            success(
                                updated_agent,
                                Some(format!("Agent '{}' stopped successfully", agent.name))
                            )
                        },
                        Ok(None) => error(HttpResponse::NotFound().status(), "Agent not found after stopping"),
                        Err(e) => {
                            error!("Failed to get agent after stopping: {}", e);
                            error(HttpResponse::InternalServerError().status(), &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to stop agent: {}", e);
                    error(HttpResponse::InternalServerError().status(), &e.to_string())
                }
            }
        },
        Ok(None) => error(HttpResponse::NotFound().status(), &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(HttpResponse::InternalServerError().status(), &e.to_string())
        }
    }
}

/// Configure the API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/agents")
            .route("", web::get().to(get_agents))
            .route("", web::post().to(create_agent))
            .route("/{agent_id}", web::get().to(get_agent))
            .route("/{agent_id}", web::put().to(update_agent))
            .route("/{agent_id}", web::delete().to(delete_agent))
            .route("/{agent_id}/start", web::post().to(start_agent))
            .route("/{agent_id}/stop", web::post().to(stop_agent))
    );

    // Task routes would be added here
}
