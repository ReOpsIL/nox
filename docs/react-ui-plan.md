# React MUI Frontend Implementation Plan

## Overview

This document outlines the implementation plan for a React-based web interface using Material-UI (MUI) for the Nox Autonomous Agent Ecosystem, with comprehensive Playwright testing.

## 1. Architecture Overview

### 1.1 System Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                    React Frontend (Port 3000)                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Dashboard     │  │   Agent Mgmt    │  │   Task Mgmt     │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   Execution     │  │   Logs          │  │   Settings      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                    API Layer (Axios + React Query)              │
├─────────────────────────────────────────────────────────────────┤
│                    WebSocket (Socket.IO Client)                 │
└─────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                  Rust Backend (Port 8080)                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   REST API      │  │   WebSocket     │  │   Core Engine   │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Technology Stack

#### Frontend Stack
```json
{
  "runtime": "Node.js 18+",
  "framework": "React 18",
  "build": "Vite 4",
  "language": "TypeScript",
  "ui": "Material-UI (MUI) 5",
  "state": "Zustand + React Query",
  "routing": "React Router 6",
  "styling": "Emotion (MUI default)",
  "testing": "Playwright + Vitest",
  "realtime": "Socket.IO Client"
}
```

#### Development Tools
```json
{
  "linting": "ESLint + Prettier",
  "testing": "Playwright + Vitest + Testing Library",
  "bundling": "Vite",
  "deployment": "Docker + Nginx"
}
```

## 2. Project Structure

### 2.1 Directory Layout
```
frontend/
├── public/
│   ├── index.html
│   └── favicon.ico
├── src/
│   ├── components/           # Reusable UI components
│   │   ├── common/          # Common components
│   │   ├── agents/          # Agent-specific components
│   │   ├── tasks/           # Task-specific components
│   │   └── layout/          # Layout components
│   ├── pages/               # Page components
│   │   ├── Dashboard.tsx
│   │   ├── Agents.tsx
│   │   ├── Tasks.tsx
│   │   ├── Execution.tsx
│   │   └── Logs.tsx
│   ├── hooks/               # Custom React hooks
│   │   ├── useAgents.ts
│   │   ├── useTasks.ts
│   │   └── useWebSocket.ts
│   ├── services/            # API services
│   │   ├── api.ts
│   │   ├── agents.ts
│   │   ├── tasks.ts
│   │   └── websocket.ts
│   ├── stores/              # State management
│   │   ├── useAgentStore.ts
│   │   ├── useTaskStore.ts
│   │   └── useUIStore.ts
│   ├── types/               # TypeScript types
│   │   ├── agent.ts
│   │   ├── task.ts
│   │   └── api.ts
│   ├── utils/               # Utility functions
│   │   ├── formatting.ts
│   │   ├── validation.ts
│   │   └── constants.ts
│   ├── App.tsx              # Main app component
│   └── main.tsx             # Entry point
├── tests/                   # Playwright tests
│   ├── e2e/
│   │   ├── dashboard.spec.ts
│   │   ├── agents.spec.ts
│   │   ├── tasks.spec.ts
│   │   └── execution.spec.ts
│   └── fixtures/            # Test fixtures
├── package.json
├── vite.config.ts
├── playwright.config.ts
└── tsconfig.json
```

## 3. UI Design and Components

### 3.1 Main Layout
```typescript
// src/components/layout/MainLayout.tsx
import { Box, AppBar, Drawer, Toolbar, Typography } from '@mui/material';
import { Outlet } from 'react-router-dom';
import { Navigation } from './Navigation';
import { StatusBar } from './StatusBar';

export const MainLayout = () => {
  return (
    <Box sx={{ display: 'flex' }}>
      <AppBar position="fixed">
        <Toolbar>
          <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
            Nox Agent Ecosystem
          </Typography>
          <StatusBar />
        </Toolbar>
      </AppBar>
      
      <Drawer variant="permanent" sx={{ width: 240 }}>
        <Navigation />
      </Drawer>
      
      <Box component="main" sx={{ flexGrow: 1, p: 3, mt: 8 }}>
        <Outlet />
      </Box>
    </Box>
  );
};
```

