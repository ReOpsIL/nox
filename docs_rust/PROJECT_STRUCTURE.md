# Nox Project Structure - Rust Backend

## Directory Layout

```
nox/
├── README.md                      # Project overview and setup
├── Cargo.toml                     # Rust package manifest
├── Cargo.lock                     # Dependency lock file
├── .gitignore                     # Git ignore rules
├── rustfmt.toml                   # Rust formatting configuration
├── clippy.toml                    # Clippy linter configuration
├── docker-compose.yml             # Docker services setup
├── Dockerfile                     # Container definition
│
├── src/                           # Source code
│   ├── main.rs                    # Main CLI entry point
│   ├── lib.rs                     # Library entry point
│   │
│   ├── types/                     # Type definitions
│   │   ├── mod.rs                 # Module exports
│   │   ├── agent.rs               # Agent types
│   │   ├── task.rs                # Task types
│   │   ├── message.rs             # Message types
│   │   └── config.rs              # Configuration types
│   │
│   ├── core/                      # Core system components
│   │   ├── mod.rs                 # Module exports
│   │   ├── agent.rs               # Agent lifecycle management
│   │   ├── messaging.rs           # Inter-agent communication
│   │   ├── registry.rs            # Agent registry operations
│   │   ├── task.rs                # Task tracking and delegation
│   │   ├── git.rs                 # Git versioning operations
│   │   ├── config.rs              # Configuration handling
│   │   └── system.rs              # Main system orchestrator
│   │
│   ├── commands/                  # CLI command implementations
│   │   ├── mod.rs                 # Module exports
│   │   ├── agent.rs               # Agent management commands
│   │   ├── task.rs                # Task management commands
│   │   ├── registry.rs            # Registry commands
│   │   ├── mcp.rs                 # MCP service commands
│   │   └── system.rs              # System commands
│   │
│   ├── mcp/                       # MCP service integration
│   │   ├── mod.rs                 # Module exports
│   │   ├── service.rs             # MCP service lifecycle
│   │   ├── docker.rs              # Docker container management
│   │   ├── capability.rs          # Service capability tracking
│   │   └── discovery.rs           # Service discovery logic
│   │
│   ├── security/                  # Security and safety components
│   │   ├── mod.rs                 # Module exports
│   │   ├── resource.rs            # Resource consumption limits
│   │   ├── approval.rs            # User approval workflows
│   │   ├── safety.rs              # Anti-runaway protection
│   │   ├── audit.rs               # Security audit logging
│   │   └── sandbox.rs             # Process isolation
│   │
│   ├── api/                       # Web API server
│   │   ├── mod.rs                 # Module exports
│   │   ├── server.rs              # Actix Web server setup
│   │   ├── websocket.rs           # WebSocket handlers
│   │   ├── routes/                # API route handlers
│   │   │   ├── mod.rs             # Module exports
│   │   │   ├── agent.rs           # Agent endpoints
│   │   │   ├── task.rs            # Task endpoints
│   │   │   ├── registry.rs        # Registry endpoints
│   │   │   └── mcp.rs             # MCP endpoints
│   │   └── middleware/            # Actix middleware
│   │       ├── mod.rs             # Module exports
│   │       ├── auth.rs            # Authentication middleware
│   │       ├── validation.rs      # Request validation
│   │       └── error.rs           # Error handling
│   │
│   ├── interfaces/                # External system interfaces
│   │   ├── mod.rs                 # Module exports
│   │   ├── claude.rs              # Claude CLI wrapper
│   │   ├── docker.rs              # Docker API wrapper
│   │   ├── git.rs                 # Git operations wrapper
│   │   └── file.rs                # File system operations
│   │
│   ├── utils/                     # Utility functions
│   │   ├── mod.rs                 # Module exports
│   │   ├── logger.rs              # Logging utilities
│   │   ├── validation.rs          # Input validation
│   │   ├── crypto.rs              # Cryptographic functions
│   │   ├── file.rs                # File system utilities
│   │   ├── process.rs             # Process management utilities
│   │   └── time.rs                # Time and date utilities
│   │
│   ├── protocols/                 # Communication protocols
│   │   ├── mod.rs                 # Module exports
│   │   ├── agent.rs               # Agent-to-agent protocols
│   │   ├── task.rs                # Task delegation protocols
│   │   └── system.rs              # System event protocols
│   │
│   └── monitoring/                # System monitoring
│       ├── mod.rs                 # Module exports
│       ├── metrics.rs             # Performance metrics
│       ├── health.rs              # System health monitoring
│       └── alerts.rs              # Alert management
│
├── tests/                         # Integration tests
│   ├── agent_lifecycle.rs         # Agent lifecycle tests
│   ├── task_delegation.rs         # Task delegation tests
│   ├── mcp_integration.rs         # MCP integration tests
│   ├── registry_operations.rs     # Registry operation tests
│   ├── api_endpoints.rs           # API endpoint tests
│   └── common/                    # Shared test utilities
│       ├── mod.rs                 # Module exports
│       ├── fixtures.rs            # Test data and fixtures
│       └── helpers.rs             # Test helper functions
│
├── benches/                       # Performance benchmarks
│   ├── agent_operations.rs        # Agent operation benchmarks
│   ├── task_processing.rs         # Task processing benchmarks
│   └── api_performance.rs         # API performance benchmarks
│
├── examples/                      # Example code
│   ├── basic_usage.rs             # Basic usage examples
│   ├── agent_communication.rs     # Agent communication examples
│   └── api_client.rs              # API client examples
│
├── config/                        # Configuration files
│   ├── default.toml               # Default configuration
│   ├── development.toml           # Development environment
│   ├── production.toml            # Production environment
│   └── test.toml                  # Test environment
│
├── scripts/                       # Build and deployment scripts
│   ├── build.sh                   # Build script
│   ├── deploy.sh                  # Deployment script
│   ├── setup-dev.sh               # Development setup
│   └── migrate-registry.rs        # Registry migration utility
│
├── docs/                          # Documentation
│   ├── CONTRIBUTING.md            # Contribution guidelines
│   ├── DEPLOYMENT.md              # Deployment instructions
│   ├── TROUBLESHOOTING.md         # Common issues and solutions
│   └── examples/                  # Usage examples
│       ├── basic_usage.md
│       ├── advanced_scenarios.md
│       └── mcp_integration.md
│
├── frontend/                      # PrimeReact frontend
│   ├── public/
│   ├── src/
│   │   ├── components/            # UI components
│   │   ├── pages/                 # Application pages
│   │   ├── hooks/                 # Custom React hooks
│   │   ├── api/                   # API client
│   │   │   ├── client.js          # Base API client
│   │   │   ├── agent.js           # Agent API methods
│   │   │   ├── task.js            # Task API methods
│   │   │   └── websocket.js       # WebSocket client
│   │   └── utils/                 # Utility functions
│   ├── package.json
│   └── vite.config.js
│
└── .nox-registry/                 # Runtime registry (gitignored)
    ├── agents.toml                # Agent configurations
    ├── mcp-services.toml          # MCP service registry
    ├── agent-relationships.toml   # Agent relationship data
    ├── resource-usage.toml        # Resource usage tracking
    ├── agents/                    # Agent-specific data
    ├── tasks/                     # Task markdown files
    ├── conversations/             # Conversation history
    └── .git/                      # Git repository
```

