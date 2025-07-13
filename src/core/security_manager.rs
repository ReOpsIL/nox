//! Security manager module for the Nox agent ecosystem
//! 
//! This module implements a comprehensive security framework including
//! user approval workflows, permission gates for sensitive agent actions,
//! and process sandboxing.

use crate::core::{agent_manager, claude_process_manager, registry_manager};
use crate::types::Agent;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

// Singleton instance of the security manager
lazy_static::lazy_static! {
    static ref SECURITY_MANAGER: Arc<Mutex<SecurityManager>> = Arc::new(Mutex::new(SecurityManager::new()));
}

/// Permission level enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    /// No permissions
    None = 0,
    /// Read-only permissions
    ReadOnly = 1,
    /// Basic permissions
    Basic = 2,
    /// Standard permissions
    Standard = 3,
    /// Advanced permissions
    Advanced = 4,
    /// Administrative permissions
    Admin = 5,
}

impl std::fmt::Display for PermissionLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionLevel::None => write!(f, "None"),
            PermissionLevel::ReadOnly => write!(f, "Read Only"),
            PermissionLevel::Basic => write!(f, "Basic"),
            PermissionLevel::Standard => write!(f, "Standard"),
            PermissionLevel::Advanced => write!(f, "Advanced"),
            PermissionLevel::Admin => write!(f, "Admin"),
        }
    }
}

/// Permission type enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PermissionType {
    /// Read agent information
    ReadAgent,
    /// Modify agent information
    ModifyAgent,
    /// Start/stop agents
    ControlAgent,
    /// Delete agents
    DeleteAgent,
    /// Create new agents
    CreateAgent,
    /// Read task information
    ReadTask,
    /// Create tasks
    CreateTask,
    /// Modify tasks
    ModifyTask,
    /// Delete tasks
    DeleteTask,
    /// Access file system
    FileSystem,
    /// Access network
    Network,
    /// Execute system commands
    ExecuteCommand,
    /// Modify system settings
    ModifySettings,
    /// Access sensitive data
    AccessSensitiveData,
    /// Spawn child agents
    SpawnAgent,
    /// Self-modify
    SelfModify,
}

impl std::fmt::Display for PermissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PermissionType::ReadAgent => write!(f, "Read Agent"),
            PermissionType::ModifyAgent => write!(f, "Modify Agent"),
            PermissionType::ControlAgent => write!(f, "Control Agent"),
            PermissionType::DeleteAgent => write!(f, "Delete Agent"),
            PermissionType::CreateAgent => write!(f, "Create Agent"),
            PermissionType::ReadTask => write!(f, "Read Task"),
            PermissionType::CreateTask => write!(f, "Create Task"),
            PermissionType::ModifyTask => write!(f, "Modify Task"),
            PermissionType::DeleteTask => write!(f, "Delete Task"),
            PermissionType::FileSystem => write!(f, "File System"),
            PermissionType::Network => write!(f, "Network"),
            PermissionType::ExecuteCommand => write!(f, "Execute Command"),
            PermissionType::ModifySettings => write!(f, "Modify Settings"),
            PermissionType::AccessSensitiveData => write!(f, "Access Sensitive Data"),
            PermissionType::SpawnAgent => write!(f, "Spawn Agent"),
            PermissionType::SelfModify => write!(f, "Self Modify"),
        }
    }
}

/// Security profile struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityProfile {
    /// Unique identifier for the profile
    pub id: String,
    /// Name of the profile
    pub name: String,
    /// Description of the profile
    pub description: String,
    /// Permission level
    pub permission_level: PermissionLevel,
    /// Set of allowed permissions
    pub allowed_permissions: HashSet<PermissionType>,
    /// Set of denied permissions
    pub denied_permissions: HashSet<PermissionType>,
    /// Whether approval is required for sensitive actions
    pub require_approval: bool,
    /// Whether sandboxing is enabled
    pub enable_sandboxing: bool,
    /// Resource access limits
    pub resource_limits: ResourceLimits,
    /// When the profile was created
    pub created_at: DateTime<Utc>,
    /// When the profile was last updated
    pub updated_at: DateTime<Utc>,
}

