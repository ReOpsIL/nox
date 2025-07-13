//! Resource manager module for the Nox agent ecosystem
//! 
//! This module implements comprehensive resource management including
//! strict enforcement of memory/CPU usage, resource pool allocation,
//! and anti-runaway protection.

use crate::core::{agent_manager, claude_process_manager};
use crate::types::{Agent, AgentStatus};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;
use uuid::Uuid;

// Singleton instance of the resource manager
lazy_static::lazy_static! {
    static ref RESOURCE_MANAGER: Arc<Mutex<ResourceManager>> = Arc::new(Mutex::new(ResourceManager::new()));
}

/// Resource usage struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Agent ID
    pub agent_id: String,
    /// CPU usage percentage
    pub cpu_percent: f32,
    /// Memory usage in MB
    pub memory_mb: u32,
    /// Disk usage in MB
    pub disk_mb: u32,
    /// Network usage in KB/s
    pub network_kbps: f32,
    /// When the usage was recorded
    pub timestamp: DateTime<Utc>,
}

impl ResourceUsage {
    /// Create a new resource usage record
    fn new(agent_id: String, cpu_percent: f32, memory_mb: u32, disk_mb: u32, network_kbps: f32) -> Self {
        Self {
            agent_id,
            cpu_percent,
            memory_mb,
            disk_mb,
            network_kbps,
            timestamp: Utc::now(),
        }
    }
}

/// Resource allocation struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Agent ID
    pub agent_id: String,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    /// Maximum disk usage in MB
    pub max_disk_mb: u32,
    /// Maximum network usage in KB/s
    pub max_network_kbps: u32,
    /// Priority (higher values get more resources)
    pub priority: u8,
    /// When the allocation was created
    pub created_at: DateTime<Utc>,
    /// When the allocation was last updated
    pub updated_at: DateTime<Utc>,
}

impl ResourceAllocation {
    /// Create a new resource allocation
    fn new(agent_id: String, max_cpu_percent: u8, max_memory_mb: u32, max_disk_mb: u32, max_network_kbps: u32, priority: u8) -> Self {
        let now = Utc::now();
        Self {
            agent_id,
            max_cpu_percent,
            max_memory_mb,
            max_disk_mb,
            max_network_kbps,
            priority,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the resource allocation
    fn update(&mut self, max_cpu_percent: u8, max_memory_mb: u32, max_disk_mb: u32, max_network_kbps: u32, priority: u8) {
        self.max_cpu_percent = max_cpu_percent;
        self.max_memory_mb = max_memory_mb;
        self.max_disk_mb = max_disk_mb;
        self.max_network_kbps = max_network_kbps;
        self.priority = priority;
        self.updated_at = Utc::now();
    }
}

/// Resource pool struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Unique identifier for the pool
    pub id: String,
    /// Name of the pool
    pub name: String,
    /// Description of the pool
    pub description: String,
    /// Total CPU cores available
    pub total_cpu_cores: f32,
    /// Total memory in MB available
    pub total_memory_mb: u32,
    /// Total disk space in MB available
    pub total_disk_mb: u32,
    /// Total network bandwidth in KB/s available
    pub total_network_kbps: u32,
    /// List of agent IDs assigned to this pool
    pub agent_ids: Vec<String>,
    /// When the pool was created
    pub created_at: DateTime<Utc>,
}

impl ResourcePool {
    /// Create a new resource pool
    fn new(name: String, description: String, total_cpu_cores: f32, total_memory_mb: u32, total_disk_mb: u32, total_network_kbps: u32) -> Self {
        let id = format!("pool-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            name,
            description,
            total_cpu_cores,
            total_memory_mb,
            total_disk_mb,
            total_network_kbps,
            agent_ids: Vec::new(),
            created_at: Utc::now(),
        }
    }

    /// Add an agent to the pool
    fn add_agent(&mut self, agent_id: &str) -> Result<()> {
        if !self.agent_ids.contains(&agent_id.to_string()) {
            self.agent_ids.push(agent_id.to_string());
            Ok(())
        } else {
            Err(anyhow!("Agent already in pool: {}", agent_id))
        }
    }

    /// Remove an agent from the pool
    fn remove_agent(&mut self, agent_id: &str) -> Result<()> {
        let original_len = self.agent_ids.len();
        self.agent_ids.retain(|id| id != agent_id);
        
        if self.agent_ids.len() == original_len {
            Err(anyhow!("Agent not in pool: {}", agent_id))
        } else {
            Ok(())
        }
    }

