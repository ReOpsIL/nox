# Nox Technical Architecture - Rust Backend

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│   CLI Commands  │ PrimeReact UI   │    WebSocket Client     │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    API & WebSocket Layer                     │
├─────────────────┬─────────────────┬─────────────────────────┤
│   Actix Web     │  WebSocket      │    API Routes           │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                    Nox Core Engine                          │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Agent Manager  │ Message Broker  │    Registry Manager     │
├─────────────────┼─────────────────┼─────────────────────────┤
│  Task Manager   │   Git Manager   │   Approval Manager      │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   Agent Processes                           │
├─────────────────┬─────────────────┬─────────────────────────┤
│   Claude CLI    │   Claude CLI    │     Claude CLI          │
│   Process #1    │   Process #2    │     Process #N          │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                   MCP Services Layer                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Docker Service │  Docker Service │   Docker Service        │
│  Container #1   │  Container #2   │   Container #N          │
└─────────────────┴─────────────────┴─────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────────┐
│                 Storage & Persistence                       │
├─────────────────┬─────────────────┬─────────────────────────┤
│  TOML Registry  │   Git History   │   Task Markdown Files   │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## Core Components

### 1. Agent Manager
**Responsibility**: Lifecycle management of Claude CLI processes

```rust
// Agent manager trait defining the interface
pub trait AgentManagerTrait {
    async fn spawn_agent(&mut self, config: &AgentConfig) -> Result<AgentProcess, Error>;
    async fn kill_agent(&mut self, agent_id: &str) -> Result<(), Error>;
    async fn restart_agent(&mut self, agent_id: &str) -> Result<(), Error>;
    async fn get_agent_status(&self, agent_id: &str) -> Result<AgentStatus, Error>;
}

// Concrete implementation
pub struct AgentManager {
    processes: Arc<Mutex<HashMap<String, AgentProcess>>>,
    health_monitor: Arc<HealthMonitor>,
}

// Agent process data structure
pub struct AgentProcess {
    pub id: String,
    pub process: Child,
    pub status: AgentStatus,
    pub start_time: DateTime<Utc>,
    pub restart_count: u32,
    pub memory_usage: u64,
    pub cpu_usage: f32,
}
```

### 2. Message Broker
**Responsibility**: Inter-agent communication routing

```rust
pub struct MessageBroker {
    subscribers: Arc<Mutex<HashMap<String, Vec<AgentSubscriber>>>>,
    message_queue: Arc<Mutex<PriorityQueue<AgentMessage>>>,
}

impl MessageBroker {
    pub async fn send_message(&self, message: Message) -> Result<(), Error>;
    pub async fn broadcast_message(&self, message: Message) -> Result<(), Error>;
    pub async fn subscribe_agent(&self, agent_id: &str, topics: Vec<String>) -> Result<(), Error>;
    pub async fn route_message(&self, message: Message) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub content: String,
    pub priority: Priority,
    pub timestamp: DateTime<Utc>,
    pub metadata: Option<MessageMetadata>,
}
```

### 3. Registry Manager
**Responsibility**: Agent configuration and state persistence

```rust
pub struct RegistryManager {
    registry_path: PathBuf,
    git_manager: Arc<Mutex<GitManager>>,
}

impl RegistryManager {
    pub async fn save_agent(&mut self, agent: &AgentConfig) -> Result<(), Error>;
    pub async fn load_agent(&self, agent_id: &str) -> Result<AgentConfig, Error>;
    pub async fn delete_agent(&mut self, agent_id: &str) -> Result<(), Error>;
    pub async fn list_agents(&self) -> Result<Vec<AgentConfig>, Error>;
    pub async fn backup(&self) -> Result<String, Error>; // Returns commit hash
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
    pub resource_limits: ResourceLimits,
    pub capabilities: Vec<String>,
    pub relationships: Vec<AgentRelationship>,
}
```

### 4. Task Manager
**Responsibility**: Task tracking and markdown file management

```rust
pub struct TaskManager {
    task_watcher: Arc<Mutex<FileWatcher>>,
    task_queue: Arc<Mutex<TaskQueue>>,
}

impl TaskManager {
    pub async fn create_task(&mut self, agent_id: &str, task: Task) -> Result<(), Error>;
    pub async fn update_task_status(&mut self, task_id: &str, status: TaskStatus) -> Result<(), Error>;
    pub async fn delegate_task(&mut self, from_agent: &str, to_agent: &str, task: Task) -> Result<(), Error>;
    pub async fn get_agent_tasks(&self, agent_id: &str) -> Result<Vec<Task>, Error>;
    pub async fn watch_task_files(&self) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>,
    pub requested_by: String,
}
```