impl SecurityProfile {
    /// Create a new security profile
    fn new(name: String, description: String, permission_level: PermissionLevel) -> Self {
        let id = format!("profile-{}", Uuid::new_v4().as_u128());
        let now = Utc::now();
        
        // Set default permissions based on level
        let allowed_permissions = match permission_level {
            PermissionLevel::None => HashSet::new(),
            PermissionLevel::ReadOnly => {
                let mut perms = HashSet::new();
                perms.insert(PermissionType::ReadAgent);
                perms.insert(PermissionType::ReadTask);
                perms
            },
            PermissionLevel::Basic => {
                let mut perms = HashSet::new();
                perms.insert(PermissionType::ReadAgent);
                perms.insert(PermissionType::ReadTask);
                perms.insert(PermissionType::CreateTask);
                perms.insert(PermissionType::ModifyTask);
                perms
            },
            PermissionLevel::Standard => {
                let mut perms = HashSet::new();
                perms.insert(PermissionType::ReadAgent);
                perms.insert(PermissionType::ModifyAgent);
                perms.insert(PermissionType::ControlAgent);
                perms.insert(PermissionType::ReadTask);
                perms.insert(PermissionType::CreateTask);
                perms.insert(PermissionType::ModifyTask);
                perms.insert(PermissionType::DeleteTask);
                perms.insert(PermissionType::Network);
                perms
            },
            PermissionLevel::Advanced => {
                let mut perms = HashSet::new();
                perms.insert(PermissionType::ReadAgent);
                perms.insert(PermissionType::ModifyAgent);
                perms.insert(PermissionType::ControlAgent);
                perms.insert(PermissionType::CreateAgent);
                perms.insert(PermissionType::ReadTask);
                perms.insert(PermissionType::CreateTask);
                perms.insert(PermissionType::ModifyTask);
                perms.insert(PermissionType::DeleteTask);
                perms.insert(PermissionType::FileSystem);
                perms.insert(PermissionType::Network);
                perms.insert(PermissionType::SpawnAgent);
                perms.insert(PermissionType::SelfModify);
                perms
            },
            PermissionLevel::Admin => {
                let mut perms = HashSet::new();
                perms.insert(PermissionType::ReadAgent);
                perms.insert(PermissionType::ModifyAgent);
                perms.insert(PermissionType::ControlAgent);
                perms.insert(PermissionType::DeleteAgent);
                perms.insert(PermissionType::CreateAgent);
                perms.insert(PermissionType::ReadTask);
                perms.insert(PermissionType::CreateTask);
                perms.insert(PermissionType::ModifyTask);
                perms.insert(PermissionType::DeleteTask);
                perms.insert(PermissionType::FileSystem);
                perms.insert(PermissionType::Network);
                perms.insert(PermissionType::ExecuteCommand);
                perms.insert(PermissionType::ModifySettings);
                perms.insert(PermissionType::AccessSensitiveData);
                perms.insert(PermissionType::SpawnAgent);
                perms.insert(PermissionType::SelfModify);
                perms
            },
        };
        
        Self {
            id,
            name,
            description,
            permission_level,
            allowed_permissions,
            denied_permissions: HashSet::new(),
            require_approval: permission_level >= PermissionLevel::Advanced,
            enable_sandboxing: permission_level <= PermissionLevel::Standard,
            resource_limits: ResourceLimits::default(),
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Check if a permission is allowed
    fn is_permission_allowed(&self, permission: PermissionType) -> bool {
        if self.denied_permissions.contains(&permission) {
            return false;
        }
        
        self.allowed_permissions.contains(&permission)
    }
    
    /// Add a permission to the allowed set
    fn allow_permission(&mut self, permission: PermissionType) {
        self.allowed_permissions.insert(permission);
        self.denied_permissions.remove(&permission);
        self.updated_at = Utc::now();
    }
    
    /// Add a permission to the denied set
    fn deny_permission(&mut self, permission: PermissionType) {
        self.denied_permissions.insert(permission);
        self.allowed_permissions.remove(&permission);
        self.updated_at = Utc::now();
    }
    
    /// Set resource limits
    fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
        self.updated_at = Utc::now();
    }
}

/// Resource limits struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum file system access (None, ReadOnly, ReadWrite)
    pub file_system_access: FileSystemAccess,
    /// Maximum network access (None, Limited, Full)
    pub network_access: NetworkAccess,
    /// Maximum memory usage in MB
    pub max_memory_mb: u32,
    /// Maximum CPU usage percentage
    pub max_cpu_percent: u8,
    /// Maximum disk usage in MB
    pub max_disk_mb: u32,
    /// Maximum execution time in seconds
    pub max_execution_time_seconds: u64,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            file_system_access: FileSystemAccess::ReadOnly,
            network_access: NetworkAccess::Limited,
            max_memory_mb: 512,
            max_cpu_percent: 50,
            max_disk_mb: 1024,
            max_execution_time_seconds: 3600, // 1 hour
        }
    }
}