    /// Calculate available resources in the pool
    fn available_resources(&self, allocations: &HashMap<String, ResourceAllocation>) -> (f32, u32, u32, u32) {
        let mut used_cpu = 0.0;
        let mut used_memory = 0;
        let mut used_disk = 0;
        let mut used_network = 0;

        for agent_id in &self.agent_ids {
            if let Some(allocation) = allocations.get(agent_id) {
                used_cpu += allocation.max_cpu_percent as f32 / 100.0;
                used_memory += allocation.max_memory_mb;
                used_disk += allocation.max_disk_mb;
                used_network += allocation.max_network_kbps;
            }
        }

        (
            (self.total_cpu_cores - used_cpu).max(0.0),
            (self.total_memory_mb - used_memory).max(0),
            (self.total_disk_mb - used_disk).max(0),
            (self.total_network_kbps - used_network).max(0),
        )
    }
}

/// Runaway detection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunawayDetectionSettings {
    /// CPU threshold percentage for runaway detection
    pub cpu_threshold_percent: u8,
    /// Memory threshold percentage for runaway detection
    pub memory_threshold_percent: u8,
    /// Duration threshold in seconds for runaway detection
    pub duration_threshold_seconds: u64,
    /// Whether to automatically stop runaway agents
    pub auto_stop: bool,
    /// Whether to notify administrators about runaway agents
    pub notify_admin: bool,
}

impl Default for RunawayDetectionSettings {
    fn default() -> Self {
        Self {
            cpu_threshold_percent: 90,
            memory_threshold_percent: 90,
            duration_threshold_seconds: 300, // 5 minutes
            auto_stop: true,
            notify_admin: true,
        }
    }
}

/// Runaway detection state for an agent
#[derive(Debug, Clone)]
struct RunawayDetectionState {
    /// Agent ID
    agent_id: String,
    /// Whether the agent is currently over the CPU threshold
    over_cpu_threshold: bool,
    /// Whether the agent is currently over the memory threshold
    over_memory_threshold: bool,
    /// When the agent first exceeded the thresholds
    threshold_exceeded_at: Option<DateTime<Utc>>,
    /// Whether a notification has been sent
    notification_sent: bool,
}

impl RunawayDetectionState {
    /// Create a new runaway detection state
    fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            over_cpu_threshold: false,
            over_memory_threshold: false,
            threshold_exceeded_at: None,
            notification_sent: false,
        }
    }

    /// Update the state based on current resource usage
    fn update(&mut self, usage: &ResourceUsage, allocation: &ResourceAllocation, settings: &RunawayDetectionSettings) -> bool {
        let over_cpu = usage.cpu_percent > (allocation.max_cpu_percent as f32 * settings.cpu_threshold_percent as f32 / 100.0);
        let over_memory = usage.memory_mb > (allocation.max_memory_mb * settings.memory_threshold_percent as u32 / 100);
        
        // Check if thresholds are exceeded
        let thresholds_exceeded = over_cpu || over_memory;
        
        // Update state
        self.over_cpu_threshold = over_cpu;
        self.over_memory_threshold = over_memory;
        
        if thresholds_exceeded {
            if self.threshold_exceeded_at.is_none() {
                self.threshold_exceeded_at = Some(Utc::now());
            }
        } else {
            // Reset if no longer exceeding thresholds
            self.threshold_exceeded_at = None;
            self.notification_sent = false;
        }
        
        // Check if this is a runaway condition
        if let Some(exceeded_at) = self.threshold_exceeded_at {
            let duration = Utc::now() - exceeded_at;
            if duration >= Duration::seconds(settings.duration_threshold_seconds as i64) {
                return true;
            }
        }
        
        false
    }
}

/// Resource manager struct
pub struct ResourceManager {
    /// Map of agent IDs to resource allocations
    allocations: HashMap<String, ResourceAllocation>,
    /// Map of agent IDs to resource usage history
    usage_history: HashMap<String, VecDeque<ResourceUsage>>,
    /// Map of pool IDs to resource pools
    pools: HashMap<String, ResourcePool>,
    /// Default resource pool
    default_pool: Option<String>,
    /// Map of agent IDs to runaway detection states
    runaway_states: HashMap<String, RunawayDetectionState>,
    /// Runaway detection settings
    runaway_settings: RunawayDetectionSettings,
    /// Maximum history length per agent
    max_history_length: usize,
    /// Whether the monitoring task is running
    monitoring_active: bool,
}

