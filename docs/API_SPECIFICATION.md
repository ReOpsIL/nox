# Nox API Specification

## CLI Commands API

### Agent Management Commands

#### `/add-agent [name] [system_prompt]`
Create a new agent with specified configuration.

**Parameters:**
- `name` (string): Unique agent identifier
- `system_prompt` (string): Agent's system prompt and instructions

**Example:**
```bash
npm run add-agent researcher "You are a research assistant specializing in AI and machine learning."
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

#### `/list-agents`
Display all registered agents and their status.

**Response:**
```json
{
  "agents": [
    {
      "id": "researcher",
      "name": "Research Assistant",
      "status": "active",
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

#### `/update-agent [name] [new_prompt]`
Modify an existing agent's system prompt.

**Example:**
```bash
npm run update-agent researcher "Enhanced research capabilities with quantum computing expertise."
```

#### `/delete-agent [name]`
Remove an agent from the registry.

**Example:**
```bash
npm run delete-agent researcher
```

### Agent Registry & Discovery Commands

#### `/query-registry [capability]`
Find agents with specific skills or capabilities.

**Example:**
```bash
/query-registry "data analysis"
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

#### `/agent-skills [name]`
List an agent's current capabilities and skills.

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

#### `list-tasks [agentId]`
List tasks for a specific agent or all agents.

**Parameters:**
- `agentId` (optional): Agent ID to filter tasks

**Example:**
```bash
npx ts-node src/nox.ts list-tasks
npx ts-node src/nox.ts list-tasks agent-1752272015775
```

#### `create-task <agentId> <title> <description>`
Create a new task for an agent.

**Parameters:**
- `agentId` (required): Target agent ID
- `title` (required): Task title
- `description` (required): Task description

**Example:**
```bash
npx ts-node src/nox.ts create-task agent-1752272015775 "Research Task" "Analyze quantum computing papers"
```

#### `update-task <taskId>`
Update task status or details.

**Parameters:**
- `taskId` (required): Task ID to update

**Example:**
```bash
npx ts-node src/nox.ts update-task task-123
```

#### `task-overview`
Show task overview across all agents.

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
      "priority": "CRITICAL"
    }
  ]
}
```

### Communication Commands

#### `/broadcast [message]`
Send message to all active agents.

**Example:**
```bash
/broadcast "New security vulnerability discovered in payment system"
```

#### `/agent-message [target] [message]`
Send direct message to specific agent.

**Example:**
```bash
/agent-message data_scientist "Can you help with the visualization task?"
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
    "status": "active",
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
    "status": "in_progress",
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
    "priority": "HIGH"
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

**Response:**
```json
{
  "agents": [...],
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

#### `GET /api/agents/{agent_id}`
Get detailed information about a specific agent.

#### `PUT /api/agents/{agent_id}`
Update agent configuration.

#### `DELETE /api/agents/{agent_id}`
Remove an agent.

### Task Management

#### `GET /api/tasks`
List all tasks across agents.

**Query Parameters:**
- `agent_id` (optional): Filter by agent
- `status` (optional): Filter by status
- `priority` (optional): Filter by priority

#### `POST /api/tasks`
Create a new task.

**Request Body:**
```json
{
  "agent_id": "researcher",
  "title": "Analyze quantum papers",
  "description": "Detailed analysis of recent quantum computing research",
  "priority": "HIGH",
  "deadline": "2024-01-16T18:00:00Z"
}
```

#### `PUT /api/tasks/{task_id}`
Update task status or details.

### Registry Management

#### `GET /api/registry/history`
Get registry change history.

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

### MCP Services

#### `GET /api/mcp/services`
List available MCP services.

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

## Internal Message Protocols

### Agent-to-Agent Messages
```typescript
interface AgentMessage {
  id: string;
  from: string;
  to: string | 'broadcast';
  type: 'task_request' | 'response' | 'broadcast' | 'direct' | 'system';
  content: string;
  priority: 'HIGH' | 'MEDIUM' | 'LOW' | 'CRITICAL';
  timestamp: string;
  requires_approval: boolean;
  metadata?: {
    task_id?: string;
    deadline?: string;
    dependencies?: string[];
    reply_to?: string;
  };
}
```

### System Messages
```typescript
interface SystemMessage {
  type: 'agent_spawn' | 'agent_kill' | 'task_create' | 'approval_request';
  source: 'user' | 'agent' | 'system';
  target?: string;
  payload: any;
  timestamp: string;
}
```

### Task Delegation Protocol
```typescript
interface TaskDelegation {
  from_agent: string;
  to_agent: string;
  task: {
    title: string;
    description: string;
    priority: Priority;
    deadline?: string;
    dependencies: string[];
  };
  delegation_type: 'request' | 'assignment' | 'collaboration';
  requires_approval: boolean;
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