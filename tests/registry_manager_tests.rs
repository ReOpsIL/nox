//! Integration tests for the registry manager module

use nox::core::{registry_manager, agent_manager, task_manager};

mod common;
use common::{fixtures::*, helpers::*};

/// Test registry initialization
#[tokio::test]
async fn test_initialize_registry() {
    setup_test_env();
    
    // Test initialization
    let result = registry_manager::initialize().await;
    assert!(result.is_ok());
}

/// Test saving and loading agents
#[tokio::test]
async fn test_agent_save_and_load() {
    setup_test_env();
    
    // Initialize registry
    registry_manager::initialize().await.unwrap();
    
    // Create a test agent
    let agent = sample_agent();
    
    // Save the agent
    let save_result = registry_manager::save_agent(&agent).await;
    assert!(save_result.is_ok());
    
    // Load the agent
    let load_result = registry_manager::get_agent(&agent.id).await;
    assert!(load_result.is_ok());
    
    let loaded_agent = load_result.unwrap().expect("Agent should exist");
    assert_eq!(loaded_agent.id, agent.id);
    assert_eq!(loaded_agent.name, agent.name);
    assert_eq!(loaded_agent.system_prompt, agent.system_prompt);
    assert_eq!(loaded_agent.status, agent.status);
}

/// Test loading all agents
#[tokio::test]
async fn test_load_all_agents() {
    setup_test_env();
    
    // Initialize registry
    registry_manager::initialize().await.unwrap();
    
    // Create multiple test agents
    let agents = sample_agents(3);
    
    // Save all agents
    for agent in &agents {
        registry_manager::save_agent(agent).await.unwrap();
    }
    
    // Load all agents
    let loaded_agents = registry_manager::get_all_agents().await.unwrap();
    
    assert_eq!(loaded_agents.len(), 3);
    
    // Verify all agents were loaded correctly
    for agent in &agents {
        let found = loaded_agents.iter().find(|a| a.id == agent.id);
        assert!(found.is_some());
        let found_agent = found.unwrap();
        assert_eq!(found_agent.name, agent.name);
        assert_eq!(found_agent.system_prompt, agent.system_prompt);
    }
}

/// Test agent deletion with task cleanup
#[tokio::test]
async fn test_agent_deletion_with_task_cleanup() {
    setup_test_env();
    
    // Initialize registry
    registry_manager::initialize().await.unwrap();
    
    // Create a test agent
    let agent = sample_agent();
    agent_manager::add_agent(agent.clone()).await.unwrap();
    
    // Create multiple tasks for this agent
    let task1 = nox::types::Task::new(agent.id.clone(), "Task 1".to_string(), "Description 1".to_string());
    let task2 = nox::types::Task::new(agent.id.clone(), "Task 2".to_string(), "Description 2".to_string());
    let task3 = nox::types::Task::new(agent.id.clone(), "Task 3".to_string(), "Description 3".to_string());
    
    task_manager::add_task(task1.clone()).await.unwrap();
    task_manager::add_task(task2.clone()).await.unwrap();
    task_manager::add_task(task3.clone()).await.unwrap();
    
    // Verify tasks were created
    let agent_tasks_before = task_manager::get_tasks_by_agent(&agent.id).await.unwrap();
    assert_eq!(agent_tasks_before.len(), 3);
    
    // Delete the agent (this should also delete all its tasks)
    let delete_result = agent_manager::delete_agent(&agent.id).await;
    assert!(delete_result.is_ok(), "Agent deletion should succeed");
    
    // Verify agent was deleted
    let agent_after = agent_manager::get_agent(&agent.id).await.unwrap();
    assert!(agent_after.is_none(), "Agent should be deleted");
    
    // Verify all tasks for this agent were also deleted
    let agent_tasks_after = task_manager::get_tasks_by_agent(&agent.id).await.unwrap();
    assert_eq!(agent_tasks_after.len(), 0, "All agent tasks should be deleted");
    
    // Verify individual tasks are gone
    for task in [task1, task2, task3] {
        let task_after = task_manager::get_task(&task.id).await.unwrap();
        assert!(task_after.is_none(), "Task {} should be deleted", task.id);
    }
}