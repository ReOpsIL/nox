import { EventEmitter } from 'events';
import { RegistryManager } from './registry-manager';
import { ConfigManager } from './config-manager';
import { GitManager } from './git-manager';
import { AgentManager } from './agent-manager';
import { TaskManager } from './task-manager';
import { MessageBroker } from './message-broker';
import { DashboardServer } from '../dashboard/server';
import { MetricsManager } from '../monitoring/metrics';
import { WebSocketServer } from '../server/websocket';
import { AgentConfig } from '../types';
import { logger } from '../utils/logger';

export interface SystemStatus {
  running: boolean;
  activeAgents: number;
  totalTasks: number;
  registryVersion: string;
  uptime: string;
  memoryUsage: number;
  errors: string[];
}

export class NoxSystem extends EventEmitter {
  private initialized = false;
  private running = false;
  private startTime?: Date;
  private systemErrors: string[] = [];

  private configManager: ConfigManager;
  private registryManager: RegistryManager;
  private gitManager: GitManager;
  private agentManager: AgentManager;
  private taskManager: TaskManager;
  private messageBroker: MessageBroker;
  private metricsManager: MetricsManager;
  private websocketServer: WebSocketServer;
  private dashboardServer: DashboardServer;

  constructor(workingDir: string = process.cwd()) {
    super();
    
    // Initialize core managers
    this.configManager = new ConfigManager();
    this.registryManager = new RegistryManager();
    this.gitManager = new GitManager();
    this.agentManager = new AgentManager(workingDir);
    this.taskManager = new TaskManager(workingDir);
    this.messageBroker = new MessageBroker(workingDir);
    
    // Initialize monitoring and dashboard components
    this.metricsManager = new MetricsManager(this.agentManager, this.messageBroker, this.taskManager, workingDir);
    this.websocketServer = new WebSocketServer(this.agentManager, this.messageBroker, this.taskManager);
    this.dashboardServer = new DashboardServer(
      this.agentManager,
      this.taskManager,
      this.messageBroker,
      this.metricsManager,
      this.websocketServer,
      this.registryManager,
      workingDir
    );

    // Setup event handlers
    this.setupEventHandlers();
  }

  private setupEventHandlers(): void {
    // Agent lifecycle events
    this.agentManager.on('agent-created', (agent: AgentConfig) => {
      logger.info(`Agent created: ${agent.id}`);
      this.emit('agent-created', agent);
    });

    this.agentManager.on('agent-deleted', (agentId: string) => {
      logger.info(`Agent deleted: ${agentId}`);
      this.emit('agent-deleted', agentId);
    });

    this.agentManager.on('agent-error', (agentId: string, error: Error) => {
      logger.error(`Agent error [${agentId}]:`, error);
      this.emit('agent-error', agentId, error);
    });

    // Task management events
    this.taskManager.on('task-created', (task) => {
      logger.info(`Task created: ${task.id} for agent ${task.agentId}`);
      this.emit('task-created', task);
    });

    this.taskManager.on('task-updated', (task) => {
      logger.info(`Task updated: ${task.id} status: ${task.status}`);
      this.emit('task-updated', task);
    });

    this.taskManager.on('task-started', (task) => {
      logger.info(`Task started: ${task.id} for agent ${task.agentId}`);
      this.emit('task-started', task);
    });

    this.taskManager.on('task-execute', async (task) => {
      logger.info(`Executing task: ${task.id} for agent ${task.agentId}`);
      await this.executeTaskOnAgent(task);
    });

    // Registry events
    this.registryManager.on('registry-updated', () => {
      logger.info('Registry updated');
      this.emit('registry-updated');
    });

    // Error handling
    this.on('error', (error) => {
      logger.error('System error:', error);
    });
  }

  async initialize(force = false): Promise<void> {
    try {
      logger.info('Initializing Nox system...');

      // Load configuration
      const config = await this.configManager.loadConfig();
      logger.info('Configuration loaded');

      // Initialize registry
      await this.registryManager.initialize(config.storage.registryPath, force);
      logger.info('Registry initialized');

      // Initialize Git repository
      await this.gitManager.initialize(config.storage.registryPath);
      logger.info('Git repository initialized');

      // Initialize other managers
      await this.agentManager.initialize(config);
      await this.taskManager.initialize(config);
      await this.messageBroker.initialize(config);

      // Initialize monitoring and dashboard components
      await this.metricsManager.initialize(config);
      await this.websocketServer.initialize(config);
      await this.dashboardServer.initialize(config);

      this.initialized = true;
      logger.info('Nox system initialized successfully');

    } catch (error) {
      logger.error('Failed to initialize Nox system:', error);
      throw error;
    }
  }