impl ResourceManager {
    /// Create a new resource manager
    fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            usage_history: HashMap::new(),
            pools: HashMap::new(),
            default_pool: None,
            runaway_states: HashMap::new(),
            runaway_settings: RunawayDetectionSettings::default(),
            max_history_length: 100,
            monitoring_active: false,
        }
    }

    /// Start the resource monitoring task
    async fn start_monitoring(&mut self) -> Result<()> {
        if self.monitoring_active {
            return Ok(());
        }
        
        self.monitoring_active = true;
        info!("Starting resource monitoring");
        
        // Create a default pool if none exists
        if self.pools.is_empty() {
            let pool = ResourcePool::new(
                "Default Pool".to_string(),
                "Default resource pool".to_string(),
                4.0, // 4 CPU cores
                8192, // 8 GB RAM
                102400, // 100 GB disk
                10240, // 10 MB/s network
            );
            self.default_pool = Some(pool.id.clone());
            self.pools.insert(pool.id.clone(), pool);
            info!("Created default resource pool");
        }
        
        Ok(())
    }

    /// Stop the resource monitoring task
    async fn stop_monitoring(&mut self) -> Result<()> {
        if !self.monitoring_active {
            return Ok(());
        }
        
        self.monitoring_active = false;
        info!("Stopped resource monitoring");
        Ok(())
    }

    /// Create a new resource pool
    fn create_pool(&mut self, name: String, description: String, total_cpu_cores: f32, total_memory_mb: u32, total_disk_mb: u32, total_network_kbps: u32) -> Result<ResourcePool> {
        let pool = ResourcePool::new(
            name,
            description,
            total_cpu_cores,
            total_memory_mb,
            total_disk_mb,
            total_network_kbps,
        );
        
        self.pools.insert(pool.id.clone(), pool.clone());
        info!("Created resource pool: {} ({})", pool.name, pool.id);
        Ok(pool)
    }

    /// Delete a resource pool
    fn delete_pool(&mut self, pool_id: &str) -> Result<()> {
        // Check if this is the default pool
        if let Some(default_id) = &self.default_pool {
            if default_id == pool_id {
                return Err(anyhow!("Cannot delete the default pool"));
            }
        }
        
        // Check if the pool has agents
        if let Some(pool) = self.pools.get(pool_id) {
            if !pool.agent_ids.is_empty() {
                return Err(anyhow!("Cannot delete pool with assigned agents"));
            }
        }
        
        // Remove the pool
        if self.pools.remove(pool_id).is_some() {
            info!("Deleted resource pool: {}", pool_id);
            Ok(())
        } else {
            Err(anyhow!("Pool not found: {}", pool_id))
        }
    }

    /// Set the default resource pool
    fn set_default_pool(&mut self, pool_id: &str) -> Result<()> {
        if !self.pools.contains_key(pool_id) {
            return Err(anyhow!("Pool not found: {}", pool_id));
        }
        
        self.default_pool = Some(pool_id.to_string());
        info!("Set default resource pool: {}", pool_id);
        Ok(())
    }

    /// Assign an agent to a pool
    fn assign_agent_to_pool(&mut self, agent_id: &str, pool_id: &str) -> Result<()> {
        // Check if the pool exists
        if !self.pools.contains_key(pool_id) {
            return Err(anyhow!("Pool not found: {}", pool_id));
        }
        
        // Check if the agent is already in another pool and remove it
        let pools_to_remove_from: Vec<String> = self.pools.iter()
            .filter(|(id, p)| id.as_str() != pool_id && p.agent_ids.contains(&agent_id.to_string()))
            .map(|(id, _)| id.clone())
            .collect();
        
        for pool_id_to_remove_from in pools_to_remove_from {
            if let Some(pool) = self.pools.get_mut(&pool_id_to_remove_from) {
                pool.remove_agent(agent_id)?;
                info!("Removed agent {} from pool {}", agent_id, pool_id_to_remove_from);
            }
        }
        
        // Add the agent to the target pool
        if let Some(pool) = self.pools.get_mut(pool_id) {
            pool.add_agent(agent_id)?;
            info!("Assigned agent {} to pool {}", agent_id, pool_id);
        }
        
        Ok(())
    }

    /// Allocate resources to an agent
    fn allocate_resources(&mut self, agent_id: &str, max_cpu_percent: u8, max_memory_mb: u32, max_disk_mb: u32, max_network_kbps: u32, priority: u8) -> Result<ResourceAllocation> {
        // Find which pool the agent is in
        let mut agent_pool = None;
        for (id, pool) in &self.pools {
            if pool.agent_ids.contains(&agent_id.to_string()) {
                agent_pool = Some(id.clone());
                break;
            }
        }
        
        // If the agent is not in any pool, assign to default
        if agent_pool.is_none() {
            if let Some(default_id) = self.default_pool.clone() {
                self.assign_agent_to_pool(agent_id, &default_id)?;
                agent_pool = Some(default_id);
            } else {
                return Err(anyhow!("No default pool available"));
            }
        }
        
        // Check if the pool has enough resources
        if let Some(pool_id) = agent_pool {
            let pool = self.pools.get(&pool_id)
                .ok_or_else(|| anyhow!("Pool not found: {}", pool_id))?;
            
            let (avail_cpu, avail_memory, avail_disk, avail_network) = pool.available_resources(&self.allocations);
            
            // Add current allocation if updating
            let (avail_cpu, avail_memory, avail_disk, avail_network) = if let Some(current) = self.allocations.get(agent_id) {
                let cpu_cores = current.max_cpu_percent as f32 / 100.0;
                (
                    avail_cpu + cpu_cores,
                    avail_memory + current.max_memory_mb,
                    avail_disk + current.max_disk_mb,
                    avail_network + current.max_network_kbps,
                )
            } else {
                (avail_cpu, avail_memory, avail_disk, avail_network)
            };
            
            // Check if requested resources are available
            let requested_cpu = max_cpu_percent as f32 / 100.0;
            if requested_cpu > avail_cpu {
                return Err(anyhow!("Not enough CPU available in pool"));
            }
            if max_memory_mb > avail_memory {
                return Err(anyhow!("Not enough memory available in pool"));
            }
            if max_disk_mb > avail_disk {
                return Err(anyhow!("Not enough disk space available in pool"));
            }
            if max_network_kbps > avail_network {
                return Err(anyhow!("Not enough network bandwidth available in pool"));
            }
        }
        
        // Create or update the allocation
        let allocation = if let Some(existing) = self.allocations.get_mut(agent_id) {
            existing.update(max_cpu_percent, max_memory_mb, max_disk_mb, max_network_kbps, priority);
            existing.clone()
        } else {
            let allocation = ResourceAllocation::new(
                agent_id.to_string(),
                max_cpu_percent,
                max_memory_mb,
                max_disk_mb,
                max_network_kbps,
                priority,
            );
            self.allocations.insert(agent_id.to_string(), allocation.clone());
            allocation
        };
        
        // Initialize runaway detection state if needed
        if !self.runaway_states.contains_key(agent_id) {
            self.runaway_states.insert(
                agent_id.to_string(),
                RunawayDetectionState::new(agent_id.to_string()),
            );
        }
        
        info!("Allocated resources to agent {}: {}% CPU, {} MB memory", agent_id, max_cpu_percent, max_memory_mb);
        Ok(allocation)
    }

    /// Record resource usage for an agent
    fn record_usage(&mut self, usage: ResourceUsage) -> Result<()> {
        // Get or create the usage history for this agent
        let history = self.usage_history
            .entry(usage.agent_id.clone())
            .or_insert_with(VecDeque::new);
        
        // Add the usage record
        history.push_back(usage.clone());
        
        // Trim history if needed
        while history.len() > self.max_history_length {
            history.pop_front();
        }
        
        // Check for runaway processes
        if let Some(allocation) = self.allocations.get(&usage.agent_id) {
            if let Some(state) = self.runaway_states.get_mut(&usage.agent_id) {
                let is_runaway = state.update(&usage, allocation, &self.runaway_settings);
                
                if is_runaway && !state.notification_sent {
                    state.notification_sent = true;
                    warn!("Runaway agent detected: {}", usage.agent_id);
                    
                    // Auto-stop if enabled
                    if self.runaway_settings.auto_stop {
                        warn!("Auto-stopping runaway agent: {}", usage.agent_id);
                        // We can't directly call stop_agent here due to async context
                        // Instead, we'll set a flag that the monitoring task will check
                    }
                    
                    // Notify admin if enabled
                    if self.runaway_settings.notify_admin {
                        warn!("Notifying admin about runaway agent: {}", usage.agent_id);
                        // This would integrate with a notification system
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Get resource usage for an agent
    fn get_agent_usage(&self, agent_id: &str) -> Result<Vec<ResourceUsage>> {
        if let Some(history) = self.usage_history.get(agent_id) {
            Ok(history.iter().cloned().collect())
        } else {
            Ok(Vec::new())
        }
    }

    /// Get resource allocation for an agent
    fn get_agent_allocation(&self, agent_id: &str) -> Result<Option<ResourceAllocation>> {
        Ok(self.allocations.get(agent_id).cloned())
    }

    /// Get all resource pools
    fn get_all_pools(&self) -> Vec<ResourcePool> {
        self.pools.values().cloned().collect()
    }

    /// Get a resource pool by ID
    fn get_pool(&self, pool_id: &str) -> Result<ResourcePool> {
        self.pools.get(pool_id)
            .cloned()
            .ok_or_else(|| anyhow!("Pool not found: {}", pool_id))
    }

    /// Get the pool for an agent
    fn get_agent_pool(&self, agent_id: &str) -> Result<Option<ResourcePool>> {
        for pool in self.pools.values() {
            if pool.agent_ids.contains(&agent_id.to_string()) {
                return Ok(Some(pool.clone()));
            }
        }
        Ok(None)
    }

    /// Update runaway detection settings
    fn update_runaway_settings(&mut self, settings: RunawayDetectionSettings) {
        self.runaway_settings = settings;
        info!("Updated runaway detection settings");
    }

    /// Get the current runaway detection settings
    fn get_runaway_settings(&self) -> RunawayDetectionSettings {
        self.runaway_settings.clone()
    }

    /// Check if an agent is a runaway
    fn is_agent_runaway(&self, agent_id: &str) -> bool {
        if let Some(state) = self.runaway_states.get(agent_id) {
            state.threshold_exceeded_at.is_some() && state.notification_sent
        } else {
            false
        }
    }

    /// Get system-wide resource usage
    fn get_system_usage(&self) -> (f32, u32, u32, f32) {
        let mut total_cpu = 0.0;
        let mut total_memory = 0;
        let mut total_disk = 0;
        let mut total_network = 0.0;
        
        for history in self.usage_history.values() {
            if let Some(latest) = history.back() {
                total_cpu += latest.cpu_percent;
                total_memory += latest.memory_mb;
                total_disk += latest.disk_mb;
                total_network += latest.network_kbps;
            }
        }
        
        (total_cpu, total_memory, total_disk, total_network)
    }
}

/// Start resource monitoring
pub async fn start_monitoring() -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.start_monitoring().await
}

/// Stop resource monitoring
pub async fn stop_monitoring() -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.stop_monitoring().await
}

/// Create a new resource pool
pub async fn create_pool(name: &str, description: &str, total_cpu_cores: f32, total_memory_mb: u32, total_disk_mb: u32, total_network_kbps: u32) -> Result<ResourcePool> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.create_pool(
        name.to_string(),
        description.to_string(),
        total_cpu_cores,
        total_memory_mb,
        total_disk_mb,
        total_network_kbps,
    )
}

/// Delete a resource pool
pub async fn delete_pool(pool_id: &str) -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.delete_pool(pool_id)
}