### 5. API Server
**Responsibility**: RESTful API endpoints and WebSocket server

```rust
pub struct ApiServer {
    config: ApiConfig,
    agent_manager: Arc<Mutex<AgentManager>>,
    registry_manager: Arc<Mutex<RegistryManager>>,
    task_manager: Arc<Mutex<TaskManager>>,
    message_broker: Arc<Mutex<MessageBroker>>,
}

impl ApiServer {
    pub async fn start(&self) -> Result<Server, Error>;
    pub async fn stop(&self) -> Result<(), Error>;
    pub fn configure_routes(cfg: &mut web::ServiceConfig);
    pub fn configure_websocket(cfg: &mut web::ServiceConfig);
}

pub async fn start_server(config: &Config) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .service(
                web::scope("/api")
                    .service(routes::agent::routes())
                    .service(routes::task::routes())
                    .service(routes::system::routes())
                    .service(routes::registry::routes())
                    .service(routes::mcp::routes())
            )
            .service(web::resource("/ws").route(web::get().to(websocket::ws_handler)))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
}
```

## Data Flow Architecture

### Agent Spawning Flow
```
User Command → Registry Manager → Agent Manager → Claude CLI Process → Health Monitor
     ↓              ↓                    ↓              ↓              ↓
Git Commit ← Agent Config ← Process Spawn ← Session Init ← Status Check
```

### Message Flow
```
Agent A → Message Broker → Routing Logic → Agent B
   ↓            ↓              ↓            ↓
Task File ← Message Log ← Priority Queue ← Task File
```

### Task Delegation Flow
```
Agent Request → Task Manager → Markdown Update → Target Agent Notification
      ↓             ↓              ↓                    ↓
  Approval? → Git Commit → File Watcher → Message Broker
```

### API Request Flow
```
Client Request → Actix Web → Middleware → Route Handler → Core Component → Response
      ↓             ↓           ↓             ↓              ↓              ↓
Authentication → Validation → Logging → Business Logic → Data Access → JSON Response
```

## Process Management Strategy

### Agent Process Lifecycle
1. **Spawn**: Create Claude CLI process with system prompt
2. **Initialize**: Send agent configuration and context
3. **Monitor**: Track health, memory, CPU usage
4. **Communicate**: Route messages to/from agent
5. **Persist**: Save state and conversations
6. **Restart**: Auto-recovery on crash
7. **Cleanup**: Graceful shutdown and resource cleanup

### Resource Management
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u8,
    pub max_concurrent_tasks: u32,
    pub max_spawn_rate: u32, // per hour
    pub max_depth: u32, // spawned agent hierarchy
}

pub struct ResourceMonitor {
    pub async fn check_limits(&self, agent_id: &str) -> Result<ResourceStatus, Error>;
    pub async fn enforce_throttling(&self, agent_id: &str) -> Result<(), Error>;
    pub async fn trigger_circuit_breaker(&self, agent_id: &str) -> Result<(), Error>;
}
```

## Security Architecture

### Sandboxing Strategy
- **Process Isolation**: Each agent in separate Claude CLI process
- **File System Limits**: Restricted to `.nox-registry/` directory
- **Network Isolation**: MCP containers on isolated Docker network
- **Resource Quotas**: Memory, CPU, and spawn rate limits
- **API Authentication**: JWT-based authentication for API endpoints

### Approval Workflow
```rust
pub struct ApprovalManager {
    pending_approvals: Arc<Mutex<HashMap<String, ApprovalRequest>>>,
}

impl ApprovalManager {
    pub async fn request_approval(&mut self, request: ApprovalRequest) -> Result<bool, Error>;
    pub async fn auto_approve(&self, request: &ApprovalRequest) -> Result<bool, Error>;
    pub async fn escalate_to_user(&self, request: &ApprovalRequest) -> Result<(), Error>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub request_type: ApprovalType,
    pub description: String,
    pub risk_level: RiskLevel,
    pub auto_approve: bool,
}
```

### API Security
```rust
// Authentication middleware
pub fn authentication() -> impl Transform<ServiceRequest, Response = ServiceResponse, Error = Error> {
    // JWT validation logic
}

// Request validation middleware
pub fn request_validation() -> impl Transform<ServiceRequest, Response = ServiceResponse, Error = Error> {
    // Input validation logic
}

