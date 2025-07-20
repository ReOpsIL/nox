# AI Agent Selector System

A comprehensive system for managing predefined AI agents and their associated tasks, organized by software development domains.

## Overview

This system consists of:

1. **YAML Configuration** (`config/predefined_agents.yaml`) - Defines specialized AI agents and their capabilities
2. **TUI Selector** (`agent-selector` binary) - Interactive terminal interface for browsing and selecting agents
3. **Task Templates** - Predefined task templates with detailed implementation steps

## Agent Categories

### 🦀 Rust CLI Development
- **Rust CLI Expert**: Building command-line applications with argument parsing, file I/O, and error handling
- **Rust TUI Specialist**: Creating terminal user interfaces with ratatui and crossterm

### 🌐 Fullstack Web Development
- **React + MUI + Rust Expert**: Frontend with React/TypeScript/Material-UI and Rust backend
- **React + Python Expert**: Frontend with React/MUI and Python FastAPI backend

### 📚 Documentation Master
- **Documentation Master**: Technical documentation, README files, PRD, SRS, API docs

### 🔧 Code Maintenance
- **Code Review Specialist**: Quality analysis, security audits, performance optimization
- **Test Engineering Specialist**: Unit testing, integration testing, E2E testing

### 📋 Project Planning
- **Project Planning Specialist**: Task breakdown, timeline planning, requirement analysis

## Usage

### Running the Agent Selector TUI

```bash
# Build and run the agent selector
cargo run --bin agent-selector
```

### TUI Navigation

| Key | Action |
|-----|--------|
| `↑↓` | Navigate through lists |
| `Enter` | Select item |
| `Esc` | Go back / Close help |
| `h` | Toggle help screen |
| `s` | Save output (when in output generation screen) |
| `q` | Quit application |

### Workflow

1. **Select Category**: Choose from development domains (Rust CLI, Fullstack Web, etc.)
2. **Browse Agents**: View available agents in the selected category
3. **Agent Details**: Review agent capabilities and default tasks
4. **Select Tasks**: Choose specific tasks for the agent
5. **Generate Config**: Create YAML configuration for selected agents
6. **Save Output**: Export configuration to timestamped markdown file

### Example Agents and Tasks

#### Rust CLI Expert
- Set up basic CLI structure with clap
- Implement file I/O operations
- Add comprehensive error handling
- Create logging and configuration systems

#### React + MUI + Rust Expert  
- Initialize React + TypeScript + MUI project
- Set up Rust backend with actix-web/axum
- Implement authentication system
- Create API endpoints and frontend integration

#### Documentation Master
- Generate comprehensive README files
- Create Product Requirements Documents (PRD)
- Write API documentation with examples
- Build architecture documentation

#### Test Engineering Specialist
- Implement comprehensive unit test suites
- Set up integration testing frameworks
- Create E2E test automation
- Configure CI/CD test integration

## Configuration Structure

### Agent Definition

```yaml
agents:
  - id: "agent_unique_id"
    name: "Human Readable Name"
    category: "category_id"
    description: "Agent specialization description"
    system_prompt: |
      Detailed prompt defining agent behavior and expertise
    capabilities:
      - "Specific capability 1"
      - "Specific capability 2"
    default_tasks:
      - template: "task_template_id"
        title: "Task Title"
        description: "Task description"
        priority: "high|medium|low"
        estimated_duration: "30m"
```

### Task Template Definition

```yaml
task_templates:
  template_id:
    title: "Template Title"
    description: "Template description"
    category: "category_id"
    tags: ["tag1", "tag2"]
    default_priority: "high"
    estimated_duration: "45m"
    steps:
      - "Step 1 description"
      - "Step 2 description"
```

## Output Generation

The TUI generates two types of output:

### 1. Markdown Summary
- Agent details and selected tasks
- Priority and duration information
- Implementation guidance

### 2. YAML Configuration
```yaml
selected_agents:
  - id: "rust_cli_expert"
    name: "Rust CLI Expert"
    category: "rust_cli"
    tasks:
      - template: "cli_basic_setup"
        title: "Set up basic CLI structure"
        priority: "high"
```

## Extending the System

### Adding New Agents

1. Define agent in `predefined_agents.yaml`
2. Specify capabilities and default tasks
3. Create associated task templates
4. Test with the TUI selector

### Adding New Categories

1. Add category definition to `categories` section
2. Create agents for the new category
3. Update filters if needed

### Custom Task Templates

1. Define template in `task_templates` section
2. Specify implementation steps
3. Associate with relevant agents

## Integration with Nox

The agent selector integrates with the broader Nox agent ecosystem:

- Selected agents can be instantiated as Nox agents
- Task configurations can be imported into task management
- Generated YAML can be used for automated agent creation

## Build and Development

```bash
# Build all binaries
cargo build

# Build just the agent selector
cargo build --bin agent-selector

# Run with development features
cargo run --bin agent-selector

# Add new dependencies
# Edit Cargo.toml and add required crates
```

## Dependencies

- `serde_yaml` - YAML configuration parsing
- `ratatui` - Terminal UI framework
- `crossterm` - Cross-platform terminal control
- `chrono` - Date/time handling for timestamps
- `anyhow` - Error handling

## Architecture

```
config/predefined_agents.yaml
├── Categories (rust_cli, fullstack_web, documentation, etc.)
├── Agents (specialized AI agents with capabilities)
├── Task Templates (reusable task definitions)
└── Filters (for browsing and selection)

src/bin/agent_selector_tui.rs
├── YAML Configuration Loading
├── TUI State Management
├── Agent/Task Browsing
├── Selection and Filtering
└── Output Generation
```

This system provides a comprehensive framework for managing specialized AI agents and their tasks, making it easy to select and configure agents for specific development needs.