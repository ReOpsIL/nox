/**
 * Task Routes - API endpoints for task management and visualization
 */

import { Router, Request, Response } from 'express';
import { TaskManager } from '../../core/task-manager';
import { logger } from '../../utils/logger';
import { Task, TaskStatus, TaskPriority } from '../../types';
import * as fs from 'fs/promises';
import * as path from 'path';
import { WebSocketServer } from '../../server/websocket';

/**
 * Serialize a task object for JSON transmission
 * Converts Date objects to ISO strings to prevent React serialization errors
 */
function serializeTask(task: Task): any {
  return {
    ...task,
    startedAt: task.startedAt?.toISOString(),
    completedAt: task.completedAt?.toISOString(),
    deadline: task.deadline?.toISOString()
  };
}





/**
 * Get conversation outputs for a task
 */
async function getTaskOutput(agentId: string, taskId: string): Promise<{ results?: string; error?: string }> {
  try {
    const conversationPath = path.join(process.cwd(), 'conversations', `${agentId}.json`);
    
    try {
      const conversationData = await fs.readFile(conversationPath, 'utf-8');
      const conversation = JSON.parse(conversationData);
      
      if (conversation.messages && Array.isArray(conversation.messages)) {
        // Find messages related to this task
        const taskMessages = conversation.messages.filter((msg: any) => 
          msg.content && (
            msg.content.includes(taskId) || 
            msg.content.includes('TASK COMPLETED') ||
            (msg.role === 'assistant' && msg.content.length > 50) // Claude responses are usually longer
          )
        );
        
        // Get the last assistant response (most likely the task result)
        const lastAssistantMessage = taskMessages
          .filter((msg: any) => msg.role === 'assistant')
          .pop();
        
        if (lastAssistantMessage) {
          const content = lastAssistantMessage.content;
          
          // Check if it's an error response
          if (content.toLowerCase().includes('error:') || 
              content.toLowerCase().includes('failed:') ||
              content.toLowerCase().includes('cannot complete') ||
              content.toLowerCase().includes('task failed')) {
            return { error: content };
          } else {
            return { results: content };
          }
        }
      }
    } catch (fileError) {
      // Conversation file doesn't exist or can't be read
      logger.debug(`No conversation file found for agent ${agentId}: ${fileError}`);
    }
    
    return {};
  } catch (error) {
    logger.error(`Error getting task output for ${taskId}:`, error);
    return {};
  }
}

/**
 * Enhanced task serialization with conversation outputs
 */
async function serializeTaskWithOutput(task: Task): Promise<any> {
  const serializedTask = serializeTask(task);
  const output = await getTaskOutput(task.agentId, task.id);
  
  return {
    ...serializedTask,
    ...output
  };
}

/**
 * Set up task routes
 */