### 3.2 Dashboard Design
```typescript
// src/pages/Dashboard.tsx
import {
  Grid,
  Card,
  CardContent,
  Typography,
  Box,
  LinearProgress,
  List,
  ListItem,
  ListItemText,
  Chip,
} from '@mui/material';
import { useAgents } from '../hooks/useAgents';
import { useTasks } from '../hooks/useTasks';
import { useSystemStatus } from '../hooks/useSystemStatus';

export const Dashboard = () => {
  const { agents } = useAgents();
  const { tasks } = useTasks();
  const { systemStatus } = useSystemStatus();

  const activeAgents = agents.filter(a => a.status === 'Active').length;
  const runningTasks = tasks.filter(t => t.status === 'InProgress').length;

  return (
    <Grid container spacing={3}>
      {/* System Status Cards */}
      <Grid item xs={12} md={4}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              System Status
            </Typography>
            <Box sx={{ mb: 2 }}>
              <Typography variant="body2">
                Status: <Chip label="Running" color="success" size="small" />
              </Typography>
              <Typography variant="body2">
                Uptime: {systemStatus.uptime}
              </Typography>
              <Typography variant="body2">CPU Usage</Typography>
              <LinearProgress 
                variant="determinate" 
                value={systemStatus.cpuUsage} 
                sx={{ mt: 1 }}
              />
              <Typography variant="body2">Memory Usage</Typography>
              <LinearProgress 
                variant="determinate" 
                value={systemStatus.memoryUsage} 
                sx={{ mt: 1 }}
              />
            </Box>
          </CardContent>
        </Card>
      </Grid>

      {/* Agent Summary */}
      <Grid item xs={12} md={4}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Agent Summary
            </Typography>
            <Typography variant="body2">
              Active: {activeAgents}/{agents.length}
            </Typography>
            <Typography variant="body2">
              Inactive: {agents.length - activeAgents}/{agents.length}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      {/* Task Summary */}
      <Grid item xs={12} md={4}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Task Summary
            </Typography>
            <Typography variant="body2">
              Running: {runningTasks}
            </Typography>
            <Typography variant="body2">
              Pending: {tasks.filter(t => t.status === 'Todo').length}
            </Typography>
            <Typography variant="body2">
              Completed: {tasks.filter(t => t.status === 'Done').length}
            </Typography>
          </CardContent>
        </Card>
      </Grid>

      {/* Recent Activity */}
      <Grid item xs={12}>
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Recent Activity
            </Typography>
            <List>
              {/* Activity items */}
            </List>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
};
```

