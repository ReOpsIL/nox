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

/// Test default seeding functionality
#[tokio::test]
async fn test_default_seeding() {
    setup_test_env();
    
    // Initialize registry (this should be empty)
    registry_manager::initialize().await.unwrap();
    
    // Verify registry is empty before seeding
    let agents_before = agent_manager::get_all_agents().await.unwrap();
    let tasks_before = task_manager::get_all_tasks().await.unwrap();
    assert_eq!(agents_before.len(), 0, "Registry should be empty before seeding");
    assert_eq!(tasks_before.len(), 0, "Registry should have no tasks before seeding");
    
    // Trigger seeding
    nox::core::seeding::seed_default_data().await.unwrap();
    
    // Verify agents were created
    let agents_after = agent_manager::get_all_agents().await.unwrap();
    assert_eq!(agents_after.len(), 5, "Should create exactly 5 default agents");
    
    // Verify agent names and specialties
    let agent_names: Vec<&str> = agents_after.iter().map(|a| a.name.as_str()).collect();
    assert!(agent_names.contains(&"RustCodeReviewer"));
    assert!(agent_names.contains(&"RustPerformanceOptimizer"));
    assert!(agent_names.contains(&"RustTestingEngineer"));
    assert!(agent_names.contains(&"RustSystemArchitect"));
    assert!(agent_names.contains(&"RustWebDeveloper"));
    
    // Verify tasks were created
    let tasks_after = task_manager::get_all_tasks().await.unwrap();
    assert_eq!(tasks_after.len(), 25, "Should create exactly 25 default tasks (5 per agent)");
    
    // Verify each agent has exactly 5 tasks
    for agent in &agents_after {
        let agent_tasks = task_manager::get_tasks_by_agent(&agent.id).await.unwrap();
        assert_eq!(agent_tasks.len(), 5, "Each agent should have exactly 5 tasks");
        
        // Verify all tasks belong to this agent
        for task in &agent_tasks {
            assert_eq!(task.agent_id, agent.id, "Task should belong to the correct agent");
        }
    }
    
    // Verify seeding is idempotent (running again should not create duplicates)
    nox::core::seeding::seed_default_data().await.unwrap();
    
    let agents_after_second = agent_manager::get_all_agents().await.unwrap();
    let tasks_after_second = task_manager::get_all_tasks().await.unwrap();
    assert_eq!(agents_after_second.len(), 5, "Second seeding should not create duplicate agents");
    assert_eq!(tasks_after_second.len(), 25, "Second seeding should not create duplicate tasks");
}