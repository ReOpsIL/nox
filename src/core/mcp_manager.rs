//! MCP (Modular Capability Provider) manager module for the Nox agent ecosystem
//! 
//! This module implements a service discovery system for dynamically expanding agent capabilities
//! by discovering and integrating with external Dockerized services.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// Singleton instance of the MCP manager
lazy_static::lazy_static! {
    static ref MCP_MANAGER: Arc<Mutex<MCPManager>> = Arc::new(Mutex::new(MCPManager::new()));
}

/// MCP service status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServiceStatus {
    /// Service is available but not running
    Available,
    /// Service is starting
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service is stopped
    Stopped,
    /// Service has an error
    Error,
}

impl std::fmt::Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceStatus::Available => write!(f, "Available"),
            ServiceStatus::Starting => write!(f, "Starting"),
            ServiceStatus::Running => write!(f, "Running"),
            ServiceStatus::Stopping => write!(f, "Stopping"),
            ServiceStatus::Stopped => write!(f, "Stopped"),
            ServiceStatus::Error => write!(f, "Error"),
        }
    }
}

/// MCP capability type enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum CapabilityType {
    /// Data processing capability
    DataProcessing,
    /// Natural language processing capability
    NLP,
    /// Computer vision capability
    Vision,
    /// Audio processing capability
    Audio,
    /// Database capability
    Database,
    /// API integration capability
    APIIntegration,
    /// Custom capability
    Custom,
}

impl std::fmt::Display for CapabilityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CapabilityType::DataProcessing => write!(f, "Data Processing"),
            CapabilityType::NLP => write!(f, "Natural Language Processing"),
            CapabilityType::Vision => write!(f, "Computer Vision"),
            CapabilityType::Audio => write!(f, "Audio Processing"),
            CapabilityType::Database => write!(f, "Database"),
            CapabilityType::APIIntegration => write!(f, "API Integration"),
            CapabilityType::Custom => write!(f, "Custom"),
        }
    }
}

/// MCP service struct representing a capability provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPService {
    /// Unique identifier for the service
    pub id: String,
    /// Name of the service
    pub name: String,
    /// Description of the service
    pub description: String,
    /// Docker image for the service
    pub docker_image: String,
    /// Docker container ID (when running)
    pub container_id: Option<String>,
    /// Service status
    pub status: ServiceStatus,
    /// Service version
    pub version: String,
    /// Service capabilities
    pub capabilities: Vec<CapabilityType>,
    /// Service endpoint URL
    pub endpoint: Option<String>,
    /// Service port
    pub port: Option<u16>,
    /// Environment variables for the service
    pub env_vars: HashMap<String, String>,
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
    /// When the service was registered
    pub registered_at: DateTime<Utc>,
    /// When the service was last started
    pub last_started: Option<DateTime<Utc>>,
    /// When the service was last stopped
    pub last_stopped: Option<DateTime<Utc>>,
}

impl MCPService {
    /// Create a new MCP service
    pub fn new(
        name: String,
        description: String,
        docker_image: String,
        version: String,
        capabilities: Vec<CapabilityType>,
        port: Option<u16>,
    ) -> Self {
        let id = format!("mcp-{}", Uuid::new_v4().as_u128());
        Self {
            id,
            name,
            description,
            docker_image,
            container_id: None,
            status: ServiceStatus::Available,
            version,
            capabilities,
            endpoint: None,
            port,
            env_vars: HashMap::new(),
            resource_requirements: ResourceRequirements::default(),
            registered_at: Utc::now(),
            last_started: None,
            last_stopped: None,
        }
    }

    /// Add an environment variable to the service
    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Set resource requirements for the service
    pub fn with_resources(mut self, resources: ResourceRequirements) -> Self {
        self.resource_requirements = resources;
        self
    }
}

/// Resource requirements for an MCP service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU cores required
    pub cpu_cores: f32,
    /// Memory in MB required
    pub memory_mb: u32,
    /// Disk space in MB required
    pub disk_mb: u32,
    /// GPU required
    pub requires_gpu: bool,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu_cores: 0.5,
            memory_mb: 512,
            disk_mb: 1024,
            requires_gpu: false,
        }
    }
}

/// MCP manager struct
pub struct MCPManager {
    /// Map of service IDs to services
    services: HashMap<String, MCPService>,
    /// Map of agent IDs to service IDs they're using
    agent_services: HashMap<String, Vec<String>>,
}

impl MCPManager {
    /// Create a new MCP manager
    fn new() -> Self {
        Self {
            services: HashMap::new(),
            agent_services: HashMap::new(),
        }
    }

    /// Register a new MCP service
    async fn register_service(&mut self, service: MCPService) -> Result<MCPService> {
        // Check if the service already exists
        if self.services.contains_key(&service.id) {
            return Err(anyhow!("Service already registered with ID: {}", service.id));
        }

        // Check if the Docker image exists
        if !self.check_docker_image(&service.docker_image).await? {
            return Err(anyhow!("Docker image not found: {}", service.docker_image));
        }

        info!("Registering MCP service: {} ({})", service.name, service.id);
        self.services.insert(service.id.clone(), service.clone());
        Ok(service)
    }