### 3.3 Agent Management Interface
```typescript
// src/pages/Agents.tsx
import {
  Box,
  Button,
  Card,
  CardContent,
  Grid,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  IconButton,
  Dialog,
} from '@mui/material';
import { 
  Add as AddIcon, 
  Edit as EditIcon, 
  Delete as DeleteIcon,
  PlayArrow as StartIcon,
  Stop as StopIcon,
} from '@mui/icons-material';
import { useState } from 'react';
import { useAgents } from '../hooks/useAgents';
import { AgentForm } from '../components/agents/AgentForm';
import { AgentDetails } from '../components/agents/AgentDetails';

export const Agents = () => {
  const { agents, createAgent, updateAgent, deleteAgent, startAgent, stopAgent } = useAgents();
  const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingAgent, setEditingAgent] = useState<Agent | null>(null);

  const handleCreateAgent = () => {
    setEditingAgent(null);
    setShowForm(true);
  };

  const handleEditAgent = (agent: Agent) => {
    setEditingAgent(agent);
    setShowForm(true);
  };

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Active': return 'success';
      case 'Inactive': return 'default';
      case 'Error': return 'error';
      default: return 'default';
    }
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">Agent Management</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={handleCreateAgent}
        >
          Create Agent
        </Button>
      </Box>

      <Grid container spacing={3}>
        {/* Agent List */}
        <Grid item xs={12} md={8}>
          <Card>
            <CardContent>
              <TableContainer component={Paper}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Name</TableCell>
                      <TableCell>Status</TableCell>
                      <TableCell>Created</TableCell>
                      <TableCell>Last Active</TableCell>
                      <TableCell>Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {agents.map((agent) => (
                      <TableRow 
                        key={agent.id}
                        hover
                        onClick={() => setSelectedAgent(agent)}
                        sx={{ cursor: 'pointer' }}
                      >
                        <TableCell>{agent.name}</TableCell>
                        <TableCell>
                          <Chip 
                            label={agent.status} 
                            color={getStatusColor(agent.status)}
                            size="small"
                          />
                        </TableCell>
                        <TableCell>{new Date(agent.created_at).toLocaleString()}</TableCell>
                        <TableCell>
                          {agent.last_active ? new Date(agent.last_active).toLocaleString() : 'Never'}
                        </TableCell>
                        <TableCell>
                          <IconButton onClick={(e) => { e.stopPropagation(); handleEditAgent(agent); }}>
                            <EditIcon />
                          </IconButton>
                          <IconButton 
                            onClick={(e) => { 
                              e.stopPropagation(); 
                              agent.status === 'Active' ? stopAgent(agent.id) : startAgent(agent.id);
                            }}
                          >
                            {agent.status === 'Active' ? <StopIcon /> : <StartIcon />}
                          </IconButton>
                          <IconButton onClick={(e) => { e.stopPropagation(); deleteAgent(agent.id); }}>
                            <DeleteIcon />
                          </IconButton>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* Agent Details Panel */}
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              {selectedAgent ? (
                <AgentDetails agent={selectedAgent} />
              ) : (
                <Typography variant="body2" color="text.secondary">
                  Select an agent to view details
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Agent Form Dialog */}
      <Dialog open={showForm} onClose={() => setShowForm(false)} maxWidth="md" fullWidth>
        <AgentForm
          agent={editingAgent}
          onSave={(agent) => {
            if (editingAgent) {
              updateAgent(editingAgent.id, agent);
            } else {
              createAgent(agent);
            }
            setShowForm(false);
          }}
          onCancel={() => setShowForm(false)}
        />
      </Dialog>
    </Box>
  );
};
```

