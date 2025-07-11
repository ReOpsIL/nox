import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { Task, NoxConfig, TaskStatus, TaskPriority } from '../types';
import { logger } from '../utils/logger';
import { PriorityQueue } from '../utils/priority-queue';
import * as chokidar from 'chokidar';

/**
 * Task Manager - Handles task tracking and markdown file management
 * Manages task creation, delegation, and status updates
 */
export class TaskManager extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private tasksDir: string;
  private tasks: Map<string, Task> = new Map();
  private tasksByAgent: Map<string, Set<string>> = new Map();
  private fileWatcher: chokidar.FSWatcher | null = null;
  private taskQueue: PriorityQueue<Task>;
  private processingInterval: NodeJS.Timeout | null = null;
  private isProcessing = false;

  constructor(workingDir: string) {
    super();
    this.workingDir = workingDir;
    this.tasksDir = path.join(workingDir, 'tasks');

    // Create priority queue for tasks
    this.taskQueue = new PriorityQueue<Task>((a, b) => {
      // Priority order: CRITICAL > HIGH > MEDIUM > LOW
      const priorityValues = { 'CRITICAL': 3, 'HIGH': 2, 'MEDIUM': 1, 'LOW': 0 };
      return priorityValues[b.priority] - priorityValues[a.priority];
    });
  }

  /**
   * Initialize the task manager
   */
  async initialize(_config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('TaskManager already initialized');
      return;
    }

    try {
      // Ensure tasks directory exists
      await fs.mkdir(this.tasksDir, { recursive: true });

      // Load existing tasks from markdown files
      await this.loadTasksFromFiles();

      // Set up file watcher for task markdown files
      await this.setupFileWatcher();

      this.initialized = true;
      logger.info('TaskManager initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize TaskManager:', error);
      throw error;
    }
  }

  /**
   * Start the task manager
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('TaskManager not initialized');
    }

    // Start task processing loop
    this.processingInterval = setInterval(() => this.processTasks(), 1000);

    logger.info('TaskManager started');
    this.emit('started');
  }

  /**
   * Shutdown the task manager
   */
  async shutdown(): Promise<void> {
    // Stop processing loop
    if (this.processingInterval) {
      clearInterval(this.processingInterval);
      this.processingInterval = null;
    }

    // Stop file watcher
    if (this.fileWatcher) {
      await this.fileWatcher.close();
      this.fileWatcher = null;
    }

    logger.info('TaskManager shutdown');
    this.emit('shutdown');
  }

  /**
   * Get total task count
   */
  async getTotalTaskCount(): Promise<number> {
    return this.tasks.size;
  }

  /**
   * Create a new task
   */
  async createTask(task: Omit<Task, 'id' | 'createdAt' | 'updatedAt'>): Promise<Task> {
    if (!this.initialized) {
      throw new Error('TaskManager not initialized');
    }

    const fullTask: Task = {
      ...task,
      id: `task_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      createdAt: new Date(),
      updatedAt: new Date(),
      status: task.status || 'todo',
      priority: task.priority || 'MEDIUM',
      dependencies: task.dependencies || [],
      progress: task.progress || 0
    };

    // Add to in-memory store
    this.tasks.set(fullTask.id, fullTask);

    // Add to agent's task set
    if (!this.tasksByAgent.has(fullTask.agentId)) {
      this.tasksByAgent.set(fullTask.agentId, new Set());
    }
    this.tasksByAgent.get(fullTask.agentId)!.add(fullTask.id);

    // Add to priority queue if not completed
    if (fullTask.status !== 'done') {
      this.taskQueue.enqueue(fullTask);
    }

    // Update markdown file
    await this.updateMarkdownFile(fullTask.agentId);

    this.emit('task-created', fullTask);
    logger.info(`Task created: ${fullTask.id} for agent ${fullTask.agentId}`);

    return fullTask;
  }

  /**
   * Update an existing task
   */
  async updateTask(taskId: string, updates: Partial<Task>): Promise<Task> {
    if (!this.initialized) {
      throw new Error('TaskManager not initialized');
    }

    const task = this.tasks.get(taskId);
    if (!task) {
      throw new Error(`Task ${taskId} not found`);
    }

    const updatedTask = {
      ...task,
      ...updates,
      updatedAt: new Date()
    };

    // Update in-memory store
    this.tasks.set(taskId, updatedTask);

    // Update task queue if priority or status changed
    if (updates.priority || updates.status) {
      // Remove old version from queue
      this.taskQueue.remove(t => t.id === taskId);

      // Add updated version if not completed
      if (updatedTask.status !== 'done') {
        this.taskQueue.enqueue(updatedTask);
      }
    }

    // Update markdown file
    await this.updateMarkdownFile(updatedTask.agentId);

    this.emit('task-updated', updatedTask);
    logger.info(`Task updated: ${taskId}`);

    return updatedTask;
  }

  /**
   * Get a task by ID
   */
  async getTask(taskId: string): Promise<Task | null> {
    return this.tasks.get(taskId) || null;
  }

  /**
   * Get all tasks for an agent
   */
  async getAgentTasks(agentId: string): Promise<Task[]> {
    const taskIds = this.tasksByAgent.get(agentId) || new Set();
    return Array.from(taskIds).map(id => this.tasks.get(id)!).filter(Boolean);
  }

  /**
   * Delegate a task from one agent to another
   */
  async delegateTask(
    fromAgentId: string, 
    toAgentId: string, 
    taskData: Omit<Task, 'id' | 'agentId' | 'createdAt' | 'updatedAt'>
  ): Promise<Task> {
    if (!this.initialized) {
      throw new Error('TaskManager not initialized');
    }

    // Create new task for target agent
    const delegatedTask = await this.createTask({
      ...taskData,
      agentId: toAgentId,
      requestedBy: fromAgentId
    });

    logger.info(`Task delegated from ${fromAgentId} to ${toAgentId}: ${delegatedTask.id}`);
    this.emit('task-delegated', fromAgentId, toAgentId, delegatedTask);

    return delegatedTask;
  }

  /**
   * Complete a task
   */
  async completeTask(taskId: string): Promise<Task> {
    return this.updateTask(taskId, { 
      status: 'done', 
      progress: 100,
      completedAt: new Date()
    });
  }

  /**
   * Get all tasks with a specific status
   */
  async getTasksByStatus(status: TaskStatus): Promise<Task[]> {
    return Array.from(this.tasks.values()).filter(task => task.status === status);
  }

  /**
   * Get all tasks with a specific priority
   */
  async getTasksByPriority(priority: TaskPriority): Promise<Task[]> {
    return Array.from(this.tasks.values()).filter(task => task.priority === priority);
  }

  /**
   * Get all blocked tasks (tasks with unmet dependencies)
   */
  async getBlockedTasks(): Promise<Task[]> {
    const blockedTasks: Task[] = [];

    for (const task of Array.from(this.tasks.values())) {
      if (task.status === 'todo' && task.dependencies.length > 0) {
        const unmetDependencies = task.dependencies.filter(depId => {
          const depTask = this.tasks.get(depId);
          return !depTask || depTask.status !== 'done';
        });

        if (unmetDependencies.length > 0) {
          blockedTasks.push(task);
        }
      }
    }

    return blockedTasks;
  }

  /**
   * Get task dashboard overview
   */
  async getTaskDashboard(): Promise<{
    total: number;
    byStatus: Record<TaskStatus, number>;
    byPriority: Record<TaskPriority, number>;
    byAgent: Record<string, { total: number; byStatus: Record<TaskStatus, number> }>;
    blocked: number;
  }> {
    const result = {
      total: this.tasks.size,
      byStatus: {
        todo: 0,
        inprogress: 0,
        done: 0,
        blocked: 0,
        cancelled: 0
      } as Record<TaskStatus, number>,
      byPriority: {
        LOW: 0,
        MEDIUM: 0,
        HIGH: 0,
        CRITICAL: 0
      } as Record<TaskPriority, number>,
      byAgent: {} as Record<string, { total: number; byStatus: Record<TaskStatus, number> }>,
      blocked: 0
    };

    // Count tasks by status and priority
    for (const task of Array.from(this.tasks.values())) {
      result.byStatus[task.status]++;
      result.byPriority[task.priority]++;

      // Initialize agent stats if needed
      if (!result.byAgent[task.agentId]) {
        result.byAgent[task.agentId] = {
          total: 0,
          byStatus: { todo: 0, inprogress: 0, done: 0, blocked: 0, cancelled: 0 }
        };
      }

      // Update agent stats
      result.byAgent[task.agentId]!.total++;
      result.byAgent[task.agentId]!.byStatus[task.status]++;
    }

    // Count blocked tasks
    result.blocked = (await this.getBlockedTasks()).length;

    return result;
  }

  /**
   * Process tasks in the queue
   */
  private async processTasks(): Promise<void> {
    if (this.isProcessing || this.taskQueue.isEmpty()) {
      return;
    }

    this.isProcessing = true;

    try {
      // Check for tasks with dependencies that are now satisfied
      const todoTasks = await this.getTasksByStatus('todo');
      for (const task of todoTasks) {
        if (task.dependencies.length > 0) {
          const allDependenciesMet = task.dependencies.every(depId => {
            const depTask = this.tasks.get(depId);
            return depTask && depTask.status === 'done';
          });

          if (allDependenciesMet) {
            // Dependencies are met, task can be started
            this.emit('task-dependencies-met', task);
            logger.info(`Dependencies met for task ${task.id}`);
          }
        }
      }
    } catch (error) {
      logger.error('Error processing tasks:', error);
    } finally {
      this.isProcessing = false;
    }
  }

  /**
   * Set up file watcher for task markdown files
   */
  private async setupFileWatcher(): Promise<void> {
    this.fileWatcher = chokidar.watch(`${this.tasksDir}/*.md`, {
      persistent: true,
      ignoreInitial: true,
      awaitWriteFinish: {
        stabilityThreshold: 1000,
        pollInterval: 100
      }
    });

    this.fileWatcher.on('change', async (filePath) => {
      try {
        const agentId = path.basename(filePath, '.md');
        logger.debug(`Task file changed for agent: ${agentId}`);

        // Reload tasks from the changed file
        await this.loadTasksFromFile(filePath, agentId);

        this.emit('task-file-changed', agentId);
      } catch (error) {
        logger.error(`Error processing task file change: ${filePath}`, error);
      }
    });

    logger.info('Task file watcher set up');
  }

  /**
   * Load tasks from all markdown files
   */
  private async loadTasksFromFiles(): Promise<void> {
    try {
      const files = await fs.readdir(this.tasksDir);
      const mdFiles = files.filter(file => file.endsWith('.md'));

      for (const file of mdFiles) {
        const agentId = path.basename(file, '.md');
        const filePath = path.join(this.tasksDir, file);
        await this.loadTasksFromFile(filePath, agentId);
      }

      logger.info(`Loaded tasks from ${mdFiles.length} markdown files`);
    } catch (error) {
      logger.error('Failed to load tasks from files:', error);
    }
  }

  /**
   * Load tasks from a single markdown file
   */
  private async loadTasksFromFile(filePath: string, agentId: string): Promise<void> {
    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const tasks = this.parseMarkdownTasks(content, agentId);

      // Update in-memory store with tasks from file
      for (const task of tasks) {
        this.tasks.set(task.id, task);

        // Add to agent's task set
        if (!this.tasksByAgent.has(task.agentId)) {
          this.tasksByAgent.set(task.agentId, new Set());
        }
        this.tasksByAgent.get(task.agentId)!.add(task.id);

        // Add to priority queue if not completed
        if (task.status !== 'done') {
          this.taskQueue.enqueue(task);
        }
      }

      logger.info(`Loaded ${tasks.length} tasks for agent ${agentId}`);
    } catch (error) {
      logger.error(`Failed to load tasks from file ${filePath}:`, error);
    }
  }

  /**
   * Parse markdown content into tasks
   */
  private parseMarkdownTasks(content: string, agentId: string): Task[] {
    const tasks: Task[] = [];
    const lines = content.split('\n');

    let currentSection: 'todo' | 'inprogress' | 'done' | null = null;
    let currentTask: Partial<Task> | null = null;

    for (let i = 0; i < lines.length; i++) {
      const line = lines[i]?.trim() || '';

      // Check for section headers
      if (line.startsWith('## 🟢 In Progress')) {
        currentSection = 'inprogress';
        continue;
      } else if (line.startsWith('## 📋 Todo')) {
        currentSection = 'todo';
        continue;
      } else if (line.startsWith('## ✅ Done')) {
        currentSection = 'done';
        continue;
      }

      // Parse task lines
      if (currentSection && line.startsWith('- [')) {
        // Save previous task if exists
        if (currentTask && currentTask.title) {
          tasks.push(this.createTaskFromMarkdown(currentTask, agentId, currentSection));
        }

        // Start new task
        const isDone = line.startsWith('- [x]');
        const taskTitle = line.substring(line.indexOf(']') + 1).trim();

        currentTask = {
          title: taskTitle,
          status: isDone ? 'done' : currentSection
        };

        // Extract priority if present
        const priorityMatch = taskTitle.match(/\(Priority: (LOW|MEDIUM|HIGH|CRITICAL)\)/);
        if (priorityMatch) {
          currentTask.priority = priorityMatch[1] as TaskPriority;
          currentTask.title = taskTitle.replace(priorityMatch[0], '').trim();
        }

      } else if (currentTask && line.startsWith('  - ')) {
        // Parse task metadata
        const metaLine = line.substring(4).trim();

        if (metaLine.startsWith('Started:')) {
          currentTask.startedAt = new Date(metaLine.substring(8).trim());
        } else if (metaLine.startsWith('Completed:')) {
          currentTask.completedAt = new Date(metaLine.substring(10).trim());
        } else if (metaLine.startsWith('Requested by:')) {
          currentTask.requestedBy = metaLine.substring(13).trim();
        } else if (metaLine.startsWith('Deadline:')) {
          currentTask.deadline = new Date(metaLine.substring(9).trim());
        } else if (metaLine.startsWith('Progress:')) {
          const progressMatch = metaLine.match(/Progress: (\d+)%/);
          if (progressMatch) {
            currentTask.progress = parseInt(progressMatch[1] || '0', 10);
          }
        } else if (metaLine.startsWith('Dependencies:')) {
          const deps = metaLine.substring(13).trim();
          if (deps) {
            currentTask.dependencies = deps.split(',').map(d => d.trim());
          }
        } else if (metaLine.startsWith('Description:')) {
          currentTask.description = metaLine.substring(12).trim();
        }
      } else if (line === '' && currentTask) {
        // Empty line ends current task
        if (currentTask.title && currentSection) {
          tasks.push(this.createTaskFromMarkdown(currentTask, agentId, currentSection));
          currentTask = null;
        }
      }
    }

    // Add final task if exists
    if (currentTask && currentTask.title && currentSection) {
      tasks.push(this.createTaskFromMarkdown(currentTask, agentId, currentSection));
    }

    return tasks;
  }

  /**
   * Create a task object from markdown parsing results
   */
  private createTaskFromMarkdown(
    taskData: Partial<Task>, 
    agentId: string, 
    status: 'todo' | 'inprogress' | 'done'
  ): Task {
    return {
      id: taskData.id || `task_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`,
      agentId,
      title: taskData.title!,
      description: taskData.description || '',
      status: taskData.status || status,
      priority: taskData.priority || 'MEDIUM',
      createdAt: taskData.createdAt || new Date(),
      updatedAt: taskData.updatedAt || new Date(),
      startedAt: taskData.startedAt || undefined,
      completedAt: taskData.completedAt || undefined,
      deadline: taskData.deadline || undefined,
      requestedBy: taskData.requestedBy || 'user',
      dependencies: taskData.dependencies || [],
      progress: taskData.progress || 0
    };
  }

  /**
   * Update the markdown file for an agent
   */
  private async updateMarkdownFile(agentId: string): Promise<void> {
    try {
      const tasks = await this.getAgentTasks(agentId);
      const filePath = path.join(this.tasksDir, `${agentId}.md`);

      // Group tasks by status
      const todoTasks = tasks.filter(t => t.status === 'todo');
      const inProgressTasks = tasks.filter(t => t.status === 'inprogress');
      const doneTasks = tasks.filter(t => t.status === 'done');

      // Generate markdown content
      let content = `# Tasks for ${agentId}\n\n`;

      // In Progress section
      content += `## 🟢 In Progress\n`;
      for (const task of inProgressTasks) {
        content += this.formatTaskAsMarkdown(task, false);
      }
      content += '\n';

      // Todo section
      content += `## 📋 Todo\n`;
      for (const task of todoTasks) {
        content += this.formatTaskAsMarkdown(task, false);
      }
      content += '\n';

      // Done section
      content += `## ✅ Done\n`;
      for (const task of doneTasks) {
        content += this.formatTaskAsMarkdown(task, true);
      }

      // Write to file
      await fs.writeFile(filePath, content);
      logger.debug(`Updated task file for agent ${agentId}`);

    } catch (error) {
      logger.error(`Failed to update markdown file for agent ${agentId}:`, error);
    }
  }

  /**
   * Format a task as markdown
   */
  private formatTaskAsMarkdown(task: Task, isDone: boolean): string {
    const checkbox = isDone ? '- [x]' : '- [ ]';
    let result = `${checkbox} ${task.title} (Priority: ${task.priority})\n`;

    if (task.startedAt) {
      result += `  - Started: ${task.startedAt.toISOString()}\n`;
    }

    if (task.completedAt) {
      result += `  - Completed: ${task.completedAt.toISOString()}\n`;
    }

    if (task.requestedBy) {
      result += `  - Requested by: ${task.requestedBy}\n`;
    }

    if (task.deadline) {
      result += `  - Deadline: ${task.deadline.toISOString()}\n`;
    }

    if (task.dependencies.length > 0) {
      result += `  - Dependencies: ${task.dependencies.join(', ')}\n`;
    }

    if (task.progress > 0) {
      result += `  - Progress: ${task.progress}%`;
      if (task.description) {
        result += ` - ${task.description}\n`;
      } else {
        result += '\n';
      }
    } else if (task.description) {
      result += `  - Description: ${task.description}\n`;
    }

    result += '\n';
    return result;
  }
}
