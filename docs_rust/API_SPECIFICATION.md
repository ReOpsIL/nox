# Nox API Specification - Rust Backend

## CLI Commands API

### Agent Management Commands

#### `agent add [name] [system_prompt]`
Create a new agent with specified configuration.

**Parameters:**
- `name` (string): Unique agent identifier
- `system_prompt` (string): Agent's system prompt and instructions

**Example:**
```bash
cargo run -- agent add researcher "You are a research assistant specializing in AI and machine learning."
```

**Response:**
```json
{
  "success": true,
  "agent_id": "researcher",
  "message": "Agent 'researcher' created successfully",
  "git_commit": "a7b3c9d"
}
```

#### `agent list`
Display all registered agents and their status.

**Example:**
```bash
cargo run -- agent list
```

**Response:**
```json
{
  "agents": [
    {
      "id": "researcher",
      "name": "Research Assistant",
      "status": "Active",
      "created_at": "2024-01-15T10:30:00Z",
      "last_active": "2024-01-15T14:25:00Z",
      "task_count": 3,
      "memory_usage": "256MB"
    }
  ],
  "total_count": 1,
  "active_count": 1
}
```

#### `agent update [name] [new_prompt]`
Modify an existing agent's system prompt.

**Example:**
```bash
cargo run -- agent update researcher "Enhanced research capabilities with quantum computing expertise."
```

#### `agent delete [name]`
Remove an agent from the registry.

**Example:**
```bash
cargo run -- agent delete researcher
```

### Agent Registry & Discovery Commands

#### `registry query [capability]`
Find agents with specific skills or capabilities.

**Example:**
```bash
cargo run -- registry query "data analysis"
```

**Response:**
```json
{
  "matches": [
    {
      "agent_id": "data_scientist",
      "relevance_score": 0.95,
      "capabilities": ["data analysis", "machine learning", "visualization"]
    }
  ]
}
```

#### `agent skills [name]`
List an agent's current capabilities and skills.

**Example:**
```bash
cargo run -- agent skills researcher
```

**Response:**
```json
{
  "agent_id": "researcher",
  "skills": [
    "research methodology",
    "academic paper analysis",
    "literature review",
    "citation management"
  ],
  "mcp_services": ["web_scraper", "database"],
  "last_updated": "2024-01-15T14:30:00Z"
}
```

### Task Management Commands

#### `task list [agent_id]`
List tasks for a specific agent or all agents.

**Parameters:**
- `agent_id` (optional): Agent ID to filter tasks

**Example:**
```bash
cargo run -- task list
cargo run -- task list agent-1752272015775
```

#### `task create <agent_id> <title> <description>`
Create a new task for an agent.

**Parameters:**
- `agent_id` (required): Target agent ID
- `title` (required): Task title
- `description` (required): Task description

**Example:**
```bash
cargo run -- task create agent-1752272015775 "Research Task" "Analyze quantum computing papers"
```

#### `task update <task_id>`
Update task status or details.

**Parameters:**
- `task_id` (required): Task ID to update

**Example:**
```bash
cargo run -- task update task-123
```

#### `task overview`
Show task overview across all agents.

**Example:**
```bash
cargo run -- task overview
```

**Response:**
```json
{
  "overview": {
    "total_tasks": 15,
    "in_progress": 5,
    "todo": 8,
    "done": 2,
    "blocked": 0
  },
  "agents": [
    {
      "agent_id": "researcher",
      "tasks": {
        "in_progress": 1,
        "todo": 2,
        "done": 5
      }
    }
  ],
  "blocked_tasks": [],
  "critical_tasks": [
    {
      "task_id": "task_123",
      "agent_id": "security_expert",
      "description": "Fix security vulnerability",
      "priority": "Critical"
    }
  ]
}
```

### Communication Commands

#### `message broadcast [message]`
Send message to all active agents.

**Example:**
```bash
cargo run -- message broadcast "New security vulnerability discovered in payment system"
```

#### `message send [target] [message]`
Send direct message to specific agent.

**Example:**
```bash
cargo run -- message send data_scientist "Can you help with the visualization task?"
```

## WebSocket API

### Connection
```
ws://localhost:3000/ws
```

### Message Types

#### Agent Status Updates
```json
{
  "type": "agent_status",
  "data": {
    "agent_id": "researcher",
    "status": "Active",
    "memory_usage": "256MB",
    "cpu_usage": 15.5,
    "last_activity": "2024-01-15T14:30:00Z"
  }
}
```