/// File system access enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FileSystemAccess {
    /// No file system access
    None,
    /// Read-only file system access
    ReadOnly,
    /// Read-write file system access
    ReadWrite,
}

impl std::fmt::Display for FileSystemAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSystemAccess::None => write!(f, "None"),
            FileSystemAccess::ReadOnly => write!(f, "Read Only"),
            FileSystemAccess::ReadWrite => write!(f, "Read Write"),
        }
    }
}

/// Network access enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkAccess {
    /// No network access
    None,
    /// Limited network access (only specific domains)
    Limited,
    /// Full network access
    Full,
}

impl std::fmt::Display for NetworkAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkAccess::None => write!(f, "None"),
            NetworkAccess::Limited => write!(f, "Limited"),
            NetworkAccess::Full => write!(f, "Full"),
        }
    }
}

/// Approval request status enum
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalStatus {
    /// Request is pending approval
    Pending,
    /// Request was approved
    Approved,
    /// Request was denied
    Denied,
    /// Request expired
    Expired,
    /// Request was cancelled
    Cancelled,
}

impl std::fmt::Display for ApprovalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApprovalStatus::Pending => write!(f, "Pending"),
            ApprovalStatus::Approved => write!(f, "Approved"),
            ApprovalStatus::Denied => write!(f, "Denied"),
            ApprovalStatus::Expired => write!(f, "Expired"),
            ApprovalStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// Approval request struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// Unique identifier for the request
    pub id: String,
    /// ID of the agent making the request
    pub agent_id: String,
    /// Type of permission being requested
    pub permission: PermissionType,
    /// Description of the action
    pub action_description: String,
    /// Additional context for the request
    pub context: serde_json::Value,
    /// Status of the request
    pub status: ApprovalStatus,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// When the request expires
    pub expires_at: DateTime<Utc>,
    /// When the request was processed
    pub processed_at: Option<DateTime<Utc>>,
    /// User who processed the request
    pub processed_by: Option<String>,
    /// Reason for approval/denial
    pub reason: Option<String>,
}

impl ApprovalRequest {
    /// Create a new approval request
    fn new(agent_id: String, permission: PermissionType, action_description: String, context: serde_json::Value, expiry_seconds: u64) -> Self {
        let id = format!("approval-{}", Uuid::new_v4().as_u128());
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(expiry_seconds as i64);
        
        Self {
            id,
            agent_id,
            permission,
            action_description,
            context,
            status: ApprovalStatus::Pending,
            created_at: now,
            expires_at,
            processed_at: None,
            processed_by: None,
            reason: None,
        }
    }
    
    /// Check if the request is expired
    fn is_expired(&self) -> bool {
        self.status == ApprovalStatus::Pending && Utc::now() > self.expires_at
    }
    
    /// Approve the request
    fn approve(&mut self, user: &str, reason: Option<String>) {
        self.status = ApprovalStatus::Approved;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(user.to_string());
        self.reason = reason;
    }
    
    /// Deny the request
    fn deny(&mut self, user: &str, reason: Option<String>) {
        self.status = ApprovalStatus::Denied;
        self.processed_at = Some(Utc::now());
        self.processed_by = Some(user.to_string());
        self.reason = reason;
    }
    
    /// Cancel the request
    fn cancel(&mut self) {
        self.status = ApprovalStatus::Cancelled;
        self.processed_at = Some(Utc::now());
    }
}

