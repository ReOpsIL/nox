import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { ClaudeInterface, ClaudeSession, ClaudeMessage } from './claude-interface';
import { AgentConfig } from '../types';
import { logger } from '../utils/logger';

export interface SessionInfo {
  sessionId: string;
  agentId: string;
  status: string;
  startTime: Date;
  lastActivity: Date;
  messageCount: number;
  conversationPath: string;
}

export interface SessionStats {
  totalSessions: number;
  activeSessions: number;
  totalMessages: number;
  oldestSession?: Date;
  newestSession?: Date;
}

/**
 * Session Manager - Manages Claude CLI sessions for all agents
 * Handles session persistence, recovery, and lifecycle management
 */
export class SessionManager extends EventEmitter {
  private sessions = new Map<string, ClaudeInterface>();
  private sessionMetadata = new Map<string, SessionInfo>();
  private initialized = false;
  private workingDir: string;
  private sessionsDir: string;

  constructor(workingDir: string) {
    super();
    this.workingDir = workingDir;
    this.sessionsDir = path.join(workingDir, 'claude-sessions');
  }

  /**
   * Initialize the session manager
   */
  async initialize(): Promise<void> {
    if (this.initialized) {
      logger.warn('SessionManager already initialized');
      return;
    }

    try {
      // Ensure sessions directory exists
      await fs.mkdir(this.sessionsDir, { recursive: true });
      await fs.mkdir(path.join(this.workingDir, 'conversations'), { recursive: true });

      // Load existing session metadata
      await this.loadSessionMetadata();

      this.initialized = true;
      logger.info(`SessionManager initialized. Working directory: ${this.workingDir}`);
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize SessionManager:', error);
      throw error;
    }
  }

  /**
   * Create a new session for an agent
   */
  async createSession(agentConfig: AgentConfig): Promise<ClaudeInterface> {
    if (!this.initialized) {
      throw new Error('SessionManager not initialized');
    }

    if (this.sessions.has(agentConfig.id)) {
      throw new Error(`Session already exists for agent: ${agentConfig.id}`);
    }

    try {
      // Create Claude interface
      const claudeInterface = new ClaudeInterface(agentConfig, this.workingDir);

      // Set up event listeners
      this.setupSessionEventListeners(claudeInterface, agentConfig.id);

      // Initialize the Claude interface
      await claudeInterface.initialize();

      // Store session
      this.sessions.set(agentConfig.id, claudeInterface);

      // Create session metadata
      const session = claudeInterface.getSession();
      if (session) {
        const sessionInfo: SessionInfo = {
          sessionId: session.id,
          agentId: agentConfig.id,
          status: session.status,
          startTime: session.startTime,
          lastActivity: session.lastActivity,
          messageCount: claudeInterface.getConversationHistory().length,
          conversationPath: session.conversationPath
        };

        this.sessionMetadata.set(agentConfig.id, sessionInfo);
        await this.saveSessionMetadata();
      }

      logger.info(`Session created for agent: ${agentConfig.id}`);
      this.emit('session-created', agentConfig.id);

      return claudeInterface;

    } catch (error) {
      logger.error(`Failed to create session for agent ${agentConfig.id}:`, error);
      throw error;
    }
  }

  /**
   * Get an existing session for an agent
   */
  getSession(agentId: string): ClaudeInterface | null {
    return this.sessions.get(agentId) || null;
  }

  /**
   * Get session metadata for an agent
   */
  getSessionInfo(agentId: string): SessionInfo | null {
    return this.sessionMetadata.get(agentId) || null;
  }

  /**
   * Get all active sessions
   */
  getActiveSessions(): Map<string, ClaudeInterface> {
    return new Map(this.sessions);
  }

  /**
   * Get all session metadata
   */
  getAllSessionInfo(): SessionInfo[] {
    return Array.from(this.sessionMetadata.values());
  }

  /**
   * Stop a session for an agent
   */
  async stopSession(agentId: string): Promise<void> {
    const session = this.sessions.get(agentId);
    if (!session) {
      logger.warn(`No session found for agent: ${agentId}`);
      return;
    }

    try {
      await session.stop();
      this.sessions.delete(agentId);
      
      // Update metadata
      const metadata = this.sessionMetadata.get(agentId);
      if (metadata) {
        metadata.status = 'stopped';
        await this.saveSessionMetadata();
      }

      logger.info(`Session stopped for agent: ${agentId}`);
      this.emit('session-stopped', agentId);

    } catch (error) {
      logger.error(`Failed to stop session for agent ${agentId}:`, error);
      throw error;
    }
  }

  /**
   * Restart a session for an agent
   */
  async restartSession(agentId: string, agentConfig: AgentConfig): Promise<ClaudeInterface> {
    logger.info(`Restarting session for agent: ${agentId}`);

    // Stop existing session if it exists
    await this.stopSession(agentId);

    // Wait a moment
    await new Promise(resolve => setTimeout(resolve, 1000));

    // Create new session
    return await this.createSession(agentConfig);
  }

  /**
   * Stop all sessions
   */
  async stopAllSessions(): Promise<void> {
    const stopPromises = Array.from(this.sessions.keys()).map(agentId => 
      this.stopSession(agentId).catch(error => {
        logger.error(`Failed to stop session for agent ${agentId}:`, error);
      })
    );

    await Promise.all(stopPromises);
    logger.info('All sessions stopped');
    this.emit('all-sessions-stopped');
  }

