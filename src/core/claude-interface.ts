import { spawn, ChildProcess } from 'child_process';
import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { logger } from '../utils/logger';
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

/**
 * Claude CLI Interface - Manages interaction with Claude CLI processes
 * Handles process spawning, session management, and conversation persistence
 */
export class ClaudeInterface extends EventEmitter {
  private process: ChildProcess | null = null;
  private session: ClaudeSession | null = null;
  private conversationHistory: ClaudeMessage[] = [];
  private isInitialized = false;
  private outputBuffer = '';
  private responseCallback: ((response: ClaudeResponse) => void) | null = null;

  constructor(
    private agentConfig: AgentConfig,
    private workingDir: string
  ) {
    super();
  }

  /**
   * Initialize and spawn Claude CLI process for this agent
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      logger.warn(`Claude interface for agent ${this.agentConfig.id} already initialized`);
      return;
    }

    try {
      // Create session
      this.session = {
        id: `session_${this.agentConfig.id}_${Date.now()}`,
        agentId: this.agentConfig.id,
        startTime: new Date(),
        lastActivity: new Date(),
        conversationPath: path.join(this.workingDir, 'conversations', `${this.agentConfig.id}.json`),
        status: 'starting'
      };

      // Ensure conversation directory exists
      await fs.mkdir(path.dirname(this.session.conversationPath), { recursive: true });

      // Load existing conversation if available
      await this.loadConversationHistory();

      // Spawn Claude CLI process
      await this.spawnClaudeProcess();

      this.isInitialized = true;
      this.session.status = 'ready';
      
      logger.info(`Claude interface initialized for agent: ${this.agentConfig.id}`);
      this.emit('initialized', this.session);

    } catch (error) {
      const errorMsg = `Failed to initialize Claude interface for agent ${this.agentConfig.id}: ${error}`;
      logger.error(errorMsg);
      if (this.session) {
        this.session.status = 'error';
      }
      throw new Error(errorMsg);
    }
  }

  /**
   * Spawn the Claude CLI process with proper configuration
   */
  private async spawnClaudeProcess(): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        // Check if claude command is available
        const claudeCommand = process.env.CLAUDE_CLI_PATH || 'claude';
        
        // Prepare Claude CLI arguments
        const args = [
          '--interactive',
          '--format', 'text',
          '--session', this.session!.id
        ];

        // Add system prompt if specified
        if (this.agentConfig.systemPrompt) {
          args.push('--system', this.agentConfig.systemPrompt);
        }

        logger.info(`Spawning Claude CLI: ${claudeCommand} ${args.join(' ')}`);

        // Spawn the process
        this.process = spawn(claudeCommand, args, {
          stdio: ['pipe', 'pipe', 'pipe'],
          env: {
            ...process.env,
            ANTHROPIC_API_KEY: process.env.ANTHROPIC_API_KEY,
            CLAUDE_SESSION_DIR: path.join(this.workingDir, 'claude-sessions')
          },
          cwd: this.workingDir
        });

        // Handle process events
        this.process.on('spawn', () => {
          logger.info(`Claude CLI process spawned for agent: ${this.agentConfig.id}`);
          resolve();
        });

        this.process.on('error', (error) => {
          logger.error(`Claude CLI process error for agent ${this.agentConfig.id}:`, error);
          this.session!.status = 'error';
          this.emit('error', error);
          reject(error);
        });

        this.process.on('exit', (code, signal) => {
          logger.warn(`Claude CLI process exited for agent ${this.agentConfig.id}. Code: ${code}, Signal: ${signal}`);
          this.session!.status = 'stopped';
          this.emit('process-exit', { code, signal });
        });

        // Handle stdout data
        this.process.stdout?.on('data', (data: Buffer) => {
          this.handleProcessOutput(data.toString());
        });

        // Handle stderr data
        this.process.stderr?.on('data', (data: Buffer) => {
          logger.warn(`Claude CLI stderr for agent ${this.agentConfig.id}: ${data.toString()}`);
        });