/// Security audit log entry struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLogEntry {
    /// Unique identifier for the log entry
    pub id: String,
    /// ID of the agent
    pub agent_id: String,
    /// Type of action
    pub action: String,
    /// Status of the action (success, failure, etc.)
    pub status: String,
    /// Details of the action
    pub details: String,
    /// When the action occurred
    pub timestamp: DateTime<Utc>,
    /// IP address (if applicable)
    pub ip_address: Option<String>,
    /// User (if applicable)
    pub user: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SecurityAuditLogEntry {
    /// Create a new security audit log entry
    fn new(agent_id: String, action: String, status: String, details: String) -> Self {
        let id = format!("audit-{}", Uuid::new_v4().as_u128());
        
        Self {
            id,
            agent_id,
            action,
            status,
            details,
            timestamp: Utc::now(),
            ip_address: None,
            user: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Add metadata to the log entry
    fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
    
    /// Set the IP address
    fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }
    
    /// Set the user
    fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }
}

/// Security manager struct
pub struct SecurityManager {
    /// Map of profile IDs to security profiles
    profiles: HashMap<String, SecurityProfile>,
    /// Map of agent IDs to profile IDs
    agent_profiles: HashMap<String, String>,
    /// Map of request IDs to approval requests
    approval_requests: HashMap<String, ApprovalRequest>,
    /// Security audit log
    audit_log: VecDeque<SecurityAuditLogEntry>,
    /// Default security profile ID
    default_profile: Option<String>,
    /// Maximum audit log length
    max_audit_log_length: usize,
    /// Default approval expiry time in seconds
    default_approval_expiry_seconds: u64,
    /// Whether auto-approval is enabled (for testing)
    auto_approve: bool,
}

