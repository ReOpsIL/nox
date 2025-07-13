//! Test fixtures and sample data for the Nox test suite

use chrono::Utc;
use nox::types::{Agent, AgentStatus, ResourceLimits, Task, TaskPriority, TaskStatus};
use std::collections::HashMap;
use uuid::Uuid;

/// Create a sample agent for testing
pub fn sample_agent() -> Agent {
    Agent {
        id: "test-agent-001".to_string(),
        name: "Test Agent".to_string(),
        system_prompt: "You are a helpful test agent.".to_string(),
        status: AgentStatus::Inactive,
        created_at: Utc::now(),
        resource_limits: ResourceLimits {
            max_memory_mb: 512,
            max_cpu_percent: 50,
        },
    }
}

/// Create a sample agent with custom parameters
pub fn sample_agent_with_params(id: &str, name: &str, status: AgentStatus) -> Agent {
    Agent {
        id: id.to_string(),
        name: name.to_string(),
        system_prompt: format!("You are {}, a test agent.", name),
        status,
        created_at: Utc::now(),
        resource_limits: ResourceLimits {
            max_memory_mb: 512,
            max_cpu_percent: 50,
        },
    }
}

/// Create a sample task for testing
pub fn sample_task() -> Task {
    Task {
        id: "test-task-001".to_string(),
        agent_id: "test-agent-001".to_string(),
        title: "Test Task".to_string(),
        description: "This is a test task for unit testing.".to_string(),
        status: TaskStatus::Todo,
        priority: TaskPriority::Medium,
        progress: 0,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        metadata: HashMap::new(),
    }
}

/// Create a sample task with custom parameters
pub fn sample_task_with_params(
    id: &str,
    agent_id: &str,
    title: &str,
    status: TaskStatus,
    priority: TaskPriority,
) -> Task {
    Task {
        id: id.to_string(),
        agent_id: agent_id.to_string(),
        title: title.to_string(),
        description: format!("Description for {}", title),
        status,
        priority,
        progress: 0,
        created_at: Utc::now(),
        started_at: None,
        completed_at: None,
        metadata: HashMap::new(),
    }
}

/// Generate a unique test ID
pub fn generate_test_id() -> String {
    format!("test-{}", Uuid::new_v4().to_string()[0..8].to_string())
}

/// Create multiple sample agents
pub fn sample_agents(count: usize) -> Vec<Agent> {
    (0..count)
        .map(|i| sample_agent_with_params(
            &format!("test-agent-{:03}", i),
            &format!("Test Agent {}", i),
            if i % 2 == 0 { AgentStatus::Active } else { AgentStatus::Inactive }
        ))
        .collect()
}

/// Create multiple sample tasks
pub fn sample_tasks(count: usize, agent_id: &str) -> Vec<Task> {
    let statuses = [TaskStatus::Todo, TaskStatus::InProgress, TaskStatus::Done, TaskStatus::Cancelled];
    let priorities = [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High];
    
    (0..count)
        .map(|i| sample_task_with_params(
            &format!("test-task-{:03}", i),
            agent_id,
            &format!("Test Task {}", i),
            statuses[i % statuses.len()].clone(),
            priorities[i % priorities.len()].clone()
        ))
        .collect()
}