### 3.4 Task Management Interface
```typescript
// src/pages/Tasks.tsx
import {
  Box,
  Button,
  Card,
  CardContent,
  Grid,
  Typography,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  IconButton,
  Dialog,
  LinearProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
} from '@mui/material';
import { 
  Add as AddIcon, 
  Edit as EditIcon, 
  Delete as DeleteIcon,
  PlayArrow as ExecuteIcon,
  Cancel as CancelIcon,
} from '@mui/icons-material';
import { useState } from 'react';
import { useTasks } from '../hooks/useTasks';
import { TaskForm } from '../components/tasks/TaskForm';
import { TaskDetails } from '../components/tasks/TaskDetails';

export const Tasks = () => {
  const { tasks, createTask, updateTask, deleteTask, executeTask, cancelTask } = useTasks();
  const [selectedTask, setSelectedTask] = useState<Task | null>(null);
  const [showForm, setShowForm] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>('all');

  const filteredTasks = tasks.filter(task => 
    statusFilter === 'all' || task.status === statusFilter
  );

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'Todo': return 'info';
      case 'InProgress': return 'warning';
      case 'Done': return 'success';
      case 'Cancelled': return 'default';
      case 'Error': return 'error';
      default: return 'default';
    }
  };

  const getPriorityColor = (priority: string) => {
    switch (priority) {
      case 'High': return 'error';
      case 'Medium': return 'warning';
      case 'Low': return 'info';
      default: return 'default';
    }
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 3 }}>
        <Typography variant="h4">Task Management</Typography>
        <Button
          variant="contained"
          startIcon={<AddIcon />}
          onClick={() => setShowForm(true)}
        >
          Create Task
        </Button>
      </Box>

      <Grid container spacing={3}>
        {/* Task List */}
        <Grid item xs={12} md={8}>
          <Card>
            <CardContent>
              <Box sx={{ mb: 2 }}>
                <FormControl size="small" sx={{ minWidth: 120 }}>
                  <InputLabel>Status</InputLabel>
                  <Select
                    value={statusFilter}
                    label="Status"
                    onChange={(e) => setStatusFilter(e.target.value)}
                  >
                    <MenuItem value="all">All</MenuItem>
                    <MenuItem value="Todo">Todo</MenuItem>
                    <MenuItem value="InProgress">In Progress</MenuItem>
                    <MenuItem value="Done">Done</MenuItem>
                    <MenuItem value="Cancelled">Cancelled</MenuItem>
                  </Select>
                </FormControl>
              </Box>

              <TableContainer component={Paper}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Title</TableCell>
                      <TableCell>Agent</TableCell>
                      <TableCell>Status</TableCell>
                      <TableCell>Priority</TableCell>
                      <TableCell>Progress</TableCell>
                      <TableCell>Created</TableCell>
                      <TableCell>Actions</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {filteredTasks.map((task) => (
                      <TableRow 
                        key={task.id}
                        hover
                        onClick={() => setSelectedTask(task)}
                        sx={{ cursor: 'pointer' }}
                      >
                        <TableCell>{task.title}</TableCell>
                        <TableCell>{task.agent_id}</TableCell>
                        <TableCell>
                          <Chip 
                            label={task.status} 
                            color={getStatusColor(task.status)}
                            size="small"
                          />
                        </TableCell>
                        <TableCell>
                          <Chip 
                            label={task.priority} 
                            color={getPriorityColor(task.priority)}
                            size="small"
                          />
                        </TableCell>
                        <TableCell sx={{ width: 120 }}>
                          {task.progress && (
                            <LinearProgress 
                              variant="determinate" 
                              value={task.progress} 
                              sx={{ width: '100%' }}
                            />
                          )}
                        </TableCell>
                        <TableCell>{new Date(task.created_at).toLocaleString()}</TableCell>
                        <TableCell>
                          <IconButton onClick={(e) => { e.stopPropagation(); executeTask(task.id); }}>
                            <ExecuteIcon />
                          </IconButton>
                          <IconButton onClick={(e) => { e.stopPropagation(); cancelTask(task.id); }}>
                            <CancelIcon />
                          </IconButton>
                          <IconButton onClick={(e) => { e.stopPropagation(); deleteTask(task.id); }}>
                            <DeleteIcon />
                          </IconButton>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </CardContent>
          </Card>
        </Grid>

        {/* Task Details Panel */}
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              {selectedTask ? (
                <TaskDetails task={selectedTask} />
              ) : (
                <Typography variant="body2" color="text.secondary">
                  Select a task to view details
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Task Form Dialog */}
      <Dialog open={showForm} onClose={() => setShowForm(false)} maxWidth="md" fullWidth>
        <TaskForm
          task={editingTask}
          onSave={(task) => {
            if (editingTask) {
              updateTask(editingTask.id, task);
            } else {
              createTask(task);
            }
            setShowForm(false);
          }}
          onCancel={() => setShowForm(false)}
        />
      </Dialog>
    </Box>
  );
};
```

## 4. State Management and API Integration

### 4.1 API Service Layer
```typescript
// src/services/api.ts
import axios from 'axios';

const API_BASE_URL = 'http://localhost:8080/api';

export const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor for logging
api.interceptors.request.use(
  (config) => {
    console.log(`API Request: ${config.method?.toUpperCase()} ${config.url}`);
    return config;
  },
  (error) => Promise.reject(error)
);

// Response interceptor for error handling
api.interceptors.response.use(
  (response) => response,
  (error) => {
    console.error('API Error:', error.response?.data || error.message);
    return Promise.reject(error);
  }
);
```

