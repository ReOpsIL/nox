//! Mock services for testing external dependencies

use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Mock Claude CLI process for testing
#[derive(Clone)]
pub struct MockClaudeProcess {
    pub id: String,
    pub status: MockProcessStatus,
    pub responses: Vec<String>,
    pub response_index: usize,
}

#[derive(Clone, Debug)]
pub enum MockProcessStatus {
    Running,
    Stopped,
    Error,
}

impl MockClaudeProcess {
    pub fn new(id: String) -> Self {
        Self {
            id,
            status: MockProcessStatus::Stopped,
            responses: vec![
                "Hello! I'm Claude, ready to help.".to_string(),
                "I understand your request.".to_string(),
                "Task completed successfully.".to_string(),
            ],
            response_index: 0,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        self.status = MockProcessStatus::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.status = MockProcessStatus::Stopped;
        Ok(())
    }

    pub fn send_message(&mut self, _message: &str) -> Result<String> {
        if matches!(self.status, MockProcessStatus::Running) {
            let response = self.responses[self.response_index % self.responses.len()].clone();
            self.response_index += 1;
            Ok(response)
        } else {
            Err(anyhow::anyhow!("Process is not running"))
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.status, MockProcessStatus::Running)
    }
}

/// Mock external service for testing network dependencies
pub struct MockExternalService {
    pub endpoints: HashMap<String, String>,
    pub call_count: HashMap<String, usize>,
    pub should_fail: bool,
}

impl MockExternalService {
    pub fn new() -> Self {
        let mut endpoints = HashMap::new();
        endpoints.insert("/health".to_string(), "OK".to_string());
        endpoints.insert("/api/status".to_string(), r#"{"status": "active"}"#.to_string());
        
        Self {
            endpoints,
            call_count: HashMap::new(),
            should_fail: false,
        }
    }

    pub fn add_endpoint(&mut self, path: String, response: String) {
        self.endpoints.insert(path, response);
    }

    pub fn set_should_fail(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }

    pub fn call_endpoint(&mut self, path: &str) -> Result<String> {
        if self.should_fail {
            return Err(anyhow::anyhow!("Mock service configured to fail"));
        }

        *self.call_count.entry(path.to_string()).or_insert(0) += 1;

        self.endpoints
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Endpoint not found: {}", path))
    }

    pub fn get_call_count(&self, path: &str) -> usize {
        self.call_count.get(path).copied().unwrap_or(0)
    }

    pub fn reset_call_counts(&mut self) {
        self.call_count.clear();
    }
}

/// Mock file system for testing file operations
pub struct MockFileSystem {
    pub files: HashMap<String, String>,
    pub directories: HashMap<String, Vec<String>>,
    pub should_fail_operations: Vec<String>,
}

impl MockFileSystem {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            directories: HashMap::new(),
            should_fail_operations: Vec::new(),
        }
    }

    pub fn add_file(&mut self, path: String, content: String) {
        self.files.insert(path, content);
    }

    pub fn add_directory(&mut self, path: String, contents: Vec<String>) {
        self.directories.insert(path, contents);
    }

    pub fn set_operation_should_fail(&mut self, operation: String) {
        self.should_fail_operations.push(operation);
    }

    pub fn read_file(&self, path: &str) -> Result<String> {
        if self.should_fail_operations.contains(&"read".to_string()) {
            return Err(anyhow::anyhow!("Mock read operation configured to fail"));
        }

        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path))
    }

    pub fn write_file(&mut self, path: String, content: String) -> Result<()> {
        if self.should_fail_operations.contains(&"write".to_string()) {
            return Err(anyhow::anyhow!("Mock write operation configured to fail"));
        }

        self.files.insert(path, content);
        Ok(())
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<String>> {
        if self.should_fail_operations.contains(&"list".to_string()) {
            return Err(anyhow::anyhow!("Mock list operation configured to fail"));
        }

        self.directories
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Directory not found: {}", path))
    }

    pub fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    pub fn directory_exists(&self, path: &str) -> bool {
        self.directories.contains_key(path)
    }
}

/// Global mock registry for managing mock services during tests
pub struct MockRegistry {
    pub claude_processes: Arc<Mutex<HashMap<String, MockClaudeProcess>>>,
    pub external_service: Arc<Mutex<MockExternalService>>,
    pub file_system: Arc<Mutex<MockFileSystem>>,
}

impl MockRegistry {
    pub fn new() -> Self {
        Self {
            claude_processes: Arc::new(Mutex::new(HashMap::new())),
            external_service: Arc::new(Mutex::new(MockExternalService::new())),
            file_system: Arc::new(Mutex::new(MockFileSystem::new())),
        }
    }

    pub fn reset(&self) {
        self.claude_processes.lock().unwrap().clear();
        *self.external_service.lock().unwrap() = MockExternalService::new();
        *self.file_system.lock().unwrap() = MockFileSystem::new();
    }

    pub fn add_claude_process(&self, id: String) -> Result<()> {
        let mut processes = self.claude_processes.lock().unwrap();
        processes.insert(id.clone(), MockClaudeProcess::new(id));
        Ok(())
    }

    pub fn get_claude_process(&self, id: &str) -> Option<MockClaudeProcess> {
        let processes = self.claude_processes.lock().unwrap();
        processes.get(id).cloned()
    }
}

// Global mock registry instance
lazy_static::lazy_static! {
    pub static ref MOCK_REGISTRY: MockRegistry = MockRegistry::new();
}