//! Types module for the Nox agent ecosystem
//! 
//! This module contains the Rust type definitions for Agent, Task, etc.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AgentStatus {
    /// Agent is active and running
    Active,
    /// Agent is inactive (not running)
    Inactive,
    /// Agent is starting up
    Starting,
    /// Agent is shutting down
    Stopping,
    /// Agent has encountered an error
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentStatus::Active => write!(f, "Active"),
            AgentStatus::Inactive => write!(f, "Inactive"),
            AgentStatus::Starting => write!(f, "Starting"),
            AgentStatus::Stopping => write!(f, "Stopping"),
            AgentStatus::Error => write!(f, "Error"),
        }
    }
}

/// Resource limits for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
    /// Maximum CPU usage as a percentage
    pub max_cpu_percent: u8,
}

/// Agent struct representing an AI agent in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent
    pub id: String,
    /// Human-readable name of the agent
    pub name: String,
    /// System prompt that defines the agent's behavior
    pub system_prompt: String,
    /// Current status of the agent
    pub status: AgentStatus,
    /// When the agent was created
    pub created_at: DateTime<Utc>,
    /// Resource limits for the agent
    pub resource_limits: ResourceLimits,
}

impl Agent {
    /// Create a new agent with the given name and system prompt
    pub fn new(name: String, system_prompt: String) -> Self {
        let id = format!("agent-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            name,
            system_prompt,
            status: AgentStatus::Inactive,
            created_at: Utc::now(),
            resource_limits: ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50,
            },
        }
    }
}

/// Task priority enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskPriority {
    /// High priority task
    High,
    /// Medium priority task
    Medium,
    /// Low priority task
    Low,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::High => write!(f, "High"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::Low => write!(f, "Low"),
        }
    }
}

/// Task status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TaskStatus {
    /// Task is pending execution
    Todo,
    /// Task is in progress
    InProgress,
    /// Task is completed
    Done,
    /// Task is cancelled
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "In Progress"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Task struct representing a task assigned to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Unique identifier for the task
    pub id: String,
    /// ID of the agent this task is assigned to
    pub agent_id: String,
    /// Title of the task
    pub title: String,
    /// Description of the task
    pub description: String,
    /// Priority of the task
    pub priority: TaskPriority,
    /// Current status of the task
    pub status: TaskStatus,
    /// When the task was created
    pub created_at: DateTime<Utc>,
    /// When the task was started (if applicable)
    pub started_at: Option<DateTime<Utc>>,
    /// When the task was completed (if applicable)
    pub completed_at: Option<DateTime<Utc>>,
    /// Progress percentage (0-100)
    pub progress: u8,
    /// Additional metadata for the task
    pub metadata: HashMap<String, String>,
}

impl Task {
    /// Create a new task with the given agent ID, title, and description
    pub fn new(agent_id: String, title: String, description: String) -> Self {
        let id = format!("task-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            agent_id,
            title,
            description,
            priority: TaskPriority::Medium,
            status: TaskStatus::Todo,
            created_at: Utc::now(),
            started_at: None,
            completed_at: None,
            progress: 0,
            metadata: HashMap::new(),
        }
    }
}

/// System status struct representing the current status of the Nox ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    /// Number of active agents
    pub active_agents: usize,
    /// Number of inactive agents
    pub inactive_agents: usize,
    /// Number of tasks by status
    pub tasks_by_status: HashMap<TaskStatus, usize>,
    /// System uptime in seconds
    pub uptime_seconds: u64,
    /// Current memory usage in MB
    pub memory_usage_mb: usize,
    /// Total memory in MB
    pub total_memory_mb: u64,
    /// Current CPU usage as a percentage
    pub cpu_usage_percent: f32,
    /// Current memory usage as a percentage
    pub memory_usage_percent: f32,
    /// Number of agents with error status
    pub error_agents: usize,
}
