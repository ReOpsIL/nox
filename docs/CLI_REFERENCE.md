# Nox CLI Reference

## Quick Reference

All commands use the format: `npx ts-node src/nox.ts <command> [options] [arguments]`

## System Commands

### `init [options]`
Initialize Nox agent ecosystem
```bash
npx ts-node src/nox.ts init
npx ts-node src/nox.ts init --force  # Force reinitialize
```

### `start [options]`
Start the Nox agent ecosystem
```bash
npx ts-node src/nox.ts start
npx ts-node src/nox.ts start --dev   # Development mode
```

### `stop`
Stop the Nox agent ecosystem
```bash
npx ts-node src/nox.ts stop
```

### `status`
Show ecosystem status
```bash
npx ts-node src/nox.ts status
```

### `health`
Check system health and status
```bash
npx ts-node src/nox.ts health
```

## Agent Management

### `add-agent <name> <systemPrompt>`
Create a new agent with specified configuration
```bash
npx ts-node src/nox.ts add-agent "NewsBot" "You are a news analysis agent specializing in AI developments"
```

### `list-agents` (alias: `ls`)
Display all registered agents and their status
```bash
npx ts-node src/nox.ts list-agents
npx ts-node src/nox.ts ls
npx ts-node src/nox.ts ls --status active  # Filter by status
```

### `show-agent <name>`
Display detailed information about an agent
```bash
npx ts-node src/nox.ts show-agent NewsBot
```

### `update-agent <name> <newPrompt>`
Modify an existing agent's system prompt
```bash
npx ts-node src/nox.ts update-agent NewsBot "Enhanced news analysis with focus on technology trends"
```

### `delete-agent <name>`
Remove an agent from the registry
```bash
npx ts-node src/nox.ts delete-agent NewsBot
npx ts-node src/nox.ts delete-agent NewsBot --force  # Skip confirmation
```

## Task Management

### `create-task <agentId> <title> <description>`
Create a new task for an agent
```bash
npx ts-node src/nox.ts create-task agent-1752272015775 "Research Task" "Analyze quantum computing papers from ArXiv"
```

### `list-tasks [agentId]`
List tasks for a specific agent or all agents
```bash
npx ts-node src/nox.ts list-tasks                    # All tasks
npx ts-node src/nox.ts list-tasks agent-1752272015775  # Specific agent
npx ts-node src/nox.ts list-tasks --status pending   # Filter by status
```

### `update-task <taskId>`
Update task status or details
```bash
npx ts-node src/nox.ts update-task task-123
npx ts-node src/nox.ts update-task task-123 --status completed
```

### `task-overview`
Show task overview across all agents
```bash
npx ts-node src/nox.ts task-overview
```

## Registry Management

### `registry-status`
Show registry information and statistics
```bash
npx ts-node src/nox.ts registry-status
```

### `registry-history [options]`
Show registry change history
```bash
npx ts-node src/nox.ts registry-history
npx ts-node src/nox.ts registry-history --limit 10
```

### `registry-backup [message]`
Create a manual registry backup
```bash
npx ts-node src/nox.ts registry-backup "Pre-experiment backup"
```

### `registry-rollback <target>`
Rollback registry to a previous state
```bash
npx ts-node src/nox.ts registry-rollback HEAD~1
npx ts-node src/nox.ts registry-rollback abc123f
npx ts-node src/nox.ts registry-rollback --confirm  # Skip confirmation
```

### `query-registry <capability>`
Find agents with specific capabilities
```bash
npx ts-node src/nox.ts query-registry "web-scraping"
npx ts-node src/nox.ts query-registry "data-analysis"
```

## Communication

### `comm`
Inter-agent communication commands
```bash
npx ts-node src/nox.ts comm --help  # See communication subcommands
```

## Configuration & Logs

### `config [options]`
Show current configuration
```bash
npx ts-node src/nox.ts config
npx ts-node src/nox.ts config --path  # Show config file path
```

### `logs [options]`
Show system logs
```bash
npx ts-node src/nox.ts logs
npx ts-node src/nox.ts logs --tail 50    # Last 50 lines
npx ts-node src/nox.ts logs --follow     # Follow logs
```

### `version`
Show version information
```bash
npx ts-node src/nox.ts version
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
npx ts-node src/nox.ts init

# Start the ecosystem
npx ts-node src/nox.ts start

# Create agents
npx ts-node src/nox.ts add-agent "NewsHunter" "Specialized in AI news analysis"
npx ts-node src/nox.ts add-agent "CodeScout" "Discovers new coding tools and frameworks"

# List agents
npx ts-node src/nox.ts list-agents

# Create tasks
npx ts-node src/nox.ts create-task agent-123 "Research Task" "Analyze latest AI developments"

# Check task status
npx ts-node src/nox.ts list-tasks

# Monitor system
npx ts-node src/nox.ts status
npx ts-node src/nox.ts health
```

### Dashboard Access
Once started, the web dashboard is available at:
- **Dashboard**: http://localhost:3001
- **WebSocket**: ws://localhost:3000

### Common Errors

**"System not initialized"**
```bash
npx ts-node src/nox.ts init
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
npx ts-node src/nox.ts list-agents
```