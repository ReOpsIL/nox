# Nox Project Structure

## Directory Layout

```
nox/
├── README.md                      # Project overview and setup
├── IMPLEMENTATION_PLAN.md         # Development roadmap
├── TECHNICAL_ARCHITECTURE.md      # System architecture
├── API_SPECIFICATION.md           # API documentation
├── TESTING_STRATEGY.md           # Testing approach
├── PROJECT_STRUCTURE.md          # This file
├── .gitignore                    # Git ignore rules
├── package.json                  # Node.js dependencies
├── tsconfig.json                 # TypeScript configuration
├── jest.config.js               # Jest testing configuration
├── eslint.config.js             # ESLint configuration
├── docker-compose.yml           # Docker services setup
├── Dockerfile                   # Container definition
│
├── src/                         # Source code
│   ├── nox.ts                   # Main CLI entry point
│   ├── types/                   # TypeScript type definitions
│   │   ├── agent.ts
│   │   ├── task.ts
│   │   ├── message.ts
│   │   └── config.ts
│   │
│   ├── core/                    # Core system components
│   │   ├── agent-manager.ts     # Agent lifecycle management
│   │   ├── message-broker.ts    # Inter-agent communication
│   │   ├── registry-manager.ts  # Agent registry operations
│   │   ├── task-manager.ts      # Task tracking and delegation
│   │   ├── git-manager.ts       # Git versioning operations
│   │   ├── config-manager.ts    # Configuration handling
│   │   └── system.ts           # Main system orchestrator
│   │
│   ├── commands/               # CLI command implementations
│   │   ├── agent.ts            # Agent management commands
│   │   ├── task.ts             # Task management commands
│   │   ├── registry.ts         # Registry commands
│   │   ├── mcp.ts              # MCP service commands
│   │   └── system.ts           # System commands
│   │
│   ├── mcp/                    # MCP service integration
│   │   ├── service-manager.ts  # MCP service lifecycle
│   │   ├── docker-manager.ts   # Docker container management
│   │   ├── capability-registry.ts # Service capability tracking
│   │   └── discovery.ts        # Service discovery logic
│   │
│   ├── security/               # Security and safety components
│   │   ├── resource-limiter.ts # Resource consumption limits
│   │   ├── approval-manager.ts # User approval workflows
│   │   ├── safety-manager.ts   # Anti-runaway protection
│   │   ├── audit-logger.ts     # Security audit logging
│   │   └── sandbox.ts          # Process isolation
│   │
│   ├── server/                 # Web server and API
│   │   ├── app.ts              # Express app setup
│   │   ├── websocket.ts        # WebSocket server
│   │   ├── routes/             # API route handlers
│   │   │   ├── agents.ts
│   │   │   ├── tasks.ts
│   │   │   ├── registry.ts
│   │   │   └── mcp.ts
│   │   └── middleware/         # Express middleware
│   │       ├── auth.ts
│   │       ├── validation.ts
│   │       └── error-handler.ts
│   │
│   ├── interfaces/             # External system interfaces
│   │   ├── claude-interface.ts # Claude CLI wrapper
│   │   ├── docker-interface.ts # Docker API wrapper
│   │   ├── git-interface.ts    # Git operations wrapper
│   │   └── file-interface.ts   # File system operations
│   │
│   ├── utils/                  # Utility functions
│   │   ├── logger.ts           # Logging utilities
│   │   ├── validation.ts       # Input validation
│   │   ├── crypto.ts           # Cryptographic functions
│   │   ├── file-utils.ts       # File system utilities
│   │   ├── process-utils.ts    # Process management utilities
│   │   └── time-utils.ts       # Time and date utilities
│   │
│   ├── protocols/              # Communication protocols
│   │   ├── agent-protocols.ts  # Agent-to-agent protocols
│   │   ├── task-protocols.ts   # Task delegation protocols
│   │   └── system-protocols.ts # System event protocols
│   │
│   └── monitoring/             # System monitoring
│       ├── metrics.ts          # Performance metrics
│       ├── health-check.ts     # System health monitoring
│       └── alerts.ts           # Alert management
│
├── tests/                      # Test files
│   ├── unit/                   # Unit tests
│   │   ├── core/
│   │   ├── commands/
│   │   ├── mcp/
│   │   ├── security/
│   │   └── utils/
│   │
│   ├── integration/            # Integration tests
│   │   ├── agent-lifecycle.test.ts
│   │   ├── task-delegation.test.ts
│   │   ├── mcp-integration.test.ts
│   │   └── registry-operations.test.ts
│   │
│   ├── e2e/                   # End-to-end tests
│   │   ├── multi-agent-scenarios.test.ts
│   │   ├── real-world-workflows.test.ts
│   │   └── performance.test.ts
│   │
│   ├── fixtures/              # Test data and fixtures
│   │   ├── agent-configs/
│   │   ├── mock-responses/
│   │   └── test-registries/
│   │
│   └── helpers/               # Test utility functions
│       ├── test-environment.ts
│       ├── mock-factory.ts
│       └── test-utils.ts
│
├── config/                    # Configuration files
│   ├── default.json          # Default configuration
│   ├── development.json      # Development environment
│   ├── production.json       # Production environment
│   └── test.json            # Test environment
│
├── scripts/                  # Build and deployment scripts
│   ├── build.sh             # Build script
│   ├── deploy.sh            # Deployment script
│   ├── setup-dev.sh         # Development setup
│   └── migrate-registry.js  # Registry migration utility
│
├── docs/                    # Additional documentation
│   ├── CONTRIBUTING.md      # Contribution guidelines
│   ├── DEPLOYMENT.md        # Deployment instructions
│   ├── TROUBLESHOOTING.md   # Common issues and solutions
│   └── examples/           # Usage examples
│       ├── basic-usage.md
│       ├── advanced-scenarios.md
│       └── mcp-integration.md
│
├── frontend/               # Web dashboard (optional)
│   ├── public/
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   └── utils/
│   ├── package.json
│   └── vite.config.ts
│
└── .nox-registry/         # Runtime registry (gitignored)
    ├── agents.json
    ├── mcp-services.json
    ├── agent-relationships.json
    ├── resource-usage.json
    ├── agents/
    ├── tasks/
    ├── conversations/
    └── .git/
```