  /**
   * Check if the system is already initialized by verifying config and registry existence
   */
  private async checkIfInitialized(): Promise<boolean> {
    try {
      // Check if config file exists
      const configExists = await this.configManager.configExists();
      if (!configExists) {
        return false;
      }

      // Check if registry directory exists
      const config = await this.configManager.loadConfig();
      const fs = await import('fs/promises');
      try {
        await fs.access(config.storage.registryPath);
        return true;
      } catch {
        return false;
      }
    } catch {
      return false;
    }
  }

  /**
   * Load existing configuration and initialize components
   */
  private async loadExistingConfiguration(): Promise<void> {
    try {
      // Load configuration
      const config = await this.configManager.loadConfig();

      // Initialize registry
      await this.registryManager.initialize(config.storage.registryPath, false);

      // Initialize Git repository
      await this.gitManager.initialize(config.storage.registryPath);

      // Initialize other components
      await this.taskManager.initialize(config);
      await this.messageBroker.initialize(config);
      await this.agentManager.initialize(config);

      // Initialize monitoring and dashboard components
      await this.metricsManager.initialize(config);
      await this.websocketServer.initialize(config);
      await this.dashboardServer.initialize(config);

      // Mark as initialized
      this.initialized = true;
      logger.info('Existing system configuration loaded successfully');
    } catch (error) {
      logger.error('Failed to load existing configuration:', error);
      throw error;
    }
  }

  async start(_background = false): Promise<void> {
    // Check if system is already initialized
    if (!this.initialized) {
      const isAlreadyInitialized = await this.checkIfInitialized();
      if (isAlreadyInitialized) {
        logger.info('System already initialized, loading existing configuration...');
        await this.loadExistingConfiguration();
      } else {
        throw new Error('System not initialized. Run "nox init" first.');
      }
    }

    try {
      logger.info('Starting Nox ecosystem...');
      this.startTime = new Date();

      // Start core services
      await this.messageBroker.start();
      await this.taskManager.start();
      await this.agentManager.start();

      // Start monitoring and dashboard services
      await this.metricsManager.start();
      await this.websocketServer.start();
      await this.dashboardServer.start();

      // Load and start existing agents
      const agents = await this.registryManager.listAgents();
      const activeAgents = agents.filter(agent => agent.status === 'active');
      
      for (const agent of activeAgents) {
        try {
          await this.agentManager.spawnAgent(agent);
          logger.info(`Restored agent: ${agent.id}`);
        } catch (error) {
          logger.error(`Failed to restore agent ${agent.id}:`, error);
        }
      }

      this.running = true;
      this.emit('system-started');
      logger.info(`Nox ecosystem started with ${activeAgents.length} agents`);

    } catch (error) {
      logger.error('Failed to start Nox ecosystem:', error);
      throw error;
    }
  }

  async shutdown(): Promise<void> {
    if (!this.running) {
      return;
    }

    try {
      logger.info('Shutting down Nox ecosystem...');

      // Stop agents
      await this.agentManager.shutdown();

      // Stop services
      await this.taskManager.shutdown();
      await this.messageBroker.shutdown();

      // Final registry backup
      await this.gitManager.commit('System shutdown - final backup');

      this.running = false;
      this.emit('system-stopped');
      logger.info('Nox ecosystem shutdown complete');

    } catch (error) {
      logger.error('Error during shutdown:', error);
      throw error;
    }
  }

  async getStatus(): Promise<SystemStatus> {
    const agents = await this.registryManager.listAgents();
    const activeAgents = agents.filter(agent => agent.status === 'active').length;
    const totalTasks = await this.taskManager.getTotalTaskCount();
    const registryVersion = await this.gitManager.getCurrentCommit();
    
    let uptime = 'Not running';
    if (this.running && this.startTime) {
      const uptimeMs = Date.now() - this.startTime.getTime();
      uptime = this.formatUptime(uptimeMs);
    }

    const memoryUsage = process.memoryUsage().heapUsed / 1024 / 1024; // MB

    return {
      running: this.running,
      activeAgents,
      totalTasks,
      registryVersion: registryVersion.slice(0, 8),
      uptime,
      memoryUsage: Math.round(memoryUsage),
      errors: this.systemErrors.slice(-10) // Show last 10 errors
    };
  }