### 4.2 Agent Service
```typescript
// src/services/agents.ts
import { api } from './api';
import { Agent, CreateAgentRequest, UpdateAgentRequest } from '../types/agent';

export const agentService = {
  // Get all agents
  async getAgents(): Promise<Agent[]> {
    const response = await api.get('/agents');
    return response.data.data;
  },

  // Get agent by ID
  async getAgent(id: string): Promise<Agent> {
    const response = await api.get(`/agents/${id}`);
    return response.data.data;
  },

  // Create new agent
  async createAgent(agent: CreateAgentRequest): Promise<Agent> {
    const response = await api.post('/agents', agent);
    return response.data.data;
  },

  // Update agent
  async updateAgent(id: string, agent: UpdateAgentRequest): Promise<Agent> {
    const response = await api.put(`/agents/${id}`, agent);
    return response.data.data;
  },

  // Delete agent
  async deleteAgent(id: string): Promise<void> {
    await api.delete(`/agents/${id}`);
  },

  // Start agent
  async startAgent(id: string): Promise<Agent> {
    const response = await api.post(`/agents/${id}/start`);
    return response.data.data;
  },

  // Stop agent
  async stopAgent(id: string): Promise<Agent> {
    const response = await api.post(`/agents/${id}/stop`);
    return response.data.data;
  },
};
```

### 4.3 React Query Hooks
```typescript
// src/hooks/useAgents.ts
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { agentService } from '../services/agents';
import { Agent, CreateAgentRequest, UpdateAgentRequest } from '../types/agent';

export const useAgents = () => {
  const queryClient = useQueryClient();

  const {
    data: agents = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ['agents'],
    queryFn: agentService.getAgents,
    refetchInterval: 5000, // Refetch every 5 seconds
  });

  const createAgentMutation = useMutation({
    mutationFn: agentService.createAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });

  const updateAgentMutation = useMutation({
    mutationFn: ({ id, agent }: { id: string; agent: UpdateAgentRequest }) =>
      agentService.updateAgent(id, agent),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });

  const deleteAgentMutation = useMutation({
    mutationFn: agentService.deleteAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });

  const startAgentMutation = useMutation({
    mutationFn: agentService.startAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });

  const stopAgentMutation = useMutation({
    mutationFn: agentService.stopAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    },
  });

  return {
    agents,
    isLoading,
    error,
    createAgent: createAgentMutation.mutate,
    updateAgent: (id: string, agent: UpdateAgentRequest) =>
      updateAgentMutation.mutate({ id, agent }),
    deleteAgent: deleteAgentMutation.mutate,
    startAgent: startAgentMutation.mutate,
    stopAgent: stopAgentMutation.mutate,
  };
};
```

### 4.4 WebSocket Integration
```typescript
// src/hooks/useWebSocket.ts
import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { io, Socket } from 'socket.io-client';

export const useWebSocket = () => {
  const queryClient = useQueryClient();
  const socketRef = useRef<Socket | null>(null);

  useEffect(() => {
    // Connect to WebSocket
    socketRef.current = io('ws://localhost:8080');

    const socket = socketRef.current;

    // Handle connection
    socket.on('connect', () => {
      console.log('WebSocket connected');
    });

    // Handle agent status updates
    socket.on('AgentStatus', (data) => {
      console.log('Agent status update:', data);
      queryClient.invalidateQueries({ queryKey: ['agents'] });
    });

    // Handle task updates
    socket.on('TaskUpdate', (data) => {
      console.log('Task update:', data);
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    });

    // Handle system events
    socket.on('SystemEvent', (data) => {
      console.log('System event:', data);
      queryClient.invalidateQueries({ queryKey: ['system-status'] });
    });

    // Handle disconnect
    socket.on('disconnect', () => {
      console.log('WebSocket disconnected');
    });

    // Cleanup on unmount
    return () => {
      socket.disconnect();
    };
  }, [queryClient]);

  return {
    socket: socketRef.current,
  };
};
```

## 5. Playwright Testing Strategy

### 5.1 Test Configuration
```typescript
// playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'firefox',
      use: { ...devices['Desktop Firefox'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
  ],

  webServer: [
    {
      command: 'npm run dev',
      port: 3000,
    },
    {
      command: 'cargo run -- serve --port 8080',
      port: 8080,
      cwd: '../',
    },
  ],
});
```

