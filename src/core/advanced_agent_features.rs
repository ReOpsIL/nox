//! Advanced agent features module for the Nox agent ecosystem
//! 
//! This module implements advanced agent capabilities such as self-modification,
//! dynamic prompt evolution, and the ability for agents to spawn other agents.

use crate::core::{agent_manager, claude_process_manager, message_broker, registry_manager};
use crate::types::Agent;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

// Singleton instance of the advanced agent features manager
lazy_static::lazy_static! {
    static ref ADVANCED_AGENT_FEATURES: Arc<Mutex<AdvancedAgentFeatures>> = Arc::new(Mutex::new(AdvancedAgentFeatures::new()));
}

/// Prompt evolution strategy enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromptEvolutionStrategy {
    /// Incremental learning from interactions
    IncrementalLearning,
    /// Feedback-based adaptation
    FeedbackAdaptation,
    /// Performance-based optimization
    PerformanceOptimization,
    /// Task-specific specialization
    TaskSpecialization,
    /// Hybrid approach combining multiple strategies
    Hybrid,
}

impl std::fmt::Display for PromptEvolutionStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptEvolutionStrategy::IncrementalLearning => write!(f, "Incremental Learning"),
            PromptEvolutionStrategy::FeedbackAdaptation => write!(f, "Feedback Adaptation"),
            PromptEvolutionStrategy::PerformanceOptimization => write!(f, "Performance Optimization"),
            PromptEvolutionStrategy::TaskSpecialization => write!(f, "Task Specialization"),
            PromptEvolutionStrategy::Hybrid => write!(f, "Hybrid"),
        }
    }
}

/// Prompt evolution record struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptEvolutionRecord {
    /// Unique identifier for the record
    pub id: String,
    /// ID of the agent
    pub agent_id: String,
    /// Original prompt
    pub original_prompt: String,
    /// Modified prompt
    pub modified_prompt: String,
    /// Evolution strategy used
    pub strategy: PromptEvolutionStrategy,
    /// Reason for the modification
    pub reason: String,
    /// Performance metrics before modification
    pub performance_before: HashMap<String, f64>,
    /// Performance metrics after modification
    pub performance_after: Option<HashMap<String, f64>>,
    /// When the modification was made
    pub created_at: DateTime<Utc>,
    /// Whether the modification was approved
    pub approved: bool,
}

impl PromptEvolutionRecord {
    /// Create a new prompt evolution record
    fn new(
        agent_id: String,
        original_prompt: String,
        modified_prompt: String,
        strategy: PromptEvolutionStrategy,
        reason: String,
        performance_before: HashMap<String, f64>,
    ) -> Self {
        let id = format!("pe-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            agent_id,
            original_prompt,
            modified_prompt,
            strategy,
            reason,
            performance_before,
            performance_after: None,
            created_at: Utc::now(),
            approved: false,
        }
    }
}

/// Agent spawn request struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpawnRequest {
    /// Unique identifier for the request
    pub id: String,
    /// ID of the parent agent
    pub parent_agent_id: String,
    /// Name for the new agent
    pub name: String,
    /// System prompt for the new agent
    pub system_prompt: String,
    /// Reason for spawning the agent
    pub reason: String,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// Whether the request was approved
    pub approved: bool,
    /// ID of the spawned agent (if approved)
    pub spawned_agent_id: Option<String>,
}

impl AgentSpawnRequest {
    /// Create a new agent spawn request
    fn new(
        parent_agent_id: String,
        name: String,
        system_prompt: String,
        reason: String,
    ) -> Self {
        let id = format!("spawn-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            parent_agent_id,
            name,
            system_prompt,
            reason,
            created_at: Utc::now(),
            approved: false,
            spawned_agent_id: None,
        }
    }
}

/// Self-modification type enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SelfModificationType {
    /// Prompt modification
    PromptModification,
    /// Behavior adjustment
    BehaviorAdjustment,
    /// Resource allocation change
    ResourceAllocation,
    /// Capability expansion
    CapabilityExpansion,
}

impl std::fmt::Display for SelfModificationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelfModificationType::PromptModification => write!(f, "Prompt Modification"),
            SelfModificationType::BehaviorAdjustment => write!(f, "Behavior Adjustment"),
            SelfModificationType::ResourceAllocation => write!(f, "Resource Allocation"),
            SelfModificationType::CapabilityExpansion => write!(f, "Capability Expansion"),
        }
    }
}

/// Self-modification request struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModificationRequest {
    /// Unique identifier for the request
    pub id: String,
    /// ID of the agent
    pub agent_id: String,
    /// Type of modification
    pub modification_type: SelfModificationType,
    /// Description of the modification
    pub description: String,
    /// Reason for the modification
    pub reason: String,
    /// Modification details as JSON
    pub details: serde_json::Value,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// Whether the request was approved
    pub approved: bool,
    /// When the request was processed
    pub processed_at: Option<DateTime<Utc>>,
}