  /**
   * Get session statistics
   */
  getSessionStats(): SessionStats {
    const allSessions = this.getAllSessionInfo();
    const activeSessions = Array.from(this.sessions.values());
    
    const totalMessages = allSessions.reduce((sum, session) => sum + session.messageCount, 0);
    const startTimes = allSessions.map(s => s.startTime);

    return {
      totalSessions: allSessions.length,
      activeSessions: activeSessions.length,
      totalMessages,
      oldestSession: startTimes.length > 0 ? new Date(Math.min(...startTimes.map(d => d.getTime()))) : undefined,
      newestSession: startTimes.length > 0 ? new Date(Math.max(...startTimes.map(d => d.getTime()))) : undefined
    };
  }

  /**
   * Health check for all sessions
   */
  async performHealthCheck(): Promise<Map<string, boolean>> {
    const healthStatus = new Map<string, boolean>();

    for (const [agentId, session] of this.sessions) {
      try {
        const health = session.getHealthStatus();
        healthStatus.set(agentId, health.healthy);
        
        // Update metadata
        const metadata = this.sessionMetadata.get(agentId);
        if (metadata) {
          metadata.status = health.status;
          if (health.lastActivity) {
            metadata.lastActivity = health.lastActivity;
          }
        }

      } catch (error) {
        logger.error(`Health check failed for agent ${agentId}:`, error);
        healthStatus.set(agentId, false);
      }
    }

    await this.saveSessionMetadata();
    return healthStatus;
  }

  /**
   * Clean up old sessions and conversation files
   */
  async cleanupOldSessions(maxAgeHours = 24 * 7): Promise<number> { // Default: 1 week
    const cutoffTime = new Date(Date.now() - (maxAgeHours * 60 * 60 * 1000));
    let cleanedCount = 0;

    try {
      // Clean up conversation files
      const conversationsDir = path.join(this.workingDir, 'conversations');
      const files = await fs.readdir(conversationsDir);

      for (const file of files) {
        if (!file.endsWith('.json')) continue;

        const filePath = path.join(conversationsDir, file);
        const stats = await fs.stat(filePath);

        if (stats.mtime < cutoffTime) {
          try {
            const content = await fs.readFile(filePath, 'utf-8');
            const data = JSON.parse(content);
            
            // Check if session is still active
            if (!this.sessions.has(data.agentId)) {
              await fs.unlink(filePath);
              cleanedCount++;
              logger.info(`Cleaned up old conversation file: ${file}`);
            }
          } catch (error) {
            logger.warn(`Failed to process conversation file ${file}:`, error);
          }
        }
      }

      // Clean up metadata for non-existent sessions
      for (const [agentId, metadata] of this.sessionMetadata) {
        if (metadata.lastActivity < cutoffTime && !this.sessions.has(agentId)) {
          this.sessionMetadata.delete(agentId);
          cleanedCount++;
        }
      }

      if (cleanedCount > 0) {
        await this.saveSessionMetadata();
        logger.info(`Cleaned up ${cleanedCount} old sessions/files`);
      }

      return cleanedCount;

    } catch (error) {
      logger.error('Failed to cleanup old sessions:', error);
      return 0;
    }
  }

  /**
   * Set up event listeners for a Claude interface
   */
  private setupSessionEventListeners(claudeInterface: ClaudeInterface, agentId: string): void {
    claudeInterface.on('response', (response) => {
      // Update session metadata
      const metadata = this.sessionMetadata.get(agentId);
      if (metadata) {
        metadata.lastActivity = response.timestamp;
        metadata.messageCount = claudeInterface.getConversationHistory().length;
      }
      
      this.emit('session-activity', agentId, response);
    });

    claudeInterface.on('error', (error) => {
      logger.error(`Session error for agent ${agentId}:`, error);
      this.emit('session-error', agentId, error);
    });

    claudeInterface.on('process-exit', (exitInfo) => {
      logger.warn(`Claude process exited for agent ${agentId}:`, exitInfo);
      this.emit('session-process-exit', agentId, exitInfo);
    });

    claudeInterface.on('stopped', () => {
      this.emit('session-stopped', agentId);
    });
  }

  /**
   * Load session metadata from file
   */
  private async loadSessionMetadata(): Promise<void> {
    try {
      const metadataPath = path.join(this.sessionsDir, 'sessions-metadata.json');
      const exists = await fs.access(metadataPath).then(() => true).catch(() => false);
      
      if (exists) {
        const data = await fs.readFile(metadataPath, 'utf-8');
        const saved = JSON.parse(data);
        
        for (const [agentId, info] of Object.entries(saved.sessions || {})) {
          const sessionInfo = info as any;
          this.sessionMetadata.set(agentId, {
            ...sessionInfo,
            startTime: new Date(sessionInfo.startTime),
            lastActivity: new Date(sessionInfo.lastActivity)
          });
        }
        
        logger.info(`Loaded metadata for ${this.sessionMetadata.size} sessions`);
      }
    } catch (error) {
      logger.warn('Failed to load session metadata:', error);
    }
  }

  /**
   * Save session metadata to file
   */
  private async saveSessionMetadata(): Promise<void> {
    try {
      const metadataPath = path.join(this.sessionsDir, 'sessions-metadata.json');
      const data = {
        lastUpdated: new Date().toISOString(),
        sessions: Object.fromEntries(this.sessionMetadata)
      };
      
      await fs.writeFile(metadataPath, JSON.stringify(data, null, 2));
    } catch (error) {
      logger.error('Failed to save session metadata:', error);
    }
  }

  /**
   * Check if session manager is initialized
   */
  isInitialized(): boolean {
    return this.initialized;
  }
}