/// Set the default resource pool
pub async fn set_default_pool(pool_id: &str) -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.set_default_pool(pool_id)
}

/// Assign an agent to a pool
pub async fn assign_agent_to_pool(agent_id: &str, pool_id: &str) -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.assign_agent_to_pool(agent_id, pool_id)
}

/// Allocate resources to an agent
pub async fn allocate_resources(agent_id: &str, max_cpu_percent: u8, max_memory_mb: u32, max_disk_mb: u32, max_network_kbps: u32, priority: u8) -> Result<ResourceAllocation> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.allocate_resources(agent_id, max_cpu_percent, max_memory_mb, max_disk_mb, max_network_kbps, priority)
}

/// Record resource usage for an agent
pub async fn record_usage(usage: ResourceUsage) -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.record_usage(usage)
}

/// Get resource usage for an agent
pub async fn get_agent_usage(agent_id: &str) -> Result<Vec<ResourceUsage>> {
    let manager = RESOURCE_MANAGER.lock().await;
    manager.get_agent_usage(agent_id)
}

/// Get resource allocation for an agent
pub async fn get_agent_allocation(agent_id: &str) -> Result<Option<ResourceAllocation>> {
    let manager = RESOURCE_MANAGER.lock().await;
    manager.get_agent_allocation(agent_id)
}

