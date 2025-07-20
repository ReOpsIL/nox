# Design Document

## Overview

The improved task management workflows feature will transform Nox from a simple task executor into a sophisticated workflow orchestration platform. This enhancement builds upon the existing task management foundation while adding advanced capabilities including task dependencies, conditional execution, parallel processing, retry mechanisms, task templates, scheduling, and comprehensive monitoring.

The design maintains backward compatibility with the current task system while introducing new workflow-specific data structures and management components. The implementation leverages Rust's async capabilities and the existing Actix Web framework to provide high-performance, concurrent workflow execution.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Workflow Management Layer                 │
├─────────────────┬─────────────────┬─────────────────────────┤
│ Workflow Engine │ Template Engine │    Scheduler Engine     │
├─────────────────┼─────────────────┼─────────────────────────┤
│ Dependency Mgr  │ Condition Eval  │   Notification Mgr      │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                 Enhanced Task Management                     │
├─────────────────┬─────────────────┬─────────────────────────┤
│ Task Executor   │ Retry Manager   │   Progress Tracker      │
├─────────────────┼─────────────────┼─────────────────────────┤
│ Parallel Coord  │ Error Handler   │   Metrics Collector     │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   Existing Nox Core                         │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Task Manager   │ Agent Manager   │   Registry Manager      │
├─────────────────┼─────────────────┼─────────────────────────┤
│ Claude Process  │ Message Broker  │    Git Manager          │
└─────────────────┴─────────────────┴─────────────────────────┘
```

### Component Integration

The workflow system integrates with existing components:
- **Task Manager**: Extended with workflow-aware task execution
- **Registry Manager**: Enhanced to store workflow definitions and templates
- **API Routes**: New endpoints for workflow operations
- **WebSocket**: Real-time workflow progress updates
- **CLI Commands**: New workflow management commands

## Components and Interfaces

### 1. Workflow Engine (`src/core/workflow_engine.rs`)

The central orchestration component that manages workflow execution:

```rust
pub struct WorkflowEngine {
    dependency_manager: Arc<DependencyManager>,
    condition_evaluator: Arc<ConditionEvaluator>,
    parallel_coordinator: Arc<ParallelCoordinator>,
    retry_manager: Arc<RetryManager>,
    progress_tracker: Arc<ProgressTracker>,
    notification_manager: Arc<NotificationManager>,
}