        // Set timeout for process spawn
        setTimeout(() => {
          if (this.process && !this.process.pid) {
            reject(new Error('Claude CLI process spawn timeout'));
          }
        }, 10000);

      } catch (error) {
        reject(error);
      }
    });
  }

  /**
   * Handle output from Claude CLI process
   */
  private handleProcessOutput(data: string): void {
    this.outputBuffer += data;
    
    // Look for complete responses (assuming Claude CLI outputs responses with specific markers)
    // This is a simplified implementation - real Claude CLI might have different output format
    const lines = this.outputBuffer.split('\n');
    
    for (let i = 0; i < lines.length - 1; i++) {
      const line = lines[i].trim();
      if (line) {
        this.processResponseLine(line);
      }
    }
    
    // Keep the last incomplete line in buffer
    this.outputBuffer = lines[lines.length - 1];
  }

  /**
   * Process a complete response line from Claude
   */
  private processResponseLine(line: string): void {
    try {
      // Update session activity
      if (this.session) {
        this.session.lastActivity = new Date();
      }

      // Create response message
      const message: ClaudeMessage = {
        id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
        timestamp: new Date(),
        role: 'assistant',
        content: line
      };

      this.conversationHistory.push(message);
      
      // Save conversation
      this.saveConversationHistory().catch(error => {
        logger.error(`Failed to save conversation for agent ${this.agentConfig.id}:`, error);
      });

      // Emit response
      const response: ClaudeResponse = {
        success: true,
        content: line,
        sessionId: this.session!.id,
        timestamp: new Date()
      };

      this.emit('response', response);
      
      // Call response callback if set
      if (this.responseCallback) {
        this.responseCallback(response);
        this.responseCallback = null;
      }

    } catch (error) {
      logger.error(`Error processing Claude response for agent ${this.agentConfig.id}:`, error);
    }
  }

  /**
   * Send a message to Claude and optionally wait for response
   */
  async sendMessage(content: string, waitForResponse = true): Promise<ClaudeResponse> {
    if (!this.isInitialized || !this.process || !this.session) {
      throw new Error(`Claude interface not initialized for agent: ${this.agentConfig.id}`);
    }

    if (this.session.status !== 'ready') {
      throw new Error(`Claude interface not ready for agent: ${this.agentConfig.id}. Status: ${this.session.status}`);
    }

    return new Promise((resolve, reject) => {
      try {
        // Update status
        this.session!.status = 'busy';
        this.session!.lastActivity = new Date();

        // Create user message
        const userMessage: ClaudeMessage = {
          id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
          timestamp: new Date(),
          role: 'user',
          content
        };

        this.conversationHistory.push(userMessage);

        if (waitForResponse) {
          // Set up response callback
          this.responseCallback = (response: ClaudeResponse) => {
            this.session!.status = 'ready';
            resolve(response);
          };

          // Set timeout for response
          setTimeout(() => {
            if (this.responseCallback) {
              this.responseCallback = null;
              this.session!.status = 'ready';
              reject(new Error(`Claude response timeout for agent: ${this.agentConfig.id}`));
            }
          }, 30000); // 30 second timeout
        }

        // Send message to Claude CLI
        this.process!.stdin?.write(content + '\n');

        if (!waitForResponse) {
          this.session!.status = 'ready';
          resolve({
            success: true,
            sessionId: this.session!.id,
            timestamp: new Date()
          });
        }

        // Save conversation
        this.saveConversationHistory().catch(error => {
          logger.error(`Failed to save conversation for agent ${this.agentConfig.id}:`, error);
        });

      } catch (error) {
        this.session!.status = 'ready';
        reject(error);
      }
    });
  }

  /**
   * Load conversation history from file
   */
  private async loadConversationHistory(): Promise<void> {
    try {
      if (!this.session) return;

      const exists = await fs.access(this.session.conversationPath).then(() => true).catch(() => false);
      if (exists) {
        const data = await fs.readFile(this.session.conversationPath, 'utf-8');
        const saved = JSON.parse(data);
        this.conversationHistory = saved.messages || [];
        logger.info(`Loaded ${this.conversationHistory.length} messages for agent: ${this.agentConfig.id}`);
      }
    } catch (error) {
      logger.warn(`Failed to load conversation history for agent ${this.agentConfig.id}:`, error);
      this.conversationHistory = [];
    }
  }

  /**
   * Save conversation history to file
   */
  private async saveConversationHistory(): Promise<void> {
    try {
      if (!this.session) return;

      const data = {
        sessionId: this.session.id,
        agentId: this.agentConfig.id,
        lastUpdated: new Date().toISOString(),
        messages: this.conversationHistory
      };

      await fs.writeFile(this.session.conversationPath, JSON.stringify(data, null, 2));
    } catch (error) {
      logger.error(`Failed to save conversation history for agent ${this.agentConfig.id}:`, error);
    }
  }

  /**
   * Get current session information
   */
  getSession(): ClaudeSession | null {
    return this.session;
  }

  /**
   * Get conversation history
   */
  getConversationHistory(): ClaudeMessage[] {
    return [...this.conversationHistory];
  }

  /**
   * Get health status
   */
  getHealthStatus(): { healthy: boolean; status: string; lastActivity?: Date } {
    if (!this.session) {
      return { healthy: false, status: 'not_initialized' };
    }

    const isHealthy = this.session.status === 'ready' || this.session.status === 'busy';
    return {
      healthy: isHealthy,
      status: this.session.status,
      lastActivity: this.session.lastActivity
    };
  }

  /**
   * Stop the Claude CLI process and cleanup
   */
  async stop(): Promise<void> {
    try {
      if (this.session) {
        this.session.status = 'stopped';
      }

      if (this.process) {
        // Send termination signal
        this.process.kill('SIGTERM');
        
        // Wait for graceful shutdown or force kill after timeout
        setTimeout(() => {
          if (this.process && !this.process.killed) {
            logger.warn(`Force killing Claude CLI process for agent: ${this.agentConfig.id}`);
            this.process.kill('SIGKILL');
          }
        }, 5000);
      }

      // Save final conversation state
      await this.saveConversationHistory();

      this.isInitialized = false;
      logger.info(`Claude interface stopped for agent: ${this.agentConfig.id}`);
      this.emit('stopped');

    } catch (error) {
      logger.error(`Error stopping Claude interface for agent ${this.agentConfig.id}:`, error);
      throw error;
    }
  }

  /**
   * Restart the Claude CLI process
   */
  async restart(): Promise<void> {
    logger.info(`Restarting Claude interface for agent: ${this.agentConfig.id}`);
    
    await this.stop();
    
    // Wait a moment before restart
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    await this.initialize();
  }
}