  private formatUptime(ms: number): string {
    const seconds = Math.floor(ms / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) return `${days}d ${hours % 24}h ${minutes % 60}m`;
    if (hours > 0) return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
    if (minutes > 0) return `${minutes}m ${seconds % 60}s`;
    return `${seconds}s`;
  }

  addError(error: string): void {
    const timestamp = new Date().toISOString();
    this.systemErrors.push(`[${timestamp}] ${error}`);
    
    // Keep only last 50 errors in memory
    if (this.systemErrors.length > 50) {
      this.systemErrors = this.systemErrors.slice(-50);
    }
    
    logger.error(error);
    this.emit('system-error', error);
  }

  clearErrors(): void {
    this.systemErrors = [];
  }

  // Getters for managers (used by commands)
  get registry(): RegistryManager { return this.registryManager; }
  get config(): ConfigManager { return this.configManager; }
  get git(): GitManager { return this.gitManager; }
  get agents(): AgentManager { return this.agentManager; }
  get tasks(): TaskManager { return this.taskManager; }
  get messages(): MessageBroker { return this.messageBroker; }

  // System state checks
  get isInitialized(): boolean { return this.initialized; }
  get isRunning(): boolean { return this.running; }

  /**
   * Execute a task on the specified agent using Claude CLI
   */
  private async executeTaskOnAgent(task: any): Promise<void> {
    try {
      // Get the agent from registry to validate it exists
      const agents = await this.registryManager.listAgents();
      const agent = agents.find(a => a.id === task.agentId);
      
      if (!agent) {
        logger.error(`Agent ${task.agentId} not found in registry for task ${task.id}`);
        await this.taskManager.updateTask(task.id, {
          status: 'cancelled'
        });
        return;
      }

      logger.info(`Starting agent ${agent.name} (${agent.id}) for task: ${task.title}`);
      
      // Spawn the agent if not already running
      try {
        await this.agentManager.spawnAgent(agent);
        logger.info(`Agent ${agent.id} spawned successfully`);
      } catch (spawnError) {
        logger.error(`Failed to spawn agent ${agent.id}:`, spawnError);
        // Agent might already be running, continue anyway
      }

      // Get Claude interface for the agent
      const claudeInterface = this.agentManager.getClaudeInterface(task.agentId);
      if (!claudeInterface) {
        logger.error(`No Claude interface available for agent ${task.agentId}`);
        await this.taskManager.updateTask(task.id, {
          status: 'cancelled'
        });
        return;
      }

      // Create task message for Claude
      const taskMessage = this.formatTaskForClaude(task, agent);
      
      logger.info(`Sending task to Claude CLI for agent ${agent.name}: ${task.title}`);
      
      // Send task to Claude
      await claudeInterface.sendMessage(taskMessage);
      
      logger.info(`Task ${task.id} sent to agent ${agent.name} successfully`);

      // The task completion will be handled by Claude CLI responses
      // For now, we just mark it as in progress
      
    } catch (error) {
      logger.error(`Failed to execute task ${task.id} on agent ${task.agentId}:`, error);
      
      // Mark task as failed
      await this.taskManager.updateTask(task.id, {
        status: 'cancelled'
      });
    }
  }

  /**
   * Format task for Claude CLI execution
   */
  private formatTaskForClaude(task: any, agent: any): string {
    const taskPrompt = `
🎯 NEW TASK ASSIGNED: ${task.title}

📋 Task Details:
- Task ID: ${task.id}
- Priority: ${task.priority}
- Created: ${task.createdAt}
- Deadline: ${task.deadline}

📝 Description:
${task.description}

🤖 Your Role as ${agent.name}:
${agent.systemPrompt}

🔧 Your Capabilities:
${agent.capabilities.join(', ')}

📊 Task Requirements:
Please complete this task using your specialized capabilities. When finished:
1. Provide a detailed summary of what you accomplished
2. Include any findings, results, or deliverables
3. Mention any challenges or limitations encountered
4. State "TASK COMPLETED: ${task.id}" at the end of your response

Begin working on this task now.`;

    return taskPrompt;
  }
}