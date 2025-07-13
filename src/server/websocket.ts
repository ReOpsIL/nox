import * as WebSocket from 'ws';
import { EventEmitter } from 'events';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';
import { AgentManager } from '../core/agent-manager';
import { MessageBroker } from '../core/message-broker';
import { TaskManager } from '../core/task-manager';

/**
 * WebSocket Server - Provides real-time updates to clients
 * Broadcasts agent status, task updates, and inter-agent messages
 */
export class WebSocketServer extends EventEmitter {
  private server: WebSocket.Server | null = null;
  private clients: Map<WebSocket, { id: string; lastPing: Date; subscriptions?: Set<string> }> = new Map();
  private pingInterval: NodeJS.Timeout | null = null;
  private initialized = false;
  private port: number;

  constructor(
    private agentManager: AgentManager,
    private messageBroker: MessageBroker,
    private taskManager: TaskManager
  ) {
    super();
    this.port = 3000; // Default port
  }

  /**
   * Initialize the WebSocket server
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('WebSocketServer already initialized');
      return;
    }

    // Check if WebSocket is enabled in config
    if (!config.server?.websocketEnabled) {
      logger.info('WebSocketServer disabled in configuration');
      return;
    }

    // Set port from config if available
    this.port = config.server?.port || 3000;

    this.initialized = true;
    logger.info('WebSocketServer initialized');
    this.emit('initialized');
  }

  /**
   * Start the WebSocket server
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('WebSocketServer not initialized');
    }

    try {
      // Create WebSocket server
      this.server = new WebSocket.Server({ port: this.port });

      // Set up event listeners
      this.server.on('connection', this.handleConnection.bind(this));
      this.server.on('error', this.handleServerError.bind(this));

      // Set up ping interval to keep connections alive and detect stale connections
      this.pingInterval = setInterval(this.pingClients.bind(this), 30000);

      // Set up event listeners for core components
      this.setupEventListeners();

      logger.info(`WebSocketServer started on port ${this.port}`);
      this.emit('started');

    } catch (error) {
      logger.error('Failed to start WebSocketServer:', error);
      throw error;
    }
  }

  /**
   * Shutdown the WebSocket server
   */
  async shutdown(): Promise<void> {
    // Clear ping interval
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }

    // Close all connections
    if (this.server) {
      for (const client of this.clients.keys()) {
        client.terminate();
      }
      this.clients.clear();

      // Close server
      await new Promise<void>((resolve, reject) => {
        if (!this.server) {
          resolve();
          return;
        }
        this.server.close((err) => {
          if (err) {
            reject(err);
          } else {
            resolve();
          }
        });
      });

      this.server = null;
    }

