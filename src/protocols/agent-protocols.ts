/**
 * Agent Protocols - Defines standardized communication protocols between agents
 * Provides structured message formats and protocol handlers
 */

import { AgentMessage, MessageType, Priority } from '../types';
import { logger } from '../utils/logger';

/**
 * Protocol Handler - Base class for all protocol handlers
 */
export abstract class ProtocolHandler {
  abstract canHandle(message: AgentMessage): boolean;
  abstract handle(message: AgentMessage): Promise<AgentMessage | null>;
}

/**
 * Task Request Protocol - Handles task delegation between agents
 */
export class TaskRequestProtocol extends ProtocolHandler {
  canHandle(message: AgentMessage): boolean {
    return message.type === 'task_request';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing task request from ${message.from} to ${message.to}`);
    
    // Create response message
    const response: AgentMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      from: message.to,
      to: message.from,
      type: 'task_response',
      content: `Task request received: ${message.content}`,
      priority: message.priority,
      timestamp: new Date(),
      requiresApproval: false,
      ...(metadata && { metadata }): {
        ...(message.metadata?.taskId && { taskId: message.metadata.taskId }),
        replyTo: message.id
      }
    };
    
    return response;
  }
}

/**
 * Information Request Protocol - Handles information exchange between agents
 */
export class InformationRequestProtocol extends ProtocolHandler {
  canHandle(message: AgentMessage): boolean {
    return message.type === 'capability_query';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing information request from ${message.from} to ${message.to}`);
    
    // Create response message
    const response: AgentMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      from: message.to,
      to: message.from,
      type: 'direct',
      content: `Information request received: ${message.content}`,
      priority: message.priority,
      timestamp: new Date(),
      requiresApproval: false,
      ...(metadata && { metadata }): {
        replyTo: message.id
      }
    };
    
    return response;
  }
}

/**
 * Collaboration Protocol - Handles collaborative work between agents
 */
export class CollaborationProtocol extends ProtocolHandler {
  canHandle(message: AgentMessage): boolean {
    return message.type === 'direct';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing collaboration message from ${message.from} to ${message.to}`);
    
    // Handle collaboration message
    const response: AgentMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      from: message.to,
      to: message.from,
      type: 'direct',
      content: `Collaboration response: ${message.content}`,
      priority: message.priority,
      timestamp: new Date(),
      requiresApproval: false,
      ...(metadata && { metadata }): {
        replyTo: message.id
      }
    };
    
    return response;
  }
}

/**
 * Status Update Protocol - Handles status updates between agents
 */
export class StatusUpdateProtocol extends ProtocolHandler {
  canHandle(message: AgentMessage): boolean {
    return message.type === 'system';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing status update from ${message.from} to ${message.to}`);
    
    // Status updates don't require a response
    return null;
  }
}

/**
 * Protocol Registry - Manages all available protocols
 */
export class ProtocolRegistry {
  private protocols: ProtocolHandler[] = [];

  constructor() {
    // Register default protocols
    this.registerProtocol(new TaskRequestProtocol());
    this.registerProtocol(new InformationRequestProtocol());
    this.registerProtocol(new CollaborationProtocol());
    this.registerProtocol(new StatusUpdateProtocol());
  }

  /**
   * Register a new protocol handler
   */
  registerProtocol(protocol: ProtocolHandler): void {
    this.protocols.push(protocol);
    logger.info(`Registered protocol handler: ${protocol.constructor.name}`);
  }

  /**
   * Find a protocol handler for a message
   */
  findProtocolHandler(message: AgentMessage): ProtocolHandler | null {
    for (const protocol of this.protocols) {
      if (protocol.canHandle(message)) {
        return protocol;
      }
    }
    return null;
  }

  /**
   * Process a message using the appropriate protocol
   */
  async processMessage(message: AgentMessage): Promise<AgentMessage | null> {
    const handler = this.findProtocolHandler(message);
    if (!handler) {
      logger.warn(`No protocol handler found for message type: ${message.type}`);
      return null;
    }
    
    try {
      return await handler.handle(message);
    } catch (error) {
      logger.error(`Error processing message with protocol ${handler.constructor.name}:`, error);
      return null;
    }
  }

  /**
   * Create a standard message using the appropriate protocol
   */
  createMessage(
    from: string,
    to: string,
    type: MessageType,
    content: string,
    priority: Priority = 'MEDIUM',
    metadata?: Record<string, any>
  ): AgentMessage {
    return {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      from,
      to,
      type,
      content,
      priority,
      timestamp: new Date(),
      requiresApproval: false,
      ...(metadata && { metadata })
    };
  }

  /**
   * Create a task request message
   */
  createTaskRequest(
    from: string,
    to: string,
    taskTitle: string,
    taskDescription: string,
    priority: Priority = 'MEDIUM',
    deadline?: Date
  ): AgentMessage {
    return this.createMessage(
      from,
      to,
      'task_request',
      taskTitle,
      priority,
      {
        taskDescription,
        deadline: deadline?.toISOString()
      }
    );
  }

  /**
   * Create an information request message
   */
  createInfoRequest(
    from: string,
    to: string,
    query: string,
    priority: Priority = 'MEDIUM'
  ): AgentMessage {
    return this.createMessage(
      from,
      to,
      'capability_query',
      query,
      priority
    );
  }

  /**
   * Create a collaboration request message
   */
  createCollaborationRequest(
    from: string,
    to: string,
    topic: string,
    details: string,
    priority: Priority = 'MEDIUM'
  ): AgentMessage {
    return this.createMessage(
      from,
      to,
      'direct',
      topic,
      priority,
      { details }
    );
  }

  /**
   * Create a status update message
   */
  createStatusUpdate(
    from: string,
    to: string,
    status: string,
    details?: string,
    priority: Priority = 'LOW'
  ): AgentMessage {
    return this.createMessage(
      from,
      to,
      'system',
      status,
      priority,
      { details }
    );
  }
}

// Export singleton instance
export const protocolRegistry = new ProtocolRegistry();