### 5.2 Page Object Model
```typescript
// tests/pages/AgentsPage.ts
import { Page, Locator, expect } from '@playwright/test';

export class AgentsPage {
  readonly page: Page;
  readonly createAgentButton: Locator;
  readonly agentTable: Locator;
  readonly agentNameInput: Locator;
  readonly agentPromptInput: Locator;
  readonly saveButton: Locator;
  readonly cancelButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.createAgentButton = page.getByRole('button', { name: 'Create Agent' });
    this.agentTable = page.getByRole('table');
    this.agentNameInput = page.getByLabel('Agent Name');
    this.agentPromptInput = page.getByLabel('System Prompt');
    this.saveButton = page.getByRole('button', { name: 'Save' });
    this.cancelButton = page.getByRole('button', { name: 'Cancel' });
  }

  async goto() {
    await this.page.goto('/agents');
  }

  async createAgent(name: string, prompt: string) {
    await this.createAgentButton.click();
    await this.agentNameInput.fill(name);
    await this.agentPromptInput.fill(prompt);
    await this.saveButton.click();
  }

  async startAgent(agentName: string) {
    const row = this.agentTable.locator('tr').filter({ hasText: agentName });
    await row.getByRole('button', { name: 'Start' }).click();
  }

  async stopAgent(agentName: string) {
    const row = this.agentTable.locator('tr').filter({ hasText: agentName });
    await row.getByRole('button', { name: 'Stop' }).click();
  }

  async deleteAgent(agentName: string) {
    const row = this.agentTable.locator('tr').filter({ hasText: agentName });
    await row.getByRole('button', { name: 'Delete' }).click();
  }

  async expectAgentToExist(agentName: string) {
    await expect(this.agentTable.locator('tr').filter({ hasText: agentName })).toBeVisible();
  }

  async expectAgentStatus(agentName: string, status: string) {
    const row = this.agentTable.locator('tr').filter({ hasText: agentName });
    await expect(row.locator(`[data-testid="status-${status}"]`)).toBeVisible();
  }
}
```

### 5.3 Test Scenarios
```typescript
// tests/e2e/agents.spec.ts
import { test, expect } from '@playwright/test';
import { AgentsPage } from '../pages/AgentsPage';

test.describe('Agent Management', () => {
  let agentsPage: AgentsPage;

  test.beforeEach(async ({ page }) => {
    agentsPage = new AgentsPage(page);
    await agentsPage.goto();
  });

  test('should create a new agent', async ({ page }) => {
    await agentsPage.createAgent('Test Agent', 'You are a test agent');
    
    await expect(page.getByText('Agent created successfully')).toBeVisible();
    await agentsPage.expectAgentToExist('Test Agent');
  });

  test('should start and stop an agent', async ({ page }) => {
    // Create agent first
    await agentsPage.createAgent('Control Agent', 'You are a control agent');
    
    // Start agent
    await agentsPage.startAgent('Control Agent');
    await expect(page.getByText('Agent started successfully')).toBeVisible();
    await agentsPage.expectAgentStatus('Control Agent', 'Active');
    
    // Stop agent
    await agentsPage.stopAgent('Control Agent');
    await expect(page.getByText('Agent stopped successfully')).toBeVisible();
    await agentsPage.expectAgentStatus('Control Agent', 'Inactive');
  });

  test('should delete an agent', async ({ page }) => {
    // Create agent first
    await agentsPage.createAgent('Delete Agent', 'You are a delete agent');
    
    // Delete agent
    await agentsPage.deleteAgent('Delete Agent');
    await expect(page.getByText('Agent deleted successfully')).toBeVisible();
    
    // Verify agent is gone
    await expect(agentsPage.agentTable.locator('tr').filter({ hasText: 'Delete Agent' })).not.toBeVisible();
  });

  test('should handle form validation', async ({ page }) => {
    await agentsPage.createAgentButton.click();
    await agentsPage.saveButton.click();
    
    await expect(page.getByText('Agent name is required')).toBeVisible();
  });

  test('should display agent details when selected', async ({ page }) => {
    // Create agent first
    await agentsPage.createAgent('Detail Agent', 'You are a detail agent');
    
    // Click on agent row
    await agentsPage.agentTable.locator('tr').filter({ hasText: 'Detail Agent' }).click();
    
    // Verify details panel shows
    await expect(page.getByText('Agent Details')).toBeVisible();
    await expect(page.getByText('Detail Agent')).toBeVisible();
    await expect(page.getByText('You are a detail agent')).toBeVisible();
  });
});
```

