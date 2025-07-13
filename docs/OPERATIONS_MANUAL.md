# Nox Agent Ecosystem - Operations Manual

## ⚠️ Implementation Status Notice

**Current System Status (Updated July 2025):**

**✅ FULLY OPERATIONAL:**
- Core CLI commands (agent/task management) 
- Claude CLI integration and task execution
- Basic Git integration and registry management

**⚠️ PARTIALLY IMPLEMENTED:**  
- Web API endpoints (structure exists, many incomplete)
- Git advanced features (rollback, branching)

**❌ KNOWN ISSUES:**
- WebSocket integration has stability issues (see WEBSOCKET_IMPROVEMENTS.md)

**🔮 FUTURE FEATURES:**
- Advanced agent features (self-modification, spawning)
- MCP service discovery  
- Resource management enforcement
- Security framework integration

## Table of Contents
1. [Overview](#overview)
2. [Prerequisites](#prerequisites)
3. [Installation and Setup](#installation-and-setup)
4. [Starting the System](#starting-the-system)
5. [Basic Operations](#basic-operations)
6. [Agent Management](#agent-management)
7. [Task Management](#task-management)
8. [API Usage](#api-usage)
9. [Monitoring and Status](#monitoring-and-status)
10. [Stopping the System](#stopping-the-system)
11. [Troubleshooting](#troubleshooting)
12. [Advanced Features](#advanced-features)

## Overview

Nox is an autonomous agent ecosystem that manages AI agents powered by Claude CLI. The system provides:
- Agent lifecycle management (create, start, stop, delete)
- Task assignment and tracking
- Resource monitoring and allocation
- Git-based versioning and change tracking
- RESTful API for integration
- WebSocket real-time communication

## Prerequisites

Before using Nox, ensure you have:
- **Rust** (1.70 or later) - [Install from rustup.rs](https://rustup.rs/)
- **Claude CLI** - [Install from Anthropic](https://docs.anthropic.com/claude/reference/claude-cli)
- **Git** - For version control and change tracking
- **Docker** (optional) - For MCP service integration

### Claude CLI Setup
```bash
# Install Claude CLI (follow Anthropic's official instructions)
# Ensure it's configured with your API key
claude auth login

# Verify installation
claude --version
```

## Installation and Setup

### 1. Build the Project
```bash
# Clone the repository (if not already done)
git clone <repository-url>
cd nox

# Build the project
cargo build --release

# Run tests to verify installation
cargo test -- --test-threads=1
```

### 2. Initialize the System
```bash
# Initialize the Nox ecosystem and registry
./target/release/nox init

# This creates:
# - .nox-registry/ directory for data persistence
# - Git repository for change tracking
# - Default configuration files
```

## Starting the System

### Method 1: Full System Start
```bash
# Start the complete Nox ecosystem
./target/release/nox start

# Start in development mode (with enhanced logging)
./target/release/nox start --dev
```

### Method 2: API Server Only
```bash
# Start just the API server for external integrations
./target/release/nox serve

# Start on custom port
./target/release/nox serve --port 8080
```

### Verification
```bash
# Check system status
./target/release/nox status

# Check system health
./target/release/nox health
```

## Basic Operations

### System Status Commands
```bash
# Show overall system status
nox status

# Check health of all components
nox health

# Stop the system gracefully
nox stop
```

### Logging
- Logs are output to stdout/stderr
- Set `RUST_LOG=debug` for detailed logging
- Development mode (`--dev`) enables enhanced logging

## Agent Management

### Creating Agents
```bash
# Create a new agent
nox agent add "Research Assistant" "You are a helpful research assistant specializing in scientific literature analysis."

# The system will:
# - Generate a unique agent ID
# - Store agent configuration in .nox-registry/
# - Commit changes to git
```

### Managing Agents
```bash
# List all agents
nox agent list

# Show detailed agent information
nox agent show "Research Assistant"

# Update agent system prompt
nox agent update "Research Assistant" "Updated system prompt here"

# Start an inactive agent
nox agent start "Research Assistant"

# Stop an active agent
nox agent stop "Research Assistant"

# Delete an agent (with confirmation)
nox agent delete "Research Assistant"

# Force delete without confirmation
nox agent delete "Research Assistant" --force
```

### Agent Lifecycle
1. **Inactive** - Created but not running
2. **Starting** - Claude process being spawned
3. **Active** - Running and ready for tasks
4. **Stopping** - Gracefully shutting down
5. **Error** - Failed to start or crashed

## Task Management

### Creating Tasks
```bash
# Create a task for an agent
nox task create AGENT_ID "Analyze Research Paper" "Analyze the attached research paper and summarize key findings"

# The system assigns a unique task ID and sets status to "todo"
```

### Managing Tasks
```bash
# List all tasks
nox task list

# List tasks for specific agent
nox task list AGENT_ID

# Update task status
nox task update TASK_ID --status "in_progress"
nox task update TASK_ID --status "done"
nox task update TASK_ID --status "cancelled"

# Get task overview across all agents
nox task overview

# Cancel a task
nox task cancel TASK_ID
```

### Task Statuses
- **todo** - Created, waiting to be started
- **in_progress** - Currently being worked on
- **done** - Completed successfully
- **cancelled** - Cancelled by user

## API Usage

When the API server is running (via `nox serve` or `nox start`), you can interact with the system via HTTP:

### Agent Endpoints
```bash
# Get all agents
curl http://localhost:8000/agents

# Create new agent
curl -X POST http://localhost:8000/agents \
  -H "Content-Type: application/json" \
  -d '{"name": "Test Agent", "system_prompt": "You are a test agent"}'

# Get specific agent
curl http://localhost:8000/agents/{agent_id}

# Update agent
curl -X PUT http://localhost:8000/agents/{agent_id} \
  -H "Content-Type: application/json" \
  -d '{"system_prompt": "Updated prompt"}'

# Delete agent
curl -X DELETE http://localhost:8000/agents/{agent_id}

# Start agent
curl -X POST http://localhost:8000/agents/{agent_id}/start

# Stop agent
curl -X POST http://localhost:8000/agents/{agent_id}/stop
```

### Task Endpoints
```bash
# Get all tasks
curl http://localhost:8000/tasks

# Create new task
curl -X POST http://localhost:8000/tasks \
  -H "Content-Type: application/json" \
  -d '{"agent_id": "AGENT_ID", "title": "Task Title", "description": "Task description"}'

# Get specific task
curl http://localhost:8000/tasks/{task_id}

# Update task
curl -X PUT http://localhost:8000/tasks/{task_id} \
  -H "Content-Type: application/json" \
  -d '{"status": "in_progress"}'

# Delete task
curl -X DELETE http://localhost:8000/tasks/{task_id}

# Start task
curl -X POST http://localhost:8000/tasks/{task_id}/start

# Complete task
curl -X POST http://localhost:8000/tasks/{task_id}/complete
```

### WebSocket Connection
```javascript
// Connect to WebSocket for real-time updates
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onmessage = function(event) {
    const message = JSON.parse(event.data);
    console.log('Received:', message);
    // Handle system events, agent status changes, task updates
};
```

## Monitoring and Status

### System Health Checks
```bash
# Overall system health
nox health
# Returns: System components status, resource usage, error counts

# Detailed status
nox status
# Returns: Agent states, task statistics, resource allocation
```

### Resource Monitoring
The system automatically monitors:
- **CPU Usage** - Per agent and system-wide
- **Memory Usage** - Agent memory consumption
- **Process Health** - Claude CLI process status
- **Task Progress** - Active and completed tasks

### Git Integration
```bash
# View change history
nox git history

# View recent changes (limit 20)
nox git history --limit 20

# Rollback to previous state
nox git rollback COMMIT_HASH --confirm

# Branch management
nox git branch list
nox git branch create feature-branch
nox git branch switch main
nox git branch delete feature-branch --force
```

## Stopping the System

### Graceful Shutdown
```bash
# Stop all agents and services gracefully
nox stop

# This will:
# - Stop all active agents
# - Terminate Claude processes
# - Save current state
# - Commit any pending changes
```

### Emergency Stop
If the system becomes unresponsive:
```bash
# Kill all Claude processes
pkill -f "claude"

# Remove any lock files
rm -f .nox-registry/.lock
```

## Troubleshooting

### Common Issues

#### "Claude CLI not found"
```bash
# Verify Claude CLI installation
which claude
claude --version

# If not found, install Claude CLI and ensure it's in PATH
```

#### "Permission denied" on startup
```bash
# Check file permissions
ls -la .nox-registry/
chmod -R 755 .nox-registry/

# Ensure Git repository is properly initialized
cd .nox-registry && git status
```

#### Agent fails to start
```bash
# Check agent status
nox agent show "Agent Name"

# Check system health
nox health

# View logs (if running with RUST_LOG=debug)
RUST_LOG=debug nox agent start "Agent Name"
```

#### API server connection refused
```bash
# Check if server is running
netstat -an | grep 8000
# or
lsof -i :8000

# Restart API server
nox serve --port 8001
```

### Debug Mode
```bash
# Enable debug logging
export RUST_LOG=debug
nox start --dev

# This provides detailed logging for troubleshooting
```

### Log Locations
- **Console Output** - All logs go to stdout/stderr
- **Git History** - Change tracking in `.nox-registry/.git`
- **Registry Files** - Agent/task data in `.nox-registry/`

## Advanced Features

### Resource Management
The system includes automatic resource monitoring:
- **Memory Limits** - Configurable per agent (default: 512MB)
- **CPU Limits** - Configurable per agent (default: 50%)
- **Runaway Protection** - Automatic detection and stopping of misbehaving agents

### Security Framework
- **Permission Levels** - Guest, User, Admin, System, Root
- **Approval Workflows** - For sensitive operations
- **Audit Logging** - All security events tracked

### MCP Service Integration
The system supports Modular Capability Providers (MCP):
- **Docker-based services** - External tool integration
- **Service discovery** - Automatic capability detection
- **Agent assignment** - Dynamic service allocation

### Message Broker
Inter-agent communication via:
- **Direct messaging** - Agent-to-agent communication
- **Topic subscriptions** - Broadcast messaging
- **Priority queues** - Message prioritization

### Self-Modification
Advanced agents can:
- **Request prompt evolution** - Improve their own prompts
- **Spawn child agents** - Create specialized sub-agents
- **Self-modify behavior** - Update their own configuration

## Configuration Files

### Main Configuration
- **Location**: `.nox-registry/config.toml`
- **Contains**: System settings, resource limits, security policies

### Agent Registry
- **Location**: `.nox-registry/agents.toml`
- **Contains**: All agent definitions and metadata

### Task Storage
- **Location**: `.nox-registry/tasks/*.json`
- **Contains**: Individual task files with full task data

## Support and Maintenance

### Regular Maintenance
```bash
# Clean up old task files (older than 30 days)
find .nox-registry/tasks -name "*.json" -mtime +30 -delete

# Compact git repository
cd .nox-registry && git gc --aggressive

# Check system integrity
nox health
```

### Backup and Recovery
```bash
# Backup the entire registry
tar -czf nox-backup-$(date +%Y%m%d).tar.gz .nox-registry/

# Restore from backup
tar -xzf nox-backup-YYYYMMDD.tar.gz
```

### Performance Tuning
- **Resource Limits** - Adjust per-agent memory/CPU limits
- **Process Pool Size** - Configure maximum concurrent agents
- **Task Queue Size** - Limit pending task backlog
- **WebSocket Connections** - Monitor and limit client connections

---

For more detailed information about specific features, refer to the README.md and inline code documentation.