/**
 * Task Routes - API endpoints for task management and visualization
 */

import { Router } from 'express';
import { TaskManager } from '../../core/task-manager';
import { logger } from '../../utils/logger';
import { Task, TaskStatus, TaskPriority } from '../../types';

/**
 * Set up task routes
 */
export function setupTaskRoutes(router: Router, taskManager: TaskManager): void {
  const taskRouter = Router();
  router.use('/tasks', taskRouter);

  /**
   * GET /api/tasks
   * Get all tasks with optional filtering
   */
  taskRouter.get('/', async (req, res) => {
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
      
      res.json({
        success: true,
        tasks
      });
    } catch (error) {
      logger.error('Error getting tasks:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get tasks',
        message: error.message
      });
    }
  });

  /**
   * GET /api/tasks/dashboard
   * Get task dashboard overview
   */
  taskRouter.get('/dashboard', async (req, res) => {
    try {
      const dashboard = await taskManager.getTaskDashboard();
      res.json({
        success: true,
        dashboard
      });
    } catch (error) {
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
  taskRouter.get('/blocked', async (req, res) => {
    try {
      const blockedTasks = await taskManager.getBlockedTasks();
      res.json({
        success: true,
        tasks: blockedTasks
      });
    } catch (error) {
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
  taskRouter.get('/:taskId', async (req, res) => {
    try {
      const taskId = req.params.taskId;
      const task = await taskManager.getTask(taskId);
      
      if (!task) {
        return res.status(404).json({
          success: false,
          error: 'Task not found',
          message: `Task ${taskId} not found`
        });
      }
      
      res.json({
        success: true,
        task
      });
    } catch (error) {
      logger.error(`Error getting task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks
   * Create a new task
   */
  taskRouter.post('/', async (req, res) => {
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
        deadline: deadline ? new Date(deadline) : undefined,
        dependencies: dependencies || [],
        requestedBy: requestedBy || 'user',
        progress: 0
      });
      
      res.status(201).json({
        success: true,
        task
      });
    } catch (error) {
      logger.error('Error creating task:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to create task',
        message: error.message
      });
    }
  });

  /**
   * PUT /api/tasks/:taskId
   * Update a task
   */
  taskRouter.put('/:taskId', async (req, res) => {
    try {
      const taskId = req.params.taskId;
      const updates = req.body;
      
      // Convert deadline to Date object if provided
      if (updates.deadline) {
        updates.deadline = new Date(updates.deadline);
      }
      
      // Update task
      const task = await taskManager.updateTask(taskId, updates);
      
      res.json({
        success: true,
        task
      });
    } catch (error) {
      logger.error(`Error updating task ${req.params.taskId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to update task',
        message: error.message
      });
    }
  });

  /**
   * POST /api/tasks/:taskId/complete
   * Mark a task as complete
   */
  taskRouter.post('/:taskId/complete', async (req, res) => {
    try {
      const taskId = req.params.taskId;
      const task = await taskManager.completeTask(taskId);
      
      res.json({
        success: true,
        task
      });
    } catch (error) {
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
  taskRouter.post('/delegate', async (req, res) => {
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
          deadline: deadline ? new Date(deadline) : undefined,
          dependencies: dependencies || [],
          progress: 0
        }
      );
      
      res.status(201).json({
        success: true,
        task
      });
    } catch (error) {
      logger.error('Error delegating task:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to delegate task',
        message: error.message
      });
    }
  });
}