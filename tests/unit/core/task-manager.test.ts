import { TaskManager } from '../../../src/core/task-manager';
import { NoxConfig, DEFAULT_CONFIG } from '../../../src/types';
import * as fs from 'fs/promises';
import * as chokidar from 'chokidar';

// Mock fs/promises and chokidar
jest.mock('fs/promises');
jest.mock('chokidar');

describe('TaskManager', () => {
  let taskManager: TaskManager;
  let tempDir: string;
  let mockConfig: NoxConfig;
  let mockWatcher: any;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    taskManager = new TaskManager(tempDir);

    mockConfig = {
      ...DEFAULT_CONFIG,
      storage: {
        ...DEFAULT_CONFIG.storage,
        registryPath: tempDir
      }
    };

    // Mock chokidar watcher
    mockWatcher = {
      on: jest.fn(),
      close: jest.fn().mockResolvedValue(undefined)
    };
    (chokidar.watch as jest.Mock).mockReturnValue(mockWatcher);

    // Mock fs operations
    (fs.mkdir as jest.Mock).mockResolvedValue(undefined);
    (fs.readdir as jest.Mock).mockResolvedValue([]);
    (fs.readFile as jest.Mock).mockResolvedValue('# Tasks for test-agent\n\n## 📋 Todo\n\n## 🟢 In Progress\n\n## ✅ Done\n');
    (fs.writeFile as jest.Mock).mockResolvedValue(undefined);
  });

  afterEach(async () => {
    try {
      await taskManager.shutdown();
    } catch (error) {
      // Ignore cleanup errors
    }
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize successfully', async () => {
      await taskManager.initialize(mockConfig);
      expect(fs.mkdir).toHaveBeenCalledWith(expect.stringContaining('tasks'), { recursive: true });
      expect(chokidar.watch).toHaveBeenCalled();
    });

    it('should handle multiple initializations', async () => {
      await taskManager.initialize(mockConfig);
      await taskManager.initialize(mockConfig);
      // Should not throw on second initialization
    });
  });

  describe('task management', () => {
    beforeEach(async () => {
      await taskManager.initialize(mockConfig);
    });

    it('should create a new task', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      const task = await taskManager.createTask(taskData);

      expect(task.id).toBeDefined();
      expect(task.title).toBe('Test Task');
      expect(task.agentId).toBe('test-agent');
      expect(task.createdAt).toBeInstanceOf(Date);
      expect(task.updatedAt).toBeInstanceOf(Date);
    });

    it('should get task by ID', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      const createdTask = await taskManager.createTask(taskData);
      const retrievedTask = await taskManager.getTask(createdTask.id);

      expect(retrievedTask).not.toBeNull();
      expect(retrievedTask!.id).toBe(createdTask.id);
    });

    it('should return null for non-existent task', async () => {
      const task = await taskManager.getTask('non-existent-id');
      expect(task).toBeNull();
    });

    it('should update task', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      const createdTask = await taskManager.createTask(taskData);

      // Add a small delay to ensure timestamps are different
      await new Promise(resolve => setTimeout(resolve, 10));

      const updatedTask = await taskManager.updateTask(createdTask.id, {
        status: 'inprogress',
        progress: 50
      });

      expect(updatedTask.status).toBe('inprogress');
      expect(updatedTask.progress).toBe(50);
      expect(updatedTask.updatedAt.getTime()).toBeGreaterThan(createdTask.updatedAt.getTime());
    });

    it('should complete task', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      const createdTask = await taskManager.createTask(taskData);
      const completedTask = await taskManager.completeTask(createdTask.id);

      expect(completedTask.status).toBe('done');
      expect(completedTask.progress).toBe(100);
      expect(completedTask.completedAt).toBeInstanceOf(Date);
    });
  });

  describe('task queries', () => {
    beforeEach(async () => {
      await taskManager.initialize(mockConfig);
    });

    it('should get agent tasks', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      await taskManager.createTask(taskData);
      const agentTasks = await taskManager.getAgentTasks('test-agent');

      expect(Array.isArray(agentTasks)).toBe(true);
      expect(agentTasks.length).toBe(1);
      expect(agentTasks[0]?.agentId).toBe('test-agent');
    });

    it('should get tasks by status', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      await taskManager.createTask(taskData);
      const todoTasks = await taskManager.getTasksByStatus('todo');

      expect(Array.isArray(todoTasks)).toBe(true);
      expect(todoTasks.length).toBe(1);
      expect(todoTasks[0]?.status).toBe('todo');
    });

    it('should get tasks by priority', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'High Priority Task',
        description: 'An important task',
        status: 'todo' as const,
        priority: 'HIGH' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      await taskManager.createTask(taskData);
      const highPriorityTasks = await taskManager.getTasksByPriority('HIGH');

      expect(Array.isArray(highPriorityTasks)).toBe(true);
      expect(highPriorityTasks.length).toBe(1);
      expect(highPriorityTasks[0]?.priority).toBe('HIGH');
    });

    it('should get blocked tasks', async () => {
      const blockedTasks = await taskManager.getBlockedTasks();
      expect(Array.isArray(blockedTasks)).toBe(true);
    });
  });

  describe('task delegation', () => {
    beforeEach(async () => {
      await taskManager.initialize(mockConfig);
    });

    it('should delegate task between agents', async () => {
      const taskData = {
        title: 'Delegated Task',
        description: 'A task to be delegated',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'agent1',
        dependencies: [],
        progress: 0
      };

      const delegatedTask = await taskManager.delegateTask('agent1', 'agent2', taskData);

      expect(delegatedTask.agentId).toBe('agent2');
      expect(delegatedTask.requestedBy).toBe('agent1');
      expect(delegatedTask.title).toBe('Delegated Task');
    });
  });

  describe('task dashboard', () => {
    beforeEach(async () => {
      await taskManager.initialize(mockConfig);
    });

    it('should provide task dashboard overview', async () => {
      const dashboard = await taskManager.getTaskDashboard();

      expect(dashboard.total).toBeGreaterThanOrEqual(0);
      expect(dashboard.byStatus).toBeDefined();
      expect(dashboard.byPriority).toBeDefined();
      expect(dashboard.byAgent).toBeDefined();
      expect(dashboard.blocked).toBeGreaterThanOrEqual(0);
    });
  });

  describe('operations when not initialized', () => {
    it('should reject operations when not initialized', async () => {
      const taskData = {
        agentId: 'test-agent',
        title: 'Test Task',
        description: 'A test task',
        status: 'todo' as const,
        priority: 'MEDIUM' as const,
        requestedBy: 'user',
        dependencies: [],
        progress: 0
      };

      await expect(taskManager.createTask(taskData)).rejects.toThrow('not initialized');
    });
  });

  describe('task count', () => {
    beforeEach(async () => {
      await taskManager.initialize(mockConfig);
    });

    it('should return total task count', async () => {
      const count = await taskManager.getTotalTaskCount();
      expect(typeof count).toBe('number');
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  describe('event handling', () => {
    it('should be an event emitter', () => {
      expect(typeof taskManager.on).toBe('function');
      expect(typeof taskManager.emit).toBe('function');
    });

    it('should emit initialization event', async () => {
      const initHandler = jest.fn();
      taskManager.on('initialized', initHandler);

      await taskManager.initialize(mockConfig);

      expect(initHandler).toHaveBeenCalled();
    });
  });
});
