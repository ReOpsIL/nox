import { MessageBroker } from '../../../src/core/message-broker';
import { AgentMessage, NoxConfig, DEFAULT_CONFIG } from '../../../src/types';
import * as fs from 'fs/promises';

// Mock fs/promises
jest.mock('fs/promises');

describe('MessageBroker', () => {
  let messageBroker: MessageBroker;
  let tempDir: string;
  let mockConfig: NoxConfig;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    messageBroker = new MessageBroker(tempDir);

    mockConfig = {
      ...DEFAULT_CONFIG,
      storage: {
        ...DEFAULT_CONFIG.storage,
        registryPath: tempDir
      },
      messaging: {
        maxHistoryPerAgent: 100,
        messageRetentionDays: 30,
        maxQueueSize: 1000,
        processingInterval: 100
      }
    };

    // Mock fs operations
    (fs.mkdir as jest.Mock).mockResolvedValue(undefined);
    (fs.access as jest.Mock).mockRejectedValue(new Error('File not found'));
    (fs.readFile as jest.Mock).mockResolvedValue('{"lastUpdated":"2024-01-01","history":{}}');
    (fs.writeFile as jest.Mock).mockResolvedValue(undefined);
  });

  afterEach(async () => {
    try {
      await messageBroker.shutdown();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize successfully', async () => {
      await messageBroker.initialize(mockConfig);
      expect(fs.mkdir).toHaveBeenCalledWith(expect.stringContaining('messages'), { recursive: true });
    });

    it('should handle multiple initializations', async () => {
      await messageBroker.initialize(mockConfig);
      await messageBroker.initialize(mockConfig);
      // Should not throw on second initialization
    });
  });

  describe('message handling', () => {
    beforeEach(async () => {
      await messageBroker.initialize(mockConfig);
    });

    it('should send message successfully', async () => {
      const message: AgentMessage = {
        id: 'test-msg-1',
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'Test message',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      await expect(messageBroker.sendMessage(message)).resolves.not.toThrow();
    });

    it('should generate message ID if not provided', async () => {
      const message: Partial<AgentMessage> = {
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'Test message',
        priority: 'MEDIUM',
        requiresApproval: false
      };

      await messageBroker.sendMessage(message as AgentMessage);
      expect(message.id).toBeDefined();
      expect(message.timestamp).toBeDefined();
    });

    it('should broadcast messages', async () => {
      const message: AgentMessage = {
        id: 'broadcast-msg-1',
        from: 'agent1',
        to: 'broadcast' as const,
        type: 'system',
        content: 'Broadcast test',
        priority: 'HIGH',
        timestamp: new Date(),
        requiresApproval: false
      };

      await expect(messageBroker.broadcastMessage(message)).resolves.not.toThrow();
    });
  });

  describe('subscription management', () => {
    beforeEach(async () => {
      await messageBroker.initialize(mockConfig);
    });

    it('should subscribe agent to message types', async () => {
      await expect(messageBroker.subscribeAgent('agent1', {
        messageType: 'task_request',
        filter: {}
      })).resolves.not.toThrow();
    });

    it('should unsubscribe agent from specific message type', async () => {
      await messageBroker.subscribeAgent('agent1', {
        messageType: 'task_request',
        filter: {}
      });

      await expect(messageBroker.unsubscribeAgent('agent1', 'task_request')).resolves.not.toThrow();
    });

    it('should unsubscribe agent from all message types', async () => {
      await messageBroker.subscribeAgent('agent1', {
        messageType: 'task_request',
        filter: {}
      });

      await expect(messageBroker.unsubscribeAgent('agent1')).resolves.not.toThrow();
    });
  });

  describe('message history', () => {
    beforeEach(async () => {
      await messageBroker.initialize(mockConfig);
    });

    it('should return empty history for new agent', async () => {
      const history = await messageBroker.getMessageHistory('agent1');
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBe(0);
    });

    it('should limit history results', async () => {
      const history = await messageBroker.getMessageHistory('agent1', 10);
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBeLessThanOrEqual(10);
    });
  });

  describe('statistics', () => {
    beforeEach(async () => {
      await messageBroker.initialize(mockConfig);
    });

    it('should provide broker statistics', () => {
      const stats = messageBroker.getStats();
      
      expect(stats.queueSize).toBeGreaterThanOrEqual(0);
      expect(stats.subscriberCount).toBeGreaterThanOrEqual(0);
      expect(stats.agentsWithHistory).toBeGreaterThanOrEqual(0);
      expect(stats.totalHistoryMessages).toBeGreaterThanOrEqual(0);
    });
  });

  describe('operations when not initialized', () => {
    it('should reject operations when not initialized', async () => {
      const message: AgentMessage = {
        id: 'test-msg',
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'Test',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      await expect(messageBroker.sendMessage(message)).rejects.toThrow('not initialized');
    });
  });

  describe('event handling', () => {
    it('should be an event emitter', () => {
      expect(typeof messageBroker.on).toBe('function');
      expect(typeof messageBroker.emit).toBe('function');
    });

    it('should emit initialization event', async () => {
      const initHandler = jest.fn();
      messageBroker.on('initialized', initHandler);
      
      await messageBroker.initialize(mockConfig);
      
      expect(initHandler).toHaveBeenCalled();
    });
  });
});