impl SecurityManager {
    /// Create a new security manager
    fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            agent_profiles: HashMap::new(),
            approval_requests: HashMap::new(),
            audit_log: VecDeque::new(),
            default_profile: None,
            max_audit_log_length: 10000,
            default_approval_expiry_seconds: 86400, // 24 hours
            auto_approve: false,
        }
    }
    
    /// Initialize the security manager
    fn initialize(&mut self) -> Result<()> {
        // Create default profiles if they don't exist
        if self.profiles.is_empty() {
            // Read-only profile
            let readonly_profile = SecurityProfile::new(
                "Read Only".to_string(),
                "Read-only access to agents and tasks".to_string(),
                PermissionLevel::ReadOnly,
            );
            self.profiles.insert(readonly_profile.id.clone(), readonly_profile.clone());
            
            // Basic profile
            let basic_profile = SecurityProfile::new(
                "Basic".to_string(),
                "Basic access to agents and tasks".to_string(),
                PermissionLevel::Basic,
            );
            self.profiles.insert(basic_profile.id.clone(), basic_profile.clone());
            
            // Standard profile
            let standard_profile = SecurityProfile::new(
                "Standard".to_string(),
                "Standard access to agents and tasks".to_string(),
                PermissionLevel::Standard,
            );
            self.profiles.insert(standard_profile.id.clone(), standard_profile.clone());
            
            // Advanced profile
            let advanced_profile = SecurityProfile::new(
                "Advanced".to_string(),
                "Advanced access with approval requirements".to_string(),
                PermissionLevel::Advanced,
            );
            self.profiles.insert(advanced_profile.id.clone(), advanced_profile.clone());
            
            // Admin profile
            let admin_profile = SecurityProfile::new(
                "Admin".to_string(),
                "Administrative access with full permissions".to_string(),
                PermissionLevel::Admin,
            );
            self.profiles.insert(admin_profile.id.clone(), admin_profile.clone());
            
            // Set the default profile to Standard
            self.default_profile = Some(standard_profile.id.clone());
            
            info!("Created default security profiles");
        }
        
        Ok(())
    }
    
    /// Set auto-approval mode
    fn set_auto_approve(&mut self, auto_approve: bool) {
        self.auto_approve = auto_approve;
        info!("Set auto-approve mode to: {}", auto_approve);
    }
    
    /// Create a new security profile
    fn create_profile(&mut self, name: String, description: String, permission_level: PermissionLevel) -> Result<SecurityProfile> {
        let profile = SecurityProfile::new(name, description, permission_level);
        
        self.profiles.insert(profile.id.clone(), profile.clone());
        info!("Created security profile: {} ({})", profile.name, profile.id);
        
        Ok(profile)
    }
    
    /// Update a security profile
    fn update_profile(&mut self, profile_id: &str, name: Option<String>, description: Option<String>, permission_level: Option<PermissionLevel>) -> Result<SecurityProfile> {
        let profile = self.profiles.get_mut(profile_id)
            .ok_or_else(|| anyhow!("Profile not found: {}", profile_id))?;
        
        if let Some(name) = name {
            profile.name = name;
        }
        
        if let Some(description) = description {
            profile.description = description;
        }
        
        if let Some(level) = permission_level {
            profile.permission_level = level;
        }
        
        profile.updated_at = Utc::now();
        info!("Updated security profile: {} ({})", profile.name, profile.id);
        
        Ok(profile.clone())
    }
    
    /// Delete a security profile
    fn delete_profile(&mut self, profile_id: &str) -> Result<()> {
        // Check if this is the default profile
        if let Some(default_id) = &self.default_profile {
            if default_id == profile_id {
                return Err(anyhow!("Cannot delete the default profile"));
            }
        }
        
        // Check if any agents are using this profile
        for (_, assigned_profile_id) in &self.agent_profiles {
            if assigned_profile_id == profile_id {
                return Err(anyhow!("Cannot delete profile that is assigned to agents"));
            }
        }
        
        // Remove the profile
        if self.profiles.remove(profile_id).is_some() {
            info!("Deleted security profile: {}", profile_id);
            Ok(())
        } else {
            Err(anyhow!("Profile not found: {}", profile_id))
        }
    }
    
    /// Set the default security profile
    fn set_default_profile(&mut self, profile_id: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_id) {
            return Err(anyhow!("Profile not found: {}", profile_id));
        }
        
        self.default_profile = Some(profile_id.to_string());
        info!("Set default security profile: {}", profile_id);
        Ok(())
    }
    
    /// Assign a security profile to an agent
    fn assign_profile_to_agent(&mut self, agent_id: &str, profile_id: &str) -> Result<()> {
        if !self.profiles.contains_key(profile_id) {
            return Err(anyhow!("Profile not found: {}", profile_id));
        }
        
        self.agent_profiles.insert(agent_id.to_string(), profile_id.to_string());
        info!("Assigned security profile {} to agent {}", profile_id, agent_id);
        
        // Log the action
        self.log_security_event(
            agent_id.to_string(),
            "PROFILE_ASSIGNED".to_string(),
            "SUCCESS".to_string(),
            format!("Security profile {} assigned to agent", profile_id),
        );
        
        Ok(())
    }
    
    /// Get the security profile for an agent
    fn get_agent_profile(&self, agent_id: &str) -> Result<SecurityProfile> {
        // Get the profile ID for this agent
        let profile_id = if let Some(id) = self.agent_profiles.get(agent_id) {
            id
        } else if let Some(default_id) = &self.default_profile {
            default_id
        } else {
            return Err(anyhow!("No profile assigned to agent and no default profile"));
        };
        
        // Get the profile
        self.profiles.get(profile_id)
            .cloned()
            .ok_or_else(|| anyhow!("Profile not found: {}", profile_id))
    }
    
    /// Check if an agent has a specific permission
    fn check_permission(&self, agent_id: &str, permission: PermissionType) -> Result<bool> {
        let profile = self.get_agent_profile(agent_id)?;
        Ok(profile.is_permission_allowed(permission))
    }
    
    /// Request approval for an action
    fn request_approval(&mut self, agent_id: &str, permission: PermissionType, action_description: String, context: serde_json::Value) -> Result<ApprovalRequest> {
        // Create a new approval request
        let request = ApprovalRequest::new(
            agent_id.to_string(),
            permission,
            action_description.clone(),
            context,
            self.default_approval_expiry_seconds,
        );
        
        // Store the request
        self.approval_requests.insert(request.id.clone(), request.clone());
        
        // Log the action
        self.log_security_event(
            agent_id.to_string(),
            "APPROVAL_REQUESTED".to_string(),
            "PENDING".to_string(),
            format!("Approval requested for action: {}", action_description),
        );
        
        info!("Approval requested for agent {}: {}", agent_id, request.id);
        
        // Auto-approve if enabled
        if self.auto_approve {
            let request_id = request.id.clone();
            self.approve_request(&request_id, "SYSTEM", None)?;
            return self.get_approval_request(&request_id);
        }
        
        Ok(request)
    }
    
    /// Get an approval request by ID
    fn get_approval_request(&self, request_id: &str) -> Result<ApprovalRequest> {
        self.approval_requests.get(request_id)
            .cloned()
            .ok_or_else(|| anyhow!("Approval request not found: {}", request_id))
    }
    
    /// Approve an approval request
    fn approve_request(&mut self, request_id: &str, user: &str, reason: Option<String>) -> Result<()> {
        let (agent_id, action_description) = {
            let request = self.approval_requests.get_mut(request_id)
                .ok_or_else(|| anyhow!("Approval request not found: {}", request_id))?;
            
            // Check if the request is still pending
            if request.status != ApprovalStatus::Pending {
                return Err(anyhow!("Request is not pending: {}", request_id));
            }
            
            // Check if the request is expired
            if request.is_expired() {
                request.status = ApprovalStatus::Expired;
                return Err(anyhow!("Request is expired: {}", request_id));
            }
            
            // Approve the request
            request.approve(user, reason);
            
            (request.agent_id.clone(), request.action_description.clone())
        };
        
        // Log the action
        self.log_security_event(
            agent_id,
            "APPROVAL_GRANTED".to_string(),
            "SUCCESS".to_string(),
            format!("Approval granted for action: {}", action_description),
        )
        .with_user(user);
        
        info!("Approved request {}", request_id);
        Ok(())
    }
    
    /// Deny an approval request
    fn deny_request(&mut self, request_id: &str, user: &str, reason: Option<String>) -> Result<()> {
        let (agent_id, action_description) = {
            let request = self.approval_requests.get_mut(request_id)
                .ok_or_else(|| anyhow!("Approval request not found: {}", request_id))?;
            
            // Check if the request is still pending
            if request.status != ApprovalStatus::Pending {
                return Err(anyhow!("Request is not pending: {}", request_id));
            }
            
            // Check if the request is expired
            if request.is_expired() {
                request.status = ApprovalStatus::Expired;
                return Err(anyhow!("Request is expired: {}", request_id));
            }
            
            // Deny the request
            request.deny(user, reason);
            
            (request.agent_id.clone(), request.action_description.clone())
        };
        
        // Log the action
        self.log_security_event(
            agent_id,
            "APPROVAL_DENIED".to_string(),
            "DENIED".to_string(),
            format!("Approval denied for action: {}", action_description),
        )
        .with_user(user);
        
        info!("Denied request {}", request_id);
        Ok(())
    }
    
    /// Cancel an approval request
    fn cancel_request(&mut self, request_id: &str) -> Result<()> {
        let (agent_id, action_description) = {
            let request = self.approval_requests.get_mut(request_id)
                .ok_or_else(|| anyhow!("Approval request not found: {}", request_id))?;
            
            // Check if the request is still pending
            if request.status != ApprovalStatus::Pending {
                return Err(anyhow!("Request is not pending: {}", request_id));
            }
            
            // Cancel the request
            request.cancel();
            
            (request.agent_id.clone(), request.action_description.clone())
        };
        
        // Log the action
        self.log_security_event(
            agent_id,
            "APPROVAL_CANCELLED".to_string(),
            "CANCELLED".to_string(),
            format!("Approval cancelled for action: {}", action_description),
        );
        
        info!("Cancelled request {}", request_id);
        Ok(())
    }
    
    /// Get all pending approval requests
    fn get_pending_requests(&self) -> Vec<ApprovalRequest> {
        self.approval_requests.values()
            .filter(|r| r.status == ApprovalStatus::Pending && !r.is_expired())
            .cloned()
            .collect()
    }
    
    /// Get all approval requests for an agent
    fn get_agent_requests(&self, agent_id: &str) -> Vec<ApprovalRequest> {
        self.approval_requests.values()
            .filter(|r| r.agent_id == agent_id)
            .cloned()
            .collect()
    }
    
    /// Log a security event
    fn log_security_event(&mut self, agent_id: String, action: String, status: String, details: String) -> SecurityAuditLogEntry {
        let entry = SecurityAuditLogEntry::new(agent_id, action, status, details);
        
        // Add to the log
        self.audit_log.push_back(entry.clone());
        
        // Trim the log if needed
        while self.audit_log.len() > self.max_audit_log_length {
            self.audit_log.pop_front();
        }
        
        entry
    }
    
    /// Get the security audit log
    fn get_audit_log(&self, limit: usize) -> Vec<SecurityAuditLogEntry> {
        self.audit_log.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Get the security audit log for an agent
    fn get_agent_audit_log(&self, agent_id: &str, limit: usize) -> Vec<SecurityAuditLogEntry> {
        self.audit_log.iter()
            .rev()
            .filter(|e| e.agent_id == agent_id)
            .take(limit)
            .cloned()
            .collect()
    }
    
    /// Check if an action requires approval
    fn action_requires_approval(&self, agent_id: &str, permission: PermissionType) -> Result<bool> {
        let profile = self.get_agent_profile(agent_id)?;
        
        // If the profile doesn't require approval, return false
        if !profile.require_approval {
            return Ok(false);
        }
        
        // Check if the permission is allowed
        if !profile.is_permission_allowed(permission) {
            return Err(anyhow!("Permission not allowed: {:?}", permission));
        }
        
        // Determine if this permission type requires approval
        let requires_approval = match permission {
            // These permissions always require approval
            PermissionType::DeleteAgent |
            PermissionType::ExecuteCommand |
            PermissionType::ModifySettings |
            PermissionType::AccessSensitiveData |
            PermissionType::SpawnAgent |
            PermissionType::SelfModify => true,
            
            // These permissions require approval for Advanced level
            PermissionType::ModifyAgent |
            PermissionType::ControlAgent |
            PermissionType::CreateAgent |
            PermissionType::DeleteTask |
            PermissionType::FileSystem => profile.permission_level == PermissionLevel::Advanced,
            
            // These permissions don't require approval
            _ => false,
        };
        
        Ok(requires_approval)
    }
    
    /// Perform a secure action with permission checking and approval if needed
    async fn perform_secure_action<F, T>(&mut self, agent_id: &str, permission: PermissionType, action_description: &str, context: serde_json::Value, action: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check if the agent has the required permission
        if !self.check_permission(agent_id, permission)? {
            // Log the denied action
            self.log_security_event(
                agent_id.to_string(),
                "PERMISSION_DENIED".to_string(),
                "DENIED".to_string(),
                format!("Permission denied for action: {}", action_description),
            );
            
            return Err(anyhow!("Permission denied: {:?}", permission));
        }
        
        // Check if approval is required
        if self.action_requires_approval(agent_id, permission)? {
            // Request approval
            let request = self.request_approval(
                agent_id,
                permission,
                action_description.to_string(),
                context,
            )?;
            
            // If auto-approve is enabled, the request will already be approved
            if request.status != ApprovalStatus::Approved {
                return Err(anyhow!("Action requires approval: {}", request.id));
            }
        }
        
        // Perform the action
        let result = action();
        
        // Log the action
        let status = if result.is_ok() { "SUCCESS" } else { "FAILURE" };
        self.log_security_event(
            agent_id.to_string(),
            "ACTION_PERFORMED".to_string(),
            status.to_string(),
            format!("Action performed: {}", action_description),
        );
        
        result
    }
}

