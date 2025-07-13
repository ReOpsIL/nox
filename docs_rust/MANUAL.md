# NOX Agent Ecosystem - User Manual (Rust Backend)

## 🚀 Overview

NOX is an autonomous AI agent ecosystem that enables you to create, manage, and coordinate multiple AI agents for various tasks. The system provides a complete platform with RESTful API endpoints, WebSocket communication, and integrated task management, with a PrimeReact frontend for visualization and control.

> **Current Implementation Status**: This manual describes the full vision of NOX with a Rust backend. The current implementation includes core agent registry, API routes, and basic CLI commands. Advanced features like MCP integration, task automation, and inter-agent communication are planned for future releases.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [System Architecture](#system-architecture)
- [PrimeReact Frontend](#primereact-frontend)
- [Command Line Interface](#command-line-interface)
- [Agent Management](#agent-management)
- [Task Management](#task-management)
- [Real-time Features](#real-time-features)
- [API Reference](#api-reference)
- [Configuration](#configuration)
- [System Management](#system-management)
- [Troubleshooting](#troubleshooting)
- [Advanced Usage](#advanced-usage)

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- Cargo package manager
- Git (for version control)
- Terminal/Command line access
- Node.js 16+ (for PrimeReact frontend)

### Installation & Setup

1. **Navigate to the NOX directory:**
```bash
cd /Users/dovcaspi/develop/nox
```

2. **Build the Rust backend:**
```bash
cargo build
```

3. **Install frontend dependencies:**
```bash
cd frontend && npm install && cd ..
```

4. **Initialize the system:**
```bash
cargo run -- init
```

5. **Start the system:**
```bash
./scripts/run-dev.sh
```

6. **Access the PrimeReact frontend:**
- Open your browser to: http://localhost:3001
- API endpoints available at: http://localhost:3000/api/*
- WebSocket server running on: ws://localhost:3000/ws

## System Architecture

### Core Components

- **Agent Manager**: Creates and manages AI agents
- **Task Manager**: Handles task distribution and execution
- **Message Broker**: Facilitates inter-agent communication
- **API Server**: RESTful endpoints and WebSocket server
- **Registry Manager**: Persistent storage and state management
- **PrimeReact Frontend**: User interface for system interaction

### Directory Structure

```
nox/
├── docs_rust/          # Rust-specific documentation
├── src/                # Rust source code
│   ├── main.rs         # CLI entry point
│   ├── lib.rs          # Library entry point
│   ├── core/           # Core managers and systems
│   ├── commands/       # CLI command implementations
│   ├── api/            # API routes and WebSocket handlers
│   ├── mcp/            # MCP service integration
│   ├── protocols/      # Agent communication protocols
│   └── types/          # Rust type definitions
├── frontend/           # PrimeReact frontend UI
├── tasks/              # Task definitions (markdown files)
├── conversations/      # Agent conversation logs
├── messages/           # Message broker storage
├── claude-sessions/    # Claude CLI sessions
├── tests/              # Integration tests
├── benches/            # Performance benchmarks
├── scripts/            # Utility scripts
└── config/             # System configuration files
```

## PrimeReact Frontend

### Accessing the Frontend

Open http://localhost:3001 in your web browser to access the NOX PrimeReact frontend.

### Frontend Features

- **System Overview**: Real-time status, health metrics, and system statistics
- **Agent Management**: Visual interface for creating, monitoring, and controlling agents
- **Task Monitoring**: Track task progress, results, and execution history
- **System Health**: Performance metrics, resource usage, and diagnostics
- **Settings**: System configuration and user preferences
- **Real-time Updates**: Live notifications and status changes via WebSocket

### Navigation

The PrimeReact frontend provides intuitive navigation through:
- **Sidebar Menu**: Quick access to all major sections
- **Header Status**: Connection status and system health indicators
- **Notification Center**: Real-time alerts and system messages
- **Responsive Design**: Mobile-friendly layout

## Command Line Interface

### Basic Commands

```bash
# Get help
cargo run -- --help

# System management
cargo run -- init          # Initialize system
cargo run -- start         # Start ecosystem
cargo run -- start --dev   # Start in development mode
cargo run -- status        # Check system status
cargo run -- stop          # Stop system gracefully
cargo run -- health        # Check system health
```

### Agent Commands

```bash
# List all agents
cargo run -- agent list
cargo run -- agent ls  # Short alias

# Create a new agent
cargo run -- agent add "agent-name" "Agent system prompt and description"

# Show agent details
cargo run -- agent show "agent-name"

# Update agent prompt
cargo run -- agent update "agent-name" "New system prompt"

# Delete an agent
cargo run -- agent delete "agent-name"
cargo run -- agent delete "agent-name" --force  # Skip confirmation

# Query agents by capability
cargo run -- registry query "web-scraping"
```

### Task Commands

```bash
# List all tasks
cargo run -- task list

# List tasks for specific agent
cargo run -- task list --agent {agentId}

# Create a task
cargo run -- task create {agentId} "Task Title" "Task description"

# Update task
cargo run -- task update {taskId}

# Task overview across all agents
cargo run -- task overview
```

### Registry Management Commands

```bash
# Show registry status and statistics
cargo run -- registry status

# Show registry change history
cargo run -- registry history
cargo run -- registry history --limit 10

# Create manual backup
cargo run -- registry backup "Pre-experiment backup"

# Rollback to previous state
cargo run -- registry rollback HEAD~1
cargo run -- registry rollback abc123f --confirm

# Show configuration
cargo run -- config
cargo run -- config --path

# Show logs
cargo run -- logs
cargo run -- logs --tail 50
```

## Agent Management

### Creating Agents

#### Via CLI
```bash
cargo run -- agent add "code-analyzer" "Analyzes code quality and suggests improvements. Specializes in code-review and static-analysis."
```

#### Via API
```bash
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code-analyzer",
    "description": "Analyzes code quality and suggests improvements",
    "capabilities": ["code-review", "static-analysis"]
  }'
```

#### Via PrimeReact Frontend
1. Open http://localhost:3001 in your browser
2. Navigate to the Agents section
3. Click "Create New Agent"
4. Fill in the agent name, description, and capabilities
5. Click "Create Agent"

### Agent Lifecycle

1. **Creation**: Agent is defined with name, system prompt, and capabilities
2. **Registration**: Agent is stored in the Nox registry (.nox-registry/agents.toml)
3. **Inactive**: Agent is registered but not currently running
4. **Starting**: Agent Claude CLI session is being initialized
5. **Active**: Agent is running and can receive tasks
6. **Error**: Agent encountered an error and needs attention
7. **Stopped**: Agent is gracefully shut down
8. **Deleted**: Agent is permanently removed from the registry

### Agent Configuration

Agents can be configured with:
- **Memory Limits**: Maximum memory usage per agent
- **Timeout Settings**: Task execution timeouts
- **Capabilities**: Specific skills or functions the agent can perform
- **Priority Levels**: Task processing priority
- **Retry Policies**: How to handle failed tasks

## Task Management

### Creating Tasks

Tasks are defined using markdown files in the `tasks/` directory.

#### Example Task File: `tasks/analyze-project.md`

```markdown
# Analyze Project Code Quality

## Objective
Review the codebase and provide suggestions for improvement.

## Requirements
- Check Rust compliance
- Review security best practices
- Suggest performance optimizations
- Generate detailed report

## Input Data
- Source code in `src/` directory
- Configuration files
- Cargo dependencies

## Expected Output
Markdown report with:
- Code quality score
- Security vulnerabilities found
- Performance bottlenecks
- Recommended improvements

## Priority
Medium

## Estimated Time
30 minutes

## Assigned Agent
code-analyzer
```

### Task Properties

- **Title**: Descriptive name for the task
- **Objective**: Clear goal and purpose
- **Requirements**: Specific deliverables and criteria
- **Input Data**: Required data sources and files
- **Expected Output**: Format and content of results
- **Priority**: Low, Medium, High, Critical
- **Estimated Time**: Expected duration
- **Assigned Agent**: Specific agent or auto-assignment

### Task Execution Flow

1. **Detection**: System monitors `tasks/` directory for new files
2. **Parsing**: Task markdown is parsed for metadata and requirements
3. **Assignment**: Task is assigned to appropriate agent
4. **Execution**: Agent processes the task using Claude CLI
5. **Results**: Output is stored and made available via API/frontend
6. **Completion**: Task status updated and notifications sent

## Real-time Features

### WebSocket Connection

Connect to the WebSocket server for real-time updates:

```javascript
// frontend/src/api/websocket.js
import { useEffect, useState } from 'react';

export const useWebSocket = () => {
  const [socket, setSocket] = useState(null);
  const [connected, setConnected] = useState(false);
  const [messages, setMessages] = useState([]);

  useEffect(() => {
    const ws = new WebSocket('ws://localhost:3000/ws');
    
    ws.onopen = () => {
      console.log('Connected to NOX WebSocket');
      setConnected(true);
    };
    
    ws.onmessage = (event) => {
      const data = JSON.parse(event.data);
      console.log('Real-time update:', data);
      setMessages(prev => [...prev, data]);
    };
    
    ws.onclose = () => {
      console.log('WebSocket connection closed');
      setConnected(false);
    };
    
    setSocket(ws);
    
    return () => {
      ws.close();
    };
  }, []);
  
  const sendMessage = (message) => {
    if (socket && connected) {
      socket.send(JSON.stringify(message));
    }
  };
  
  return { connected, messages, sendMessage };
};
```

### Notification Types

- **Agent Events**: Creation, status changes, task assignments
- **Task Events**: Creation, progress updates, completion, failures
- **System Events**: Health status, resource usage, errors
- **User Events**: Configuration changes, manual interventions

## API Reference

### System Endpoints

```bash
# System status and health
GET /api/system/status      # Overall system status
GET /api/system/health      # Health check and metrics
GET /api/system/info        # System information
POST /api/system/shutdown   # Graceful shutdown
```

### Agent Endpoints

```bash
# Agent management
GET /api/agents                    # List all agents
POST /api/agents                   # Create new agent
GET /api/agents/{id}              # Get agent details
PUT /api/agents/{id}              # Update agent
DELETE /api/agents/{id}           # Delete agent
POST /api/agents/{id}/start       # Start agent
POST /api/agents/{id}/stop        # Stop agent
GET /api/agents/{id}/status       # Get agent status
GET /api/agents/{id}/tasks        # Get agent's tasks
```

### Task Endpoints

```bash
# Task management
GET /api/tasks                     # List all tasks
GET /api/tasks/{id}               # Get task details
GET /api/tasks/{id}/status        # Get task status
GET /api/tasks/{id}/results       # Get task results
POST /api/tasks/{id}/cancel       # Cancel task
GET /api/tasks/{id}/logs          # Get task execution logs
```

### Metrics Endpoints

```bash
# Performance metrics
GET /api/metrics/system           # System performance metrics
GET /api/metrics/agents           # Agent performance metrics
GET /api/metrics/tasks            # Task execution metrics
GET /api/metrics/resources        # Resource usage metrics
```

## Configuration

### Main Configuration File: `config/default.toml`

```toml
[security]
max_agents = 50
spawn_rate_limit = 10
require_approval_for = ["agent_creation", "mcp_installation"]

[security.resource_limits]
memory_per_agent = "2GB"
claude_calls_per_minute = 60
max_concurrent_tasks = 10
max_docker_containers = 20
disk_space_limit = "10GB"
sandbox_mode = true
allow_external_communication = false

[claude_cli]
session_timeout = 3600
auto_restart_on_crash = true
backup_conversations = true
cli_path = "claude"
default_args = ["--interactive"]
health_check_interval = 30

[server]
port = 3000
frontend_port = 3001
host = "localhost"
websocket_enabled = true
frontend_enabled = true
api_enabled = true
cors_origins = ["http://localhost:3001"]

[server.rate_limiting]
window_ms = 900000
max_requests = 1000
skip_successful_requests = false

[storage]
format = "toml"
compression = true
backup_retention_days = 30
registry_path = ".nox-registry"
encrypt_sensitive_data = false

[mcp]
enabled = true
docker_hub_registry = "https://hub.docker.com/v2/repositories/mcp/"
auto_discovery = true
service_timeout = 30
max_services_per_agent = 5
network_isolation = true

[mcp.resource_limits]
memory = "1GB"
cpu = "0.5"
storage = "2GB"
network_bandwidth = "100MB"

[logging]
level = "info"
format = "json"

[[logging.outputs]]
type = "console"

[[logging.outputs]]
type = "file"
path = "logs/nox.log"

[logging.retention]
days = 30
max_size_mb = 100
compress = true
```

### Environment Variables

```bash
# Override configuration via environment variables
export NOX_CONFIG_PATH="/path/to/config.toml"
export RUST_LOG="debug"
export NOX_SERVER_PORT="3000"
export NOX_FRONTEND_PORT="3001"
export NOX_MAX_AGENTS="100"
```

## System Management

### Monitoring System Health

#### Via PrimeReact Frontend
Visit http://localhost:3001 and check the system health section for:
- CPU and memory usage
- Active agents count
- Task queue status
- Error rates and logs

#### Via CLI
```bash
# Check overall status
cargo run -- status

# Get detailed health information
curl http://localhost:3000/api/system/health
```

#### Via Logs
```bash
# View real-time logs
tail -f logs/nox.log

# Search for errors
grep "ERROR" logs/nox.log

# Monitor specific component
grep "AgentManager" logs/nox.log
```

### Backup and Recovery

#### Automatic Backups
The system automatically:
- Commits changes to git repository
- Backs up agent configurations
- Preserves conversation history
- Maintains task execution logs

#### Manual Backup
```bash
# Create manual backup
git add . && git commit -m "Manual backup $(date)"

# Export system state
curl http://localhost:3000/api/system/export > backup.json
```

#### Recovery
```bash
# Restore from git history
git checkout HEAD~1  # Go back one commit

# Restore from backup file
curl -X POST http://localhost:3000/api/system/import \
  -H "Content-Type: application/json" \
  -d @backup.json
```

### Performance Optimization

#### Resource Monitoring
```bash
# Check resource usage
curl http://localhost:3000/api/metrics/resources

# Monitor in real-time
watch -n 5 'curl -s http://localhost:3000/api/metrics/system'
```

#### Optimization Tips
- Limit concurrent agents based on available memory
- Adjust task timeout values for better resource utilization
- Enable compression for large data transfers
- Use task prioritization for better throughput
- Monitor Claude CLI call rates to avoid limits

## Troubleshooting

### Common Issues

#### System Won't Start
```bash
# Check if ports are already in use
lsof -i :3000
lsof -i :3001

# Check system logs for errors
tail -50 logs/nox.log

# Verify configuration
cargo run -- config
```

#### Agents Not Responding
```bash
# Check agent status
cargo run -- agent list
curl http://localhost:3000/api/agents

# Check specific agent details
cargo run -- agent show {agentName}

# Check Claude CLI connection
claude --version
```

#### Frontend Not Loading
```bash
# Verify frontend build
ls -la frontend/build/

# Rebuild frontend if needed
cd frontend && npm run build && cd ..

# Check API server logs
grep "ApiServer" logs/nox.log
```

#### WebSocket Connection Issues
```bash
# Test WebSocket connection
wscat -c ws://localhost:3000/ws

# Check WebSocket server status
curl http://localhost:3000/api/websocket-info
```

### Debug Mode

Enable debug logging for detailed troubleshooting:

```bash
# Start with debug logging
RUST_LOG=debug cargo run -- start --dev

# Or modify config file
[logging]
level = "debug"
```

### Error Codes

- **ENOENT**: File or directory not found
- **EADDRINUSE**: Port already in use
- **ECONNREFUSED**: Connection refused (service not running)
- **TIMEOUT**: Operation timed out
- **UNAUTHORIZED**: Authentication required
- **RATELIMIT**: Rate limit exceeded

## Advanced Usage

### Custom Agent Types

Create specialized agents for specific domains:

```bash
# Data analysis agent
cargo run -- agent add "data-scientist" "You are a data scientist specializing in data-analysis, visualization, and statistics. You excel at Python data science workflows."

# DevOps agent
cargo run -- agent add "devops-engineer" "You are a DevOps engineer specializing in deployment, monitoring, and automation. You have expertise in Docker, cloud platforms, and CI/CD."
```

### Task Automation

Set up automated task execution:

```bash
# Create recurring task
cat > tasks/daily-health-check.md << EOF
# Daily System Health Check

## Schedule
Every day at 09:00

## Objective
Perform automated system health verification

## Tasks
- Check all agent status
- Verify resource usage
- Test API endpoints
- Generate health report

## Alert Conditions
- Any agent failures
- Resource usage > 80%
- API response time > 2s
EOF
```

### Integration Examples

#### Slack Integration
```javascript
// frontend/src/integrations/slack.js
import axios from 'axios';

export const sendSlackNotification = async (message, channel = '#nox-alerts') => {
  const webhook = 'https://hooks.slack.com/services/...';
  
  try {
    await axios.post(webhook, {
      text: `NOX Alert: ${message}`,
      channel
    });
    return true;
  } catch (error) {
    console.error('Failed to send Slack notification:', error);
    return false;
  }
};
```

#### GitHub Integration
```bash
# Automated code review agent
cargo run -- agent add "code-reviewer" "You are a code review specialist focusing on code-review, security-scan, and documentation. You provide thorough analysis of pull requests."

# Set up webhook for PR reviews (API endpoint would need to be implemented)
curl -X POST http://localhost:3000/api/webhooks/github \
  -H "Content-Type: application/json" \
  -d '{"event": "pull_request", "action": "opened"}'
```

### Performance Tuning

#### High-Throughput Configuration
```toml
[security]
max_agents = 100

[security.resource_limits]
max_concurrent_tasks = 50
claude_calls_per_minute = 120

[server.rate_limiting]
max_requests = 5000
```

#### Low-Resource Configuration
```toml
[security]
max_agents = 10

[security.resource_limits]
memory_per_agent = "512MB"
max_concurrent_tasks = 3
```

---

## 📞 Support and Community

### Getting Help
- **Documentation**: Check this manual and API documentation
- **Logs**: Review system logs for detailed error information
- **Health Checks**: Use built-in health monitoring endpoints
- **GitHub Issues**: Report bugs and feature requests

### Best Practices
- Always test configuration changes in development mode
- Monitor system resources regularly
- Keep backups of important agent configurations
- Use task prioritization for critical operations
- Implement proper error handling in custom integrations

### Version Information
- **NOX Version**: 0.1.0
- **Rust**: 1.70+ required
- **Actix Web**: 4.3+
- **PrimeReact**: 9.0+

---

*Last updated: July 11, 2025*
*NOX Agent Ecosystem - Autonomous AI for Everyone* 🚀