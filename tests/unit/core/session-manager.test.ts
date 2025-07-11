import { SessionManager } from '../../../src/core/session-manager';
import { AgentConfig } from '../../../src/types/agent';
import * as fs from 'fs/promises';

// Mock child_process
jest.mock('child_process', () => ({
  spawn: jest.fn()
}));

// Mock fs/promises
jest.mock('fs/promises');

describe('SessionManager', () => {
  let sessionManager: SessionManager;
  let tempDir: string;
  let mockAgentConfig: AgentConfig;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    sessionManager = new SessionManager(tempDir);

    mockAgentConfig = {
      id: 'test-agent',
      name: 'Test Agent',
      systemPrompt: 'You are a test assistant',
      status: 'inactive',
      createdAt: new Date(),
      lastModified: new Date(),
      resourceLimits: {
        maxMemoryMB: 1024,
        maxCPUPercent: 50,
        maxConcurrentTasks: 5,
        maxSpawnRate: 2,
        maxDepth: 3
      },
      capabilities: [],
      relationships: [],
      mcpServices: []
    };

    // Mock fs operations
    (fs.mkdir as jest.Mock).mockResolvedValue(undefined);
    (fs.access as jest.Mock).mockRejectedValue(new Error('File not found'));
    (fs.readdir as jest.Mock).mockResolvedValue([]);
    (fs.readFile as jest.Mock).mockResolvedValue('{}');
    (fs.writeFile as jest.Mock).mockResolvedValue(undefined);
  });

  afterEach(async () => {
    try {
      await sessionManager.stopAllSessions();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize successfully', async () => {
      expect(sessionManager.isInitialized()).toBe(false);
      
      await sessionManager.initialize();
      
      expect(sessionManager.isInitialized()).toBe(true);
    });

    it('should handle multiple initializations', async () => {
      await sessionManager.initialize();
      expect(sessionManager.isInitialized()).toBe(true);
      
      // Should not throw on second initialization
      await sessionManager.initialize();
      expect(sessionManager.isInitialized()).toBe(true);
    });

    it('should create required directories', async () => {
      await sessionManager.initialize();
      
      expect(fs.mkdir).toHaveBeenCalledWith(
        expect.stringContaining('claude-sessions'),
        { recursive: true }
      );
      expect(fs.mkdir).toHaveBeenCalledWith(
        expect.stringContaining('conversations'),
        { recursive: true }
      );
    });
  });

  describe('session management', () => {
    beforeEach(async () => {
      await sessionManager.initialize();
    });

    it('should track sessions correctly', async () => {
      const activeSessions = sessionManager.getActiveSessions();
      expect(activeSessions.size).toBe(0);
      
      const allSessionInfo = sessionManager.getAllSessionInfo();
      expect(allSessionInfo.length).toBe(0);
    });

    it('should handle session creation failure gracefully', async () => {
      // Mock Claude CLI spawn failure
      const { spawn } = require('child_process');
      spawn.mockReturnValue({
        on: jest.fn((event, callback) => {
          if (event === 'error') {
            setTimeout(() => callback(new Error('Claude CLI not found')), 10);
          }
        })
      });

      await expect(sessionManager.createSession(mockAgentConfig))
        .rejects.toThrow();
    });

    it('should return null for non-existent sessions', () => {
      const session = sessionManager.getSession('non-existent');
      expect(session).toBeNull();
      
      const sessionInfo = sessionManager.getSessionInfo('non-existent');
      expect(sessionInfo).toBeNull();
    });

    it('should handle session stopping gracefully', async () => {
      await expect(sessionManager.stopSession('non-existent'))
        .resolves.not.toThrow();
    });
  });

  describe('session statistics', () => {
    beforeEach(async () => {
      await sessionManager.initialize();
    });

    it('should provide accurate statistics', () => {
      const stats = sessionManager.getSessionStats();
      
      expect(stats.totalSessions).toBe(0);
      expect(stats.activeSessions).toBe(0);
      expect(stats.totalMessages).toBe(0);
      expect(stats.oldestSession).toBeUndefined();
      expect(stats.newestSession).toBeUndefined();
    });
  });

  describe('health checks', () => {
    beforeEach(async () => {
      await sessionManager.initialize();
    });

    it('should perform health checks on empty session list', async () => {
      const healthResults = await sessionManager.performHealthCheck();
      expect(healthResults.size).toBe(0);
    });
  });

  describe('cleanup operations', () => {
    beforeEach(async () => {
      await sessionManager.initialize();
    });

    it('should cleanup old sessions', async () => {
      const cleanedCount = await sessionManager.cleanupOldSessions(1); // 1 hour
      expect(cleanedCount).toBeGreaterThanOrEqual(0);
    });

    it('should handle cleanup errors gracefully', async () => {
      // Mock readdir to throw error
      (fs.readdir as jest.Mock).mockRejectedValueOnce(new Error('Access denied'));
      
      const cleanedCount = await sessionManager.cleanupOldSessions();
      expect(cleanedCount).toBe(0);
    });
  });

  describe('error handling', () => {
    it('should handle initialization errors', async () => {
      // Mock mkdir to fail
      (fs.mkdir as jest.Mock).mockRejectedValueOnce(new Error('Permission denied'));
      
      await expect(sessionManager.initialize()).rejects.toThrow('Permission denied');
    });

    it('should handle operations when not initialized', async () => {
      await expect(sessionManager.createSession(mockAgentConfig))
        .rejects.toThrow('not initialized');
    });
  });

  describe('event handling', () => {
    it('should be an event emitter', () => {
      expect(typeof sessionManager.on).toBe('function');
      expect(typeof sessionManager.emit).toBe('function');
    });

    it('should emit initialization event', async () => {
      const initHandler = jest.fn();
      sessionManager.on('initialized', initHandler);
      
      await sessionManager.initialize();
      
      expect(initHandler).toHaveBeenCalled();
    });
  });
});