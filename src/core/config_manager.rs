//! Configuration manager module for the Nox agent ecosystem
//! 
//! This module handles loading and managing configuration settings.

use anyhow::{anyhow, Result};
use config::{Config, ConfigError, Environment, File as ConfigFile};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

// Singleton instance of the configuration manager
lazy_static! {
    static ref CONFIG_MANAGER: Arc<RwLock<ConfigManager>> = {
        let manager = ConfigManager::new();
        Arc::new(RwLock::new(manager))
    };
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub frontend_port: u16,
    pub host: String,
    pub websocket_enabled: bool,
    pub api_enabled: bool,
    pub cors_origins: Vec<String>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub registry_path: String,
}

/// Claude CLI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCliConfig {
    pub session_timeout: u64,
    pub auto_restart_on_crash: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub storage: StorageConfig,
    pub claude_cli: ClaudeCliConfig,
    pub logging: LoggingConfig,
}

/// Configuration manager struct
struct ConfigManager {
    config: Option<AppConfig>,
    config_path: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    fn new() -> Self {
        let config_path = PathBuf::from("config/default.toml");

        Self {
            config: None,
            config_path,
        }
    }

    /// Load configuration from file and environment variables
    async fn load_config(&mut self) -> Result<()> {
        debug!("Loading configuration from {:?}", self.config_path);

        // Create config builder
        let mut builder = Config::builder();

        // Add default configuration
        builder = builder.set_default("server.port", 8080)?;
        builder = builder.set_default("server.frontend_port", 5173)?;
        builder = builder.set_default("server.host", "localhost")?;
        builder = builder.set_default("server.websocket_enabled", true)?;
        builder = builder.set_default("server.api_enabled", true)?;
        builder = builder.set_default("server.cors_origins", vec!["http://localhost:5173"])?;

        builder = builder.set_default("storage.registry_path", ".nox-registry")?;

        builder = builder.set_default("claude_cli.session_timeout", 3600)?;
        builder = builder.set_default("claude_cli.auto_restart_on_crash", true)?;

        builder = builder.set_default("logging.level", "info")?;
        builder = builder.set_default("logging.format", "json")?;

        // Add configuration from file if it exists
        if self.config_path.exists() {
            builder = builder.add_source(ConfigFile::from(self.config_path.clone()));
        } else {
            warn!("Configuration file not found at {:?}, using defaults", self.config_path);
        }

        // Add configuration from environment variables
        // Format: NOX_SERVER__PORT, NOX_STORAGE__REGISTRY_PATH, etc.
        builder = builder.add_source(Environment::with_prefix("NOX").separator("__"));

        // Build the configuration
        let config = builder.build()?;
        info!("{:?}", config);
        // Deserialize the configuration
        let app_config: AppConfig = config.try_deserialize()?;

        info!("Configuration loaded successfully");
        self.config = Some(app_config);

        Ok(())
    }

    /// Get the current configuration
    fn get_config(&self) -> Result<AppConfig> {
        match &self.config {
            Some(config) => Ok(config.clone()),
            None => Err(anyhow!("Configuration not loaded")),
        }
    }

    /// Save the current configuration to file
    async fn save_config(&self) -> Result<()> {
        match &self.config {
            Some(config) => {
                // Create parent directory if it doesn't exist
                if let Some(parent) = self.config_path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }

                // Serialize the configuration to TOML
                let toml_string = toml::to_string_pretty(config)?;

                // Write the configuration to file
                fs::write(&self.config_path, toml_string)?;

                info!("Configuration saved to {:?}", self.config_path);
                Ok(())
            },
            None => Err(anyhow!("Configuration not loaded")),
        }
    }

    /// Update a specific configuration value
    async fn update_value<T: serde::Serialize + std::fmt::Debug + Into<config::Value> + Clone>(&mut self, key: &str, value: T) -> Result<()> {
        // Ensure configuration is loaded
        if self.config.is_none() {
            self.load_config().await?;
        }

        // Create a new Config instance
        let mut builder = Config::builder();

        // Add current configuration
        if let Some(config) = &self.config {
            let config_str = toml::to_string(config)?;
            builder = builder.add_source(config::File::from_str(&config_str, config::FileFormat::Toml));
        }

        // Clone the value for logging later
        let value_for_log = value.clone();

        // Add the new value
        builder = builder.set_override(key, value)?;

        // Build the configuration
        let config = builder.build()?;

        // Deserialize the configuration
        let app_config: AppConfig = config.try_deserialize()?;

        // Update the configuration
        self.config = Some(app_config);

        // Save the configuration to file
        self.save_config().await?;

        info!("Configuration updated: {} = {:?}", key, value_for_log);
        Ok(())
    }
}

