import { EventEmitter } from 'events';
import { AgentConfig } from '../types';

export interface ClaudeSession {
  id: string;
  agentId: string;
  startTime: Date;
  lastActivity: Date;
  conversationPath: string;
  status: 'starting' | 'ready' | 'busy' | 'error' | 'stopped';
}

export interface ClaudeMessage {
  id: string;
  timestamp: Date;
  role: 'user' | 'assistant';
  content: string;
  metadata?: Record<string, any>;
}

export interface ClaudeResponse {
  success: boolean;
  content?: string;
  error?: string;
  sessionId: string;
  timestamp: Date;
}

export interface ClaudeHealthStatus {
  status: string;
  lastActivity: Date;
  messageCount: number;
  healthy?: boolean;
}

/**
 * Base interface for Claude CLI implementations
 */
export abstract class ClaudeInterfaceBase extends EventEmitter {
  protected agentConfig: AgentConfig;
  protected workingDir: string;

  constructor(agentConfig: AgentConfig, workingDir: string) {
    super();
    this.agentConfig = agentConfig;
    this.workingDir = workingDir;
  }

  /**
   * Initialize the Claude interface
   */
  abstract initialize(): Promise<void>;

  /**
   * Send a message to Claude and get response
   */
  abstract sendMessage(message: string): Promise<ClaudeResponse>;

  /**
   * Get session information
   */
  abstract getSession(): ClaudeSession | null;

  /**
   * Get conversation history
   */
  abstract getConversationHistory(): ClaudeMessage[];

  /**
   * Get health status
   */
  abstract getHealthStatus(): ClaudeHealthStatus;

  /**
   * Stop/shutdown the interface
   */
  abstract stop(): Promise<void>;

  /**
   * Shutdown the interface (alias for stop)
   */
  async shutdown(): Promise<void> {
    return this.stop();
  }
}