    /// Check if a Docker image exists
    async fn check_docker_image(&self, image: &str) -> Result<bool> {
        let output = Command::new("docker")
            .args(&["image", "inspect", image])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        Ok(output.success())
    }

    /// Unregister an MCP service
    async fn unregister_service(&mut self, service_id: &str) -> Result<()> {
        // Check if the service is in use by any agents
        for (agent_id, services) in &self.agent_services {
            if services.contains(&service_id.to_string()) {
                return Err(anyhow!("Service is in use by agent: {}", agent_id));
            }
        }

        // Check if the service is running
        if let Some(service) = self.services.get(service_id) {
            if service.status == ServiceStatus::Running || service.status == ServiceStatus::Starting {
                return Err(anyhow!("Service is running. Stop it before unregistering."));
            }
        }

        // Remove the service
        if self.services.remove(service_id).is_some() {
            info!("Unregistered MCP service: {}", service_id);
            Ok(())
        } else {
            Err(anyhow!("Service not found: {}", service_id))
        }
    }

    /// Get an MCP service by ID
    fn get_service(&self, service_id: &str) -> Result<MCPService> {
        self.services.get(service_id)
            .cloned()
            .ok_or_else(|| anyhow!("Service not found: {}", service_id))
    }

    /// Get all MCP services
    fn get_all_services(&self) -> Vec<MCPService> {
        self.services.values().cloned().collect()
    }

    /// Start an MCP service
    async fn start_service(&mut self, service_id: &str) -> Result<()> {
        let service = self.get_service(service_id)?;

        // Check if the service is already running
        if service.status == ServiceStatus::Running || service.status == ServiceStatus::Starting {
            return Err(anyhow!("Service is already running: {}", service.name));
        }

        info!("Starting MCP service: {} ({})", service.name, service.id);

        // Update service status to Starting
        if let Some(service) = self.services.get_mut(service_id) {
            service.status = ServiceStatus::Starting;
        }

        // Build the Docker run command
        let mut cmd = Command::new("docker");
        cmd.arg("run")
            .arg("--detach")
            .arg("--rm");

        // Add port mapping if specified
        if let Some(port) = service.port {
            cmd.arg("-p")
                .arg(format!("{}:{}", port, port));
        }

        // Add environment variables
        for (key, value) in &service.env_vars {
            cmd.arg("-e")
                .arg(format!("{}={}", key, value));
        }

        // Add resource limits
        cmd.arg("--cpus")
            .arg(service.resource_requirements.cpu_cores.to_string())
            .arg("--memory")
            .arg(format!("{}m", service.resource_requirements.memory_mb));

        // Add the image name
        cmd.arg(&service.docker_image);

        // Run the container
        let output = cmd.output().await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to start service {}: {}", service.name, error_msg);

            // Update service status to Error
            if let Some(service) = self.services.get_mut(service_id) {
                service.status = ServiceStatus::Error;
            }

            return Err(anyhow!("Failed to start service: {}", error_msg));
        }

        // Get the container ID
        let container_id = String::from_utf8_lossy(&output.stdout).trim().to_string();
        debug!("Started container: {}", container_id);

        // Update the service with the container ID and status
        if let Some(service) = self.services.get_mut(service_id) {
            service.container_id = Some(container_id);
            service.status = ServiceStatus::Running;
            service.last_started = Some(Utc::now());

            // Set the endpoint URL if port is specified
            if let Some(port) = service.port {
                service.endpoint = Some(format!("http://localhost:{}", port));
            }
        }