## Key File Purposes

### Core Entry Points

#### `src/nox.ts`
Main CLI application entry point. Handles command parsing and routing.

```typescript
// Main CLI entry point
import { Command } from 'commander';
import { AgentCommands } from './commands/agent';
import { TaskCommands } from './commands/task';
import { SystemCommands } from './commands/system';

const program = new Command();
program
  .name('nox')
  .description('Autonomous AI Agent Ecosystem')
  .version('1.0.0');

// Register command groups
AgentCommands.register(program);
TaskCommands.register(program);
SystemCommands.register(program);

program.parse();
```

#### `src/core/system.ts`
Main system orchestrator that coordinates all components.

```typescript
export class NoxSystem {
  private agentManager: AgentManager;
  private messageBroker: MessageBroker;
  private registryManager: RegistryManager;
  private taskManager: TaskManager;
  private mcpManager: MCPServiceManager;
  
  async initialize(): Promise<void>
  async shutdown(): Promise<void>
  async executeCommand(command: string, args: any[]): Promise<any>
}
```

### Core Components

#### `src/core/agent-manager.ts`
Manages Claude CLI processes and agent lifecycle.

#### `src/core/message-broker.ts`
Handles inter-agent communication and message routing.

#### `src/core/registry-manager.ts`
Manages agent configurations and registry operations.

#### `src/core/task-manager.ts`
Handles task creation, delegation, and markdown file updates.

### Type Definitions

#### `src/types/agent.ts`
```typescript
export interface AgentConfig {
  id: string;
  name: string;
  systemPrompt: string;
  status: AgentStatus;
  createdAt: Date;
  resourceLimits: ResourceLimits;
  capabilities: string[];
}

export type AgentStatus = 'active' | 'inactive' | 'error' | 'crashed';
```

