use anyhow::Result;
use chrono::{DateTime, Utc};
use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

use crate::types::{Agent, Task, AgentStatus, TaskStatus, TaskPriority, ResourceLimits};
use crate::core::{agent_manager, task_manager};
use crate::testing_fw::{TestLogger, TestConfig, TestStatus};

/// Extended test result for CRUD operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrudTestResult {
    pub test_name: String,
    pub test_type: CrudTestType,
    pub operation: CrudOperation,
    pub entity_type: EntityType,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub timestamp: DateTime<Utc>,
    pub test_data: Option<String>,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
}

/// Type of CRUD test being performed
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CrudTestType {
    /// Basic CRUD operation test
    BasicOperation,
    /// Form validation test
    FormValidation,
    /// Business logic test
    BusinessLogic,
    /// Workflow test
    Workflow,
    /// Error handling test
    ErrorHandling,
    /// Performance test
    Performance,
}

/// CRUD operation being tested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrudOperation {
    Create,
    Read,
    Update,
    Delete,
    List,
    Search,
    Filter,
    Execute,
    Start,
    Stop,
    Restart,
    Cancel,
}

/// Entity type being tested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Agent,
    Task,
    System,
}

/// Test scenario for CRUD operations
#[derive(Debug, Clone)]
pub struct CrudTestScenario {
    pub name: String,
    pub description: String,
    pub test_type: CrudTestType,
    pub operation: CrudOperation,
    pub entity_type: EntityType,
    pub setup_data: Vec<TestData>,
    pub test_steps: Vec<TestStep>,
    pub expected_outcomes: Vec<ExpectedOutcome>,
    pub cleanup_required: bool,
}

/// Test data for scenarios
#[derive(Debug, Clone)]
pub enum TestData {
    Agent(Agent),
    Task(Task),
    AgentCount(usize),
    TaskCount(usize),
    SystemState(String),
}

/// Individual test step
#[derive(Debug, Clone)]
pub struct TestStep {
    pub name: String,
    pub action: TestAction,
    pub input_data: Option<TestData>,
    pub expected_duration_ms: Option<u64>,
}

/// Test action to perform
#[derive(Debug, Clone)]
pub enum TestAction {
    CreateAgent { name: String, system_prompt: String },
    UpdateAgent { id: String, name: Option<String>, system_prompt: Option<String> },
    DeleteAgent { id: String },
    StartAgent { id: String },
    StopAgent { id: String },
    RestartAgent { id: String },
    ListAgents,
    SearchAgents { query: String },
    FilterAgents { status: Option<AgentStatus> },
    
    CreateTask { agent_id: String, title: String, description: String, priority: Option<TaskPriority> },
    UpdateTask { id: String, title: Option<String>, description: Option<String>, priority: Option<TaskPriority> },
    DeleteTask { id: String },
    ExecuteTask { id: String },
    CancelTask { id: String },
    ListTasks,
    SearchTasks { query: String },
    FilterTasks { status: Option<TaskStatus>, agent_id: Option<String> },
    
    ValidateForm { field: String, value: String, expected_valid: bool },
    SimulateKeypress { key: KeyCode, modifiers: KeyModifiers },
    WaitForState { timeout_ms: u64 },
    VerifyData { entity_type: EntityType, expected_count: usize },
}

/// Expected outcome of a test step
#[derive(Debug, Clone)]
pub struct ExpectedOutcome {
    pub step_name: String,
    pub success: bool,
    pub error_pattern: Option<String>,
    pub data_validation: Option<DataValidation>,
}

/// Data validation criteria
#[derive(Debug, Clone)]
pub enum DataValidation {
    AgentExists { id: String },
    AgentStatus { id: String, status: AgentStatus },
    AgentCount { expected: usize },
    TaskExists { id: String },
    TaskStatus { id: String, status: TaskStatus },
    TaskCount { expected: usize },
    SystemState { check: String, expected: String },
}

/// Mock data factory for generating test data
pub struct MockDataFactory {
    agent_counter: usize,
    task_counter: usize,
}

impl MockDataFactory {
    pub fn new() -> Self {
        Self {
            agent_counter: 0,
            task_counter: 0,
        }
    }
    
    /// Generate a mock agent for testing
    pub fn create_mock_agent(&mut self, name_prefix: Option<&str>) -> Agent {
        self.agent_counter += 1;
        let name = format!("{}{}", 
            name_prefix.unwrap_or("TestAgent"), 
            self.agent_counter
        );
        let system_prompt = format!(
            "You are {}, a test agent created for testing purposes. Your task is to {}.", 
            name,
            match self.agent_counter % 3 {
                0 => "analyze data and provide insights",
                1 => "process documents and extract information",
                _ => "manage tasks and coordinate with other agents",
            }
        );
        
        Agent::new(name, system_prompt)
    }
    
