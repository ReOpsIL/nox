import { WebSocketServer } from '../../../src/server/websocket';
import { AgentManager } from '../../../src/core/agent-manager';
import { MessageBroker } from '../../../src/core/message-broker';
import { TaskManager } from '../../../src/core/task-manager';
import { DEFAULT_CONFIG } from '../../../src/types';

// Mock WebSocket
jest.mock('ws', () => {
  const mockWebSocket = {
    OPEN: 1,
    Server: jest.fn().mockImplementation(() => ({
      on: jest.fn(),
      close: jest.fn((callback) => callback && callback())
    }))
  };
  return mockWebSocket;
});

describe('WebSocketServer', () => {
  let websocketServer: WebSocketServer;
  let agentManager: AgentManager;
  let messageBroker: MessageBroker;
  let taskManager: TaskManager;
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    
    agentManager = new AgentManager(tempDir);
    messageBroker = new MessageBroker(tempDir);
    taskManager = new TaskManager(tempDir);
    websocketServer = new WebSocketServer(agentManager, messageBroker, taskManager);
  });

  afterEach(async () => {
    try {
      await websocketServer.shutdown();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize successfully', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true,
          port: 3001
        }
      };

      await expect(websocketServer.initialize(config)).resolves.not.toThrow();
    });

    it('should handle disabled WebSocket in config', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: false
        }
      };

      await websocketServer.initialize(config);
      // Should not throw, but WebSocket should remain disabled
    });

    it('should not initialize twice', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true
        }
      };

      await websocketServer.initialize(config);
      await websocketServer.initialize(config);
      // Should not throw on second initialization
    });
  });

  describe('server lifecycle', () => {
    beforeEach(async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true,
          port: 3001
        }
      };
      await websocketServer.initialize(config);
    });

    it('should start successfully', async () => {
      await expect(websocketServer.start()).resolves.not.toThrow();
      expect(websocketServer.isRunning()).toBe(true);
    });

    it('should not start when not initialized', async () => {
      const uninitializedServer = new WebSocketServer(agentManager, messageBroker, taskManager);
      await expect(uninitializedServer.start()).rejects.toThrow('not initialized');
    });

    it('should shutdown gracefully', async () => {
      await websocketServer.start();
      await expect(websocketServer.shutdown()).resolves.not.toThrow();
      expect(websocketServer.isRunning()).toBe(false);
    });
  });

  describe('client management', () => {
    beforeEach(async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true,
          port: 3001
        }
      };
      await websocketServer.initialize(config);
    });

    it('should track client count', () => {
      expect(websocketServer.getClientCount()).toBe(0);
    });

    it('should handle broadcasting when no clients', () => {
      expect(() => {
        websocketServer.broadcast('test_event', { data: 'test' });
      }).not.toThrow();
    });

    it('should handle sending to specific client when no clients', () => {
      expect(() => {
        websocketServer.sendToClient('nonexistent', 'test_event', { data: 'test' });
      }).not.toThrow();
    });
  });

  describe('event integration', () => {
    beforeEach(async () => {
      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true,
          port: 3001
        }
      };
      await websocketServer.initialize(config);
      await websocketServer.start();
    });

    it('should be an event emitter', () => {
      expect(typeof websocketServer.on).toBe('function');
      expect(typeof websocketServer.emit).toBe('function');
    });

    it('should emit initialization event', (done) => {
      const newServer = new WebSocketServer(agentManager, messageBroker, taskManager);
      newServer.on('initialized', () => {
        done();
      });

      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true
        }
      };
      newServer.initialize(config);
    });

    it('should emit started event', (done) => {
      const newServer = new WebSocketServer(agentManager, messageBroker, taskManager);
      
      newServer.on('started', () => {
        newServer.shutdown().then(() => done());
      });

      const config = {
        ...DEFAULT_CONFIG,
        server: {
          ...DEFAULT_CONFIG.server,
          websocketEnabled: true,
          port: 3002
        }
      };
      
      newServer.initialize(config).then(() => {
        newServer.start();
      });
    });
  });

  describe('error handling', () => {
    it('should handle shutdown when not running', async () => {
      await expect(websocketServer.shutdown()).resolves.not.toThrow();
    });

    it('should handle operations when server is null', () => {
      expect(() => {
        websocketServer.broadcast('test', {});
        websocketServer.sendToClient('test', 'test', {});
      }).not.toThrow();
    });
  });
});