#### Task Updates
```json
{
  "type": "task_update",
  "data": {
    "task_id": "task_123",
    "agent_id": "researcher",
    "status": "InProgress",
    "progress": 60,
    "updated_at": "2024-01-15T14:30:00Z"
  }
}
```

#### Inter-Agent Messages
```json
{
  "type": "agent_message",
  "data": {
    "from": "researcher",
    "to": "data_scientist",
    "content": "Need help with data visualization",
    "timestamp": "2024-01-15T14:30:00Z",
    "priority": "High"
  }
}
```

#### System Events
```json
{
  "type": "system_event",
  "data": {
    "event": "agent_created",
    "agent_id": "new_agent",
    "timestamp": "2024-01-15T14:30:00Z",
    "details": {
      "created_by": "user",
      "system_prompt": "..."
    }
  }
}
```

## REST API Endpoints

### Agent Management

#### `GET /api/agents`
List all agents with their current status.

**Query Parameters:**
- `page` (optional): Page number for pagination (default: 1)
- `limit` (optional): Number of items per page (default: 20)
- `status` (optional): Filter by agent status

**Response:**
```json
{
  "agents": [
    {
      "id": "researcher",
      "name": "Research Assistant",
      "status": "Active",
      "created_at": "2024-01-15T10:30:00Z"
    },
    {
      "id": "data_scientist",
      "name": "Data Scientist",
      "status": "Inactive",
      "created_at": "2024-01-14T09:15:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 5
  }
}
```

#### `POST /api/agents`
Create a new agent.

**Request Body:**
```json
{
  "name": "researcher",
  "system_prompt": "You are a research assistant...",
  "resource_limits": {
    "max_memory_mb": 512,
    "max_cpu_percent": 50
  }
}
```

**Response:**
```json
{
  "success": true,
  "agent": {
    "id": "researcher",
    "name": "researcher",
    "status": "Inactive",
    "created_at": "2024-01-15T14:30:00Z"
  }
}
```

#### `GET /api/agents/{agent_id}`
Get detailed information about a specific agent.

**Parameters:**
- `agent_id` (path): The unique identifier of the agent

**Response:**
```json
{
  "id": "researcher",
  "name": "Research Assistant",
  "system_prompt": "You are a research assistant...",
  "status": "Active",
  "created_at": "2024-01-15T10:30:00Z",
  "last_active": "2024-01-15T14:25:00Z",
  "resource_limits": {
    "max_memory_mb": 512,
    "max_cpu_percent": 50
  },
  "capabilities": ["research", "analysis", "summarization"],
  "tasks": {
    "total": 5,
    "in_progress": 1,
    "todo": 2,
    "done": 2
  }
}
```

#### `PUT /api/agents/{agent_id}`
Update agent configuration.

**Parameters:**
- `agent_id` (path): The unique identifier of the agent

**Request Body:**
```json
{
  "system_prompt": "Updated system prompt...",
  "resource_limits": {
    "max_memory_mb": 1024
  }
}
```

**Response:**
```json
{
  "success": true,
  "agent": {
    "id": "researcher",
    "name": "researcher",
    "status": "Active",
    "updated_at": "2024-01-15T15:30:00Z"
  }
}
```

#### `DELETE /api/agents/{agent_id}`
Remove an agent from the system.

**Parameters:**
- `agent_id` (path): The unique identifier of the agent to delete

**Response (Success):**
```json
{
  "success": true,
  "message": "Agent {agent_id} deleted successfully"
}
```

**Response (Not Found):**
```json
{
  "success": false,
  "error": "Agent not found",
  "message": "Agent {agent_id} not found"
}
```

**Notes:**
- Automatically stops the agent process if it's currently running
- Removes agent from the registry
- This operation cannot be undone without registry rollback

#### `POST /api/agents/{agent_id}/start`
Start an inactive agent.

**Parameters:**
- `agent_id` (path): The unique identifier of the agent

**Response:**
```json
{
  "success": true,
  "agent_id": "researcher",
  "status": "Active",
  "started_at": "2024-01-15T15:30:00Z"
}
```

#### `POST /api/agents/{agent_id}/stop`
Stop a running agent.

**Parameters:**
- `agent_id` (path): The unique identifier of the agent