        info!("MCP service started: {} ({})", service.name, service.id);
        Ok(())
    }

    /// Stop an MCP service
    async fn stop_service(&mut self, service_id: &str) -> Result<()> {
        let service = self.get_service(service_id)?;

        // Check if the service is running
        if service.status != ServiceStatus::Running {
            return Err(anyhow!("Service is not running: {}", service.name));
        }

        // Get the container ID
        let container_id = service.container_id
            .clone()
            .ok_or_else(|| anyhow!("No container ID found for service: {}", service.name))?;

        info!("Stopping MCP service: {} ({})", service.name, service.id);

        // Update service status to Stopping
        if let Some(service) = self.services.get_mut(service_id) {
            service.status = ServiceStatus::Stopping;
        }

        // Stop the container
        let output = Command::new("docker")
            .args(&["stop", &container_id])
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            error!("Failed to stop service {}: {}", service.name, error_msg);

            // Update service status to Error
            if let Some(service) = self.services.get_mut(service_id) {
                service.status = ServiceStatus::Error;
            }

            return Err(anyhow!("Failed to stop service: {}", error_msg));
        }

        // Update the service status
        if let Some(service) = self.services.get_mut(service_id) {
            service.container_id = None;
            service.status = ServiceStatus::Stopped;
            service.last_stopped = Some(Utc::now());
            service.endpoint = None;
        }

        info!("MCP service stopped: {} ({})", service.name, service.id);
        Ok(())
    }

    /// Assign a service to an agent
    fn assign_service_to_agent(&mut self, service_id: &str, agent_id: &str) -> Result<()> {
        // Check if the service exists
        if !self.services.contains_key(service_id) {
            return Err(anyhow!("Service not found: {}", service_id));
        }

        // Add the service to the agent's list
        self.agent_services
            .entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(service_id.to_string());

        info!("Assigned service {} to agent {}", service_id, agent_id);
        Ok(())
    }

    /// Unassign a service from an agent
    fn unassign_service_from_agent(&mut self, service_id: &str, agent_id: &str) -> Result<()> {
        // Check if the agent has any services
        if let Some(services) = self.agent_services.get_mut(agent_id) {
            // Remove the service from the agent's list
            let original_len = services.len();
            services.retain(|id| id != service_id);

            if services.len() == original_len {
                return Err(anyhow!("Service {} not assigned to agent {}", service_id, agent_id));
            }

            info!("Unassigned service {} from agent {}", service_id, agent_id);
            Ok(())
        } else {
            Err(anyhow!("Agent has no assigned services: {}", agent_id))
        }
    }

    /// Get all services assigned to an agent
    fn get_agent_services(&self, agent_id: &str) -> Vec<MCPService> {
        if let Some(service_ids) = self.agent_services.get(agent_id) {
            service_ids.iter()
                .filter_map(|id| self.services.get(id))
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Find services by capability
    fn find_services_by_capability(&self, capability: CapabilityType) -> Vec<MCPService> {
        self.services.values()
            .filter(|service| service.capabilities.contains(&capability))
            .cloned()
            .collect()
    }

    /// Check the health of a service
    async fn check_service_health(&self, service_id: &str) -> Result<bool> {
        let service = self.get_service(service_id)?;

        // If the service is not running, it's not healthy
        if service.status != ServiceStatus::Running {
            return Ok(false);
        }

        // If the service has no container ID, it's not healthy
        let container_id = match &service.container_id {
            Some(id) => id,
            None => return Ok(false),
        };

        // Check if the container is running
        let output = Command::new("docker")
            .args(&["inspect", "--format={{.State.Running}}", container_id])
            .output()
            .await?;

        if !output.status.success() {
            return Ok(false);
        }

        let running = String::from_utf8_lossy(&output.stdout).trim() == "true";
        Ok(running)
    }
}

/// Register a new MCP service
pub async fn register_service(service: MCPService) -> Result<MCPService> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.register_service(service).await
}

/// Unregister an MCP service
pub async fn unregister_service(service_id: &str) -> Result<()> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.unregister_service(service_id).await
}

/// Get an MCP service by ID
pub async fn get_service(service_id: &str) -> Result<MCPService> {
    let manager = MCP_MANAGER.lock().await;
    manager.get_service(service_id)
}

/// Get all MCP services
pub async fn get_all_services() -> Result<Vec<MCPService>> {
    let manager = MCP_MANAGER.lock().await;
    Ok(manager.get_all_services())
}

/// Start an MCP service
pub async fn start_service(service_id: &str) -> Result<()> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.start_service(service_id).await
}

/// Stop an MCP service
pub async fn stop_service(service_id: &str) -> Result<()> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.stop_service(service_id).await
}

/// Assign a service to an agent
pub async fn assign_service_to_agent(service_id: &str, agent_id: &str) -> Result<()> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.assign_service_to_agent(service_id, agent_id)
}

/// Unassign a service from an agent
pub async fn unassign_service_from_agent(service_id: &str, agent_id: &str) -> Result<()> {
    let mut manager = MCP_MANAGER.lock().await;
    manager.unassign_service_from_agent(service_id, agent_id)
}

/// Get all services assigned to an agent
pub async fn get_agent_services(agent_id: &str) -> Result<Vec<MCPService>> {
    let manager = MCP_MANAGER.lock().await;
    Ok(manager.get_agent_services(agent_id))
}

/// Find services by capability
pub async fn find_services_by_capability(capability: CapabilityType) -> Result<Vec<MCPService>> {
    let manager = MCP_MANAGER.lock().await;
    Ok(manager.find_services_by_capability(capability))
}

/// Check the health of a service
pub async fn check_service_health(service_id: &str) -> Result<bool> {
    let manager = MCP_MANAGER.lock().await;
    manager.check_service_health(service_id).await
}

/// Create a new MCP service
pub fn create_service(
    name: &str,
    description: &str,
    docker_image: &str,
    version: &str,
    capabilities: Vec<CapabilityType>,
    port: Option<u16>,
) -> MCPService {
    MCPService::new(
        name.to_string(),
        description.to_string(),
        docker_image.to_string(),
        version.to_string(),
        capabilities,
        port,
    )
}