impl SelfModificationRequest {
    /// Create a new self-modification request
    fn new(
        agent_id: String,
        modification_type: SelfModificationType,
        description: String,
        reason: String,
        details: serde_json::Value,
    ) -> Self {
        let id = format!("mod-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            agent_id,
            modification_type,
            description,
            reason,
            details,
            created_at: Utc::now(),
            approved: false,
            processed_at: None,
        }
    }
}

/// Advanced agent features struct
pub struct AdvancedAgentFeatures {
    /// Map of agent IDs to their prompt evolution records
    prompt_evolution_records: HashMap<String, Vec<PromptEvolutionRecord>>,
    /// Map of request IDs to agent spawn requests
    agent_spawn_requests: HashMap<String, AgentSpawnRequest>,
    /// Map of request IDs to self-modification requests
    self_modification_requests: HashMap<String, SelfModificationRequest>,
    /// Whether automatic approval is enabled
    auto_approve: bool,
}

impl AdvancedAgentFeatures {
    /// Create a new advanced agent features manager
    fn new() -> Self {
        Self {
            prompt_evolution_records: HashMap::new(),
            agent_spawn_requests: HashMap::new(),
            self_modification_requests: HashMap::new(),
            auto_approve: false,
        }
    }

    /// Set automatic approval
    fn set_auto_approve(&mut self, auto_approve: bool) {
        self.auto_approve = auto_approve;
    }

    /// Request prompt evolution for an agent
    async fn request_prompt_evolution(
        &mut self,
        agent_id: &str,
        modified_prompt: String,
        strategy: PromptEvolutionStrategy,
        reason: String,
        performance_metrics: HashMap<String, f64>,
    ) -> Result<PromptEvolutionRecord> {
        // Get the agent
        let agent = agent_manager::get_agent(agent_id).await?
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;

        // Create a new prompt evolution record
        let record = PromptEvolutionRecord::new(
            agent_id.to_string(),
            agent.system_prompt.clone(),
            modified_prompt.clone(),
            strategy,
            reason,
            performance_metrics,
        );

        // Store the record
        self.prompt_evolution_records
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(record.clone());

        info!("Prompt evolution requested for agent {}: {}", agent_id, record.id);

        // If auto-approve is enabled, apply the evolution immediately
        if self.auto_approve {
            self.apply_prompt_evolution(&record.id).await?;
        }

        Ok(record)
    }

    /// Apply a prompt evolution
    async fn apply_prompt_evolution(&mut self, record_id: &str) -> Result<()> {
        // Find the record
        let mut record = None;
        let mut agent_id = String::new();

        for (id, records) in &mut self.prompt_evolution_records {
            if let Some(pos) = records.iter().position(|r| r.id == record_id) {
                record = Some(records[pos].clone());
                agent_id = id.clone();
                records[pos].approved = true;
                break;
            }
        }

        let record = record.ok_or_else(|| anyhow!("Prompt evolution record not found: {}", record_id))?;

        // Get the agent
        let mut agent = agent_manager::get_agent(&agent_id).await?
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;

        // Update the agent's system prompt
        agent.system_prompt = record.modified_prompt.clone();

        // Save the updated agent
        agent_manager::update_agent(agent).await?;

        info!("Applied prompt evolution for agent {}: {}", agent_id, record_id);
        Ok(())
    }

    /// Request an agent to spawn a new agent
    async fn request_agent_spawn(
        &mut self,
        parent_agent_id: &str,
        name: String,
        system_prompt: String,
        reason: String,
    ) -> Result<AgentSpawnRequest> {
        // Check if the parent agent exists
        let _parent_agent = agent_manager::get_agent(parent_agent_id).await?
            .ok_or_else(|| anyhow!("Parent agent not found: {}", parent_agent_id))?;

        // Create a new agent spawn request
        let request = AgentSpawnRequest::new(
            parent_agent_id.to_string(),
            name.clone(),
            system_prompt.clone(),
            reason,
        );

        // Store the request
        self.agent_spawn_requests.insert(request.id.clone(), request.clone());

        info!("Agent spawn requested by agent {}: {}", parent_agent_id, request.id);

        // If auto-approve is enabled, spawn the agent immediately
        if self.auto_approve {
            self.approve_agent_spawn(&request.id).await?;
        }

        Ok(request)
    }

    /// Approve an agent spawn request
    async fn approve_agent_spawn(&mut self, request_id: &str) -> Result<Agent> {
        // Get the request
        let request = self.agent_spawn_requests.get_mut(request_id)
            .ok_or_else(|| anyhow!("Agent spawn request not found: {}", request_id))?;

        // Create the new agent
        let agent = Agent::new(request.name.clone(), request.system_prompt.clone());

        // Add the agent to the registry
        agent_manager::add_agent(agent.clone()).await?;

        // Update the request
        request.approved = true;
        request.spawned_agent_id = Some(agent.id.clone());

        info!("Approved agent spawn request {}: created agent {}", request_id, agent.id);
        Ok(agent)
    }

