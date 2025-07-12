# NOX Agent Ecosystem - User Manual

## 🚀 Overview

NOX is an autonomous AI agent ecosystem that enables you to create, manage, and coordinate multiple AI agents for various tasks. The system provides a complete platform with web dashboard, API endpoints, real-time communication, and integrated task management.

> **Current Implementation Status**: This manual describes the full vision of NOX. The current implementation includes core agent registry, dashboard interface, and basic CLI commands. Advanced features like MCP integration, task automation, and inter-agent communication are planned for future releases.

## 📋 Table of Contents

- [Quick Start](#quick-start)
- [System Architecture](#system-architecture)
- [Dashboard Interface](#dashboard-interface)
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

- Node.js 16+ installed
- NPM or Yarn package manager
- Git (for version control)
- Terminal/Command line access

### Installation & Setup

1. **Navigate to the NOX directory:**
```bash
cd /Users/dovcaspi/develop/nox
```

2. **Install dependencies:**
```bash
npm install
cd frontend && npm install && cd ..
```

3. **Initialize the system:**
```bash
npx ts-node src/nox.ts init
```

4. **Start the system:**
```bash
./scripts/run-dev.sh
```

5. **Access the dashboard:**
- Open your browser to: http://localhost:3001
- API endpoints available at: http://localhost:3001/api/*
- WebSocket server running on: ws://localhost:3000

## System Architecture

### Core Components

- **Agent Manager**: Creates and manages AI agents
- **Task Manager**: Handles task distribution and execution
- **Message Broker**: Facilitates inter-agent communication
- **Dashboard Server**: Web interface and API endpoints
- **WebSocket Server**: Real-time updates and notifications
- **Registry Manager**: Persistent storage and state management

### Directory Structure

```
nox/
├── docs/               # Documentation
├── src/                # Core system code
│   ├── core/           # Core managers and systems
│   ├── commands/       # CLI command implementations
│   ├── dashboard/      # Web dashboard and API routes
│   ├── mcp/           # MCP service integration
│   ├── protocols/     # Agent communication protocols
│   └── types/         # TypeScript type definitions
├── frontend/          # React dashboard UI
├── tasks/             # Task definitions (markdown files)
├── conversations/     # Agent conversation logs
├── messages/          # Message broker storage
├── claude-sessions/   # Claude CLI sessions
├── tests/             # Test suites
├── scripts/           # Utility scripts
└── nox.config.json    # System configuration
```

## Dashboard Interface

### Accessing the Dashboard

Open http://localhost:3001 in your web browser to access the NOX dashboard.

### Dashboard Features

- **System Overview**: Real-time status, health metrics, and system statistics
- **Agent Management**: Visual interface for creating, monitoring, and controlling agents
- **Task Monitoring**: Track task progress, results, and execution history
- **System Health**: Performance metrics, resource usage, and diagnostics
- **Settings**: System configuration and user preferences
- **Real-time Updates**: Live notifications and status changes

### Navigation

The dashboard provides intuitive navigation through:
- **Sidebar Menu**: Quick access to all major sections
- **Header Status**: Connection status and system health indicators
- **Notification Center**: Real-time alerts and system messages

## Command Line Interface

### Basic Commands

```bash
# Get help
npx ts-node src/nox.ts --help

# System management
npx ts-node src/nox.ts init          # Initialize system
npx ts-node src/nox.ts start         # Start ecosystem
npx ts-node src/nox.ts start --dev   # Start in development mode
npx ts-node src/nox.ts status        # Check system status
npx ts-node src/nox.ts stop          # Stop system gracefully
npx ts-node src/nox.ts health        # Check system health
```

### Agent Commands

```bash
# List all agents
npx ts-node src/nox.ts list-agents
npx ts-node src/nox.ts ls  # Short alias

# Create a new agent
npx ts-node src/nox.ts add-agent "agent-name" "Agent system prompt and description"

# Show agent details
npx ts-node src/nox.ts show-agent "agent-name"

# Update agent prompt
npx ts-node src/nox.ts update-agent "agent-name" "New system prompt"

# Delete an agent
npx ts-node src/nox.ts delete-agent "agent-name"
npx ts-node src/nox.ts delete-agent "agent-name" --force  # Skip confirmation

# Query agents by capability
npx ts-node src/nox.ts query-registry "web-scraping"
```

### Task Commands

```bash
# List all tasks
npx ts-node src/nox.ts list-tasks

# List tasks for specific agent
npx ts-node src/nox.ts list-tasks {agentId}

# Create a task
npx ts-node src/nox.ts create-task {agentId} "Task Title" "Task description"

# Update task
npx ts-node src/nox.ts update-task {taskId}

# Task overview across all agents
npx ts-node src/nox.ts task-overview
```

### Registry Management Commands

```bash
# Show registry status and statistics
npx ts-node src/nox.ts registry-status

# Show registry change history
npx ts-node src/nox.ts registry-history
npx ts-node src/nox.ts registry-history --limit 10

# Create manual backup
npx ts-node src/nox.ts registry-backup "Pre-experiment backup"

# Rollback to previous state
npx ts-node src/nox.ts registry-rollback HEAD~1
npx ts-node src/nox.ts registry-rollback abc123f --confirm

# Show configuration
npx ts-node src/nox.ts config
npx ts-node src/nox.ts config --path

# Show logs
npx ts-node src/nox.ts logs
npx ts-node src/nox.ts logs --tail 50
```

## Agent Management

### Creating Agents

#### Via CLI
```bash
npx ts-node src/nox.ts add-agent "code-analyzer" "Analyzes code quality and suggests improvements. Specializes in code-review and static-analysis."
```

#### Via API
```bash
curl -X POST http://localhost:3001/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "code-analyzer",
    "description": "Analyzes code quality and suggests improvements",
    "capabilities": ["code-review", "static-analysis"]
  }'
```

#### Via Dashboard
1. Open http://localhost:3001 in your browser
2. Navigate to the Agents section
3. Click "Create New Agent"
4. Fill in the agent name, description, and capabilities
5. Click "Create Agent"

### Agent Lifecycle

1. **Creation**: Agent is defined with name, system prompt, and capabilities
2. **Registration**: Agent is stored in the Nox registry (.nox-registry/agents.json)
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
- Check TypeScript compliance
- Review security best practices
- Suggest performance optimizations
- Generate detailed report

## Input Data
- Source code in `src/` directory
- Configuration files
- Package dependencies

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
5. **Results**: Output is stored and made available via API/dashboard
6. **Completion**: Task status updated and notifications sent

## Real-time Features

### WebSocket Connection

Connect to the WebSocket server for real-time updates:

```javascript
const socket = new WebSocket('ws://localhost:3000');

socket.onopen = () => {
  console.log('Connected to NOX WebSocket');
};

socket.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('Real-time update:', data);
  
  // Handle different message types
  switch(data.type) {
    case 'agent_status_changed':
      updateAgentDisplay(data.data);
      break;
    case 'task_completed':
      showTaskResults(data.data);
      break;
    case 'system_health_update':
      updateHealthMetrics(data.data);
      break;
  }
};

socket.onclose = () => {
  console.log('WebSocket connection closed');
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

### Main Configuration File: `nox.config.json`

```json
{
  "security": {
    "maxAgents": 50,
    "spawnRateLimit": 10,
    "requireApprovalFor": ["agent_creation", "mcp_installation"],
    "resourceLimits": {
      "memoryPerAgent": "2GB",
      "claudeCallsPerMinute": 60,
      "maxConcurrentTasks": 10,
      "maxDockerContainers": 20,
      "diskSpaceLimit": "10GB"
    },
    "sandboxMode": true,
    "allowExternalCommunication": false
  },
  "claudeCli": {
    "sessionTimeout": 3600,
    "autoRestartOnCrash": true,
    "backupConversations": true,
    "cliPath": "claude",
    "defaultArgs": ["--interactive"],
    "healthCheckInterval": 30
  },
  "server": {
    "port": 3000,
    "dashboardPort": 3001,
    "host": "localhost",
    "websocketEnabled": true,
    "dashboardEnabled": true,
    "apiEnabled": true,
    "corsOrigins": ["http://localhost:3000"],
    "rateLimiting": {
      "windowMs": 900000,
      "maxRequests": 1000,
      "skipSuccessfulRequests": false
    }
  },
  "storage": {
    "format": "json",
    "compression": true,
    "backupRetentionDays": 30,
    "registryPath": ".nox-registry",
    "encryptSensitiveData": false
  },
  "mcp": {
    "enabled": true,
    "dockerHubRegistry": "https://hub.docker.com/v2/repositories/mcp/",
    "autoDiscovery": true,
    "serviceTimeout": 30,
    "maxServicesPerAgent": 5,
    "networkIsolation": true,
    "resourceLimits": {
      "memory": "1GB",
      "cpu": "0.5",
      "storage": "2GB",
      "networkBandwidth": "100MB"
    }
  },
  "logging": {
    "level": "info",
    "format": "json",
    "outputs": [
      { "type": "console", "config": {} },
      { "type": "file", "config": { "path": "logs/nox.log" } }
    ],
    "retention": {
      "days": 30,
      "maxSizeMB": 100,
      "compress": true
    }
  }
}
```

### Environment Variables

```bash
# Override configuration via environment variables
export NOX_CONFIG_PATH="/path/to/config.json"
export NOX_LOG_LEVEL="debug"
export NOX_SERVER_PORT="3001"
export NOX_WEBSOCKET_PORT="3000"
export NOX_MAX_AGENTS="100"
```

## System Management

### Monitoring System Health

#### Via Dashboard
Visit http://localhost:3001 and check the system health section for:
- CPU and memory usage
- Active agents count
- Task queue status
- Error rates and logs

#### Via CLI
```bash
# Check overall status
npx ts-node src/nox.ts status

# Get detailed health information
curl http://localhost:3001/api/system/health
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
curl http://localhost:3001/api/system/export > backup.json
```

#### Recovery
```bash
# Restore from git history
git checkout HEAD~1  # Go back one commit

# Restore from backup file
curl -X POST http://localhost:3001/api/system/import \
  -H "Content-Type: application/json" \
  -d @backup.json
```

### Performance Optimization

#### Resource Monitoring
```bash
# Check resource usage
curl http://localhost:3001/api/metrics/resources

# Monitor in real-time
watch -n 5 'curl -s http://localhost:3001/api/metrics/system'
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
npx ts-node src/nox.ts config
```

#### Agents Not Responding
```bash
# Check agent status
npx ts-node src/nox.ts list-agents
curl http://localhost:3001/api/agents

# Check specific agent details
npx ts-node src/nox.ts show-agent {agentName}

# Check Claude CLI connection
claude --version
```

#### Dashboard Not Loading
```bash
# Verify frontend build
ls -la frontend/build/

# Rebuild frontend if needed
cd frontend && npm run build && cd ..

# Check dashboard server logs
grep "DashboardServer" logs/nox.log
```

#### WebSocket Connection Issues
```bash
# Test WebSocket connection
wscat -c ws://localhost:3000

# Check WebSocket server status
curl http://localhost:3001/api/websocket-info
```

### Debug Mode

Enable debug logging for detailed troubleshooting:

```bash
# Start with debug logging
NOX_LOG_LEVEL=debug npx ts-node src/nox.ts start --dev

# Or modify config file
{
  "logging": {
    "level": "debug"
  }
}
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
npx ts-node src/nox.ts add-agent "data-scientist" "You are a data scientist specializing in data-analysis, visualization, and statistics. You excel at Python data science workflows."

# DevOps agent
npx ts-node src/nox.ts add-agent "devops-engineer" "You are a DevOps engineer specializing in deployment, monitoring, and automation. You have expertise in Docker, cloud platforms, and CI/CD."
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
// Webhook for Slack notifications
const webhook = 'https://hooks.slack.com/services/...';

fetch(webhook, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    text: `NOX Alert: ${message}`,
    channel: '#nox-alerts'
  })
});
```

#### GitHub Integration
```bash
# Automated code review agent
npx ts-node src/nox.ts add-agent "code-reviewer" "You are a code review specialist focusing on code-review, security-scan, and documentation. You provide thorough analysis of pull requests."

# Set up webhook for PR reviews (API endpoint would need to be implemented)
curl -X POST http://localhost:3001/api/webhooks/github \
  -H "Content-Type: application/json" \
  -d '{"event": "pull_request", "action": "opened"}'
```

### Performance Tuning

#### High-Throughput Configuration
```json
{
  "security": {
    "maxAgents": 100,
    "resourceLimits": {
      "maxConcurrentTasks": 50,
      "claudeCallsPerMinute": 120
    }
  },
  "server": {
    "rateLimiting": {
      "maxRequests": 5000
    }
  }
}
```

#### Low-Resource Configuration
```json
{
  "security": {
    "maxAgents": 10,
    "resourceLimits": {
      "memoryPerAgent": "512MB",
      "maxConcurrentTasks": 3
    }
  }
}
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
- **NOX Version**: 1.0.0
- **Node.js**: 16+ required
- **TypeScript**: 4.9+
- **React**: 18.2+

---

*Last updated: July 11, 2025*
*NOX Agent Ecosystem - Autonomous AI for Everyone* 🚀