### 5.4 Task Management Tests
```typescript
// tests/e2e/tasks.spec.ts
import { test, expect } from '@playwright/test';
import { TasksPage } from '../pages/TasksPage';
import { AgentsPage } from '../pages/AgentsPage';

test.describe('Task Management', () => {
  let tasksPage: TasksPage;
  let agentsPage: AgentsPage;

  test.beforeEach(async ({ page }) => {
    tasksPage = new TasksPage(page);
    agentsPage = new AgentsPage(page);
    
    // Create an agent first
    await agentsPage.goto();
    await agentsPage.createAgent('Task Agent', 'You are a task agent');
    
    // Navigate to tasks
    await tasksPage.goto();
  });

  test('should create a new task', async ({ page }) => {
    await tasksPage.createTask('Test Task', 'This is a test task', 'Task Agent');
    
    await expect(page.getByText('Task created successfully')).toBeVisible();
    await tasksPage.expectTaskToExist('Test Task');
  });

  test('should execute a task', async ({ page }) => {
    // Create task first
    await tasksPage.createTask('Execute Task', 'This task will be executed', 'Task Agent');
    
    // Execute task
    await tasksPage.executeTask('Execute Task');
    await expect(page.getByText('Task execution started')).toBeVisible();
    
    // Task should show as InProgress
    await tasksPage.expectTaskStatus('Execute Task', 'InProgress');
  });

  test('should filter tasks by status', async ({ page }) => {
    // Create tasks with different statuses
    await tasksPage.createTask('Todo Task', 'Todo task', 'Task Agent');
    await tasksPage.createTask('Active Task', 'Active task', 'Task Agent');
    await tasksPage.executeTask('Active Task');
    
    // Filter by InProgress
    await tasksPage.filterByStatus('InProgress');
    await tasksPage.expectTaskToExist('Active Task');
    await expect(tasksPage.taskTable.locator('tr').filter({ hasText: 'Todo Task' })).not.toBeVisible();
    
    // Filter by Todo
    await tasksPage.filterByStatus('Todo');
    await tasksPage.expectTaskToExist('Todo Task');
    await expect(tasksPage.taskTable.locator('tr').filter({ hasText: 'Active Task' })).not.toBeVisible();
  });

  test('should cancel a running task', async ({ page }) => {
    // Create and execute task
    await tasksPage.createTask('Cancel Task', 'This task will be cancelled', 'Task Agent');
    await tasksPage.executeTask('Cancel Task');
    
    // Cancel task
    await tasksPage.cancelTask('Cancel Task');
    await expect(page.getByText('Task cancelled successfully')).toBeVisible();
    await tasksPage.expectTaskStatus('Cancel Task', 'Cancelled');
  });
});
```