impl WorkflowEngine {
    pub async fn execute_workflow(&self, workflow_id: &str) -> Result<WorkflowExecution>;
    pub async fn pause_workflow(&self, workflow_id: &str) -> Result<()>;
    pub async fn resume_workflow(&self, workflow_id: &str) -> Result<()>;
    pub async fn cancel_workflow(&self, workflow_id: &str) -> Result<()>;
    pub async fn get_workflow_status(&self, workflow_id: &str) -> Result<WorkflowStatus>;
}
```

### 2. Dependency Manager (`src/core/dependency_manager.rs`)

Manages task dependencies and execution ordering:

```rust
pub struct DependencyManager {
    dependency_graph: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl DependencyManager {
    pub async fn add_dependency(&self, task_id: &str, depends_on: &str) -> Result<()>;
    pub async fn remove_dependency(&self, task_id: &str, depends_on: &str) -> Result<()>;
    pub async fn get_ready_tasks(&self, workflow_id: &str) -> Result<Vec<String>>;
    pub async fn mark_task_completed(&self, task_id: &str) -> Result<Vec<String>>;
    pub async fn validate_dependencies(&self, workflow_id: &str) -> Result<()>;
}
```

### 3. Template Engine (`src/core/template_engine.rs`)

Handles task template creation and instantiation:

```rust
pub struct TemplateEngine {
    template_registry: Arc<Mutex<HashMap<String, TaskTemplate>>>,
}

impl TemplateEngine {
    pub async fn create_template(&self, template: TaskTemplate) -> Result<()>;
    pub async fn instantiate_template(&self, template_id: &str, params: HashMap<String, String>) -> Result<Vec<Task>>;
    pub async fn list_templates(&self) -> Result<Vec<TaskTemplate>>;
    pub async fn validate_template(&self, template: &TaskTemplate) -> Result<()>;
}
```

### 4. Scheduler Engine (`src/core/scheduler_engine.rs`)

Manages scheduled and recurring workflows:

```rust
pub struct SchedulerEngine {
    scheduled_workflows: Arc<Mutex<HashMap<String, ScheduledWorkflow>>>,
    cron_scheduler: Arc<Mutex<CronScheduler>>,
}

impl SchedulerEngine {
    pub async fn schedule_workflow(&self, workflow_id: &str, schedule: Schedule) -> Result<()>;
    pub async fn unschedule_workflow(&self, workflow_id: &str) -> Result<()>;
    pub async fn get_scheduled_workflows(&self) -> Result<Vec<ScheduledWorkflow>>;
    pub async fn trigger_scheduled_workflows(&self) -> Result<()>;
}
```

### 5. Parallel Coordinator (`src/core/parallel_coordinator.rs`)

Manages concurrent task execution with resource limits:

```rust
pub struct ParallelCoordinator {
    execution_semaphore: Arc<Semaphore>,
    agent_semaphores: Arc<Mutex<HashMap<String, Arc<Semaphore>>>>,
    active_tasks: Arc<Mutex<HashMap<String, TaskExecution>>>,
}

impl ParallelCoordinator {
    pub async fn execute_task_parallel(&self, task: Task) -> Result<TaskHandle>;
    pub async fn wait_for_completion(&self, handles: Vec<TaskHandle>) -> Result<Vec<TaskResult>>;
    pub async fn set_concurrency_limit(&self, agent_id: Option<&str>, limit: usize) -> Result<()>;
}
```

### 6. Enhanced API Routes (`src/api/workflow_routes.rs`)

New API endpoints for workflow management:

```rust
// Workflow endpoints
POST   /api/workflows                    // Create workflow
GET    /api/workflows                    // List workflows
GET    /api/workflows/{id}               // Get workflow details
PUT    /api/workflows/{id}               // Update workflow
DELETE /api/workflows/{id}               // Delete workflow
POST   /api/workflows/{id}/execute       // Execute workflow
POST   /api/workflows/{id}/pause         // Pause workflow
POST   /api/workflows/{id}/resume        // Resume workflow
POST   /api/workflows/{id}/cancel        // Cancel workflow

// Template endpoints
POST   /api/templates                    // Create template
GET    /api/templates                    // List templates
GET    /api/templates/{id}               // Get template
PUT    /api/templates/{id}               // Update template
DELETE /api/templates/{id}               // Delete template
POST   /api/templates/{id}/instantiate   // Instantiate template

// Schedule endpoints
POST   /api/schedules                    // Create schedule
GET    /api/schedules                    // List schedules
PUT    /api/schedules/{id}               // Update schedule
DELETE /api/schedules/{id}               // Delete schedule

// Analytics endpoints
GET    /api/workflows/analytics          // Workflow analytics
GET    /api/workflows/{id}/history       // Workflow execution history
```

## Data Models

### Enhanced Task Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    // Existing fields
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub progress: u8,
    pub metadata: HashMap<String, String>,
    
    // New workflow fields
    pub workflow_id: Option<String>,
    pub dependencies: Vec<String>,
    pub conditions: Vec<TaskCondition>,
    pub retry_config: Option<RetryConfig>,
    pub timeout_seconds: Option<u64>,
    pub parallel_group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCondition {
    pub condition_type: ConditionType,
    pub target_task_id: String,
    pub operator: ComparisonOperator,
    pub expected_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionType {
    TaskStatus,
    TaskOutput,
    TaskMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    Contains,
    GreaterThan,
    LessThan,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_strategy: BackoffStrategy,
    pub retry_on_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    Fixed { delay_seconds: u64 },
    Exponential { initial_delay_seconds: u64, multiplier: f64 },
    Linear { initial_delay_seconds: u64, increment_seconds: u64 },
}
```

### Workflow Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub tasks: Vec<Task>,
    pub status: WorkflowStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: String,
    pub metadata: HashMap<String, String>,
    pub notification_config: Option<NotificationConfig>,
    pub concurrency_limits: HashMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Draft,
    Ready,
    Running,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub on_completion: bool,
    pub on_failure: bool,
    pub on_delay: bool,
    pub delay_threshold_minutes: u32,
    pub webhook_url: Option<String>,
}
```

### Task Template Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub parameters: Vec<TemplateParameter>,
    pub task_template: Task,
    pub created_at: DateTime<Utc>,
    pub created_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub description: String,
    pub validation_regex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Integer,
    Boolean,
    AgentId,
    TaskId,
}
```

