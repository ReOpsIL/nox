export interface AgentConfig {
  id: string;
  name: string;
  systemPrompt: string;
  status: AgentStatus;
  createdAt: Date;
  lastModified: Date;
  resourceLimits: ResourceLimits;
  capabilities: string[];
  relationships: AgentRelationship[];
  mcpServices: string[];
}

export type AgentStatus = 'active' | 'inactive' | 'error' | 'crashed' | 'starting' | 'stopping';

export interface ResourceLimits {
  maxMemoryMB: number;
  maxCPUPercent: number;
  maxConcurrentTasks: number;
  maxSpawnRate: number;
  maxDepth: number;
}

export interface AgentRelationship {
  targetAgentId: string;
  relationshipType: 'collaborator' | 'supervisor' | 'subordinate' | 'peer';
  createdAt: Date;
  metadata?: Record<string, unknown>;
}

export interface AgentProcess {
  id: string;
  pid?: number;
  sessionId?: string;
  status: 'running' | 'crashed' | 'stopped' | 'starting';
  startTime: Date;
  restartCount: number;
  memoryUsage: number;
  cpuUsage: number;
  lastHealthCheck: Date;
}

export interface AgentMetrics {
  id: string;
  uptime: number;
  totalTasks: number;
  completedTasks: number;
  failedTasks: number;
  averageResponseTime: number;
  memoryUsage: number;
  cpuUsage: number;
  lastActivity: Date;
}

export const DEFAULT_RESOURCE_LIMITS: ResourceLimits = {
  maxMemoryMB: 2048,
  maxCPUPercent: 50,
  maxConcurrentTasks: 10,
  maxSpawnRate: 10,
  maxDepth: 5
};