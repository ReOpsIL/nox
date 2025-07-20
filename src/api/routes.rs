//! API routes module for the Nox agent ecosystem
//! 
//! This module contains the API route handlers.

use crate::api::websocket;
use crate::core::{agent_manager, task_manager};
use crate::types::{Agent, AgentStatus, Task, TaskStatus};
use actix_web::{web, HttpResponse, Responder};
use log::error;
use serde::{Deserialize, Serialize};
use serde_json::json;

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
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
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
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Get an agent by ID
async fn get_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => success(agent, None),
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
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

                    success(agent.clone(), Some(format!("Agent '{}' updated successfully", agent.name)))
                },
                Err(e) => {
                    error!("Failed to update agent: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Delete an agent
async fn delete_agent(path: web::Path<String>) -> impl Responder {
    let agent_id = path.into_inner();

    // Get the agent name for the response message
    let agent_name = match agent_manager::get_agent(&agent_id).await {
        Ok(Some(agent)) => agent.name,
        Ok(None) => return error(actix_web::http::StatusCode::NOT_FOUND, &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            return error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
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
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
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
                return error(actix_web::http::StatusCode::BAD_REQUEST, "Agent is already active");
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
                        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, "Agent not found after starting"),
                        Err(e) => {
                            error!("Failed to get agent after starting: {}", e);
                            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to start agent: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
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
                return error(actix_web::http::StatusCode::BAD_REQUEST, "Agent is already inactive");
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
                        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, "Agent not found after stopping"),
                        Err(e) => {
                            error!("Failed to get agent after stopping: {}", e);
                            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to stop agent: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Agent '{}' not found", agent_id)),
        Err(e) => {
            error!("Failed to get agent: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// System status endpoint
async fn get_system_status() -> impl Responder {
    success(json!({
        "status": "running",
        "version": "0.1.0",
        "uptime": "unknown"
    }), None)
}

/// Configure the API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    // System routes
    cfg.service(
        web::scope("/system")
            .route("/status", web::get().to(get_system_status))
    );

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

    // Task routes
    cfg.service(
        web::scope("/tasks")
            .route("", web::get().to(get_tasks))
            .route("", web::post().to(create_task))
            .route("/{task_id}", web::get().to(get_task))
            .route("/{task_id}", web::put().to(update_task))
            .route("/{task_id}", web::delete().to(delete_task))
            .route("/{task_id}/start", web::post().to(start_task))
            .route("/{task_id}/complete", web::post().to(complete_task))
            .route("/{task_id}/cancel", web::post().to(cancel_task))
    );
}

/// Get all tasks
async fn get_tasks() -> impl Responder {
    match task_manager::get_all_tasks().await {
        Ok(tasks) => success(tasks, None),
        Err(e) => {
            error!("Failed to get tasks: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Create a new task
async fn create_task(req: web::Json<CreateTaskRequest>) -> impl Responder {
    // Verify agent exists
    match agent_manager::get_agent(&req.agent_id).await {
        Ok(Some(_)) => {
            // Create a new task
            let task = Task::new(
                req.agent_id.clone(),
                req.title.clone(),
                req.description.clone(),
            );

            // Add the task to the registry
            match task_manager::add_task(task.clone()).await {
                Ok(_) => {
                    // Broadcast the task creation event
                    let _ = websocket::broadcast_system_event(
                        "TaskCreated",
                        json!({
                            "task_id": task.id,
                            "agent_id": task.agent_id,
                            "title": task.title
                        })
                    ).await;

                    success(task, Some(format!("Task '{}' created successfully", req.title)))
                },
                Err(e) => {
                    error!("Failed to create task: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::BAD_REQUEST, &format!("Agent '{}' not found", req.agent_id)),
        Err(e) => {
            error!("Failed to verify agent: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Get a task by ID
async fn get_task(path: web::Path<String>) -> impl Responder {
    let task_id = path.into_inner();

    match task_manager::get_task(&task_id).await {
        Ok(Some(task)) => success(task, None),
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Update a task
async fn update_task(path: web::Path<String>, req: web::Json<UpdateTaskRequest>) -> impl Responder {
    let task_id = path.into_inner();

    // Get the task from the registry
    match task_manager::get_task(&task_id).await {
        Ok(Some(mut task)) => {
            // Update the task status if provided
            if let Some(status_str) = &req.status {
                match status_str.as_str() {
                    "todo" => task.status = TaskStatus::Todo,
                    "in_progress" => task.status = TaskStatus::InProgress,
                    "done" => task.status = TaskStatus::Done,
                    "cancelled" => task.status = TaskStatus::Cancelled,
                    _ => return error(actix_web::http::StatusCode::BAD_REQUEST, "Invalid task status"),
                }
            }

            // Save the updated task
            match task_manager::update_task(task.clone()).await {
                Ok(_) => {
                    // Broadcast the task update event
                    let _ = websocket::broadcast_system_event(
                        "TaskUpdated",
                        json!({
                            "task_id": task.id,
                            "agent_id": task.agent_id,
                            "status": format!("{:?}", task.status)
                        })
                    ).await;

                    success(task.clone(), Some(format!("Task '{}' updated successfully", task.title)))
                },
                Err(e) => {
                    error!("Failed to update task: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Delete a task
async fn delete_task(path: web::Path<String>) -> impl Responder {
    let task_id = path.into_inner();

    // Get the task title for the response message
    let task_title = match task_manager::get_task(&task_id).await {
        Ok(Some(task)) => task.title,
        Ok(None) => return error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            return error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string());
        }
    };

    // Delete the task
    match task_manager::delete_task(&task_id).await {
        Ok(_) => {
            // Broadcast the task deletion event
            let _ = websocket::broadcast_system_event(
                "TaskDeleted",
                json!({
                    "task_id": task_id,
                    "title": task_title
                })
            ).await;

            success(json!({"task_id": task_id}), Some(format!("Task '{}' deleted successfully", task_title)))
        },
        Err(e) => {
            error!("Failed to delete task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Start a task
async fn start_task(path: web::Path<String>) -> impl Responder {
    let task_id = path.into_inner();

    // Get the task to check if it exists and current status
    match task_manager::get_task(&task_id).await {
        Ok(Some(task)) => {
            // Check if the task is already in progress
            if task.status == TaskStatus::InProgress {
                return error(actix_web::http::StatusCode::BAD_REQUEST, "Task is already in progress");
            }

            // Start the task
            match task_manager::start_task(&task_id).await {
                Ok(_) => {
                    // Get the updated task
                    match task_manager::get_task(&task_id).await {
                        Ok(Some(updated_task)) => {
                            success(
                                updated_task,
                                Some(format!("Task '{}' started successfully", task.title))
                            )
                        },
                        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, "Task not found after starting"),
                        Err(e) => {
                            error!("Failed to get task after starting: {}", e);
                            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to start task: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Complete a task
async fn complete_task(path: web::Path<String>) -> impl Responder {
    let task_id = path.into_inner();

    // Get the task to check if it exists and current status
    match task_manager::get_task(&task_id).await {
        Ok(Some(task)) => {
            // Check if the task is already completed
            if task.status == TaskStatus::Done {
                return error(actix_web::http::StatusCode::BAD_REQUEST, "Task is already completed");
            }

            // Complete the task
            match task_manager::complete_task(&task_id).await {
                Ok(_) => {
                    // Get the updated task
                    match task_manager::get_task(&task_id).await {
                        Ok(Some(updated_task)) => {
                            success(
                                updated_task,
                                Some(format!("Task '{}' completed successfully", task.title))
                            )
                        },
                        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, "Task not found after completing"),
                        Err(e) => {
                            error!("Failed to get task after completing: {}", e);
                            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to complete task: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

/// Cancel a task
async fn cancel_task(path: web::Path<String>) -> impl Responder {
    let task_id = path.into_inner();

    // Get the task to check if it exists and current status
    match task_manager::get_task(&task_id).await {
        Ok(Some(task)) => {
            // Check if the task is already cancelled
            if task.status == TaskStatus::Cancelled {
                return error(actix_web::http::StatusCode::BAD_REQUEST, "Task is already cancelled");
            }

            // Cancel the task
            match task_manager::cancel_task(&task_id).await {
                Ok(_) => {
                    // Get the updated task
                    match task_manager::get_task(&task_id).await {
                        Ok(Some(updated_task)) => {
                            success(
                                updated_task,
                                Some(format!("Task '{}' cancelled successfully", task.title))
                            )
                        },
                        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, "Task not found after cancelling"),
                        Err(e) => {
                            error!("Failed to get task after cancelling: {}", e);
                            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to cancel task: {}", e);
                    error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                }
            }
        },
        Ok(None) => error(actix_web::http::StatusCode::NOT_FOUND, &format!("Task '{}' not found", task_id)),
        Err(e) => {
            error!("Failed to get task: {}", e);
            error(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}
