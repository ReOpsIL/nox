//! Integration tests for the registry manager module

use nox::core::registry_manager;

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