/// Initialize the configuration manager
pub async fn initialize() -> Result<()> {
    let mut manager = CONFIG_MANAGER.write().await;
    
    // If config file doesn't exist, create it first
    if !manager.config_path.exists() {
        drop(manager); // Release the lock temporarily
        create_default_config().await?;
        manager = CONFIG_MANAGER.write().await; // Re-acquire the lock
    }
    
    manager.load_config().await
}

/// Ensure configuration is initialized (safe to call multiple times)
pub async fn ensure_initialized() -> Result<()> {
    // Check if configuration is already loaded
    {
        let manager = CONFIG_MANAGER.read().await;
        if manager.config.is_some() {
            return Ok(());
        }
    }
    
    // Initialize if not already done
    initialize().await
}

/// Get the current configuration
pub async fn get_config() -> Result<AppConfig> {
    ensure_initialized().await?;
    let manager = CONFIG_MANAGER.read().await;
    manager.get_config()
}

/// Save the current configuration to file
pub async fn save_config() -> Result<()> {
    let manager = CONFIG_MANAGER.read().await;
    manager.save_config().await
}

/// Update a specific configuration value
pub async fn update_value<T: serde::Serialize + std::fmt::Debug + Into<config::Value> + Clone>(key: &str, value: T) -> Result<()> {
    let mut manager = CONFIG_MANAGER.write().await;
    manager.update_value(key, value).await
}

/// Get the registry path
pub async fn get_registry_path() -> Result<PathBuf> {
    ensure_initialized().await?;
    let config = get_config().await?;
    Ok(PathBuf::from(config.storage.registry_path))
}

/// Get the server configuration
pub async fn get_server_config() -> Result<ServerConfig> {
    ensure_initialized().await?;
    let config = get_config().await?;
    Ok(config.server)
}

/// Get the Claude CLI configuration
pub async fn get_claude_cli_config() -> Result<ClaudeCliConfig> {
    ensure_initialized().await?;
    let config = get_config().await?;
    Ok(config.claude_cli)
}

/// Get the logging configuration
pub async fn get_logging_config() -> Result<LoggingConfig> {
    ensure_initialized().await?;
    let config = get_config().await?;
    Ok(config.logging)
}

/// Create default configuration file if it doesn't exist
pub async fn create_default_config() -> Result<()> {
    let mut manager = CONFIG_MANAGER.write().await;

    // Check if configuration file exists
    if manager.config_path.exists() {
        debug!("Configuration file already exists at {:?}", manager.config_path);
        return Ok(());
    }

    // Create parent directory if it doesn't exist
    if let Some(parent) = manager.config_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    // Create default configuration
    let default_config = AppConfig {
        server: ServerConfig {
            port: 8080,
            frontend_port: 5173,
            host: "localhost".to_string(),
            websocket_enabled: true,
            api_enabled: true,
            cors_origins: vec!["*".to_string()],
        },
        storage: StorageConfig {
            registry_path: ".nox-registry".to_string(),
        },
        claude_cli: ClaudeCliConfig {
            session_timeout: 3600,
            auto_restart_on_crash: true,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
    };

    // Serialize the configuration to TOML
    let toml_string = toml::to_string_pretty(&default_config)?;

    // Write the configuration to file
    fs::write(&manager.config_path, toml_string)?;

    // Update the manager's configuration
    manager.config = Some(default_config);

    info!("Created default configuration file at {:?}", manager.config_path);
    Ok(())
}
