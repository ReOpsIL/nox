import { spawn, ChildProcess } from 'child_process';
import * as path from 'path';
import * as fs from 'fs/promises';
import { logger } from '../utils/logger';
import { AgentConfig } from '../types';
import { 
  ClaudeInterfaceBase, 
  ClaudeSession, 
  ClaudeMessage, 
  ClaudeResponse, 
  ClaudeHealthStatus 
} from './claude-interface-base';

/**
 * Claude CLI Interface - Manages interaction with Claude CLI processes
 * Handles process spawning, session management, and conversation persistence
 */
export class ClaudeInterface extends ClaudeInterfaceBase {
  private process: ChildProcess | null = null;
  private session: ClaudeSession | null = null;
  private conversationHistory: ClaudeMessage[] = [];
  private isInitialized = false;
  private outputBuffer = '';
  private responseCallback: ((response: ClaudeResponse) => void) | null = null;

  constructor(agentConfig: AgentConfig, workingDir: string) {
    super(agentConfig, workingDir);
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
        
        // Prepare Claude CLI arguments - Use --print mode for non-interactive usage
        const args: string[] = ['--print'];

        // Add model if specified in agent config
        if (this.agentConfig.model) {
          args.push('--model', this.agentConfig.model);
        }

        logger.info(`Spawning Claude CLI: ${claudeCommand} ${args.join(' ')}`);

        // Spawn the process
        this.process = spawn(claudeCommand, args, {
          stdio: ['pipe', 'pipe', 'pipe'],
          env: {
            ...process.env,
            ANTHROPIC_API_KEY: process.env.ANTHROPIC_API_KEY
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
          const output = data.toString();
          logger.info(`Claude CLI stdout for agent ${this.agentConfig.id}: ${JSON.stringify(output)}`);
          this.handleProcessOutput(output);
        });

        // Handle stderr data
        this.process.stderr?.on('data', (data: Buffer) => {
          const error = data.toString();
          logger.warn(`Claude CLI stderr for agent ${this.agentConfig.id}: ${JSON.stringify(error)}`);
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
    // Clean terminal escape sequences
    const cleanData = data.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '').replace(/\][0-9];[^\x07]*\x07/g, '');
    this.outputBuffer += cleanData;
    
    // Look for complete responses - Claude CLI typically outputs complete responses
    const lines = this.outputBuffer.split('\n');
    
    for (let i = 0; i < lines.length - 1; i++) {
      const line = lines[i]?.trim();
      if (line && line.length > 0) {
        this.processResponseLine(line);
      }
    }
    
    // Keep the last incomplete line in buffer
    this.outputBuffer = lines[lines.length - 1] || '';
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
    if (!this.isInitialized || !this.session) {
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

        if (!waitForResponse) {
          this.session!.status = 'ready';
          resolve({
            success: true,
            sessionId: this.session!.id,
            timestamp: new Date()
          });
          return;
        }

        // Spawn Claude CLI with --print mode for this message
        const claudeCommand = process.env.CLAUDE_CLI_PATH || 'claude';
        const args = ['--print'];
        
        // Add model if specified
        if (this.agentConfig.model) {
          args.push('--model', this.agentConfig.model);
        }

        // Include system prompt context if this is the first message
        let messageContent = content;
        if (this.conversationHistory.length === 1 && this.agentConfig.systemPrompt) {
          messageContent = `${this.agentConfig.systemPrompt}\n\nUser: ${content}`;
        }

        const { spawn } = require('child_process');
        const claudeProcess = spawn(claudeCommand, args, {
          stdio: ['pipe', 'pipe', 'pipe'],
          env: {
            ...process.env,
            ANTHROPIC_API_KEY: process.env.ANTHROPIC_API_KEY
          },
          cwd: this.workingDir
        });

        let output = '';
        let errorOutput = '';

        claudeProcess.stdout.on('data', (data: Buffer) => {
          output += data.toString();
        });

        claudeProcess.stderr.on('data', (data: Buffer) => {
          errorOutput += data.toString();
        });

        claudeProcess.on('close', (code: number) => {
          this.session!.status = 'ready';
          
          if (code === 0) {
            // Clean output
            const cleanOutput = output.replace(/\x1b\[[0-9;]*[a-zA-Z]/g, '').replace(/\][0-9];[^\x07]*\x07/g, '').trim();
            
            if (cleanOutput) {
              // Create response message
              const responseMessage: ClaudeMessage = {
                id: `msg_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
                timestamp: new Date(),
                role: 'assistant',
                content: cleanOutput
              };

              this.conversationHistory.push(responseMessage);
              
              const response: ClaudeResponse = {
                success: true,
                content: cleanOutput,
                sessionId: this.session!.id,
                timestamp: new Date()
              };

              this.emit('response', response);
              resolve(response);
            } else {
              reject(new Error(`Empty response from Claude CLI for agent: ${this.agentConfig.id}`));
            }
          } else {
            reject(new Error(`Claude CLI error for agent ${this.agentConfig.id}: ${errorOutput || 'Unknown error'}`));
          }
        });

        claudeProcess.on('error', (error: Error) => {
          this.session!.status = 'ready';
          reject(new Error(`Failed to spawn Claude CLI for agent ${this.agentConfig.id}: ${error.message}`));
        });

        // Send the message
        claudeProcess.stdin.write(messageContent);
        claudeProcess.stdin.end();

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
  getHealthStatus(): ClaudeHealthStatus {
    if (!this.session) {
      return { 
        healthy: false, 
        status: 'not_initialized',
        lastActivity: new Date(),
        messageCount: 0
      };
    }

    const isHealthy = this.session.status === 'ready' || this.session.status === 'busy';
    return {
      healthy: isHealthy,
      status: this.session.status,
      lastActivity: this.session.lastActivity,
      messageCount: this.conversationHistory.length
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