    /// Reject an agent spawn request
    fn reject_agent_spawn(&mut self, request_id: &str) -> Result<()> {
        // Get the request
        let request = self.agent_spawn_requests.get_mut(request_id)
            .ok_or_else(|| anyhow!("Agent spawn request not found: {}", request_id))?;

        // Update the request
        request.approved = false;

        info!("Rejected agent spawn request {}", request_id);
        Ok(())
    }

    /// Request self-modification for an agent
    async fn request_self_modification(
        &mut self,
        agent_id: &str,
        modification_type: SelfModificationType,
        description: String,
        reason: String,
        details: serde_json::Value,
    ) -> Result<SelfModificationRequest> {
        // Check if the agent exists
        let _agent = agent_manager::get_agent(agent_id).await?
            .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;

        // Create a new self-modification request
        let request = SelfModificationRequest::new(
            agent_id.to_string(),
            modification_type,
            description,
            reason,
            details,
        );

        // Store the request
        self.self_modification_requests.insert(request.id.clone(), request.clone());

        info!("Self-modification requested by agent {}: {}", agent_id, request.id);

        // If auto-approve is enabled, apply the modification immediately
        if self.auto_approve {
            self.apply_self_modification(&request.id).await?;
        }

        Ok(request)
    }

    /// Apply a self-modification request
    async fn apply_self_modification(&mut self, request_id: &str) -> Result<()> {
        // Get the request
        let request = self.self_modification_requests.get_mut(request_id)
            .ok_or_else(|| anyhow!("Self-modification request not found: {}", request_id))?;

        // Mark as approved
        request.approved = true;
        request.processed_at = Some(Utc::now());

        // Get the agent
        let mut agent = agent_manager::get_agent(&request.agent_id).await?
            .ok_or_else(|| anyhow!("Agent not found: {}", request.agent_id))?;

        // Apply the modification based on type
        match request.modification_type {
            SelfModificationType::PromptModification => {
                // Extract the new prompt from the details
                let new_prompt = request.details.get("new_prompt")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow!("New prompt not found in modification details"))?;

                // Update the agent's system prompt
                agent.system_prompt = new_prompt.to_string();
                agent_manager::update_agent(agent).await?;

                info!("Applied prompt modification for agent {}", request.agent_id);
            },
            SelfModificationType::ResourceAllocation => {
                // Extract the resource limits from the details
                let cpu_percent = request.details.get("cpu_percent")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow!("CPU percent not found in modification details"))?;
                
                let memory_mb = request.details.get("memory_mb")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| anyhow!("Memory MB not found in modification details"))?;

                // Update the agent's resource limits
                agent.resource_limits.max_cpu_percent = cpu_percent as u8;
                agent.resource_limits.max_memory_mb = memory_mb as u32;
                agent_manager::update_agent(agent).await?;

                info!("Applied resource allocation modification for agent {}", request.agent_id);
            },
            SelfModificationType::BehaviorAdjustment => {
                // This is a more complex modification that might involve multiple changes
                // For now, we'll just log it
                info!("Applied behavior adjustment for agent {}", request.agent_id);
            },
            SelfModificationType::CapabilityExpansion => {
                // This might involve assigning MCP services to the agent
                if let Some(service_id) = request.details.get("service_id").and_then(|v| v.as_str()) {
                    // Assign the service to the agent
                    crate::core::mcp_manager::assign_service_to_agent(service_id, &request.agent_id).await?;
                    info!("Assigned service {} to agent {}", service_id, request.agent_id);
                }
            },
        }

        Ok(())
    }

    /// Reject a self-modification request
    fn reject_self_modification(&mut self, request_id: &str) -> Result<()> {
        // Get the request
        let request = self.self_modification_requests.get_mut(request_id)
            .ok_or_else(|| anyhow!("Self-modification request not found: {}", request_id))?;

        // Update the request
        request.approved = false;
        request.processed_at = Some(Utc::now());

        info!("Rejected self-modification request {}", request_id);
        Ok(())
    }

    /// Get all prompt evolution records for an agent
    fn get_prompt_evolution_records(&self, agent_id: &str) -> Vec<PromptEvolutionRecord> {
        self.prompt_evolution_records
            .get(agent_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Get all agent spawn requests by a parent agent
    fn get_agent_spawn_requests(&self, parent_agent_id: &str) -> Vec<AgentSpawnRequest> {
        self.agent_spawn_requests
            .values()
            .filter(|r| r.parent_agent_id == parent_agent_id)
            .cloned()
            .collect()
    }

    /// Get all self-modification requests for an agent
    fn get_self_modification_requests(&self, agent_id: &str) -> Vec<SelfModificationRequest> {
        self.self_modification_requests
            .values()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }

    /// Get all pending approval requests
    fn get_pending_approval_requests(&self) -> (Vec<PromptEvolutionRecord>, Vec<AgentSpawnRequest>, Vec<SelfModificationRequest>) {
        // Get pending prompt evolution records
        let pending_evolutions: Vec<PromptEvolutionRecord> = self.prompt_evolution_records
            .values()
            .flat_map(|records| records.iter())
            .filter(|r| !r.approved)
            .cloned()
            .collect();

        // Get pending agent spawn requests
        let pending_spawns: Vec<AgentSpawnRequest> = self.agent_spawn_requests
            .values()
            .filter(|r| !r.approved && r.spawned_agent_id.is_none())
            .cloned()
            .collect();

        // Get pending self-modification requests
        let pending_modifications: Vec<SelfModificationRequest> = self.self_modification_requests
            .values()
            .filter(|r| !r.approved && r.processed_at.is_none())
            .cloned()
            .collect();

        (pending_evolutions, pending_spawns, pending_modifications)
    }
}

/// Set automatic approval for advanced agent features
pub async fn set_auto_approve(auto_approve: bool) -> Result<()> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.set_auto_approve(auto_approve);
    info!("Set auto-approve for advanced agent features to: {}", auto_approve);
    Ok(())
}

