export interface NoxConfig {
  security: SecurityConfig;
  claudeCli: ClaudeCliConfig;
  git: GitConfig;
  server: ServerConfig;
  storage: StorageConfig;
  mcp: MCPConfig;
  logging: LoggingConfig;
}

export interface SecurityConfig {
  maxAgents: number;
  spawnRateLimit: number;
  requireApprovalFor: ApprovalType[];
  resourceLimits: GlobalResourceLimits;
  sandboxMode: boolean;
  allowExternalCommunication: boolean;
}

export type ApprovalType = 
  | 'agent_creation' 
  | 'mcp_installation' 
  | 'external_communication'
  | 'resource_expansion'
  | 'registry_modification';

export interface GlobalResourceLimits {
  memoryPerAgent: string;
  claudeCallsPerMinute: number;
  maxConcurrentTasks: number;
  maxDockerContainers: number;
  diskSpaceLimit: string;
}

export interface ClaudeCliConfig {
  sessionTimeout: number;
  autoRestartOnCrash: boolean;
  backupConversations: boolean;
  cliPath: string;
  defaultArgs: string[];
  healthCheckInterval: number;
}

export interface GitConfig {
  autoCommit: boolean;
  backupInterval: number;
  remoteBackup?: string;
  commitMessagePrefix: string;
  branchStrategy: 'main' | 'feature' | 'agent-specific';
}

export interface ServerConfig {
  port: number;
  host: string;
  websocketEnabled: boolean;
  dashboardEnabled: boolean;
  apiEnabled: boolean;
  corsOrigins: string[];
  rateLimiting: RateLimitConfig;
}

export interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  skipSuccessfulRequests: boolean;
  keyGenerator?: string;
}

export interface StorageConfig {
  format: 'json' | 'sqlite' | 'mongodb';
  compression: boolean;
  backupRetentionDays: number;
  registryPath: string;
  encryptSensitiveData: boolean;
}

export interface MCPConfig {
  dockerHubRegistry: string;
  autoDiscovery: boolean;
  serviceTimeout: number;
  maxServicesPerAgent: number;
  networkIsolation: boolean;
  resourceLimits: MCPResourceLimits;
}

export interface MCPResourceLimits {
  memory: string;
  cpu: string;
  storage: string;
  networkBandwidth: string;
}

export interface LoggingConfig {
  level: 'debug' | 'info' | 'warn' | 'error' | 'fatal';
  format: 'json' | 'text';
  outputs: LogOutput[];
  retention: LogRetentionConfig;
}

export interface LogOutput {
  type: 'console' | 'file' | 'elasticsearch' | 'webhook';
  config: Record<string, unknown>;
}

export interface LogRetentionConfig {
  days: number;
  maxSizeMB: number;
  compress: boolean;
}

export const DEFAULT_CONFIG: NoxConfig = {
  security: {
    maxAgents: 50,
    spawnRateLimit: 10,
    requireApprovalFor: ['agent_creation', 'mcp_installation', 'external_communication'],
    resourceLimits: {
      memoryPerAgent: '2GB',
      claudeCallsPerMinute: 60,
      maxConcurrentTasks: 10,
      maxDockerContainers: 20,
      diskSpaceLimit: '10GB'
    },
    sandboxMode: true,
    allowExternalCommunication: false
  },
  claudeCli: {
    sessionTimeout: 3600,
    autoRestartOnCrash: true,
    backupConversations: true,
    cliPath: 'claude',
    defaultArgs: ['--interactive'],
    healthCheckInterval: 30
  },
  git: {
    autoCommit: true,
    backupInterval: 300,
    commitMessagePrefix: '[nox]',
    branchStrategy: 'main'
  },
  server: {
    port: 3000,
    host: 'localhost',
    websocketEnabled: true,
    dashboardEnabled: true,
    apiEnabled: true,
    corsOrigins: ['http://localhost:3000'],
    rateLimiting: {
      windowMs: 900000, // 15 minutes
      maxRequests: 1000,
      skipSuccessfulRequests: false
    }
  },
  storage: {
    format: 'json',
    compression: true,
    backupRetentionDays: 30,
    registryPath: '.nox-registry',
    encryptSensitiveData: false
  },
  mcp: {
    dockerHubRegistry: 'https://hub.docker.com/v2/repositories/mcp/',
    autoDiscovery: true,
    serviceTimeout: 30,
    maxServicesPerAgent: 5,
    networkIsolation: true,
    resourceLimits: {
      memory: '1GB',
      cpu: '0.5',
      storage: '2GB',
      networkBandwidth: '100MB'
    }
  },
  logging: {
    level: 'info',
    format: 'json',
    outputs: [
      { type: 'console', config: {} },
      { type: 'file', config: { path: 'logs/nox.log' } }
    ],
    retention: {
      days: 30,
      maxSizeMB: 100,
      compress: true
    }
  }
};