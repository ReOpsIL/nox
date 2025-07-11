# Nox Technical Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────┐
│                        User Interface                        │
├─────────────────┬─────────────────┬─────────────────────────┤
│   CLI Commands  │   Web Dashboard │    WebSocket Client     │
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
│  JSON Registry  │   Git History   │   Task Markdown Files  │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## Core Components

### 1. Agent Manager
**Responsibility**: Lifecycle management of Claude CLI processes

```typescript
class AgentManager {
  private processes: Map<string, AgentProcess>;
  private healthMonitor: HealthMonitor;
  
  async spawnAgent(config: AgentConfig): Promise<AgentProcess>
  async killAgent(agentId: string): Promise<void>
  async restartAgent(agentId: string): Promise<void>
  async getAgentStatus(agentId: string): Promise<AgentStatus>
}

interface AgentProcess {
  id: string;
  process: ChildProcess;
  status: 'running' | 'crashed' | 'stopped';
  startTime: Date;
  restartCount: number;
  memoryUsage: number;
  cpuUsage: number;
}
```

### 2. Message Broker
**Responsibility**: Inter-agent communication routing

```typescript
class MessageBroker {
  private subscribers: Map<string, AgentSubscriber[]>;
  private messageQueue: PriorityQueue<AgentMessage>;
  
  async sendMessage(message: AgentMessage): Promise<void>
  async broadcastMessage(message: AgentMessage): Promise<void>
  async subscribeAgent(agentId: string, topics: string[]): Promise<void>
  async routeMessage(message: AgentMessage): Promise<void>
}

interface AgentMessage {
  id: string;
  from: string;
  to: string | 'broadcast';
  type: MessageType;
  content: string;
  priority: Priority;
  timestamp: Date;
  metadata?: MessageMetadata;
}
```

### 3. Registry Manager
**Responsibility**: Agent configuration and state persistence

```typescript
class RegistryManager {
  private registryPath: string;
  private gitManager: GitManager;
  
  async saveAgent(agent: AgentConfig): Promise<void>
  async loadAgent(agentId: string): Promise<AgentConfig>
  async deleteAgent(agentId: string): Promise<void>
  async listAgents(): Promise<AgentConfig[]>
  async backup(): Promise<string> // Returns commit hash
}

interface AgentConfig {
  id: string;
  name: string;
  systemPrompt: string;
  status: 'active' | 'inactive' | 'error';
  createdAt: Date;
  lastModified: Date;
  resourceLimits: ResourceLimits;
  capabilities: string[];
  relationships: AgentRelationship[];
}
```

### 4. Task Manager
**Responsibility**: Task tracking and markdown file management

```typescript
class TaskManager {
  private taskWatcher: FileWatcher;
  private taskQueue: TaskQueue;
  
  async createTask(agentId: string, task: Task): Promise<void>
  async updateTaskStatus(taskId: string, status: TaskStatus): Promise<void>
  async delegateTask(fromAgent: string, toAgent: string, task: Task): Promise<void>
  async getAgentTasks(agentId: string): Promise<Task[]>
  async watchTaskFiles(): Promise<void>
}

interface Task {
  id: string;
  agentId: string;
  title: string;
  description: string;
  status: 'todo' | 'inprogress' | 'done';
  priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
  createdAt: Date;
  deadline?: Date;
  dependencies: string[];
  requestedBy: string;
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
```typescript
interface ResourceLimits {
  maxMemoryMB: number;
  maxCPUPercent: number;
  maxConcurrentTasks: number;
  maxSpawnRate: number; // per hour
  maxDepth: number; // spawned agent hierarchy
}

class ResourceMonitor {
  async checkLimits(agentId: string): Promise<ResourceStatus>
  async enforceThrottling(agentId: string): Promise<void>
  async triggerCircuitBreaker(agentId: string): Promise<void>
}
```

## Security Architecture

### Sandboxing Strategy
- **Process Isolation**: Each agent in separate Claude CLI process
- **File System Limits**: Restricted to `.nox-registry/` directory
- **Network Isolation**: MCP containers on isolated Docker network
- **Resource Quotas**: Memory, CPU, and spawn rate limits

### Approval Workflow
```typescript
class ApprovalManager {
  private pendingApprovals: Map<string, ApprovalRequest>;
  
  async requestApproval(request: ApprovalRequest): Promise<boolean>
  async autoApprove(request: ApprovalRequest): Promise<boolean>
  async escalateToUser(request: ApprovalRequest): Promise<void>
}

interface ApprovalRequest {
  id: string;
  agentId: string;
  type: 'agent_creation' | 'mcp_installation' | 'external_communication';
  description: string;
  riskLevel: 'LOW' | 'MEDIUM' | 'HIGH';
  autoApprove: boolean;
}
```

## Storage Architecture

### JSON Registry Structure
```
.nox-registry/
├── agents.json                 # Agent configurations
├── mcp-services.json          # MCP service registry
├── agent-relationships.json   # Inter-agent connections
├── resource-usage.json        # Resource consumption data
├── approval-history.json      # User approval decisions
├── system-config.json         # System configuration
├── agents/[agent_id]/
│   ├── conversations/          # Chat history (JSON files)
│   │   ├── 2024-01-15.json
│   │   └── 2024-01-16.json
│   ├── state.json             # Current agent state
│   ├── tasks.json             # Task history and metrics
│   └── memory.json            # Long-term memory
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
```typescript
class MCPServiceManager {
  private docker: DockerApi;
  private serviceRegistry: Map<string, MCPService>;
  
  async discoverServices(query: string): Promise<MCPService[]>
  async installService(serviceName: string, agentId: string): Promise<void>
  async configureService(service: MCPService, agent: AgentConfig): Promise<void>
  async removeService(serviceName: string, agentId: string): Promise<void>
}
```

## Performance Considerations

### Scalability Targets
- **Concurrent Agents**: Up to 50 agents
- **Message Throughput**: 1000 messages/minute
- **Task Updates**: Real-time markdown file updates
- **Memory Usage**: <4GB total system memory
- **Startup Time**: <30 seconds for full system

### Optimization Strategies
- **Connection Pooling**: Reuse Claude CLI connections
- **Message Batching**: Batch low-priority messages
- **Lazy Loading**: Load agent configurations on demand
- **Caching**: Cache frequently accessed registry data
- **Compression**: Compress conversation logs

## Monitoring & Observability

### Health Checks
- Agent process health
- Resource consumption
- Message queue backlog
- Git repository status
- Docker container health

### Metrics Collection
```typescript
interface SystemMetrics {
  agentCount: number;
  activeAgents: number;
  messagesPerMinute: number;
  averageResponseTime: number;
  resourceUsage: ResourceUsage;
  errorRate: number;
  uptime: number;
}
```

### Logging Strategy
- **Structured Logging**: JSON format with correlation IDs
- **Log Levels**: DEBUG, INFO, WARN, ERROR, FATAL
- **Audit Trail**: All user actions and agent decisions
- **Retention**: 30 days default, configurable