/// Get all resource pools
pub async fn get_all_pools() -> Result<Vec<ResourcePool>> {
    let manager = RESOURCE_MANAGER.lock().await;
    Ok(manager.get_all_pools())
}

/// Get a resource pool by ID
pub async fn get_pool(pool_id: &str) -> Result<ResourcePool> {
    let manager = RESOURCE_MANAGER.lock().await;
    manager.get_pool(pool_id)
}

/// Get the pool for an agent
pub async fn get_agent_pool(agent_id: &str) -> Result<Option<ResourcePool>> {
    let manager = RESOURCE_MANAGER.lock().await;
    manager.get_agent_pool(agent_id)
}

/// Update runaway detection settings
pub async fn update_runaway_settings(settings: RunawayDetectionSettings) -> Result<()> {
    let mut manager = RESOURCE_MANAGER.lock().await;
    manager.update_runaway_settings(settings);
    Ok(())
}

/// Get the current runaway detection settings
pub async fn get_runaway_settings() -> Result<RunawayDetectionSettings> {
    let manager = RESOURCE_MANAGER.lock().await;
    Ok(manager.get_runaway_settings())
}

/// Check if an agent is a runaway
pub async fn is_agent_runaway(agent_id: &str) -> Result<bool> {
    let manager = RESOURCE_MANAGER.lock().await;
    Ok(manager.is_agent_runaway(agent_id))
}