// Rate limiting middleware
pub fn rate_limiting(config: RateLimitConfig) -> impl Transform<ServiceRequest, Response = ServiceResponse, Error = Error> {
    // Rate limiting logic
}
```

## Storage Architecture

### TOML Registry Structure
```
.nox-registry/
├── agents.toml                # Agent configurations
├── mcp-services.toml          # MCP service registry
├── agent-relationships.toml   # Inter-agent connections
├── resource-usage.toml        # Resource consumption data
├── approval-history.toml      # User approval decisions
├── system-config.toml         # System configuration
├── agents/[agent_id]/
│   ├── conversations/          # Chat history (TOML files)
│   │   ├── 2024-01-15.toml
│   │   └── 2024-01-16.toml
│   ├── state.toml             # Current agent state
│   ├── tasks.toml             # Task history and metrics
│   └── memory.toml            # Long-term memory
└── tasks/                     # Markdown task files
    ├── research_agent.md
    └── data_scientist.md
```

### Git Integration Strategy
- **Auto-commit**: Before any registry modification
- **Branching**: Experimental agent configurations
- **Rollback**: Time-based and commit-based recovery
- **Remote Backup**: Optional GitHub/GitLab integration

## MCP Integration Architecture

### Service Discovery Flow
```
Agent Request → Docker Hub API → Service Evaluation → User Approval → Container Pull → Integration
```

### Container Management
```rust
pub struct MCPServiceManager {
    docker: Arc<DockerApi>,
    service_registry: Arc<Mutex<HashMap<String, MCPService>>>,
}

impl MCPServiceManager {
    pub async fn discover_services(&self, query: &str) -> Result<Vec<MCPService>, Error>;
    pub async fn install_service(&mut self, service_name: &str, agent_id: &str) -> Result<(), Error>;
    pub async fn configure_service(&mut self, service: &MCPService, agent: &AgentConfig) -> Result<(), Error>;
    pub async fn remove_service(&mut self, service_name: &str, agent_id: &str) -> Result<(), Error>;
}
```

## Performance Considerations

### Scalability Targets
- **Concurrent Agents**: Up to 50 agents
- **Message Throughput**: 1000 messages/minute
- **Task Updates**: Real-time markdown file updates
- **Memory Usage**: <4GB total system memory
- **Startup Time**: <30 seconds for full system
- **API Response Time**: <100ms for standard requests

### Optimization Strategies
- **Async/Await**: Non-blocking I/O operations
- **Connection Pooling**: Reuse Claude CLI connections
- **Message Batching**: Batch low-priority messages
- **Lazy Loading**: Load agent configurations on demand
- **Caching**: Cache frequently accessed registry data
- **Compression**: Compress conversation logs
- **Actix Web Workers**: Multiple worker threads for API server

## Monitoring & Observability

### Health Checks
- Agent process health
- Resource consumption
- Message queue backlog
- Git repository status
- Docker container health
- API server status

### Metrics Collection
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub agent_count: u32,
    pub active_agents: u32,
    pub messages_per_minute: u32,
    pub average_response_time: f64,
    pub resource_usage: ResourceUsage,
    pub error_rate: f64,
    pub uptime: u64,
    pub api_requests_per_minute: u32,
    pub websocket_connections: u32,
}
```

### Logging Strategy
- **Structured Logging**: JSON format with correlation IDs
- **Log Levels**: Trace, Debug, Info, Warn, Error
- **Audit Trail**: All user actions and agent decisions
- **Retention**: 30 days default, configurable
- **Tracing**: OpenTelemetry integration for distributed tracing

## Frontend Integration

### PrimeReact Frontend
- **API Client**: TypeScript client for Rust backend API
- **WebSocket Client**: Real-time updates from backend
- **Authentication**: JWT-based authentication
- **State Management**: React Context API or Redux
- **Responsive Design**: Mobile-friendly UI

### API Integration
```typescript
// frontend/src/api/client.ts
import axios from 'axios';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:3000/api';

const apiClient = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Add authentication interceptor
apiClient.interceptors.request.use(config => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

export default apiClient;
```

### WebSocket Integration
```typescript
// frontend/src/hooks/useWebSocket.ts
import { useState, useEffect, useCallback } from 'react';

export const useWebSocket = (url: string) => {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [messages, setMessages] = useState<any[]>([]);

  useEffect(() => {
    const ws = new WebSocket(url);

    ws.onopen = () => {
      setIsConnected(true);
    };

    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      setMessages(prev => [...prev, data]);
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    setSocket(ws);

    return () => {
      ws.close();
    };
  }, [url]);

  const sendMessage = useCallback((data: any) => {
    if (socket && isConnected) {
      socket.send(JSON.stringify(data));
    }
  }, [socket, isConnected]);

  return { isConnected, messages, sendMessage };
};
```