export function setupTaskRoutes(router: Router, taskManager: TaskManager, websocketServer?: WebSocketServer): void {
  const taskRouter = Router();
  router.use('/tasks', taskRouter);

  /**
   * GET /api/tasks
   * Get all tasks with optional filtering
   */
  taskRouter.get('/', async (req: Request, res: Response) => {
    try {
      // Parse query parameters for filtering
      const agentId = req.query.agentId as string;
      const status = req.query.status as TaskStatus;
      const priority = req.query.priority as TaskPriority;

      let tasks: Task[] = [];

      // Get tasks based on filters
      if (agentId) {
        tasks = await taskManager.getAgentTasks(agentId);
      } else if (status) {
        tasks = await taskManager.getTasksByStatus(status);
      } else if (priority) {
        tasks = await taskManager.getTasksByPriority(priority);
      } else {
        // Get all tasks (this is a simplified approach - in a real system,
        // you would implement pagination and more sophisticated filtering)
        const totalCount = await taskManager.getTotalTaskCount();
        if (totalCount > 1000) {
          return res.status(400).json({
            success: false,
            error: 'Too many tasks',
            message: 'Please use filters to narrow down the results'
          });
        }

        // Get tasks for all agents
        const dashboard = await taskManager.getTaskDashboard();
        const agentIds = Object.keys(dashboard.byAgent);

        for (const id of agentIds) {
          const agentTasks = await taskManager.getAgentTasks(id);
          tasks = tasks.concat(agentTasks);
        }
      }

      // Apply additional filtering if multiple filters are provided
      if (status && tasks.length > 0) {
        tasks = tasks.filter(task => task.status === status);
      }

      if (priority && tasks.length > 0) {
        tasks = tasks.filter(task => task.priority === priority);
      }

      // Enhance tasks with output for display
      const enhancedTasks = await Promise.all(tasks.map(serializeTaskWithOutput));
      
      res.json({
        success: true,
        tasks: enhancedTasks
      });
      return;
    } catch (error: any) {
      logger.error('Error getting tasks:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get tasks',
        message: error.message
      });
      return;
    }
  });

  /**
   * GET /api/tasks/dashboard
   * Get task dashboard overview
   */
  taskRouter.get('/dashboard', async (_req: Request, res: Response) => {
    try {
      const dashboard = await taskManager.getTaskDashboard();
      res.json({
        success: true,
        dashboard
      });
    } catch (error: any) {
      logger.error('Error getting task dashboard:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get task dashboard',
        message: error.message
      });
    }
  });

  /**
   * GET /api/tasks/blocked
   * Get blocked tasks
   */
  taskRouter.get('/blocked', async (_req: Request, res: Response) => {
    try {
      const blockedTasks = await taskManager.getBlockedTasks();
      res.json({
        success: true,
        tasks: blockedTasks.map(serializeTask)
      });
    } catch (error: any) {
      logger.error('Error getting blocked tasks:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get blocked tasks',
        message: error.message
      });
    }
  });

  /**
   * GET /api/tasks/:taskId
   * Get a specific task
   */
  taskRouter.get('/:taskId', async (req: Request, res: Response) => {
    try {
      const taskId = req.params.taskId as string;
      const task = await taskManager.getTask(taskId);

      if (!task) {
        return res.status(404).json({
          success: false,
          error: 'Task not found',
          message: `Task ${taskId} not found`
        });
      }

      // Enhance task with output for display
      const enhancedTask = await serializeTaskWithOutput(task);
      
      res.json({
        success: true,
        task: enhancedTask
      });
      return;
    } catch (error: any) {
      logger.error(`Error getting task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get task',
        message: error.message
      });
      return;
    }
  });

  /**
   * POST /api/tasks
   * Create a new task
   */
  taskRouter.post('/', async (req: Request, res: Response) => {
    try {
      const {
        agentId,
        title,
        description,
        priority,
        deadline,
        dependencies,
        requestedBy
      } = req.body;

      // Validate required fields
      if (!agentId || !title) {
        return res.status(400).json({
          success: false,
          error: 'Bad Request',
          message: 'Agent ID and title are required'
        });
      }

      // Create task
      const task = await taskManager.createTask({
        agentId,
        title,
        description: description || '',
        status: 'todo',
        priority: priority || 'MEDIUM',
        deadline: deadline ? new Date(deadline) : new Date(),
        dependencies: dependencies || [],
        requestedBy: requestedBy || 'user',
        progress: 0
      });

      // Broadcast task creation event to connected clients
      if (websocketServer) {
        websocketServer.broadcast('task_created', serializeTask(task));
      }

      res.status(201).json({
        success: true,
        task: serializeTask(task)
      });
      return;
    } catch (error: any) {
      logger.error('Error creating task:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to create task',
        message: error.message
      });
      return;
    }
  });

  /**
   * PUT /api/tasks/:taskId
   * Update a task
   */
  taskRouter.put('/:taskId', async (req: Request, res: Response) => {
    try {
      const taskId = req.params.taskId as string;
      const updates = req.body;

      // Convert deadline to Date object if provided
      if (updates.deadline) {
        updates.deadline = new Date(updates.deadline);
      }

      // Update task
      const task = await taskManager.updateTask(taskId, updates);

      res.json({
        success: true,
        task: serializeTask(task)
      });
    } catch (error: any) {
      logger.error(`Error updating task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to update task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks/:taskId/start
   * Start executing a task
   */
  taskRouter.post('/:taskId/start', async (req: Request, res: Response) => {
    try {
      const taskId = req.params.taskId as string;
      const task = await taskManager.startTask(taskId);

      res.json({
        success: true,
        task,
        message: `Task ${taskId} started successfully`
      });
    } catch (error: any) {
      logger.error(`Error starting task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to start task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks/:taskId/execute
   * Execute a task (send to agent)
   */
  taskRouter.post('/:taskId/execute', async (req: Request, res: Response) => {
    try {
      const taskId = req.params.taskId as string;
      await taskManager.executeTask(taskId);

      res.json({
        success: true,
        message: `Task ${taskId} sent to agent for execution`
      });
    } catch (error: any) {
      logger.error(`Error executing task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to execute task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks/:taskId/complete
   * Mark a task as complete
   */
  taskRouter.post('/:taskId/complete', async (req: Request, res: Response) => {
    try {
      const taskId = req.params.taskId as string;
      const task = await taskManager.completeTask(taskId);

      res.json({
        success: true,
        task: serializeTask(task)
      });
    } catch (error: any) {
      logger.error(`Error completing task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to complete task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks/delegate
   * Delegate a task from one agent to another
   */
  taskRouter.post('/delegate', async (req: Request, res: Response) => {
    try {
      const {
        fromAgentId,
        toAgentId,
        title,
        description,
        priority,
        deadline,
        dependencies
      } = req.body;

      // Validate required fields
      if (!fromAgentId || !toAgentId || !title) {
        return res.status(400).json({
          success: false,
          error: 'Bad Request',
          message: 'From agent ID, to agent ID, and title are required'
        });
      }

      // Delegate task
      const task = await taskManager.delegateTask(
        fromAgentId,
        toAgentId,
        {
          title,
          description: description || '',
          status: 'todo',
          priority: priority || 'MEDIUM',
          deadline: deadline ? new Date(deadline) : new Date(),
          dependencies: dependencies || [],
          requestedBy: fromAgentId,
          progress: 0
        }
      );

      res.status(201).json({
        success: true,
        task: serializeTask(task)
      });
      return;
    } catch (error: any) {
      logger.error('Error delegating task:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to delegate task',
        message: error.message
      });
      return;
    }
  });
}