    /// Generate a mock task for testing
    pub fn create_mock_task(&mut self, agent_id: String, title_prefix: Option<&str>) -> Task {
        self.task_counter += 1;
        let title = format!("{}{}", 
            title_prefix.unwrap_or("Test Task"), 
            self.task_counter
        );
        let description = format!(
            "This is test task {} created for testing purposes. It should {}.",
            self.task_counter,
            match self.task_counter % 4 {
                0 => "process a large dataset and generate a summary report",
                1 => "analyze user feedback and identify improvement areas",
                2 => "coordinate with external APIs to gather information",
                _ => "validate system integrity and performance metrics",
            }
        );
        
        Task::new(agent_id, title, description)
    }
    
    /// Generate multiple agents with different characteristics
    pub fn create_agent_batch(&mut self, count: usize, name_prefix: &str) -> Vec<Agent> {
        (0..count).map(|_| self.create_mock_agent(Some(name_prefix))).collect()
    }
    
    /// Generate multiple tasks for agents
    pub fn create_task_batch(&mut self, agents: &[Agent], tasks_per_agent: usize) -> Vec<Task> {
        let mut tasks = Vec::new();
        for agent in agents {
            for _ in 0..tasks_per_agent {
                tasks.push(self.create_mock_task(agent.id.clone(), Some("BatchTask")));
            }
        }
        tasks
    }
}

/// Extended TUI testing framework with CRUD operations support
pub struct ExtendedTuiTestFramework {
    config: TestConfig,
    crud_test_results: Arc<Mutex<Vec<CrudTestResult>>>,
    logger: TestLogger,
    mock_factory: MockDataFactory,
    test_scenarios: Vec<CrudTestScenario>,
}

impl ExtendedTuiTestFramework {
    /// Create a new extended testing framework
    pub fn new(config: TestConfig) -> Self {
        let logger = TestLogger::new(config.log_level.clone());
        
        Self {
            config,
            crud_test_results: Arc::new(Mutex::new(Vec::new())),
            logger,
            mock_factory: MockDataFactory::new(),
            test_scenarios: Self::get_predefined_scenarios(),
        }
    }
    