/// Request prompt evolution for an agent
pub async fn request_prompt_evolution(
    agent_id: &str,
    modified_prompt: &str,
    strategy: PromptEvolutionStrategy,
    reason: &str,
    performance_metrics: HashMap<String, f64>,
) -> Result<PromptEvolutionRecord> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.request_prompt_evolution(
        agent_id,
        modified_prompt.to_string(),
        strategy,
        reason.to_string(),
        performance_metrics,
    ).await
}

/// Apply a prompt evolution
pub async fn apply_prompt_evolution(record_id: &str) -> Result<()> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.apply_prompt_evolution(record_id).await
}

/// Request an agent to spawn a new agent
pub async fn request_agent_spawn(
    parent_agent_id: &str,
    name: &str,
    system_prompt: &str,
    reason: &str,
) -> Result<AgentSpawnRequest> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.request_agent_spawn(
        parent_agent_id,
        name.to_string(),
        system_prompt.to_string(),
        reason.to_string(),
    ).await
}

/// Approve an agent spawn request
pub async fn approve_agent_spawn(request_id: &str) -> Result<Agent> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.approve_agent_spawn(request_id).await
}

/// Reject an agent spawn request
pub async fn reject_agent_spawn(request_id: &str) -> Result<()> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.reject_agent_spawn(request_id)
}

/// Request self-modification for an agent
pub async fn request_self_modification(
    agent_id: &str,
    modification_type: SelfModificationType,
    description: &str,
    reason: &str,
    details: serde_json::Value,
) -> Result<SelfModificationRequest> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.request_self_modification(
        agent_id,
        modification_type,
        description.to_string(),
        reason.to_string(),
        details,
    ).await
}

/// Apply a self-modification request
pub async fn apply_self_modification(request_id: &str) -> Result<()> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.apply_self_modification(request_id).await
}

/// Reject a self-modification request
pub async fn reject_self_modification(request_id: &str) -> Result<()> {
    let mut features = ADVANCED_AGENT_FEATURES.lock().await;
    features.reject_self_modification(request_id)
}

/// Get all prompt evolution records for an agent
pub async fn get_prompt_evolution_records(agent_id: &str) -> Result<Vec<PromptEvolutionRecord>> {
    let features = ADVANCED_AGENT_FEATURES.lock().await;
    Ok(features.get_prompt_evolution_records(agent_id))
}

/// Get all agent spawn requests by a parent agent
pub async fn get_agent_spawn_requests(parent_agent_id: &str) -> Result<Vec<AgentSpawnRequest>> {
    let features = ADVANCED_AGENT_FEATURES.lock().await;
    Ok(features.get_agent_spawn_requests(parent_agent_id))
}

/// Get all self-modification requests for an agent
pub async fn get_self_modification_requests(agent_id: &str) -> Result<Vec<SelfModificationRequest>> {
    let features = ADVANCED_AGENT_FEATURES.lock().await;
    Ok(features.get_self_modification_requests(agent_id))
}

/// Get all pending approval requests
pub async fn get_pending_approval_requests() -> Result<(Vec<PromptEvolutionRecord>, Vec<AgentSpawnRequest>, Vec<SelfModificationRequest>)> {
    let features = ADVANCED_AGENT_FEATURES.lock().await;
    Ok(features.get_pending_approval_requests())
}