/// Initialize the security manager
pub async fn initialize() -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.initialize()
}

/// Set auto-approval mode (for testing)
pub async fn set_auto_approve(auto_approve: bool) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.set_auto_approve(auto_approve);
    Ok(())
}

/// Create a new security profile
pub async fn create_profile(name: &str, description: &str, permission_level: PermissionLevel) -> Result<SecurityProfile> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.create_profile(name.to_string(), description.to_string(), permission_level)
}

/// Update a security profile
pub async fn update_profile(profile_id: &str, name: Option<&str>, description: Option<&str>, permission_level: Option<PermissionLevel>) -> Result<SecurityProfile> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.update_profile(
        profile_id,
        name.map(|s| s.to_string()),
        description.map(|s| s.to_string()),
        permission_level,
    )
}

/// Delete a security profile
pub async fn delete_profile(profile_id: &str) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.delete_profile(profile_id)
}

/// Set the default security profile
pub async fn set_default_profile(profile_id: &str) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.set_default_profile(profile_id)
}

/// Assign a security profile to an agent
pub async fn assign_profile_to_agent(agent_id: &str, profile_id: &str) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.assign_profile_to_agent(agent_id, profile_id)
}

/// Get the security profile for an agent
pub async fn get_agent_profile(agent_id: &str) -> Result<SecurityProfile> {
    let manager = SECURITY_MANAGER.lock().await;
    manager.get_agent_profile(agent_id)
}

