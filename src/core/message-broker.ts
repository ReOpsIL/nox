import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { AgentMessage, NoxConfig, AgentSubscription, MessageType } from '../types';
import { logger } from '../utils/logger';
import { PriorityQueue } from '../utils/priority-queue';

/**
 * Message Broker - Handles inter-agent communication
 * Manages message routing, subscriptions, and persistence
 */
export class MessageBroker extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private messagesDir: string;
  private messageQueue: PriorityQueue<AgentMessage>;
  private subscribers: Map<string, Set<AgentSubscription>> = new Map();
  private processingInterval: NodeJS.Timeout | null = null;
  private persistenceInterval: NodeJS.Timeout | null = null;
  private messageHistory: Map<string, AgentMessage[]> = new Map();
  private maxHistoryPerAgent = 1000;
  private isProcessing = false;

  constructor(workingDir: string) {
    super();
    this.workingDir = workingDir;
    this.messagesDir = path.join(workingDir, 'messages');
    this.messageQueue = new PriorityQueue<AgentMessage>((a, b) => {
      // Priority order: CRITICAL > HIGH > MEDIUM > LOW
      const priorityValues = { 'CRITICAL': 3, 'HIGH': 2, 'MEDIUM': 1, 'LOW': 0 };
      return priorityValues[b.priority] - priorityValues[a.priority];
    });
  }

  /**
   * Initialize the message broker
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('MessageBroker already initialized');
      return;
    }

    try {
      // Ensure messages directory exists
      await fs.mkdir(this.messagesDir, { recursive: true });

      // Load message history from disk
      await this.loadMessageHistory();

      // Set configuration
      this.maxHistoryPerAgent = config.messaging?.maxHistoryPerAgent || 1000;

      this.initialized = true;
      logger.info('MessageBroker initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize MessageBroker:', error);
      throw error;
    }
  }

  /**
   * Start the message broker
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('MessageBroker not initialized');
    }

    // Start message processing loop
    this.processingInterval = setInterval(() => this.processMessages(), 100);

    // Start persistence loop
    this.persistenceInterval = setInterval(() => this.saveMessageHistory(), 60000); // Save every minute

    logger.info('MessageBroker started');
    this.emit('started');
  }

  /**
   * Shutdown the message broker
   */
  async shutdown(): Promise<void> {
    // Stop processing loops
    if (this.processingInterval) {
      clearInterval(this.processingInterval);
      this.processingInterval = null;
    }

    if (this.persistenceInterval) {
      clearInterval(this.persistenceInterval);
      this.persistenceInterval = null;
    }

    // Save final state
    await this.saveMessageHistory();

    logger.info('MessageBroker shutdown');
    this.emit('shutdown');
  }

  /**
   * Send a message from one agent to another
   */
  async sendMessage(message: AgentMessage): Promise<void> {
    if (!this.initialized) {
      throw new Error('MessageBroker not initialized');
    }

    // Ensure message has required fields
    if (!message.id) {
      message.id = `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    if (!message.timestamp) {
      message.timestamp = new Date();
    }

    // Add to queue
    this.messageQueue.enqueue(message);

    // Store in history
    this.addToHistory(message);

    // Emit event
    this.emit('message-sent', message);

    logger.info(`Message sent from ${message.from} to ${message.to}: ${message.type}`);
  }

  /**
   * Broadcast a message to all agents
   */
  async broadcastMessage(message: AgentMessage): Promise<void> {
    const broadcastMessage = { ...message, to: 'broadcast' as const };
    await this.sendMessage(broadcastMessage);
  }

  /**
   * Subscribe an agent to receive messages of specific types
   */
  async subscribeAgent(agentId: string, subscription: AgentSubscription): Promise<void> {
    if (!this.subscribers.has(agentId)) {
      this.subscribers.set(agentId, new Set());
    }

    this.subscribers.get(agentId)!.add(subscription);
    logger.info(`Agent ${agentId} subscribed to ${subscription.messageType} messages`);
    this.emit('agent-subscribed', agentId, subscription);
  }

  /**
   * Unsubscribe an agent from specific message types
   */
  async unsubscribeAgent(agentId: string, messageType?: MessageType): Promise<void> {
    if (!this.subscribers.has(agentId)) {
      return;
    }

    if (messageType) {
      // Remove specific subscription
      const subscriptions = this.subscribers.get(agentId)!;
      for (const subscription of Array.from(subscriptions)) {
        if (subscription.messageType === messageType) {
          subscriptions.delete(subscription);
          break;
        }
      }
    } else {
      // Remove all subscriptions
      this.subscribers.delete(agentId);
    }

    logger.info(`Agent ${agentId} unsubscribed from ${messageType || 'all'} messages`);
    this.emit('agent-unsubscribed', agentId, messageType);
  }

  /**
   * Get message history for an agent
   */
  async getMessageHistory(agentId: string, limit = 50): Promise<AgentMessage[]> {
    const history = this.messageHistory.get(agentId) || [];
    return history.slice(-limit);
  }

  /**
   * Get all pending messages for an agent
   */
  async getAgentMessages(agentId: string): Promise<AgentMessage[]> {
    // Get all messages in queue that are addressed to this agent
    return this.messageQueue.toArray().filter(msg => 
      msg.to === agentId || msg.to === 'broadcast'
    );
  }

  /**
   * Process messages in the queue
   */
  private async processMessages(): Promise<void> {
    if (this.isProcessing || this.messageQueue.isEmpty()) {
      return;
    }

    this.isProcessing = true;

    try {
      // Process up to 10 messages at a time
      for (let i = 0; i < 10 && !this.messageQueue.isEmpty(); i++) {
        const message = this.messageQueue.dequeue();
        if (!message) continue;

        // Route message to appropriate subscribers
        await this.routeMessage(message);
      }
    } catch (error) {
      logger.error('Error processing messages:', error);
    } finally {
      this.isProcessing = false;
    }
  }

  /**
   * Route a message to appropriate subscribers
   */
  private async routeMessage(message: AgentMessage): Promise<void> {
    // Direct message to specific agent
    if (message.to !== 'broadcast') {
      await this.routeDirectMessage(message);
      return;
    }

    // Broadcast message to all subscribers
    for (const [agentId, subscriptions] of Array.from(this.subscribers.entries())) {
      // Skip sender
      if (agentId === message.from) continue;

      // Check if agent is subscribed to this message type
      let shouldDeliver = false;
      for (const subscription of Array.from(subscriptions)) {
        if (subscription.messageType === message.type || subscription.messageType === 'all') {
          shouldDeliver = true;
          break;
        }
      }

      if (shouldDeliver) {
        // Create a copy of the message specifically for this agent
        const deliveredMessage: AgentMessage = {
          ...message,
          to: agentId
        };

        this.emit('message-delivered', deliveredMessage);
      }
    }
  }

  /**
   * Add a message to the history for both sender and recipient
   */
  private addToHistory(message: AgentMessage): void {
    // Add to sender's history
    if (!this.messageHistory.has(message.from)) {
      this.messageHistory.set(message.from, []);
    }
    const senderHistory = this.messageHistory.get(message.from)!;
    senderHistory.push(message);

    // Trim if exceeds max history
    if (senderHistory.length > this.maxHistoryPerAgent) {
      this.messageHistory.set(message.from, senderHistory.slice(-this.maxHistoryPerAgent));
    }

    // Add to recipient's history (if not broadcast)
    if (message.to !== 'broadcast') {
      if (!this.messageHistory.has(message.to)) {
        this.messageHistory.set(message.to, []);
      }
      const recipientHistory = this.messageHistory.get(message.to)!;
      recipientHistory.push(message);

      // Trim if exceeds max history
      if (recipientHistory.length > this.maxHistoryPerAgent) {
        this.messageHistory.set(message.to, recipientHistory.slice(-this.maxHistoryPerAgent));
      }
    }
  }

  /**
   * Load message history from disk
   */
  private async loadMessageHistory(): Promise<void> {
    try {
      const historyFile = path.join(this.messagesDir, 'message-history.json');
      const exists = await fs.access(historyFile).then(() => true).catch(() => false);

      if (exists) {
        const data = await fs.readFile(historyFile, 'utf-8');
        const saved = JSON.parse(data);

        for (const [agentId, messages] of Object.entries(saved.history || {})) {
          this.messageHistory.set(agentId, (messages as any[]).map(msg => ({
            ...msg,
            timestamp: new Date(msg.timestamp)
          })));
        }

        logger.info(`Loaded message history for ${this.messageHistory.size} agents`);
      }
    } catch (error) {
      logger.warn('Failed to load message history:', error);
    }
  }

  /**
   * Save message history to disk
   */
  private async saveMessageHistory(): Promise<void> {
    try {
      const historyFile = path.join(this.messagesDir, 'message-history.json');
      const data = {
        lastUpdated: new Date().toISOString(),
        history: Object.fromEntries(this.messageHistory)
      };

      await fs.writeFile(historyFile, JSON.stringify(data, null, 2));
      logger.debug('Message history saved to disk');
    } catch (error) {
      logger.error('Failed to save message history:', error);
    }
  }

  /**
   * Set reference to agent manager for enhanced routing
   */
  setAgentManager(agentManager: any): void {
    // This allows the message broker to directly communicate with agents
    // through the agent manager when needed
    this.agentManager = agentManager;
  }

  private agentManager: any = null;

  /**
   * Route message directly through agent manager if available
   */
  private async routeDirectMessage(message: AgentMessage): Promise<void> {
    if (this.agentManager && this.agentManager.sendInterAgentMessage) {
      try {
        await this.agentManager.sendInterAgentMessage(message);
        logger.debug(`Message routed through agent manager: ${message.from} -> ${message.to}`);
      } catch (error) {
        logger.error('Failed to route message through agent manager:', error);
        // Fall back to regular event emission
        this.emit('message-delivered', message);
      }
    } else {
      // Regular event-based delivery
      this.emit('message-delivered', message);
    }
  }

  /**
   * Get statistics about the message broker
   */
  getStats(): {
    queueSize: number;
    subscriberCount: number;
    agentsWithHistory: number;
    totalHistoryMessages: number;
  } {
    const totalHistoryMessages = Array.from(this.messageHistory.values())
      .reduce((sum, messages) => sum + messages.length, 0);

    return {
      queueSize: this.messageQueue.size(),
      subscriberCount: this.subscribers.size,
      agentsWithHistory: this.messageHistory.size,
      totalHistoryMessages
    };
  }
}
