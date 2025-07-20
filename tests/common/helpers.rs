//! Test helper functions for the Nox test suite

use anyhow::Result;
use std::fs;
use std::path::Path;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

/// Create a temporary test directory
pub fn create_temp_dir() -> Result<TempDir> {
    Ok(TempDir::new()?)
}

/// Create a temporary test directory with a specific name pattern
pub fn create_temp_dir_with_prefix(prefix: &str) -> Result<TempDir> {
    Ok(TempDir::with_prefix(prefix)?)
}

/// Setup test environment variables
pub fn setup_test_env() {
    std::env::set_var("NOX_ENV", "test");
    std::env::set_var("NOX_LOG_LEVEL", "debug");
}

/// Cleanup test environment variables
pub fn cleanup_test_env() {
    std::env::remove_var("NOX_ENV");
    std::env::remove_var("NOX_LOG_LEVEL");
}

/// Wait for a condition to be true with timeout
pub async fn wait_for_condition<F>(
    mut condition: F,
    timeout_ms: u64,
    check_interval_ms: u64,
) -> bool
where
    F: FnMut() -> bool,
{
    let timeout = Duration::from_millis(timeout_ms);
    let interval = Duration::from_millis(check_interval_ms);
    let start = std::time::Instant::now();
    
    while start.elapsed() < timeout {
        if condition() {
            return true;
        }
        sleep(interval).await;
    }
    
    false
}

/// Create a test configuration file
pub fn create_test_config(temp_dir: &Path) -> Result<String> {
    let config_content = r#"
[server]
host = "127.0.0.1"
port = 8080
api_enabled = true
websocket_enabled = true
cors_origins = []

[claude]
executable_path = "-claude-"
default_timeout = 30000
max_retries = 3
retry_delay = 1000

[system]
registry_path = "./test_registry"
log_level = "debug"
max_agents = 100
"#;
    
    let config_path = temp_dir.join("test_config.toml");
    fs::write(&config_path, config_content)?;
    Ok(config_path.to_string_lossy().to_string())
}

/// Assert that a Result is Ok and return the value
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(val) => val,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a Result is Err
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(val) => panic!("Expected Err, got Ok: {:?}", val),
            Err(_) => {},
        }
    };
}

/// Assert that two values are approximately equal (for floating point comparisons)
#[macro_export]
macro_rules! assert_approx_eq {
    ($left:expr, $right:expr, $epsilon:expr) => {
        let diff = ($left - $right).abs();
        if diff > $epsilon {
            panic!("Values are not approximately equal: {} != {} (diff: {}, epsilon: {})", 
                   $left, $right, diff, $epsilon);
        }
    };
}

/// Run test with timeout
pub async fn with_timeout<F, R>(future: F, timeout_ms: u64) -> Result<R>
where
    F: std::future::Future<Output = R>,
{
    tokio::time::timeout(Duration::from_millis(timeout_ms), future)
        .await
        .map_err(|_| anyhow::anyhow!("Test timed out after {}ms", timeout_ms))
}

/// Initialize test logging
pub fn init_test_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Debug)
        .try_init();
}