**Response:**
```json
{
  "success": true,
  "agent_id": "researcher",
  "status": "Inactive",
  "stopped_at": "2024-01-15T15:30:00Z"
}
```

### Task Management

#### `GET /api/tasks`
List all tasks across agents.

**Query Parameters:**
- `agent_id` (optional): Filter by agent
- `status` (optional): Filter by status (Todo, InProgress, Done, Blocked)
- `priority` (optional): Filter by priority (Low, Medium, High, Critical)
- `page` (optional): Page number for pagination
- `limit` (optional): Number of items per page

**Response:**
```json
{
  "tasks": [
    {
      "id": "task_123",
      "agent_id": "researcher",
      "title": "Analyze quantum papers",
      "description": "Detailed analysis of recent quantum computing research",
      "status": "InProgress",
      "priority": "High",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T14:30:00Z",
      "deadline": "2024-01-16T18:00:00Z",
      "progress": 60
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 1
  }
}
```

#### `POST /api/tasks`
Create a new task.

**Request Body:**
```json
{
  "agent_id": "researcher",
  "title": "Analyze quantum papers",
  "description": "Detailed analysis of recent quantum computing research",
  "priority": "High",
  "deadline": "2024-01-16T18:00:00Z"
}
```

**Response:**
```json
{
  "success": true,
  "task": {
    "id": "task_123",
    "agent_id": "researcher",
    "title": "Analyze quantum papers",
    "status": "Todo",
    "created_at": "2024-01-15T15:30:00Z"
  }
}
```

#### `GET /api/tasks/{task_id}`
Get detailed information about a specific task.

**Parameters:**
- `task_id` (path): The unique identifier of the task

**Response:**
```json
{
  "id": "task_123",
  "agent_id": "researcher",
  "title": "Analyze quantum papers",
  "description": "Detailed analysis of recent quantum computing research",
  "status": "InProgress",
  "priority": "High",
  "created_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T14:30:00Z",
  "deadline": "2024-01-16T18:00:00Z",
  "progress": 60,
  "dependencies": [],
  "subtasks": [
    {
      "id": "subtask_1",
      "title": "Literature review",
      "status": "Done"
    },
    {
      "id": "subtask_2",
      "title": "Analysis",
      "status": "InProgress"
    }
  ]
}
```

#### `PUT /api/tasks/{task_id}`
Update task status or details.

**Parameters:**
- `task_id` (path): The unique identifier of the task

**Request Body:**
```json
{
  "status": "InProgress",
  "progress": 75,
  "description": "Updated description..."
}
```

**Response:**
```json
{
  "success": true,
  "task": {
    "id": "task_123",
    "status": "InProgress",
    "progress": 75,
    "updated_at": "2024-01-15T16:30:00Z"
  }
}
```

#### `DELETE /api/tasks/{task_id}`
Delete a task.

**Parameters:**
- `task_id` (path): The unique identifier of the task

**Response:**
```json
{
  "success": true,
  "message": "Task {task_id} deleted successfully"
}
```

#### `GET /api/tasks/{task_id}/status`
Get the current status of a task.

**Parameters:**
- `task_id` (path): The unique identifier of the task

**Response:**
```json
{
  "task_id": "task_123",
  "status": "InProgress",
  "progress": 75,
  "updated_at": "2024-01-15T16:30:00Z"
}
```

#### `POST /api/tasks/{task_id}/cancel`
Cancel a task.

**Parameters:**
- `task_id` (path): The unique identifier of the task

**Response:**
```json
{
  "success": true,
  "task_id": "task_123",
  "status": "Cancelled",
  "cancelled_at": "2024-01-15T16:30:00Z"
}
```

### Registry Management

#### `GET /api/registry/history`
Get registry change history.

**Query Parameters:**
- `limit` (optional): Number of commits to return (default: 20)

**Response:**
```json
{
  "commits": [
    {
      "hash": "a7b3c9d",
      "message": "Created researcher agent",
      "timestamp": "2024-01-15T14:30:00Z",
      "author": "system"
    }
  ]
}
```

#### `POST /api/registry/rollback`
Rollback registry to a previous state.

**Request Body:**
```json
{
  "commit_hash": "a7b3c9d",
  "confirm": true
}
```

**Response:**
```json
{
  "success": true,
  "message": "Registry rolled back to commit a7b3c9d",
  "timestamp": "2024-01-15T16:30:00Z"
}
```

### MCP Services

#### `GET /api/mcp/services`
List available MCP services.