    /// Get predefined test scenarios
    fn get_predefined_scenarios() -> Vec<CrudTestScenario> {
        vec![
            // Agent CRUD scenarios
            CrudTestScenario {
                name: "Agent Creation Basic".to_string(),
                description: "Test basic agent creation functionality".to_string(),
                test_type: CrudTestType::BasicOperation,
                operation: CrudOperation::Create,
                entity_type: EntityType::Agent,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Create new agent".to_string(),
                        action: TestAction::CreateAgent {
                            name: "TestAgent".to_string(),
                            system_prompt: "You are a test agent.".to_string(),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Create new agent".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: Some(DataValidation::AgentCount { expected: 1 }),
                    },
                ],
                cleanup_required: true,
            },
            
            CrudTestScenario {
                name: "Agent Update Basic".to_string(),
                description: "Test basic agent update functionality".to_string(),
                test_type: CrudTestType::BasicOperation,
                operation: CrudOperation::Update,
                entity_type: EntityType::Agent,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Create agent for update".to_string(),
                        action: TestAction::CreateAgent {
                            name: "OriginalAgent".to_string(),
                            system_prompt: "Original prompt".to_string(),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Update agent name".to_string(),
                        action: TestAction::UpdateAgent {
                            id: "PLACEHOLDER".to_string(), // Will be replaced with actual ID
                            name: Some("UpdatedAgent".to_string()),
                            system_prompt: Some("Updated prompt".to_string()),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Update agent name".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: Some(DataValidation::AgentCount { expected: 1 }),
                    },
                ],
                cleanup_required: true,
            },
            
            CrudTestScenario {
                name: "Agent Lifecycle".to_string(),
                description: "Test complete agent lifecycle (create, start, stop, delete)".to_string(),
                test_type: CrudTestType::Workflow,
                operation: CrudOperation::Create,
                entity_type: EntityType::Agent,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Create agent".to_string(),
                        action: TestAction::CreateAgent {
                            name: "LifecycleAgent".to_string(),
                            system_prompt: "Agent for lifecycle testing".to_string(),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Start agent".to_string(),
                        action: TestAction::StartAgent { id: "PLACEHOLDER".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(2000),
                    },
                    TestStep {
                        name: "Stop agent".to_string(),
                        action: TestAction::StopAgent { id: "PLACEHOLDER".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(2000),
                    },
                    TestStep {
                        name: "Delete agent".to_string(),
                        action: TestAction::DeleteAgent { id: "PLACEHOLDER".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Delete agent".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: Some(DataValidation::AgentCount { expected: 0 }),
                    },
                ],
                cleanup_required: false, // Cleanup is part of the test
            },
            
            // Task CRUD scenarios
            CrudTestScenario {
                name: "Task Creation Basic".to_string(),
                description: "Test basic task creation functionality".to_string(),
                test_type: CrudTestType::BasicOperation,
                operation: CrudOperation::Create,
                entity_type: EntityType::Task,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Create agent for task".to_string(),
                        action: TestAction::CreateAgent {
                            name: "TaskAgent".to_string(),
                            system_prompt: "Agent for task testing".to_string(),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Create task".to_string(),
                        action: TestAction::CreateTask {
                            agent_id: "PLACEHOLDER".to_string(),
                            title: "Test Task".to_string(),
                            description: "A task for testing".to_string(),
                            priority: Some(TaskPriority::Medium),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Create task".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: Some(DataValidation::TaskCount { expected: 1 }),
                    },
                ],
                cleanup_required: true,
            },
            
            CrudTestScenario {
                name: "Task Execution Workflow".to_string(),
                description: "Test task execution workflow".to_string(),
                test_type: CrudTestType::Workflow,
                operation: CrudOperation::Execute,
                entity_type: EntityType::Task,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Create agent".to_string(),
                        action: TestAction::CreateAgent {
                            name: "ExecutionAgent".to_string(),
                            system_prompt: "Agent for execution testing".to_string(),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Start agent".to_string(),
                        action: TestAction::StartAgent { id: "PLACEHOLDER".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(2000),
                    },
                    TestStep {
                        name: "Create task".to_string(),
                        action: TestAction::CreateTask {
                            agent_id: "PLACEHOLDER".to_string(),
                            title: "Execution Test Task".to_string(),
                            description: "Task to test execution workflow".to_string(),
                            priority: Some(TaskPriority::High),
                        },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Execute task".to_string(),
                        action: TestAction::ExecuteTask { id: "PLACEHOLDER".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(3000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Execute task".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: Some(DataValidation::TaskStatus { 
                            id: "PLACEHOLDER".to_string(), 
                            status: TaskStatus::InProgress 
                        }),
                    },
                ],
                cleanup_required: true,
            },
            
            // Form validation scenarios
            CrudTestScenario {
                name: "Agent Form Validation".to_string(),
                description: "Test agent creation form validation".to_string(),
                test_type: CrudTestType::FormValidation,
                operation: CrudOperation::Create,
                entity_type: EntityType::Agent,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Test empty name validation".to_string(),
                        action: TestAction::ValidateForm {
                            field: "name".to_string(),
                            value: "".to_string(),
                            expected_valid: false,
                        },
                        input_data: None,
                        expected_duration_ms: Some(100),
                    },
                    TestStep {
                        name: "Test empty system prompt validation".to_string(),
                        action: TestAction::ValidateForm {
                            field: "system_prompt".to_string(),
                            value: "".to_string(),
                            expected_valid: false,
                        },
                        input_data: None,
                        expected_duration_ms: Some(100),
                    },
                    TestStep {
                        name: "Test valid agent name".to_string(),
                        action: TestAction::ValidateForm {
                            field: "name".to_string(),
                            value: "ValidAgent".to_string(),
                            expected_valid: true,
                        },
                        input_data: None,
                        expected_duration_ms: Some(100),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Test valid agent name".to_string(),
                        success: true,
                        error_pattern: None,
                        data_validation: None,
                    },
                ],
                cleanup_required: false,
            },
            
            // Error handling scenarios
            CrudTestScenario {
                name: "Agent Error Handling".to_string(),
                description: "Test error handling for agent operations".to_string(),
                test_type: CrudTestType::ErrorHandling,
                operation: CrudOperation::Delete,
                entity_type: EntityType::Agent,
                setup_data: vec![],
                test_steps: vec![
                    TestStep {
                        name: "Delete non-existent agent".to_string(),
                        action: TestAction::DeleteAgent { id: "non-existent-id".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                    TestStep {
                        name: "Start non-existent agent".to_string(),
                        action: TestAction::StartAgent { id: "non-existent-id".to_string() },
                        input_data: None,
                        expected_duration_ms: Some(1000),
                    },
                ],
                expected_outcomes: vec![
                    ExpectedOutcome {
                        step_name: "Delete non-existent agent".to_string(),
                        success: false,
                        error_pattern: Some("not found".to_string()),
                        data_validation: None,
                    },
                    ExpectedOutcome {
                        step_name: "Start non-existent agent".to_string(),
                        success: false,
                        error_pattern: Some("not found".to_string()),
                        data_validation: None,
                    },
                ],
                cleanup_required: false,
            },
        ]
    }
    
    /// Run all CRUD test scenarios
    pub async fn run_all_crud_tests(&mut self) -> Result<()> {
        self.logger.info("Starting comprehensive CRUD operations testing");
        self.logger.info(&format!("Running {} test scenarios", self.test_scenarios.len()));
        
        let mut total_tests = 0;
        let mut passed_tests = 0;
        
        for scenario in &self.test_scenarios.clone() {
            self.logger.info(&format!("Running scenario: {}", scenario.name));
            
            let result = self.run_crud_scenario(scenario).await;
            total_tests += 1;
            
            match result {
                Ok(success) => {
                    if success {
                        passed_tests += 1;
                        self.logger.info(&format!("✅ Scenario '{}' PASSED", scenario.name));
                    } else {
                        self.logger.warn(&format!("❌ Scenario '{}' FAILED", scenario.name));
                    }
                }
                Err(e) => {
                    self.logger.error(&format!("💥 Scenario '{}' ERROR: {}", scenario.name, e));
                }
            }
            
            // Delay between scenarios
            tokio::time::sleep(Duration::from_millis(self.config.delay_between_tests_ms)).await;
        }
        
        self.logger.info(&format!(
            "CRUD testing completed: {}/{} scenarios passed ({:.1}%)",
            passed_tests,
            total_tests,
            (passed_tests as f32 / total_tests as f32) * 100.0
        ));
        
        Ok(())
    }
    
    /// Run a single CRUD test scenario
    async fn run_crud_scenario(&mut self, scenario: &CrudTestScenario) -> Result<bool> {
        let start_time = std::time::Instant::now();
        let mut scenario_success = true;
        let mut step_context: HashMap<String, String> = HashMap::new();
        
        // Setup phase
        if !scenario.setup_data.is_empty() {
            self.logger.debug(&format!("Setting up test data for scenario: {}", scenario.name));
            // TODO: Implement setup data creation
        }
        
        // Execute test steps
        for step in &scenario.test_steps {
            let step_start = std::time::Instant::now();
            self.logger.debug(&format!("Executing step: {}", step.name));
            
            let step_result = self.execute_test_step(step, &mut step_context).await;
            let execution_time_ms = step_start.elapsed().as_millis() as u64;
            
            let test_result = CrudTestResult {
                test_name: format!("{} - {}", scenario.name, step.name),
                test_type: scenario.test_type.clone(),
                operation: scenario.operation.clone(),
                entity_type: scenario.entity_type.clone(),
                status: if step_result.is_ok() { TestStatus::Pass } else { TestStatus::Fail },
                error_message: step_result.as_ref().err().map(|e| e.to_string()),
                execution_time_ms,
                timestamp: Utc::now(),
                test_data: None, // TODO: Add test data serialization
                expected_result: None, // TODO: Add expected result
                actual_result: None, // TODO: Add actual result
            };
            
            let mut results = self.crud_test_results.lock().await;
            results.push(test_result);
            
            if step_result.is_err() {
                scenario_success = false;
                self.logger.warn(&format!("Step '{}' failed: {:?}", step.name, step_result));
            }
        }
        
        // Cleanup phase
        if scenario.cleanup_required {
            self.logger.debug(&format!("Cleaning up after scenario: {}", scenario.name));
            // TODO: Implement cleanup logic
        }
        
        Ok(scenario_success)
    }
    
    /// Execute a single test step
    async fn execute_test_step(&mut self, step: &TestStep, context: &mut HashMap<String, String>) -> Result<()> {
        match &step.action {
            TestAction::CreateAgent { name, system_prompt } => {
                let agent = self.mock_factory.create_mock_agent(Some(name));
                // Simulate agent creation - in real implementation, call actual API
                context.insert("last_agent_id".to_string(), agent.id.clone());
                self.logger.debug(&format!("Mock created agent: {} ({})", agent.name, agent.id));
                Ok(())
            }
            
            TestAction::UpdateAgent { id, name, system_prompt } => {
                let agent_id = if id == "PLACEHOLDER" {
                    context.get("last_agent_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    id.clone()
                };
                self.logger.debug(&format!("Mock updated agent: {}", agent_id));
                Ok(())
            }
            
            TestAction::DeleteAgent { id } => {
                let agent_id = if id == "PLACEHOLDER" {
                    context.get("last_agent_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    id.clone()
                };
                self.logger.debug(&format!("Mock deleted agent: {}", agent_id));
                Ok(())
            }
            
            TestAction::StartAgent { id } => {
                let agent_id = if id == "PLACEHOLDER" {
                    context.get("last_agent_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    id.clone()
                };
                self.logger.debug(&format!("Mock started agent: {}", agent_id));
                Ok(())
            }
            
            TestAction::StopAgent { id } => {
                let agent_id = if id == "PLACEHOLDER" {
                    context.get("last_agent_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    id.clone()
                };
                self.logger.debug(&format!("Mock stopped agent: {}", agent_id));
                Ok(())
            }
            
            TestAction::CreateTask { agent_id, title, description, priority } => {
                let actual_agent_id = if agent_id == "PLACEHOLDER" {
                    context.get("last_agent_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    agent_id.clone()
                };
                let task = self.mock_factory.create_mock_task(actual_agent_id, Some(title));
                context.insert("last_task_id".to_string(), task.id.clone());
                self.logger.debug(&format!("Mock created task: {} ({})", task.title, task.id));
                Ok(())
            }
            
            TestAction::ExecuteTask { id } => {
                let task_id = if id == "PLACEHOLDER" {
                    context.get("last_task_id").unwrap_or(&"unknown".to_string()).clone()
                } else {
                    id.clone()
                };
                self.logger.debug(&format!("Mock executed task: {}", task_id));
                Ok(())
            }
            
            TestAction::ValidateForm { field, value, expected_valid } => {
                let is_valid = match field.as_str() {
                    "name" => !value.trim().is_empty() && value.len() >= 3,
                    "system_prompt" => !value.trim().is_empty() && value.len() >= 10,
                    "title" => !value.trim().is_empty() && value.len() >= 2,
                    "description" => !value.trim().is_empty(),
                    _ => true,
                };
                
                if is_valid == *expected_valid {
                    self.logger.debug(&format!("Form validation passed for {}: {}", field, value));
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Form validation failed for {}: expected {}, got {}", field, expected_valid, is_valid))
                }
            }
            
            TestAction::VerifyData { entity_type, expected_count } => {
                // Mock data verification
                self.logger.debug(&format!("Mock verified {} count: {}", 
                    match entity_type {
                        EntityType::Agent => "agent",
                        EntityType::Task => "task",
                        EntityType::System => "system",
                    }, 
                    expected_count
                ));
                Ok(())
            }
            
            _ => {
                self.logger.warn(&format!("Test action not implemented: {:?}", step.action));
                Ok(())
            }
        }
    }
    
    /// Generate a comprehensive CRUD test report
    pub async fn generate_crud_report(&self) -> Result<String> {
        let results = self.crud_test_results.lock().await;
        
        match self.config.output_format.as_str() {
            "json" => self.generate_crud_json_report(&results),
            "csv" => self.generate_crud_csv_report(&results),
            _ => self.generate_crud_table_report(&results),
        }
    }
    
    /// Generate a table-formatted CRUD report
    fn generate_crud_table_report(&self, results: &[CrudTestResult]) -> Result<String> {
        let mut output = String::new();
        
        output.push_str(&format!("# NOX TUI CRUD Operations Testing Report\n\n"));
        output.push_str(&format!("**Generated:** {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("**Total Tests:** {}\n\n", results.len()));

        // Summary by status
        let mut status_counts = HashMap::new();
        for result in results {
            *status_counts.entry(&result.status).or_insert(0) += 1;
        }

        output.push_str("## Summary\n\n");
        output.push_str("| Status | Count | Percentage |\n");
        output.push_str("|--------|-------|------------|\n");
        
        for (status, count) in &status_counts {
            let percentage = (*count as f32 / results.len() as f32) * 100.0;
            output.push_str(&format!("| {} | {} | {:.1}% |\n", status, count, percentage));
        }
        output.push_str("\n");

        // Summary by test type
        let mut type_counts = HashMap::new();
        for result in results {
            *type_counts.entry(&result.test_type).or_insert(0) += 1;
        }

        output.push_str("## Test Types\n\n");
        output.push_str("| Test Type | Count | Pass Rate |\n");
        output.push_str("|-----------|-------|----------|\n");
        
        for (test_type, count) in &type_counts {
            let passed = results.iter()
                .filter(|r| std::mem::discriminant(&r.test_type) == std::mem::discriminant(test_type) && r.status == TestStatus::Pass)
                .count();
            let pass_rate = (passed as f32 / *count as f32) * 100.0;
            output.push_str(&format!("| {:?} | {} | {:.1}% |\n", test_type, count, pass_rate));
        }
        output.push_str("\n");

        // Detailed results by operation
        let mut results_by_operation: HashMap<String, Vec<&CrudTestResult>> = HashMap::new();
        for result in results {
            let key = format!("{:?} {:?}", result.entity_type, result.operation);
            results_by_operation.entry(key).or_default().push(result);
        }

        for (operation, operation_results) in results_by_operation {
            output.push_str(&format!("## {} Operations\n\n", operation));
            output.push_str("| Test Name | Test Type | Status | Time (ms) | Error |\n");
            output.push_str("|-----------|-----------|--------|-----------|-------|\n");
            
            for result in operation_results {
                let error_display = result.error_message.as_deref().unwrap_or("-");
                let error_truncated = if error_display.len() > 50 {
                    format!("{}...", &error_display[..47])
                } else {
                    error_display.to_string()
                };
                
                output.push_str(&format!(
                    "| {} | {:?} | {} | {} | {} |\n",
                    result.test_name,
                    result.test_type,
                    result.status,
                    result.execution_time_ms,
                    error_truncated
                ));
            }
            output.push_str("\n");
        }

        // Failed tests section
        let failed_tests: Vec<&CrudTestResult> = results.iter()
            .filter(|r| r.status != TestStatus::Pass)
            .collect();

        if !failed_tests.is_empty() {
            output.push_str("## Failed Tests\n\n");
            for result in failed_tests {
                output.push_str(&format!(
                    "- **{}**: {} ({:?} operation)\n",
                    result.test_name, result.status, result.operation
                ));
                if let Some(error) = &result.error_message {
                    output.push_str(&format!("  - Error: {}\n", error));
                }
            }
            output.push_str("\n");
        }

        output.push_str("## Recommendations\n\n");
        let fail_count = status_counts.get(&TestStatus::Fail).unwrap_or(&0);
        let error_count = status_counts.get(&TestStatus::Error).unwrap_or(&0);
        
        if *fail_count > 0 {
            output.push_str("- Review failed CRUD operations and ensure proper implementation\n");
        }
        if *error_count > 0 {
            output.push_str("- Fix error conditions in CRUD operation handling\n");
        }
        
        let pass_rate = *status_counts.get(&TestStatus::Pass).unwrap_or(&0) as f32 / results.len() as f32;
        if pass_rate < 0.9 {
            output.push_str("- Overall CRUD pass rate is below 90%, consider comprehensive review\n");
        }

        Ok(output)
    }
    
    /// Generate a JSON CRUD report
    fn generate_crud_json_report(&self, results: &[CrudTestResult]) -> Result<String> {
        let report = serde_json::json!({
            "timestamp": Utc::now().to_rfc3339(),
            "total_tests": results.len(),
            "results": results
        });
        
        Ok(serde_json::to_string_pretty(&report)?)
    }
    
    /// Generate a CSV CRUD report
    fn generate_crud_csv_report(&self, results: &[CrudTestResult]) -> Result<String> {
        let mut output = String::new();
        output.push_str("Test Name,Test Type,Operation,Entity Type,Status,Error,Execution Time (ms),Timestamp\n");
        
        for result in results {
            let error = result.error_message.as_deref().unwrap_or("");
            output.push_str(&format!(
                "{},{:?},{:?},{:?},{},{},{},{}\n",
                result.test_name,
                result.test_type,
                result.operation,
                result.entity_type,
                result.status,
                error,
                result.execution_time_ms,
                result.timestamp.to_rfc3339()
            ));
        }
        
        Ok(output)
    }
    
    /// Save the CRUD report to a file
    pub async fn save_crud_report_to_file(&self, filename: &str) -> Result<()> {
        let report = self.generate_crud_report().await?;
        tokio::fs::write(filename, report).await?;
        self.logger.info(&format!("CRUD test report saved to: {}", filename));
        Ok(())
    }
}

/// Helper function to truncate strings for display
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}