/// Check if an agent has a specific permission
pub async fn check_permission(agent_id: &str, permission: PermissionType) -> Result<bool> {
    let manager = SECURITY_MANAGER.lock().await;
    manager.check_permission(agent_id, permission)
}

/// Request approval for an action
pub async fn request_approval(agent_id: &str, permission: PermissionType, action_description: &str, context: serde_json::Value) -> Result<ApprovalRequest> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.request_approval(agent_id, permission, action_description.to_string(), context)
}

/// Get an approval request by ID
pub async fn get_approval_request(request_id: &str) -> Result<ApprovalRequest> {
    let manager = SECURITY_MANAGER.lock().await;
    manager.get_approval_request(request_id)
}

/// Approve an approval request
pub async fn approve_request(request_id: &str, user: &str, reason: Option<&str>) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.approve_request(request_id, user, reason.map(|s| s.to_string()))
}

/// Deny an approval request
pub async fn deny_request(request_id: &str, user: &str, reason: Option<&str>) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.deny_request(request_id, user, reason.map(|s| s.to_string()))
}

/// Cancel an approval request
pub async fn cancel_request(request_id: &str) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.cancel_request(request_id)
}

/// Get all pending approval requests
pub async fn get_pending_requests() -> Result<Vec<ApprovalRequest>> {
    let manager = SECURITY_MANAGER.lock().await;
    Ok(manager.get_pending_requests())
}

