//! API endpoint tests for the Nox agent ecosystem
//!
//! This module contains comprehensive tests for all API endpoints as specified in the test plan.

use actix_web::{test, web, App};
use nox::api::routes;
use nox::core::{agent_manager, task_manager};
use nox::types::{Agent, Task};
use serde_json::json;
use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the test environment
async fn setup_test_env() {
    INIT.call_once(|| {
        env_logger::init();
    });
    
    // Clean up any existing registry for tests
    let _ = std::fs::remove_dir_all(".nox-registry");
    
    // Initialize the system
    let _ = nox::core::initialize().await;
}

/// Create a test app
fn create_test_app() -> App<
    impl actix_web::dev::ServiceFactory<
        actix_web::dev::ServiceRequest,
        Config = (),
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
        InitError = (),
    >,
> {
    App::new()
        .service(web::scope("/api").configure(routes::configure))
}

#[actix_web::test]
async fn test_get_agents_empty() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/agents")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[actix_web::test]
async fn test_create_agent_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    let agent_data = json!({
        "name": "TestAgent",
        "system_prompt": "You are a test agent"
    });
    
    let req = test::TestRequest::post()
        .uri("/api/agents")
        .set_json(&agent_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "TestAgent");
    assert_eq!(body["data"]["system_prompt"], "You are a test agent");
    assert_eq!(body["data"]["status"], "Inactive");
}

#[actix_web::test]
async fn test_get_agent_by_id_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/agents/{}", agent_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], agent_id);
    assert_eq!(body["data"]["name"], "TestAgent");
}

#[actix_web::test]
async fn test_get_agent_by_id_not_found() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/agents/nonexistent-id")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[actix_web::test]
async fn test_update_agent_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent
    let agent = Agent::new("TestAgent".to_string(), "Original prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let update_data = json!({
        "system_prompt": "Updated prompt"
    });
    
    let req = test::TestRequest::put()
        .uri(&format!("/api/agents/{}", agent_id))
        .set_json(&update_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["system_prompt"], "Updated prompt");
}

#[actix_web::test]
async fn test_delete_agent_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let req = test::TestRequest::delete()
        .uri(&format!("/api/agents/{}", agent_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["message"].as_str().unwrap().contains("deleted successfully"));
}

#[actix_web::test]
async fn test_start_agent_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/agents/{}/start", agent_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "Active");
}

#[actix_web::test]
async fn test_stop_agent_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create and start an agent
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    agent_manager::start_agent(&agent_id).await.unwrap();
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/agents/{}/stop", agent_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "Inactive");
}

// Task endpoint tests

#[actix_web::test]
async fn test_get_tasks_empty() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    let req = test::TestRequest::get()
        .uri("/api/tasks")
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[actix_web::test]
async fn test_create_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task_data = json!({
        "agent_id": agent_id,
        "title": "Test Task",
        "description": "This is a test task"
    });
    
    let req = test::TestRequest::post()
        .uri("/api/tasks")
        .set_json(&task_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["title"], "Test Task");
    assert_eq!(body["data"]["description"], "This is a test task");
    assert_eq!(body["data"]["agent_id"], agent_id);
    assert_eq!(body["data"]["status"], "Todo");
}

#[actix_web::test]
async fn test_create_task_invalid_agent() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    let task_data = json!({
        "agent_id": "nonexistent-agent",
        "title": "Test Task",
        "description": "This is a test task"
    });
    
    let req = test::TestRequest::post()
        .uri("/api/tasks")
        .set_json(&task_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], false);
    assert!(body["error"].as_str().unwrap().contains("not found"));
}

#[actix_web::test]
async fn test_get_task_by_id_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let req = test::TestRequest::get()
        .uri(&format!("/api/tasks/{}", task_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], task_id);
    assert_eq!(body["data"]["title"], "Test Task");
}

#[actix_web::test]
async fn test_update_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let update_data = json!({
        "status": "in_progress"
    });
    
    let req = test::TestRequest::put()
        .uri(&format!("/api/tasks/{}", task_id))
        .set_json(&update_data)
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "InProgress");
}

#[actix_web::test]
async fn test_delete_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let req = test::TestRequest::delete()
        .uri(&format!("/api/tasks/{}", task_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert!(body["message"].as_str().unwrap().contains("deleted successfully"));
}

#[actix_web::test]
async fn test_start_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/tasks/{}/start", task_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "InProgress");
}

#[actix_web::test]
async fn test_complete_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/tasks/{}/complete", task_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "Done");
}

#[actix_web::test]
async fn test_cancel_task_success() {
    setup_test_env().await;
    let app = test::init_service(create_test_app()).await;
    
    // First create an agent and task
    let agent = Agent::new("TestAgent".to_string(), "Test prompt".to_string());
    let agent_id = agent.id.clone();
    agent_manager::add_agent(agent).await.unwrap();
    
    let task = Task::new(agent_id, "Test Task".to_string(), "Test description".to_string());
    let task_id = task.id.clone();
    task_manager::add_task(task).await.unwrap();
    
    let req = test::TestRequest::post()
        .uri(&format!("/api/tasks/{}/cancel", task_id))
        .to_request();
    
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "Cancelled");
}