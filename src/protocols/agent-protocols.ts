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
      metadata: {
        taskId: message.metadata?.taskId,
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
    return message.type === 'info_request';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing information request from ${message.from} to ${message.to}`);
    
    // Create response message
    const response: AgentMessage = {
      id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      from: message.to,
      to: message.from,
      type: 'info_response',
      content: `Information request received: ${message.content}`,
      priority: message.priority,
      timestamp: new Date(),
      metadata: {
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
    return message.type === 'collaboration_request' || message.type === 'collaboration_update';
  }

  async handle(message: AgentMessage): Promise<AgentMessage | null> {
    logger.info(`Processing collaboration message from ${message.from} to ${message.to}`);
    
    if (message.type === 'collaboration_request') {
      // Handle initial collaboration request
      const response: AgentMessage = {
        id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        from: message.to,
        to: message.from,
        type: 'collaboration_response',
        content: `Collaboration request accepted: ${message.content}`,
        priority: message.priority,
        timestamp: new Date(),
        metadata: {
          collaborationId: `collab_${Date.now()}`,
          replyTo: message.id
        }
      };
      
      return response;
    } else if (message.type === 'collaboration_update') {
      // Handle collaboration update
      const response: AgentMessage = {
        id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        from: message.to,
        to: message.from,
        type: 'collaboration_update',
        content: `Update acknowledged: ${message.content}`,
        priority: message.priority,
        timestamp: new Date(),
        metadata: {
          collaborationId: message.metadata?.collaborationId,
          replyTo: message.id
        }
      };
      
      return response;
    }
    
    return null;
  }
}

/**
 * Status Update Protocol - Handles status updates between agents
 */
export class StatusUpdateProtocol extends ProtocolHandler {
  canHandle(message: AgentMessage): boolean {
    return message.type === 'status_update';
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
      metadata
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
      'info_request',
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
      'collaboration_request',
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
      'status_update',
      status,
      priority,
      { details }
    );
  }
}

// Export singleton instance
export const protocolRegistry = new ProtocolRegistry();