**Response:**
```json
{
  "services": [
    {
      "name": "web_scraper",
      "description": "Web scraping and data extraction service",
      "version": "1.0.0",
      "capabilities": ["web_scraping", "data_extraction"],
      "resource_requirements": {
        "memory_mb": 256,
        "cpu_percent": 25
      }
    }
  ]
}
```

#### `POST /api/mcp/install`
Install an MCP service for an agent.

**Request Body:**
```json
{
  "service_name": "web_scraper",
  "agent_id": "researcher",
  "auto_approve": false
}
```

**Response:**
```json
{
  "success": true,
  "service": "web_scraper",
  "agent_id": "researcher",
  "status": "pending_approval",
  "approval_id": "approval_123"
}
```

### System Management

#### `GET /api/system/status`
Get the current system status.

**Response:**
```json
{
  "status": "running",
  "uptime": 3600,
  "agents": {
    "total": 5,
    "active": 3,
    "inactive": 2
  },
  "tasks": {
    "total": 15,
    "in_progress": 5,
    "todo": 8,
    "done": 2
  },
  "resources": {
    "memory_usage": "1.2GB",
    "cpu_usage": 35.5,
    "disk_usage": "500MB"
  },
  "version": "0.1.0"
}
```

#### `GET /api/system/health`
Get detailed health metrics.

**Response:**
```json
{
  "status": "healthy",
  "components": {
    "api_server": "healthy",
    "websocket_server": "healthy",
    "agent_manager": "healthy",
    "task_manager": "healthy",
    "registry": "healthy"
  },
  "metrics": {
    "memory": {
      "total": "8GB",
      "used": "1.2GB",
      "free": "6.8GB"
    },
    "cpu": {
      "usage": 35.5,
      "cores": 8
    },
    "disk": {
      "total": "100GB",
      "used": "15GB",
      "free": "85GB"
    }
  },
  "last_updated": "2024-01-15T16:30:00Z"
}
```

## Rust Type Definitions

### Agent Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub resource_limits: ResourceLimits,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Error,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u8,
    pub max_concurrent_tasks: Option<u32>,
}
```

### Task Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub progress: u8,
    pub dependencies: Vec<String>,
    pub requested_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Blocked,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}
```

### Message Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub content: String,
    pub priority: Priority,
    pub timestamp: DateTime<Utc>,
    pub requires_approval: bool,
    pub metadata: Option<MessageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    TaskRequest,
    Response,
    Broadcast,
    Direct,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub task_id: Option<String>,
    pub deadline: Option<DateTime<Utc>>,
    pub dependencies: Option<Vec<String>>,
    pub reply_to: Option<String>,
}
```

## Error Handling

### Error Response Format
```json
{
  "success": false,
  "error": {
    "code": "AGENT_NOT_FOUND",
    "message": "Agent 'researcher' does not exist",
    "details": {
      "agent_id": "researcher",
      "available_agents": ["data_scientist", "web_scraper"]
    }
  },
  "timestamp": "2024-01-15T14:30:00Z"
}
```

### Error Codes
- `AGENT_NOT_FOUND`: Specified agent doesn't exist
- `AGENT_ALREADY_EXISTS`: Agent name already in use
- `INVALID_SYSTEM_PROMPT`: System prompt validation failed
- `RESOURCE_LIMIT_EXCEEDED`: Agent hit resource constraints
- `APPROVAL_REQUIRED`: User approval needed for action
- `MCP_SERVICE_UNAVAILABLE`: Requested MCP service not available
- `TASK_DEPENDENCY_CYCLE`: Circular task dependencies detected
- `GIT_OPERATION_FAILED`: Git backup/rollback failed
- `INVALID_REQUEST`: Request validation failed
- `UNAUTHORIZED`: Authentication required
- `FORBIDDEN`: Insufficient permissions
- `INTERNAL_SERVER_ERROR`: Unexpected server error

## Rate Limiting

### CLI Commands
- **Agent Creation**: 10 per hour per user
- **Registry Modifications**: 100 per hour per user
- **Task Operations**: 1000 per hour per agent

### API Endpoints
- **Standard Endpoints**: 1000 requests/hour per IP
- **Agent Creation**: 10 requests/hour per IP
- **Registry Rollback**: 5 requests/hour per IP

### WebSocket
- **Message Rate**: 100 messages/minute per connection
- **Connection Limit**: 10 concurrent connections per IP