### Schedule Model

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledWorkflow {
    pub id: String,
    pub workflow_id: String,
    pub schedule: Schedule,
    pub enabled: bool,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: DateTime<Utc>,
    pub execution_count: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schedule {
    pub schedule_type: ScheduleType,
    pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    Once { execute_at: DateTime<Utc> },
    Cron { expression: String },
    Interval { interval_seconds: u64 },
}
```

## Error Handling

### Workflow-Specific Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Circular dependency detected in workflow {workflow_id}")]
    CircularDependency { workflow_id: String },
    
    #[error("Task {task_id} dependency {dependency_id} not found")]
    DependencyNotFound { task_id: String, dependency_id: String },
    
    #[error("Condition evaluation failed for task {task_id}: {reason}")]
    ConditionEvaluationFailed { task_id: String, reason: String },
    
    #[error("Template parameter {parameter} is required but not provided")]
    MissingTemplateParameter { parameter: String },
    
    #[error("Invalid cron expression: {expression}")]
    InvalidCronExpression { expression: String },
    
    #[error("Workflow {workflow_id} is not in a state that allows {operation}")]
    InvalidWorkflowState { workflow_id: String, operation: String },
    
    #[error("Concurrency limit exceeded for agent {agent_id}")]
    ConcurrencyLimitExceeded { agent_id: String },
}
```

### Error Recovery Strategies

1. **Dependency Failures**: When a task fails, dependent tasks are automatically blocked and marked as "Blocked"
2. **Condition Failures**: Failed condition evaluations are logged and treated as condition not met
3. **Template Errors**: Validation errors during template instantiation prevent workflow creation
4. **Concurrency Limits**: Tasks wait in queue when limits are exceeded
5. **Retry Exhaustion**: Tasks are marked as failed after all retry attempts are exhausted

## Testing Strategy

### Unit Tests

1. **Dependency Manager Tests**
   - Circular dependency detection
   - Correct task ordering
   - Dependency resolution after task completion

2. **Condition Evaluator Tests**
   - Various condition types and operators
   - Complex logical expressions
   - Error handling for invalid conditions

3. **Template Engine Tests**
   - Parameter substitution
   - Validation of required parameters
   - Template versioning

4. **Scheduler Tests**
   - Cron expression parsing
   - Schedule calculation
   - Timezone handling

### Integration Tests

1. **End-to-End Workflow Tests**
   - Simple linear workflows
   - Complex branching workflows
   - Parallel execution workflows
   - Error recovery scenarios

2. **API Integration Tests**
   - Workflow CRUD operations
   - Template management
   - Schedule management
   - Real-time progress updates

3. **Performance Tests**
   - Large workflow execution
   - High concurrency scenarios
   - Memory usage under load
   - Database performance with many workflows

### Test Data and Fixtures

```rust
// Test workflow with dependencies
pub fn create_test_workflow_with_dependencies() -> Workflow {
    let mut workflow = Workflow::new("test-workflow".to_string());
    
    let task1 = Task::new("agent-1".to_string(), "Task 1".to_string(), "First task".to_string());
    let mut task2 = Task::new("agent-2".to_string(), "Task 2".to_string(), "Second task".to_string());
    task2.dependencies = vec![task1.id.clone()];
    
    workflow.tasks = vec![task1, task2];
    workflow
}

// Test template with parameters
pub fn create_test_template() -> TaskTemplate {
    TaskTemplate {
        id: "test-template".to_string(),
        name: "Test Template".to_string(),
        parameters: vec![
            TemplateParameter {
                name: "agent_id".to_string(),
                parameter_type: ParameterType::AgentId,
                required: true,
                default_value: None,
                description: "Target agent ID".to_string(),
                validation_regex: None,
            }
        ],
        task_template: Task::new("{{agent_id}}".to_string(), "{{title}}".to_string(), "{{description}}".to_string()),
        // ... other fields
    }
}
```

### Monitoring and Observability

1. **Metrics Collection**
   - Workflow execution times
   - Task success/failure rates
   - Concurrency utilization
   - Queue depths

2. **Logging Strategy**
   - Structured logging with workflow/task context
   - Performance logging for bottleneck identification
   - Error logging with full context

3. **Health Checks**
   - Workflow engine health
   - Scheduler health
   - Database connectivity
   - Resource utilization

The design ensures scalability, maintainability, and robust error handling while building upon Nox's existing architecture. The modular approach allows for incremental implementation and testing of individual components.