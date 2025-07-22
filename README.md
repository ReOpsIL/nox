# Nox - AI Agent Management System

A Rust-based agent management platform for coordinating AI agents powered by Claude CLI.

![Nox TUI](nox1.jpg)

## Overview

Nox provides infrastructure for creating, managing, and executing tasks through AI agents. It features both a command-line interface and a terminal user interface (TUI) for agent and task management.

## Quick Start

### Prerequisites

- Rust 1.70+ with Cargo
- Claude CLI (installed and configured)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/user/nox.git
cd nox

# Build the project
cargo build

# Initialize the system
cargo run -- init

# Start TUI interface
cargo run --bin nox-tui
```

## Core Features

### Agent Management
- Create and manage AI agents with custom prompts
- Agent lifecycle management (start/stop/status)
- Resource monitoring and limits
- Predefined agent templates

### Task Execution
- Task creation and assignment to agents
- Claude CLI integration for task execution
- Task status tracking and progress monitoring
- Markdown-based task logging

### User Interface
- **CLI**: Full command-line interface
- **TUI**: Terminal user interface with real-time updates
- **API**: RESTful API for web integration (partial)

## CLI Commands

### System Commands
```bash
cargo run -- init                    # Initialize Nox registry
cargo run -- start [--dev]           # Start the ecosystem
cargo run -- stop                    # Stop the ecosystem
cargo run -- status                  # Show system status
cargo run -- health                  # Health check
cargo run -- serve [--port PORT]     # Start API server
```

### Agent Management
```bash
cargo run -- agent add <name> <prompt>     # Create agent
cargo run -- agent list                    # List all agents
cargo run -- agent show <name>             # Show agent details
cargo run -- agent update <name> <prompt>  # Update agent prompt
cargo run -- agent delete <name>           # Delete agent
cargo run -- agent start <name>            # Start agent
cargo run -- agent stop <name>             # Stop agent
```

### Task Management
```bash
cargo run -- task create <agent_id> <title> <description>  # Create task
cargo run -- task list [agent_id]                          # List tasks
cargo run -- task show <task_id>                           # Show task details
cargo run -- task execute <task_id>                        # Execute task
cargo run -- task update <task_id> --status <status>       # Update task
cargo run -- task cancel <task_id>                         # Cancel task
cargo run -- task overview                                 # Task overview
```

## TUI Interface

Launch the TUI with:
```bash
cargo run --bin nox-tui
```

### Navigation
- **Tab**: Switch between screens (Agents, Tasks, Logs, etc.)
- **Enter**: Interact with selected items
- **Esc**: Go back or cancel
- **q**: Quit application
- **h**: Show help

### Screens
- **Dashboard**: System overview and quick actions
- **Agents**: Agent management and status
- **Tasks**: Task creation and monitoring
- **Execution**: Task execution with real-time output
- **Logs**: System and task logs
- **Predefined Agents**: Agent templates

## Architecture

```
CLI/TUI Interface
       ↓
Core Management Layer
├── Agent Manager
├── Task Manager
├── Registry Manager
└── Claude Process Manager
       ↓
Claude CLI Integration
       ↓
File Storage (TOML/JSON/Markdown)
```

## Configuration

Configuration is stored in `config/default.toml`:

```toml
[server]
port = 3000
host = "localhost"

[storage]
registry_path = ".nox-registry"

[claude_cli]
session_timeout = 3600
auto_restart_on_crash = true

[logging]
level = "info"
```

## Data Storage

- **Agents**: `.nox-registry/agents.toml` (TOML configuration)
- **Tasks**: `.nox-registry/tasks/<agent_id>.json` (JSON data)
- **Logs**: `logs/task_<task_id>.md` (Markdown format)

## Development

### Build Options
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run specific binary
cargo run --bin nox-tui
cargo run --bin agent-selector
```

### Available Binaries
- `nox` - Main CLI interface
- `nox-tui` - Terminal user interface
- `agent-selector` - Agent selection utility
- Various test binaries for development

## Implementation Status

- ✅ **Core Agent Management**: Fully implemented
- ✅ **Task Management**: Complete with Claude CLI integration
- ✅ **TUI Interface**: Full terminal interface with all screens
- ✅ **CLI Commands**: All documented commands working
- ⚠️ **API Server**: Basic structure, some endpoints incomplete
- ⚠️ **WebSocket**: Has stability issues, not recommended for production

## Dependencies

**Core Dependencies:**
- `tokio` - Async runtime
- `clap` - CLI parsing
- `serde` - Serialization
- `actix-web` - Web server
- `ratatui` - TUI framework
- `crossterm` - Terminal manipulation

**Claude Integration:**
- Direct command execution via `claude` CLI
- Automatic response parsing and storage
- Error handling and timeout management

## License

See LICENSE file for details.