### 5.5 End-to-End Workflow Tests
```typescript
// tests/e2e/workflow.spec.ts
import { test, expect } from '@playwright/test';
import { DashboardPage } from '../pages/DashboardPage';
import { AgentsPage } from '../pages/AgentsPage';
import { TasksPage } from '../pages/TasksPage';
import { ExecutionPage } from '../pages/ExecutionPage';

test.describe('Complete Workflow', () => {
  test('should complete full agent and task lifecycle', async ({ page }) => {
    const dashboardPage = new DashboardPage(page);
    const agentsPage = new AgentsPage(page);
    const tasksPage = new TasksPage(page);
    const executionPage = new ExecutionPage(page);

    // Start at dashboard
    await dashboardPage.goto();
    await expect(page.getByText('Nox Agent Ecosystem')).toBeVisible();

    // Create agent
    await agentsPage.goto();
    await agentsPage.createAgent('Workflow Agent', 'You are a workflow test agent');
    await expect(page.getByText('Agent created successfully')).toBeVisible();

    // Start agent
    await agentsPage.startAgent('Workflow Agent');
    await expect(page.getByText('Agent started successfully')).toBeVisible();

    // Create task
    await tasksPage.goto();
    await tasksPage.createTask('Workflow Task', 'Complete workflow test task', 'Workflow Agent');
    await expect(page.getByText('Task created successfully')).toBeVisible();

    // Execute task
    await tasksPage.executeTask('Workflow Task');
    await expect(page.getByText('Task execution started')).toBeVisible();

    // Monitor execution
    await executionPage.goto();
    await executionPage.expectRunningTask('Workflow Task');

    // Wait for completion (or timeout)
    await page.waitForTimeout(10000);

    // Verify task completed
    await tasksPage.goto();
    await tasksPage.expectTaskStatus('Workflow Task', 'Done');

    // Check dashboard shows updated stats
    await dashboardPage.goto();
    await expect(page.getByText('Completed: 1')).toBeVisible();
  });
});
```

## 6. Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Set up React + TypeScript + Vite project
- [ ] Configure MUI theme and basic layout
- [ ] Implement main layout with navigation
- [ ] Set up API service layer
- [ ] Create basic TypeScript types
- [ ] Set up Playwright testing framework

### Phase 2: Core Features (Week 3-4)
- [ ] Implement Dashboard page
- [ ] Create Agent management interface
- [ ] Implement Task management interface
- [ ] Add CRUD operations for agents and tasks
- [ ] Set up React Query for state management
- [ ] Add basic Playwright tests

### Phase 3: Advanced Features (Week 5-6)
- [ ] Implement real-time updates with WebSocket
- [ ] Add Execution monitoring interface
- [ ] Create System logs viewer
- [ ] Add form validation and error handling
- [ ] Implement comprehensive Playwright test suite
- [ ] Add visual regression tests

### Phase 4: Polish and Optimization (Week 7-8)
- [ ] Optimize performance and bundle size
- [ ] Add comprehensive error handling
- [ ] Implement responsive design
- [ ] Add accessibility features
- [ ] Complete test coverage
- [ ] Add deployment configuration

## 7. Development Setup

### 7.1 Initial Setup Commands
```bash
# Create React app
npm create vite@latest nox-frontend -- --template react-ts
cd nox-frontend

# Install dependencies
npm install @mui/material @emotion/react @emotion/styled
npm install @mui/icons-material
npm install @tanstack/react-query
npm install axios
npm install socket.io-client
npm install react-router-dom
npm install zustand

# Install dev dependencies
npm install -D @playwright/test
npm install -D @types/node
npm install -D eslint-config-prettier prettier

# Initialize Playwright
npx playwright install
```

### 7.2 Package.json Scripts
```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "playwright test",
    "test:ui": "playwright test --ui",
    "test:headed": "playwright test --headed",
    "lint": "eslint src --ext .ts,.tsx",
    "lint:fix": "eslint src --ext .ts,.tsx --fix",
    "format": "prettier --write src/**/*.{ts,tsx}"
  }
}
```

## 8. Deployment Strategy

### 8.1 Docker Configuration
```dockerfile
# Dockerfile
FROM node:18-alpine as builder

WORKDIR /app
COPY package*.json ./
RUN npm ci

COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

### 8.2 Development Docker Compose
```yaml
# docker-compose.dev.yml
version: '3.8'

services:
  frontend:
    build: .
    ports:
      - "3000:80"
    depends_on:
      - backend
    environment:
      - REACT_APP_API_URL=http://localhost:8080

  backend:
    build: ../
    ports:
      - "8080:8080"
    volumes:
      - ../:/app
    command: cargo run -- serve --port 8080
```

This comprehensive plan provides a modern, professional React MUI frontend with robust Playwright testing that will greatly enhance the user experience of your Nox Agent Ecosystem. The implementation is designed to be built incrementally and includes comprehensive testing from the start.

Would you like me to start implementing any specific part of this plan?