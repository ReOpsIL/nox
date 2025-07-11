import { AgentManager } from '../../src/core/agent-manager';
import { MessageBroker } from '../../src/core/message-broker';
import { TaskManager } from '../../src/core/task-manager';
import { WebSocketServer } from '../../src/server/websocket';
import { protocolRegistry } from '../../src/protocols/agent-protocols';
import { DEFAULT_CONFIG } from '../../src/types';

describe('Phase 3: Inter-Agent Communication Integration', () => {
  let agentManager: AgentManager;
  let messageBroker: MessageBroker;
  let taskManager: TaskManager;
  let websocketServer: WebSocketServer;
  let tempDir: string;

  beforeAll(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    
    // Initialize components
    agentManager = new AgentManager(tempDir);
    messageBroker = new MessageBroker(tempDir);
    taskManager = new TaskManager(tempDir);
    websocketServer = new WebSocketServer(agentManager, messageBroker, taskManager);

    // Configure message broker to work with agent manager
    messageBroker.setAgentManager(agentManager);

    const config = {
      ...DEFAULT_CONFIG,
      storage: {
        ...DEFAULT_CONFIG.storage,
        registryPath: tempDir
      }
    };

    // Initialize all components
    await agentManager.initialize(config);
    await messageBroker.initialize(config);
    await taskManager.initialize(config);
    await websocketServer.initialize(config);
  });

  afterAll(async () => {
    try {
      await websocketServer.shutdown();
      await agentManager.shutdown();
      await messageBroker.shutdown();
      await taskManager.shutdown();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('Protocol Registry', () => {
    it('should handle task request messages', async () => {
      const message = protocolRegistry.createTaskRequest(
        'agent1',
        'agent2',
        'Test Task',
        'This is a test task',
        'HIGH'
      );

      expect(message.type).toBe('task_request');
      expect(message.from).toBe('agent1');
      expect(message.to).toBe('agent2');
      expect(message.priority).toBe('HIGH');

      const response = await protocolRegistry.processMessage(message);
      expect(response).not.toBeNull();
      expect(response?.type).toBe('task_response');
      expect(response?.from).toBe('agent2');
      expect(response?.to).toBe('agent1');
    });

    it('should handle information requests', async () => {
      const message = protocolRegistry.createInfoRequest(
        'agent1',
        'agent2',
        'What are your capabilities?'
      );

      expect(message.type).toBe('capability_query');
      const response = await protocolRegistry.processMessage(message);
      expect(response).not.toBeNull();
      expect(response?.type).toBe('direct');
    });

    it('should handle collaboration requests', async () => {
      const message = protocolRegistry.createCollaborationRequest(
        'agent1',
        'agent2',
        'Joint Analysis',
        'Let us work together on this analysis'
      );

      expect(message.type).toBe('direct');
      const response = await protocolRegistry.processMessage(message);
      expect(response).not.toBeNull();
    });

    it('should handle status updates', async () => {
      const message = protocolRegistry.createStatusUpdate(
        'agent1',
        'agent2',
        'Task completed',
        'All analysis is done'
      );

      expect(message.type).toBe('system');
      const response = await protocolRegistry.processMessage(message);
      expect(response).toBeNull(); // Status updates don't generate responses
    });
  });

  describe('Message Broker Integration', () => {
    it('should initialize successfully', () => {
      const stats = messageBroker.getStats();
      expect(stats.queueSize).toBe(0);
      expect(stats.subscriberCount).toBe(0);
    });

    it('should handle message sending', async () => {
      const message = protocolRegistry.createTaskRequest(
        'test-agent-1',
        'test-agent-2',
        'Integration Test',
        'Testing message sending'
      );

      await expect(messageBroker.sendMessage(message)).resolves.not.toThrow();

      const stats = messageBroker.getStats();
      expect(stats.queueSize).toBeGreaterThan(0);
    });

    it('should support agent subscriptions', async () => {
      await expect(messageBroker.subscribeAgent('test-agent', {
        messageType: 'task_request',
        filter: {}
      })).resolves.not.toThrow();

      const stats = messageBroker.getStats();
      expect(stats.subscriberCount).toBe(1);
    });
  });

  describe('Task Manager Integration', () => {
    it('should handle task creation', async () => {
      const task = await taskManager.createTask({
        agentId: 'test-agent',
        title: 'Integration Test Task',
        description: 'Testing task creation in integration',
        status: 'todo',
        priority: 'MEDIUM',
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      });

      expect(task.id).toBeDefined();
      expect(task.title).toBe('Integration Test Task');
      expect(task.agentId).toBe('test-agent');
    });

    it('should handle task delegation', async () => {
      const delegatedTask = await taskManager.delegateTask(
        'agent1',
        'agent2',
        {
          title: 'Delegated Task',
          description: 'Task delegated for testing',
          status: 'todo',
          priority: 'HIGH',
          requestedBy: 'agent1',
          dependencies: [],
          progress: 0
        }
      );

      expect(delegatedTask.agentId).toBe('agent2');
      expect(delegatedTask.requestedBy).toBe('agent1');
      expect(delegatedTask.priority).toBe('HIGH');
    });

    it('should provide task dashboard', async () => {
      const dashboard = await taskManager.getTaskDashboard();
      
      expect(dashboard.total).toBeGreaterThanOrEqual(0);
      expect(dashboard.byStatus).toBeDefined();
      expect(dashboard.byPriority).toBeDefined();
      expect(dashboard.byAgent).toBeDefined();
    });
  });

  describe('WebSocket Server Integration', () => {
    it('should initialize without starting in test environment', async () => {
      expect(websocketServer.getClientCount()).toBe(0);
      expect(websocketServer.isRunning()).toBe(false);
    });

    it('should handle events from core components', (done) => {
      let eventCount = 0;
      const checkCompletion = () => {
        eventCount++;
        if (eventCount >= 2) done();
      };

      // Set up event listeners
      agentManager.on('test-event', checkCompletion);
      taskManager.on('test-event', checkCompletion);

      // Emit test events
      agentManager.emit('test-event', { test: true });
      taskManager.emit('test-event', { test: true });
    });
  });

  describe('End-to-End Communication Flow', () => {
    it('should handle complete message flow', async () => {
      // Create a message
      const message = protocolRegistry.createTaskRequest(
        'sender-agent',
        'receiver-agent',
        'E2E Test Task',
        'End-to-end testing of message flow'
      );

      // Send through message broker
      await messageBroker.sendMessage(message);

      // Verify message was queued
      const stats = messageBroker.getStats();
      expect(stats.queueSize).toBeGreaterThan(0);

      // Verify message history is tracked
      const history = await messageBroker.getMessageHistory('sender-agent', 10);
      expect(Array.isArray(history)).toBe(true);
    });

    it('should handle task creation and delegation workflow', async () => {
      // Create initial task
      const task = await taskManager.createTask({
        agentId: 'primary-agent',
        title: 'Complex Task',
        description: 'A task that needs delegation',
        status: 'inprogress',
        priority: 'HIGH',
        requestedBy: 'user',
        dependencies: [],
        progress: 25
      });

      // Delegate subtask
      const subtask = await taskManager.delegateTask(
        'primary-agent',
        'helper-agent',
        {
          title: 'Subtask of Complex Task',
          description: 'Helper task for the main task',
          status: 'todo',
          priority: 'MEDIUM',
          requestedBy: 'primary-agent',
          dependencies: [task.id],
          progress: 0
        }
      );

      // Verify delegation worked
      expect(subtask.requestedBy).toBe('primary-agent');
      expect(subtask.dependencies).toContain(task.id);

      // Update task progress
      const updatedTask = await taskManager.updateTask(task.id, {
        progress: 75,
        status: 'inprogress'
      });

      expect(updatedTask.progress).toBe(75);
      expect(updatedTask.status).toBe('inprogress');

      // Complete the task
      const completedTask = await taskManager.completeTask(task.id);
      expect(completedTask.status).toBe('done');
      expect(completedTask.progress).toBe(100);
      expect(completedTask.completedAt).toBeInstanceOf(Date);
    });
  });

  describe('Performance and Reliability', () => {
    it('should handle multiple concurrent messages', async () => {
      const promises: Promise<void>[] = [];
      
      for (let i = 0; i < 10; i++) {
        const message = protocolRegistry.createTaskRequest(
          `sender-${i}`,
          `receiver-${i}`,
          `Concurrent Task ${i}`,
          `Testing concurrent message handling ${i}`
        );
        
        promises.push(messageBroker.sendMessage(message));
      }

      await expect(Promise.all(promises)).resolves.not.toThrow();
      
      const stats = messageBroker.getStats();
      expect(stats.queueSize).toBeGreaterThanOrEqual(10);
    });

    it('should maintain message history correctly', async () => {
      const agentId = 'history-test-agent';
      
      // Send several messages
      for (let i = 0; i < 5; i++) {
        const message = protocolRegistry.createStatusUpdate(
          agentId,
          'other-agent',
          `Status Update ${i}`,
          `Update number ${i}`
        );
        await messageBroker.sendMessage(message);
      }

      // Check history
      const history = await messageBroker.getMessageHistory(agentId, 10);
      expect(history.length).toBeGreaterThanOrEqual(5);
      
      // Verify messages are in chronological order
      for (let i = 1; i < history.length; i++) {
        expect(history[i]?.timestamp.getTime()).toBeGreaterThanOrEqual(
          history[i - 1]?.timestamp.getTime() || 0
        );
      }
    });
  });
});