## Key File Purposes

### Core Entry Points

#### `src/main.rs`
Main CLI application entry point. Handles command parsing and routing.

```rust
use clap::{App, AppSettings};
use nox::commands::{agent, task, system};

fn main() {
    let app = App::new("nox")
        .about("Autonomous AI Agent Ecosystem")
        .version("1.0.0")
        .setting(AppSettings::SubcommandRequiredElseHelp);
    
    // Register command groups
    let app = agent::register(app);
    let app = task::register(app);
    let app = system::register(app);
    
    let matches = app.get_matches();
    
    // Execute command based on matches
    // ...
}
```

#### `src/lib.rs`
Library entry point that exports all modules.

```rust
pub mod types;
pub mod core;
pub mod commands;
pub mod mcp;
pub mod security;
pub mod api;
pub mod interfaces;
pub mod utils;
pub mod protocols;
pub mod monitoring;

pub use core::system::NoxSystem;
```

#### `src/core/system.rs`
Main system orchestrator that coordinates all components.

```rust
pub struct NoxSystem {
    agent_manager: AgentManager,
    message_broker: MessageBroker,
    registry_manager: RegistryManager,
    task_manager: TaskManager,
    mcp_manager: MCPServiceManager,
}

impl NoxSystem {
    pub fn new() -> Result<Self, Error> {
        // Initialize components
    }
    
    pub async fn initialize(&mut self) -> Result<(), Error> {
        // Initialize system
    }
    
    pub async fn shutdown(&mut self) -> Result<(), Error> {
        // Shutdown system
    }
    
    pub async fn execute_command(&mut self, command: &str, args: &[&str]) -> Result<(), Error> {
        // Execute command
    }
}
```