/// Get all approval requests for an agent
pub async fn get_agent_requests(agent_id: &str) -> Result<Vec<ApprovalRequest>> {
    let manager = SECURITY_MANAGER.lock().await;
    Ok(manager.get_agent_requests(agent_id))
}

/// Get the security audit log
pub async fn get_audit_log(limit: usize) -> Result<Vec<SecurityAuditLogEntry>> {
    let manager = SECURITY_MANAGER.lock().await;
    Ok(manager.get_audit_log(limit))
}

/// Get the security audit log for an agent
pub async fn get_agent_audit_log(agent_id: &str, limit: usize) -> Result<Vec<SecurityAuditLogEntry>> {
    let manager = SECURITY_MANAGER.lock().await;
    Ok(manager.get_agent_audit_log(agent_id, limit))
}

/// Check if an action requires approval
pub async fn action_requires_approval(agent_id: &str, permission: PermissionType) -> Result<bool> {
    let manager = SECURITY_MANAGER.lock().await;
    manager.action_requires_approval(agent_id, permission)
}

/// Perform a secure action with permission checking and approval if needed
pub async fn perform_secure_action<F, T>(agent_id: &str, permission: PermissionType, action_description: &str, context: serde_json::Value, action: F) -> Result<T>
where
    F: FnOnce() -> Result<T>,
{
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.perform_secure_action(agent_id, permission, action_description, context, action).await
}

/// Log a security event
pub async fn log_security_event(agent_id: &str, action: &str, status: &str, details: &str) -> Result<()> {
    let mut manager = SECURITY_MANAGER.lock().await;
    manager.log_security_event(
        agent_id.to_string(),
        action.to_string(),
        status.to_string(),
        details.to_string(),
    );
    Ok(())
}

/// Initialize security for a new agent
pub async fn initialize_agent_security(agent_id: &str) -> Result<()> {
    let manager = SECURITY_MANAGER.lock().await;
    
    // Get the default profile or create one if it doesn't exist
    let default_profile_id = {
        let manager_guard = manager;
        if let Some(id) = &manager_guard.default_profile {
            id.clone()
        } else {
            drop(manager_guard);
            // If no default profile, create one
            let profile = create_profile(
                "Standard",
                "Standard access to agents and tasks",
                PermissionLevel::Standard,
            ).await?;
            let profile_id = profile.id.clone();
            set_default_profile(&profile_id).await?;
            profile_id
        }
    };
    
    // Assign the default profile to the agent
    assign_profile_to_agent(agent_id, &default_profile_id).await?;
    
    // Log the action
    log_security_event(
        agent_id,
        "SECURITY_INITIALIZED",
        "SUCCESS",
        "Security initialized for agent",
    ).await?;
    
    Ok(())
}