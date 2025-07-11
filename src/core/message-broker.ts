import { EventEmitter } from 'events';
import { AgentMessage, NoxConfig } from '../types';
import { logger } from '../utils/logger';

export class MessageBroker extends EventEmitter {
  private initialized = false;
  private messageQueue: AgentMessage[] = [];

  constructor() {
    super();
  }

  async initialize(_config: NoxConfig): Promise<void> {
    this.initialized = true;
    logger.info('MessageBroker initialized');
  }

  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('MessageBroker not initialized');
    }
    logger.info('MessageBroker started');
  }

  async shutdown(): Promise<void> {
    logger.info('MessageBroker shutdown');
  }

  async sendMessage(message: AgentMessage): Promise<void> {
    if (!this.initialized) {
      throw new Error('MessageBroker not initialized');
    }

    this.messageQueue.push(message);
    this.emit('message-sent', message);
    
    logger.info(`Message sent from ${message.from} to ${message.to}: ${message.type}`);
  }

  async broadcastMessage(message: AgentMessage): Promise<void> {
    const broadcastMessage = { ...message, to: 'broadcast' as const };
    await this.sendMessage(broadcastMessage);
  }

  async getMessageHistory(agentId: string, limit = 50): Promise<AgentMessage[]> {
    return this.messageQueue
      .filter(msg => msg.from === agentId || msg.to === agentId)
      .slice(-limit);
  }
}