### Core Components

#### `src/core/agent.rs`
Manages Claude CLI processes and agent lifecycle.

#### `src/core/messaging.rs`
Handles inter-agent communication and message routing.

#### `src/core/registry.rs`
Manages agent configurations and registry operations.

#### `src/core/task.rs`
Handles task creation, delegation, and markdown file updates.

### API Server

#### `src/api/server.rs`
Actix Web server setup for RESTful API endpoints.

```rust
use actix_web::{web, App, HttpServer};
use crate::api::routes::{agent, task, registry, mcp};
use crate::api::middleware::{auth, error};

pub async fn start_server(config: &Config) -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(error::error_handler())
            .wrap(auth::authentication())
            .service(
                web::scope("/api")
                    .service(agent::routes())
                    .service(task::routes())
                    .service(registry::routes())
                    .service(mcp::routes())
            )
            .service(web::resource("/ws").route(web::get().to(websocket::ws_handler)))
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .run()
    .await
}
```

#### `src/api/routes/agent.rs`
RESTful API endpoints for agent management.

```rust
use actix_web::{web, HttpResponse, Responder, Scope};
use crate::types::agent::{AgentConfig, AgentStatus};

pub fn routes() -> Scope {
    web::scope("/agents")
        .route("", web::get().to(list_agents))
        .route("", web::post().to(create_agent))
        .route("/{id}", web::get().to(get_agent))
        .route("/{id}", web::put().to(update_agent))
        .route("/{id}", web::delete().to(delete_agent))
        .route("/{id}/status", web::get().to(get_agent_status))
        .route("/{id}/tasks", web::get().to(get_agent_tasks))
}

async fn list_agents() -> impl Responder {
    // List all agents
    HttpResponse::Ok().json(/* agents list */)
}

async fn create_agent(agent: web::Json<AgentConfig>) -> impl Responder {
    // Create new agent
    HttpResponse::Created().json(/* created agent */)
}

// Other handler functions...
```

### Type Definitions

#### `src/types/agent.rs`
```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
    pub status: AgentStatus,
    pub created_at: DateTime<Utc>,
    pub resource_limits: ResourceLimits,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus {
    Active,
    Inactive,
    Error,
    Crashed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u32,
    pub max_cpu_percent: u8,
    pub max_runtime_seconds: Option<u32>,
}
```

#### `src/types/task.rs`
```rust
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub agent_id: String,
    pub title: String,
    pub description: String,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}
```

## Configuration Files