    logger.info('WebSocketServer shutdown');
    this.emit('shutdown');
  }

  /**
   * Broadcast a message to all connected clients
   */
  broadcast(type: string, data: any): void {
    if (!this.server || this.clients.size === 0) {
      return;
    }

    const message = JSON.stringify({ type, data, timestamp: new Date().toISOString() });

    for (const client of this.clients.keys()) {
      if (client.readyState === WebSocket.OPEN) {
        client.send(message);
      }
    }
  }

  /**
   * Send a message to a specific client
   */
  sendToClient(clientId: string, type: string, data: any): void {
    if (!this.server) {
      return;
    }

    const message = JSON.stringify({ type, data, timestamp: new Date().toISOString() });

    for (const [client, info] of this.clients.entries()) {
      if (info.id === clientId && client.readyState === WebSocket.OPEN) {
        client.send(message);
        break;
      }
    }
  }

  /**
   * Send a message directly to a WebSocket
   */
  private sendToWebSocket(ws: WebSocket, data: any): void {
    if (ws.readyState === WebSocket.OPEN) {
      const message = JSON.stringify({ ...data, timestamp: new Date().toISOString() });
      ws.send(message);
    }
  }

  /**
   * Handle a new WebSocket connection
   */
  private handleConnection(ws: WebSocket, _req: any): void {
    // Generate client ID
    const clientId = `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    
    // Store client info
    this.clients.set(ws, { id: clientId, lastPing: new Date() });
    
    logger.info(`WebSocket client connected: ${clientId}`);
    
    // Set up client event listeners
    ws.on('message', (message: string) => this.handleClientMessage(ws, message));
    ws.on('close', () => this.handleClientDisconnect(ws));
    ws.on('error', (error) => this.handleClientError(ws, error));
    ws.on('pong', () => this.handleClientPong(ws));
    
    // Send welcome message
    ws.send(JSON.stringify({
      type: 'connection_established',
      data: {
        clientId,
        serverTime: new Date().toISOString()
      }
    }));
    
    // Send initial state
    this.sendInitialState(ws);
  }

  /**
   * Handle a message from a client
   */
  private handleClientMessage(ws: WebSocket, message: string): void {
    try {
      const clientInfo = this.clients.get(ws);
      if (!clientInfo) return;

      const data = JSON.parse(message);
      logger.debug(`WebSocket message from client ${clientInfo.id}:`, data);

      // Update last ping time
      clientInfo.lastPing = new Date();
      
      // Handle different message types
      switch (data.type) {
        case 'ping':
          ws.send(JSON.stringify({ type: 'pong', timestamp: new Date().toISOString() }));
          break;
          
        case 'subscribe':
          // Handle subscription requests
          this.handleSubscription(ws, data);
          break;
          
        default:
          logger.warn(`Unknown WebSocket message type: ${data.type}`);
      }
      
    } catch (error) {
      logger.error('Error handling WebSocket message:', error);
    }
  }

  /**
   * Handle client subscription requests
   */
  private handleSubscription(ws: WebSocket, data: any): void {
    try {
      const clientInfo = this.clients.get(ws);
      if (!clientInfo) {
        logger.warn('Subscription request from unknown client');
        return;
      }

      const { type, filters } = data;
      
      // Initialize subscriptions if not exists
      if (!clientInfo.subscriptions) {
        clientInfo.subscriptions = new Set();
      }

      // Handle different subscription types
      switch (type) {
        case 'agents':
          // Subscribe to agent events (status changes, creation, deletion)
          if (filters?.agentIds) {
            filters.agentIds.forEach((agentId: string) => {
              clientInfo.subscriptions?.add(`agent:${agentId}`);
            });
          } else {
            clientInfo.subscriptions?.add('agents:all');
          }
          break;

        case 'tasks':
          // Subscribe to task events (creation, updates, completion)
          if (filters?.agentIds) {
            filters.agentIds.forEach((agentId: string) => {
              clientInfo.subscriptions?.add(`tasks:agent:${agentId}`);
            });
          } else {
            clientInfo.subscriptions?.add('tasks:all');
          }
          break;

        case 'system':
          // Subscribe to system events (health, status, metrics)
          clientInfo.subscriptions?.add('system:all');
          break;

        case 'messages':
          // Subscribe to inter-agent messages
          if (filters?.agentIds) {
            filters.agentIds.forEach((agentId: string) => {
              clientInfo.subscriptions?.add(`messages:${agentId}`);
            });
          } else {
            clientInfo.subscriptions?.add('messages:all');
          }
          break;

        default:
          logger.warn(`Unknown subscription type: ${type}`);
          this.sendToWebSocket(ws, {
            type: 'error',
            message: `Unknown subscription type: ${type}`
          });
          return;
      }

      // Send confirmation
      this.sendToWebSocket(ws, {
        type: 'subscription_confirmed',
        subscriptionType: type,
        filters: filters || {}
      });

      logger.debug(`Client ${clientInfo.id} subscribed to ${type} events`);

    } catch (error) {
      logger.error('Error handling subscription:', error);
      this.sendToWebSocket(ws, {
        type: 'error',
        message: 'Failed to process subscription request'
      });
    }
  }

  /**
   * Handle client disconnect
   */
  private handleClientDisconnect(ws: WebSocket): void {
    const clientInfo = this.clients.get(ws);
    if (clientInfo) {
      logger.info(`WebSocket client disconnected: ${clientInfo.id}`);
      this.clients.delete(ws);
    }
  }

  /**
   * Handle client error
   */
  private handleClientError(ws: WebSocket, error: Error): void {
    const clientInfo = this.clients.get(ws);
    logger.error(`WebSocket client error ${clientInfo ? clientInfo.id : 'unknown'}:`, error);
  }

  /**
   * Handle client pong response
   */
  private handleClientPong(ws: WebSocket): void {
    const clientInfo = this.clients.get(ws);
    if (clientInfo) {
      clientInfo.lastPing = new Date();
    }
  }

  /**
   * Handle server error
   */
  private handleServerError(error: Error): void {
    logger.error('WebSocketServer error:', error);
    this.emit('error', error);
  }

  /**
   * Send initial state to a new client
   */
  private async sendInitialState(ws: WebSocket): Promise<void> {
    try {
      // Send agent status
      const agents = await this.agentManager.listRunningAgents();
      ws.send(JSON.stringify({
        type: 'agent_status_list',
        data: agents
      }));

      // Send task dashboard
      const taskDashboard = await this.taskManager.getTaskDashboard();
      ws.send(JSON.stringify({
        type: 'task_dashboard',
        data: taskDashboard
      }));

    } catch (error) {
      logger.error('Error sending initial state to client:', error);
    }
  }

  /**
   * Ping all clients to keep connections alive and detect stale connections
   */
  private pingClients(): void {
    const now = new Date();
    
    for (const [client, info] of this.clients.entries()) {
      // Check if client hasn't responded for more than 60 seconds
      if (now.getTime() - info.lastPing.getTime() > 60000) {
        logger.warn(`Terminating stale WebSocket connection: ${info.id}`);
        client.terminate();
        this.clients.delete(client);
        continue;
      }
      
      // Send ping
      if (client.readyState === WebSocket.OPEN) {
        client.ping();
      }
    }
  }

  /**
   * Set up event listeners for core components
   */
  private setupEventListeners(): void {
    // Agent Manager events
    this.agentManager.on('agent-created', (config) => {
      this.broadcast('agent_created', {
        agentId: config.id,
        name: config.name,
        timestamp: new Date().toISOString()
      });
    });

    this.agentManager.on('agent-deleted', (agentId) => {
      this.broadcast('agent_deleted', {
        agentId,
        timestamp: new Date().toISOString()
      });
    });

    this.agentManager.on('agent-response', (agentId, response) => {
      this.broadcast('agent_response', {
        agentId,
        response,
        timestamp: new Date().toISOString()
      });
    });

    // Message Broker events
    this.messageBroker.on('message-sent', (message) => {
      this.broadcast('agent_message', {
        from: message.from,
        to: message.to,
        type: message.type,
        content: message.content,
        priority: message.priority,
        timestamp: message.timestamp
      });
    });

    // Task Manager events
    this.taskManager.on('task-created', (task) => {
      this.broadcast('task_created', {
        taskId: task.id,
        agentId: task.agentId,
        title: task.title,
        status: task.status,
        priority: task.priority,
        timestamp: new Date().toISOString()
      });
    });

    this.taskManager.on('task-updated', (task) => {
      this.broadcast('task_updated', {
        taskId: task.id,
        agentId: task.agentId,
        title: task.title,
        status: task.status,
        priority: task.priority,
        progress: task.progress,
        startedAt: task.startedAt?.toISOString(),
        completedAt: task.completedAt?.toISOString(),
        deadline: task.deadline?.toISOString(),
        timestamp: new Date().toISOString()
      });
    });

    this.taskManager.on('task-started', (task) => {
      this.broadcast('task_started', {
        taskId: task.id,
        agentId: task.agentId,
        title: task.title,
        status: task.status,
        priority: task.priority,
        progress: task.progress,
        startedAt: task.startedAt?.toISOString(),
        timestamp: new Date().toISOString()
      });
    });

    this.taskManager.on('task-execute', (task) => {
      this.broadcast('task_execute', {
        taskId: task.id,
        agentId: task.agentId,
        title: task.title,
        status: task.status,
        priority: task.priority,
        timestamp: new Date().toISOString()
      });
    });

    this.taskManager.on('task-delegated', (fromAgentId, toAgentId, task) => {
      this.broadcast('task_delegated', {
        taskId: task.id,
        fromAgentId,
        toAgentId,
        title: task.title,
        priority: task.priority,
        timestamp: new Date().toISOString()
      });
    });
  }

  /**
   * Get the number of connected clients
   */
  getClientCount(): number {
    return this.clients.size;
  }

  /**
   * Check if the server is running
   */
  isRunning(): boolean {
    return this.server !== null;
  }
}