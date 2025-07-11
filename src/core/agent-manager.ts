import { EventEmitter } from 'events';
import { AgentConfig, AgentProcess, NoxConfig, AgentMessage } from '../types';
import { logger } from '../utils/logger';
import { SessionManager } from './session-manager';
import { ProcessMonitor } from '../utils/process-monitor';
import { ClaudeInterface } from './claude-interface';
import { protocolRegistry } from '../protocols/agent-protocols';

export class AgentManager extends EventEmitter {
  private processes: Map<string, AgentProcess> = new Map();
  private claudeInterfaces: Map<string, ClaudeInterface> = new Map();
  private initialized = false;
  private sessionManager: SessionManager;
  private processMonitor: ProcessMonitor;
  private autoRestart: boolean = true;

  constructor(workingDir: string) {
    super();
    this.sessionManager = new SessionManager(workingDir);
    this.processMonitor = new ProcessMonitor({
      checkInterval: 10000, // 10 seconds
      cpuThreshold: 80,
      memoryThreshold: 512, // 512MB
      unresponsiveTimeout: 60000 // 1 minute
    });

    this.setupEventListeners();
  }

  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('AgentManager already initialized');
      return;
    }

    try {
      // Initialize session manager
      await this.sessionManager.initialize();

      // Start process monitor
      this.processMonitor.start();

      // Set auto-restart from config
      this.autoRestart = config.agents?.autoRestart !== false;

      this.initialized = true;
      logger.info('AgentManager initialized with Claude CLI integration');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize AgentManager:', error);
      throw error;
    }
  }

  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }
    logger.info('AgentManager started');
  }


  async spawnAgent(config: AgentConfig): Promise<AgentProcess> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    if (this.processes.has(config.id)) {
      throw new Error(`Agent ${config.id} is already running`);
    }

    try {
      logger.info(`Spawning agent with Claude CLI: ${config.id}`);

      // Create Claude CLI session
      const claudeInterface = await this.sessionManager.createSession(config);
      this.claudeInterfaces.set(config.id, claudeInterface);

      // Get the Claude CLI process info
      const session = claudeInterface.getSession();
      if (!session) {
        throw new Error(`Failed to create session for agent: ${config.id}`);
      }

      // Create agent process record
      const agentProcess: AgentProcess = {
        id: config.id,
        status: 'running',
        startTime: new Date(),
        restartCount: 0,
        memoryUsage: 0,
        cpuUsage: 0,
        lastHealthCheck: new Date(),
        sessionId: session.id,
        pid: process.pid // Claude CLI process PID (if available)
      };

      this.processes.set(config.id, agentProcess);

      // Add to process monitor if we have a PID
      if (agentProcess.pid) {
        this.processMonitor.addProcess(config.id, agentProcess.pid);
      }

      // Set up Claude interface event listeners
      this.setupClaudeInterfaceListeners(config.id, claudeInterface);

      logger.info(`Agent successfully spawned with Claude CLI: ${config.id}`);
      this.emit('agent-created', config);

      return agentProcess;

    } catch (error) {
      // Cleanup on failure
      this.claudeInterfaces.delete(config.id);
      this.processes.delete(config.id);

      logger.error(`Failed to spawn agent ${config.id}:`, error);
      throw error;
    }
  }

  async killAgent(agentId: string): Promise<void> {
    const agentProcess = this.processes.get(agentId);
    if (!agentProcess) {
      logger.warn(`Agent ${agentId} not found for termination`);
      return;
    }

    try {
      logger.info(`Terminating agent: ${agentId}`);

      // Stop Claude interface
      const claudeInterface = this.claudeInterfaces.get(agentId);
      if (claudeInterface) {
        await claudeInterface.stop();
        this.claudeInterfaces.delete(agentId);
      }

      // Stop session
      await this.sessionManager.stopSession(agentId);

      // Remove from process monitor
      this.processMonitor.removeProcess(agentId);

      // Update status and remove
      agentProcess.status = 'stopped';
      this.processes.delete(agentId);

      logger.info(`Agent terminated: ${agentId}`);
      this.emit('agent-deleted', agentId);

    } catch (error) {
      logger.error(`Failed to kill agent ${agentId}:`, error);
      throw error;
    }
  }

  async getAgentStatus(agentId: string): Promise<AgentProcess | null> {
    return this.processes.get(agentId) || null;
  }

  async listRunningAgents(): Promise<AgentProcess[]> {
    return Array.from(this.processes.values());
  }

  /**
   * Send a message to a specific agent
   */
  async sendMessageToAgent(agentId: string, message: string): Promise<string> {
    const claudeInterface = this.claudeInterfaces.get(agentId);
    if (!claudeInterface) {
      throw new Error(`Agent ${agentId} not found or not running`);
    }

    try {
      const response = await claudeInterface.sendMessage(message, true);
      if (!response.success) {
        throw new Error(`Agent ${agentId} error: ${response.error}`);
      }

      return response.content || '';

    } catch (error) {
      logger.error(`Failed to send message to agent ${agentId}:`, error);
      throw error;
    }
  }

  /**
   * Get Claude interface for an agent
   */
  getClaudeInterface(agentId: string): ClaudeInterface | null {
    return this.claudeInterfaces.get(agentId) || null;
  }

  /**
   * Restart an agent
   */
  async restartAgent(agentId: string, config: AgentConfig): Promise<AgentProcess> {
    logger.info(`Restarting agent: ${agentId}`);

    // Kill existing agent
    await this.killAgent(agentId);

    // Wait a moment
    await new Promise(resolve => setTimeout(resolve, 2000));

    // Increment restart count if process existed
    const restartCount = this.processes.has(agentId) ? 
      (this.processes.get(agentId)?.restartCount || 0) + 1 : 0;

    // Spawn new agent
    const newProcess = await this.spawnAgent(config);
    newProcess.restartCount = restartCount;

    return newProcess;
  }

  /**
   * Perform health check on all agents
   */
  async performHealthCheck(): Promise<Map<string, boolean>> {
    const healthResults = new Map<string, boolean>();

    // Update process metrics from monitor
    const processMetrics = this.processMonitor.getAllProcessMetrics();

    for (const [agentId, agentProcess] of Array.from(this.processes)) {
      try {
        // Check Claude interface health
        const claudeInterface = this.claudeInterfaces.get(agentId);
        const claudeHealth = claudeInterface ? claudeInterface.getHealthStatus() : null;

        // Check process metrics
        const metrics = processMetrics.get(agentId);

        // Determine overall health
        const isHealthy = claudeHealth?.healthy === true && 
                         agentProcess.status === 'running' &&
                         (!metrics || metrics.status !== 'unresponsive');

        healthResults.set(agentId, isHealthy);

        // Update agent process health info
        agentProcess.lastHealthCheck = new Date();
        if (metrics) {
          agentProcess.cpuUsage = metrics.cpuUsage;
          agentProcess.memoryUsage = metrics.memoryUsageMB;
        }

        // Auto-restart if unhealthy and auto-restart is enabled
        if (!isHealthy && this.autoRestart) {
          logger.warn(`Agent ${agentId} unhealthy, scheduling restart`);
          this.emit('agent-unhealthy', agentId, claudeHealth);
          // Note: Actual restart logic would be handled by a separate service
        }

      } catch (error) {
        logger.error(`Health check failed for agent ${agentId}:`, error);
        healthResults.set(agentId, false);
      }
    }

    return healthResults;
  }

  /**
   * Get session manager
   */
  getSessionManager(): SessionManager {
    return this.sessionManager;
  }

  /**
   * Get process monitor
   */
  getProcessMonitor(): ProcessMonitor {
    return this.processMonitor;
  }

  /**
   * Set up event listeners for AgentManager
   */
  private setupEventListeners(): void {
    // Process monitor events
    this.processMonitor.on('process-died', (processId: string, pid: number) => {
      logger.warn(`Process died for agent ${processId} (PID: ${pid})`);
      this.emit('agent-process-died', processId, pid);
    });

    this.processMonitor.on('process-warning', (processId: string, type: string, message: string) => {
      logger.warn(`Process warning for agent ${processId} (${type}): ${message}`);
      this.emit('agent-process-warning', processId, type, message);
    });

    // Session manager events
    this.sessionManager.on('session-error', (agentId: string, error: any) => {
      logger.error(`Session error for agent ${agentId}:`, error);
      this.emit('agent-session-error', agentId, error);
    });

    this.sessionManager.on('session-process-exit', (agentId: string, exitInfo: any) => {
      logger.warn(`Claude CLI process exited for agent ${agentId}:`, exitInfo);
      this.emit('agent-claude-exit', agentId, exitInfo);

      // Auto-restart if enabled
      if (this.autoRestart) {
        this.emit('agent-needs-restart', agentId);
      }
    });
  }

  /**
   * Set up event listeners for a Claude interface
   */
  private setupClaudeInterfaceListeners(agentId: string, claudeInterface: ClaudeInterface): void {
    claudeInterface.on('response', (response) => {
      this.emit('agent-response', agentId, response);
    });

    claudeInterface.on('error', (error) => {
      logger.error(`Claude interface error for agent ${agentId}:`, error);
      this.emit('agent-error', agentId, error);
    });

    claudeInterface.on('stopped', () => {
      logger.info(`Claude interface stopped for agent ${agentId}`);
      this.emit('agent-stopped', agentId);
    });
  }

  /**
   * Send a message from one agent to another
   */
  async sendInterAgentMessage(message: AgentMessage): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    const { from, to } = message;

    // Check if both agents exist
    if (!this.processes.has(from) || !this.processes.has(to)) {
      logger.error(`Cannot send message: Agent ${from} or ${to} not found`);
      return false;
    }

    try {
      // Process the message through protocols
      const response = await protocolRegistry.processMessage(message);

      // Get Claude interface for the target agent
      const targetInterface = this.claudeInterfaces.get(to);
      if (!targetInterface) {
        logger.error(`No Claude interface found for agent ${to}`);
        return false;
      }

      // Send the message to the target agent
      await targetInterface.sendMessage(message.content);

      // If there's a response, send it back
      if (response) {
        const sourceInterface = this.claudeInterfaces.get(from);
        if (sourceInterface) {
          await sourceInterface.sendMessage(response.content);
        }
      }

      logger.info(`Message sent from ${from} to ${to}: ${message.type}`);
      this.emit('message-sent', message);
      return true;

    } catch (error) {
      logger.error(`Failed to send message from ${from} to ${to}:`, error);
      return false;
    }
  }

  /**
   * Broadcast a message to all agents
   */
  async broadcastMessage(message: AgentMessage): Promise<number> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    let successCount = 0;
    const agentIds = Array.from(this.processes.keys());

    for (const agentId of agentIds) {
      if (agentId === message.from) continue; // Don't send to sender

      const agentMessage: AgentMessage = {
        ...message,
        to: agentId
      };

      if (await this.sendInterAgentMessage(agentMessage)) {
        successCount++;
      }
    }

    logger.info(`Broadcast message sent to ${successCount}/${agentIds.length - 1} agents`);
    return successCount;
  }

  /**
   * Delegate a task from one agent to another
   */
  async delegateTask(
    fromAgentId: string,
    toAgentId: string,
    taskTitle: string,
    taskDescription: string,
    priority: 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL' = 'MEDIUM'
  ): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    // Create task delegation message
    const message = protocolRegistry.createTaskRequest(
      fromAgentId,
      toAgentId,
      taskTitle,
      taskDescription,
      priority
    );

    const success = await this.sendInterAgentMessage(message);

    if (success) {
      logger.info(`Task delegated from ${fromAgentId} to ${toAgentId}: ${taskTitle}`);
      this.emit('task-delegated', fromAgentId, toAgentId, {
        title: taskTitle,
        description: taskDescription,
        priority
      });
    }

    return success;
  }

  /**
   * Get information from another agent
   */
  async queryAgentCapabilities(
    fromAgentId: string,
    toAgentId: string,
    query: string
  ): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    const message = protocolRegistry.createInfoRequest(
      fromAgentId,
      toAgentId,
      query
    );

    return await this.sendInterAgentMessage(message);
  }

  /**
   * Initiate collaboration between agents
   */
  async initiateCollaboration(
    fromAgentId: string,
    toAgentId: string,
    topic: string,
    details: string
  ): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('AgentManager not initialized');
    }

    const message = protocolRegistry.createCollaborationRequest(
      fromAgentId,
      toAgentId,
      topic,
      details
    );

    return await this.sendInterAgentMessage(message);
  }

  /**
   * Shutdown the agent manager
   */
  async shutdown(): Promise<void> {
    if (!this.initialized) {
      return;
    }

    logger.info('Shutting down AgentManager...');

    try {
      // Stop all agents
      const agentIds = Array.from(this.processes.keys());
      const stopPromises = agentIds.map(agentId => 
        this.killAgent(agentId).catch(error => {
          logger.error(`Failed to stop agent ${agentId} during shutdown:`, error);
        })
      );

      await Promise.all(stopPromises);

      // Stop session manager
      await this.sessionManager.stopAllSessions();

      // Stop process monitor
      this.processMonitor.stop();

      this.initialized = false;
      logger.info('AgentManager shutdown complete');
      this.emit('shutdown');

    } catch (error) {
      logger.error('Error during AgentManager shutdown:', error);
      throw error;
    }
  }
}
