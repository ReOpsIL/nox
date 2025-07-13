# Nox CLI Reference - Rust Backend

## Quick Reference

All commands use the format: `cargo run -- <command> [subcommand] [options] [arguments]`

## System Commands

### `init [options]`
Initialize Nox agent ecosystem
```bash
cargo run -- init
cargo run -- init --force  # Force reinitialize
```

### `start [options]`
Start the Nox agent ecosystem
```bash
cargo run -- start
cargo run -- start --dev   # Development mode
```

### `stop`
Stop the Nox agent ecosystem
```bash
cargo run -- stop
```

### `status`
Show ecosystem status
```bash
cargo run -- status
```

### `health`
Check system health and status
```bash
cargo run -- health
```

### `serve`
Start the API server for frontend integration
```bash
cargo run -- serve
cargo run -- serve --port 8080  # Custom port
```

## Agent Management

### `agent add <name> <system_prompt>`
Create a new agent with specified configuration
```bash
cargo run -- agent add "NewsBot" "You are a news analysis agent specializing in AI developments"
```

### `agent list` (alias: `agent ls`)
Display all registered agents and their status
```bash
cargo run -- agent list
cargo run -- agent ls
cargo run -- agent ls --status active  # Filter by status
```

### `agent show <name>`
Display detailed information about an agent
```bash
cargo run -- agent show NewsBot
```

### `agent update <name> <new_prompt>`
Modify an existing agent's system prompt
```bash
cargo run -- agent update NewsBot "Enhanced news analysis with focus on technology trends"
```

### `agent delete <name>`
Remove an agent from the registry
```bash
cargo run -- agent delete NewsBot
cargo run -- agent delete NewsBot --force  # Skip confirmation
```

### `agent start <name>`
Start a specific agent
```bash
cargo run -- agent start NewsBot
```

### `agent stop <name>`
Stop a specific agent
```bash
cargo run -- agent stop NewsBot
```

## Task Management

### `task create <agent_id> <title> <description>`
Create a new task for an agent
```bash
cargo run -- task create agent-1752272015775 "Research Task" "Analyze quantum computing papers from ArXiv"
```

### `task list [agent_id]`
List tasks for a specific agent or all agents
```bash
cargo run -- task list                    # All tasks
cargo run -- task list agent-1752272015775  # Specific agent
cargo run -- task list --status pending   # Filter by status
```

### `task update <task_id>`
Update task status or details
```bash
cargo run -- task update task-123
cargo run -- task update task-123 --status completed
```

### `task overview`
Show task overview across all agents
```bash
cargo run -- task overview
```

### `task cancel <task_id>`
Cancel a task
```bash
cargo run -- task cancel task-123
```

## API Endpoints for Task Control

### Manual Task Execution
```bash
# Start a specific task
curl -X POST http://localhost:3000/api/tasks/{task_id}/start

# Execute a task on its assigned agent
curl -X POST http://localhost:3000/api/tasks/{task_id}/execute
```

### Agent Deletion
```bash
# Delete an agent via API
curl -X DELETE http://localhost:3000/api/agents/{agent_id}
```

## Registry Management

### `registry status`
Show registry information and statistics
```bash
cargo run -- registry status
```

### `registry history [options]`
Show registry change history
```bash
cargo run -- registry history
cargo run -- registry history --limit 10
```

### `registry backup [message]`
Create a manual registry backup
```bash
cargo run -- registry backup "Pre-experiment backup"
```

### `registry rollback <target>`
Rollback registry to a previous state
```bash
cargo run -- registry rollback HEAD~1
cargo run -- registry rollback abc123f
cargo run -- registry rollback --confirm  # Skip confirmation
```

### `registry query <capability>`
Find agents with specific capabilities
```bash
cargo run -- registry query "web-scraping"
cargo run -- registry query "data-analysis"
```

## Communication

### `message broadcast <message>`
Send a message to all agents
```bash
cargo run -- message broadcast "Important announcement for all agents"
```

### `message send <agent_id> <message>`
Send a message to a specific agent
```bash
cargo run -- message send NewsBot "Please analyze today's tech news"
```

## Configuration & Logs

### `config [options]`
Show current configuration
```bash
cargo run -- config
cargo run -- config --path  # Show config file path
```

### `logs [options]`
Show system logs
```bash
cargo run -- logs
cargo run -- logs --tail 50    # Last 50 lines
cargo run -- logs --follow     # Follow logs
```

### `version`
Show version information
```bash
cargo run -- version
```

## Global Options

- `-V, --version` - Output the version number
- `-v, --verbose` - Enable verbose logging
- `--dev` - Enable development mode
- `--config <path>` - Path to configuration file
- `-h, --help` - Display help for command

## Examples

### Basic Workflow
```bash
# Initialize the system
cargo run -- init

# Start the ecosystem
cargo run -- start

# Start the API server for frontend
cargo run -- serve

# Create agents
cargo run -- agent add "NewsHunter" "Specialized in AI news analysis"
cargo run -- agent add "CodeScout" "Discovers new coding tools and frameworks"

# List agents
cargo run -- agent list

# Create tasks
cargo run -- task create agent-123 "Research Task" "Analyze latest AI developments"

# Check task status
cargo run -- task list

# Monitor system
cargo run -- status
cargo run -- health
```

### Dashboard Access
Once started, the web dashboard and API are available at:
- **PrimeReact Frontend**: http://localhost:3001
- **API Server**: http://localhost:3000/api
- **WebSocket**: ws://localhost:3000/ws

### Common Errors

**"System not initialized"**
```bash
cargo run -- init
```

**"Port already in use"**
```bash
# Kill conflicting processes
lsof -ti:3000 | xargs kill -9
lsof -ti:3001 | xargs kill -9
```

**"Agent not found"**
```bash
# List agents to see correct IDs
cargo run -- agent list
```

**"Compilation error"**
```bash
# Clean and rebuild
cargo clean
cargo build
```

**"Database error"**
```bash
# Check registry path
cargo run -- config --path
# Verify registry files exist
ls -la .nox-registry/
```