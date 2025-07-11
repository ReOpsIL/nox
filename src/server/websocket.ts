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
  private clients: Map<WebSocket, { id: string; lastPing: Date }> = new Map();
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
  private handleSubscription(_ws: WebSocket, _data: any): void {
    // TODO: Implement subscription logic
    // This would allow clients to subscribe to specific events or agents
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