### `Cargo.toml`
```toml
[package]
name = "nox"
version = "0.1.0"
edition = "2021"
authors = ["Your Name <your.email@example.com>"]
description = "Autonomous AI Agent Ecosystem"

[dependencies]
tokio = { version = "1.28", features = ["full"] }
clap = { version = "4.3", features = ["derive"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.7"
actix-web = "4.3"
actix-rt = "2.8"
actix-cors = "0.6"
actix-ws = "0.2"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
async-trait = "0.1"
futures = "0.3"
git2 = "0.17"
reqwest = { version = "0.11", features = ["json"] }
bollard = "0.14"
uuid = { version = "1.4", features = ["v4", "serde"] }
config = "0.13"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.11"
criterion = "0.5"
actix-rt-test = "0.1"
wiremock = "0.5"

[[bench]]
name = "agent_operations"
harness = false

[[bench]]
name = "task_processing"
harness = false

[[bench]]
name = "api_performance"
harness = false
```

### `config/default.toml`
```toml
[server]
host = "127.0.0.1"
port = 3000
workers = 4

[registry]
path = "./.nox-registry"
git_enabled = true
auto_commit = true

[logging]
level = "info"
file = "nox.log"

[security]
enable_sandbox = true
resource_limits_enabled = true
require_approval = true

[claude]
timeout_seconds = 60
retry_attempts = 3
```

## Development Workflow

### Initial Setup
```bash
# Clone repository
git clone <repo-url>
cd nox

# Build project
cargo build

# Initialize development environment
cargo run -- init

# Run development server
cargo run -- serve
```

### Adding New Features

1. **Create Types**: Define structs and enums in `src/types/`
2. **Implement Core Logic**: Add functionality in `src/core/`
3. **Add Commands**: Create CLI commands in `src/commands/`
4. **Add API Routes**: Create API endpoints in `src/api/routes/`
5. **Write Tests**: Add tests in `tests/` and unit tests in source files
6. **Update Documentation**: Modify relevant `.md` files

### File Naming Conventions

- **Rust Files**: snake_case (e.g., `agent_manager.rs`)
- **Module Files**: `mod.rs` for module exports
- **Test Files**: Same name or in `tests/` directory
- **Type Files**: Singular nouns (e.g., `agent.rs`, `task.rs`)
- **Configuration Files**: lowercase with extensions (e.g., `config.toml`)

### Import Organization

```rust
// 1. Rust standard library
use std::fs;
use std::process::Command;

// 2. Third-party crates
use tokio::process::Command as TokioCommand;
use serde::{Serialize, Deserialize};
use actix_web::{web, App, HttpServer};

// 3. Internal crates/modules
use crate::types::{Agent, Task};
use crate::core::{Registry, MessageBroker};
```

## Build and Deployment

### Build Process
```bash
# Development build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Environment Variables
```bash
# Development
RUST_LOG=debug
NOX_CONFIG=config/development.toml
NOX_REGISTRY_PATH=./.nox-registry

# Production
RUST_LOG=info
NOX_CONFIG=config/production.toml
NOX_REGISTRY_PATH=/var/lib/nox
```

## Frontend Integration

The PrimeReact frontend communicates with the Rust backend through:

1. **RESTful API Endpoints**: For CRUD operations on agents, tasks, etc.
2. **WebSocket Connections**: For real-time updates and events
3. **Authentication**: JWT-based authentication for secure access

### API Client Example

```javascript
// frontend/src/api/client.js
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

```javascript
// frontend/src/api/agent.js
import apiClient from './client';

export const getAgents = async () => {
  const response = await apiClient.get('/agents');
  return response.data;
};

export const createAgent = async (agentData) => {
  const response = await apiClient.post('/agents', agentData);
  return response.data;
};

export const updateAgent = async (id, agentData) => {
  const response = await apiClient.put(`/agents/${id}`, agentData);
  return response.data;
};

export const deleteAgent = async (id) => {
  await apiClient.delete(`/agents/${id}`);
};
```

This structure provides a clear separation of concerns, making the codebase maintainable and scalable while following Rust best practices and providing a solid foundation for the PrimeReact frontend integration.