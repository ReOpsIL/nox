import { ClaudeInterface } from '../../../src/core/claude-interface';
import { AgentConfig } from '../../../src/types/agent';
import * as fs from 'fs/promises';
import * as path from 'path';

// Mock child_process
jest.mock('child_process', () => ({
  spawn: jest.fn()
}));

// Mock fs/promises
jest.mock('fs/promises');

describe('ClaudeInterface', () => {
  let claudeInterface: ClaudeInterface;
  let mockAgentConfig: AgentConfig;
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    
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

    claudeInterface = new ClaudeInterface(mockAgentConfig, tempDir);
  });

  afterEach(async () => {
    try {
      await claudeInterface.stop();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize without starting process in test environment', async () => {
      // Mock the spawn to simulate Claude CLI not being available
      const { spawn } = require('child_process');
      const mockProcess = {
        pid: 12345,
        stdin: { write: jest.fn() },
        stdout: { on: jest.fn() },
        stderr: { on: jest.fn() },
        on: jest.fn((event, callback) => {
          if (event === 'error') {
            // Immediately trigger error for faster test
            callback(new Error('Claude CLI not found'));
          }
        })
      };
      spawn.mockReturnValue(mockProcess);

      // Mock fs operations
      (fs.mkdir as jest.Mock).mockResolvedValue(undefined);
      (fs.access as jest.Mock).mockRejectedValue(new Error('File not found'));

      await expect(claudeInterface.initialize()).rejects.toThrow('Claude CLI not found');
    }, 10000);

    it('should track session state', () => {
      expect(claudeInterface.getSession()).toBeNull();
      expect(claudeInterface.getHealthStatus().healthy).toBe(false);
      expect(claudeInterface.getHealthStatus().status).toBe('not_initialized');
    });

    it('should handle conversation history', () => {
      const history = claudeInterface.getConversationHistory();
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBe(0);
    });
  });

  describe('error handling', () => {
    it('should reject operations when not initialized', async () => {
      await expect(claudeInterface.sendMessage('test')).rejects.toThrow('not initialized');
    });

    it('should handle stop when not initialized', async () => {
      await expect(claudeInterface.stop()).resolves.not.toThrow();
    });

    it('should handle restart when not initialized', async () => {
      await expect(claudeInterface.restart()).rejects.toThrow();
    }, 5000);
  });

  describe('message handling', () => {
    beforeEach(() => {
      // Mock successful initialization
      (claudeInterface as any).session = {
        id: 'test-session',
        agentId: 'test-agent',
        startTime: new Date(),
        lastActivity: new Date(),
        conversationPath: path.join(tempDir, 'test.json'),
        status: 'ready'
      };
      (claudeInterface as any).isInitialized = true;
    });

    it('should handle message processing', () => {
      const session = claudeInterface.getSession();
      expect(session).not.toBeNull();
      expect(session!.id).toBe('test-session');
      expect(session!.status).toBe('ready');
    });

    it('should track health status when initialized', () => {
      const health = claudeInterface.getHealthStatus();
      expect(health.healthy).toBe(true);
      expect(health.status).toBe('ready');
    });
  });

  describe('configuration', () => {
    it('should use provided agent config', () => {
      expect((claudeInterface as any).agentConfig.id).toBe('test-agent');
      expect((claudeInterface as any).agentConfig.systemPrompt).toBe('You are a test assistant');
    });

    it('should use provided working directory', () => {
      expect((claudeInterface as any).workingDir).toBe(tempDir);
    });
  });

  describe('event handling', () => {
    it('should be an event emitter', () => {
      expect(typeof claudeInterface.on).toBe('function');
      expect(typeof claudeInterface.emit).toBe('function');
    });

    it('should handle event listeners', (done) => {
      claudeInterface.on('test-event', (data) => {
        expect(data).toBe('test-data');
        done();
      });

      claudeInterface.emit('test-event', 'test-data');
    });
  });
});