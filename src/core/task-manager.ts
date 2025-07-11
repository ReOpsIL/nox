import { EventEmitter } from 'events';
import { Task, NoxConfig } from '../types';
import { logger } from '../utils/logger';

export class TaskManager extends EventEmitter {
  private initialized = false;
  private tasks: Map<string, Task> = new Map();

  constructor() {
    super();
  }

  async initialize(_config: NoxConfig): Promise<void> {
    this.initialized = true;
    logger.info('TaskManager initialized');
  }

  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('TaskManager not initialized');
    }
    logger.info('TaskManager started');
  }

  async shutdown(): Promise<void> {
    logger.info('TaskManager shutdown');
  }

  async getTotalTaskCount(): Promise<number> {
    return this.tasks.size;
  }

  async createTask(task: Omit<Task, 'id' | 'createdAt' | 'updatedAt'>): Promise<Task> {
    const fullTask: Task = {
      ...task,
      id: `task_${Date.now()}`,
      createdAt: new Date(),
      updatedAt: new Date()
    };

    this.tasks.set(fullTask.id, fullTask);
    this.emit('task-created', fullTask);
    
    logger.info(`Task created: ${fullTask.id} for agent ${fullTask.agentId}`);
    return fullTask;
  }

  async updateTask(taskId: string, updates: Partial<Task>): Promise<Task> {
    const task = this.tasks.get(taskId);
    if (!task) {
      throw new Error(`Task ${taskId} not found`);
    }

    const updatedTask = {
      ...task,
      ...updates,
      updatedAt: new Date()
    };

    this.tasks.set(taskId, updatedTask);
    this.emit('task-updated', updatedTask);
    
    logger.info(`Task updated: ${taskId}`);
    return updatedTask;
  }

  async getTask(taskId: string): Promise<Task | null> {
    return this.tasks.get(taskId) || null;
  }

  async getAgentTasks(agentId: string): Promise<Task[]> {
    return Array.from(this.tasks.values()).filter(task => task.agentId === agentId);
  }
}