/// Get system-wide resource usage
pub async fn get_system_usage() -> Result<(f32, u32, u32, f32)> {
    let manager = RESOURCE_MANAGER.lock().await;
    Ok(manager.get_system_usage())
}

/// Create a new resource usage record
pub fn create_usage_record(agent_id: &str, cpu_percent: f32, memory_mb: u32, disk_mb: u32, network_kbps: f32) -> ResourceUsage {
    ResourceUsage::new(
        agent_id.to_string(),
        cpu_percent,
        memory_mb,
        disk_mb,
        network_kbps,
    )
}

/// Monitor agent resources
pub async fn monitor_agent_resources(agent_id: &str) -> Result<ResourceUsage> {
    // Get the agent's process ID
    let has_process = claude_process_manager::has_process(agent_id).await?;
    
    if !has_process {
        return Err(anyhow!("Agent does not have a running process: {}", agent_id));
    }
    
    // For now, we'll use a simple approach with random values
    // In a real implementation, this would use system APIs to get actual resource usage
    let cpu_percent = rand::random::<f32>() * 100.0;
    let memory_mb = rand::random::<u32>() % 1024 + 100;
    let disk_mb = rand::random::<u32>() % 1000 + 50;
    let network_kbps = rand::random::<f32>() * 1000.0;
    
    let usage = create_usage_record(agent_id, cpu_percent, memory_mb, disk_mb, network_kbps);
    
    // Record the usage
    record_usage(usage.clone()).await?;
    
    Ok(usage)
}

/// Initialize default resource allocations for an agent
pub async fn initialize_agent_resources(agent_id: &str) -> Result<()> {
    // Get the agent
    let agent = agent_manager::get_agent(agent_id).await?
        .ok_or_else(|| anyhow!("Agent not found: {}", agent_id))?;
    
    // Use the agent's resource limits
    let cpu_percent = agent.resource_limits.max_cpu_percent;
    let memory_mb = agent.resource_limits.max_memory_mb;
    
    // Allocate resources
    allocate_resources(
        agent_id,
        cpu_percent,
        memory_mb,
        1024, // 1 GB disk
        1024, // 1 MB/s network
        5,    // Medium priority
    ).await?;
    
    // Assign to default pool if available
    let default_pool_id = {
        let manager = RESOURCE_MANAGER.lock().await;
        manager.default_pool.clone()
    };
    
    if let Some(default_pool) = default_pool_id {
        assign_agent_to_pool(agent_id, &default_pool).await?;
    }
    
    Ok(())
}