#### `src/types/task.ts`
```typescript
export interface Task {
  id: string;
  agentId: string;
  title: string;
  description: string;
  status: TaskStatus;
  priority: TaskPriority;
  createdAt: Date;
  deadline?: Date;
}

export type TaskStatus = 'todo' | 'inprogress' | 'done' | 'blocked';
export type TaskPriority = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
```

## Configuration Files

### `package.json`
```json
{
  "name": "nox",
  "version": "1.0.0",
  "description": "Autonomous AI Agent Ecosystem",
  "main": "dist/nox.js",
  "scripts": {
    "build": "tsc",
    "start": "node dist/nox.js",
    "dev": "ts-node src/nox.ts",
    "test": "jest",
    "lint": "eslint src/**/*.ts"
  },
  "dependencies": {
    "commander": "^11.0.0",
    "ws": "^8.14.0",
    "dockerode": "^4.0.0",
    "simple-git": "^3.19.0",
    "chokidar": "^3.5.0",
    "express": "^4.18.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "@types/jest": "^29.5.0",
    "typescript": "^5.2.0",
    "jest": "^29.7.0",
    "eslint": "^8.50.0"
  }
}
```

### `tsconfig.json`
```json
{
  "compilerOptions": {
    "target": "ES2022",
    "module": "commonjs",
    "lib": ["ES2022"],
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "tests"]
}
```

### `jest.config.js`
```javascript
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/src', '<rootDir>/tests'],
  testMatch: ['**/__tests__/**/*.ts', '**/?(*.)+(spec|test).ts'],
  transform: {
    '^.+\\.ts$': 'ts-jest',
  },
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.d.ts',
    '!src/**/*.test.ts'
  ],
  coverageThreshold: {
    global: {
      branches: 80,
      functions: 80,
      lines: 80,
      statements: 80
    }
  }
};
```

## Development Workflow

### Initial Setup
```bash
# Clone repository
git clone <repo-url>
cd nox

# Install dependencies
npm install

# Build TypeScript
npm run build

# Initialize development environment
npm run init

# Start development server
npm run dev
```

### Adding New Features

1. **Create Types**: Define interfaces in `src/types/`
2. **Implement Core Logic**: Add functionality in `src/core/`
3. **Add Commands**: Create CLI commands in `src/commands/`
4. **Write Tests**: Add tests in `tests/`
5. **Update Documentation**: Modify relevant `.md` files

### File Naming Conventions

- **TypeScript Files**: kebab-case (e.g., `agent-manager.ts`)
- **Test Files**: Same name + `.test.ts` (e.g., `agent-manager.test.ts`)
- **Type Files**: Singular nouns (e.g., `agent.ts`, `task.ts`)
- **Configuration Files**: lowercase with extensions (e.g., `tsconfig.json`)

### Import Organization

```typescript
// 1. Node.js built-in modules
import { spawn } from 'child_process';
import * as fs from 'fs/promises';

// 2. Third-party libraries
import { Command } from 'commander';
import * as git from 'simple-git';

// 3. Internal types
import { AgentConfig, Task } from '../types';

// 4. Internal modules
import { RegistryManager } from './registry-manager';
import { MessageBroker } from './message-broker';
```

## Build and Deployment

### Build Process
```bash
# Clean previous build
rm -rf dist/

# Compile TypeScript
npm run build

# Run tests
npm test

# Create production bundle
npm run package
```

### Environment Variables
```bash
# Development
NODE_ENV=development
NOX_REGISTRY_PATH=./.nox-registry
NOX_LOG_LEVEL=debug
NOX_PORT=3000

# Production
NODE_ENV=production
NOX_REGISTRY_PATH=/var/lib/nox
NOX_LOG_LEVEL=info
NOX_PORT=80
```

This structure provides a clear separation of concerns, making the codebase maintainable and scalable while following Node.js/TypeScript best practices.