/**
 * System Routes - API endpoints for system information and management
 */

import { Router } from 'express';
import * as os from 'os';
import { AgentManager } from '../../core/agent-manager';
import { MessageBroker } from '../../core/message-broker';
import { TaskManager } from '../../core/task-manager';
import { logger } from '../../utils/logger';

/**
 * Set up system routes
 */
export function setupSystemRoutes(
  router: Router,
  agentManager: AgentManager,
  messageBroker: MessageBroker,
  taskManager: TaskManager
): void {
  const systemRouter = Router();
  router.use('/system', systemRouter);

  /**
   * GET /api/system/info
   * Get system information
   */
  systemRouter.get('/info', async (req, res) => {
    try {
      const info = {
        hostname: os.hostname(),
        platform: os.platform(),
        arch: os.arch(),
        cpus: os.cpus().length,
        memory: {
          total: os.totalmem(),
          free: os.freemem(),
          used: os.totalmem() - os.freemem()
        },
        uptime: os.uptime(),
        loadAverage: os.loadavg(),
        nodeVersion: process.version,
        processUptime: process.uptime(),
        processMemoryUsage: process.memoryUsage()
      };
      
      res.json({
        success: true,
        info
      });
    } catch (error) {
      logger.error('Error getting system info:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system info',
        message: error.message
      });
    }
  });

  /**
   * GET /api/system/status
   * Get system status
   */
  systemRouter.get('/status', async (req, res) => {
    try {
      // Get agent status
      const agents = await agentManager.listRunningAgents();
      const agentCount = agents.length;
      const activeAgentCount = agents.filter(a => a.status === 'running').length;
      
      // Get message broker status
      const messageStats = messageBroker.getStats();
      
      // Get task status
      const taskDashboard = await taskManager.getTaskDashboard();
      
      // Calculate overall system health
      const systemHealth = calculateSystemHealth(
        activeAgentCount / Math.max(agentCount, 1),
        messageStats.queueSize,
        taskDashboard.blocked
      );
      
      res.json({
        success: true,
        status: {
          health: systemHealth,
          agents: {
            total: agentCount,
            active: activeAgentCount,
            inactive: agentCount - activeAgentCount
          },
          messages: {
            queueSize: messageStats.queueSize,
            subscriberCount: messageStats.subscriberCount,
            totalHistoryMessages: messageStats.totalHistoryMessages
          },
          tasks: {
            total: taskDashboard.total,
            todo: taskDashboard.byStatus.todo,
            inProgress: taskDashboard.byStatus.inprogress,
            done: taskDashboard.byStatus.done,
            blocked: taskDashboard.blocked
          },
          system: {
            memoryUsage: (os.totalmem() - os.freemem()) / os.totalmem() * 100,
            cpuCount: os.cpus().length,
            uptime: os.uptime()
          }
        }
      });
    } catch (error) {
      logger.error('Error getting system status:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system status',
        message: error.message
      });
    }
  });

  /**
   * POST /api/system/shutdown
   * Shutdown the system
   */
  systemRouter.post('/shutdown', async (req, res) => {
    try {
      // This is a placeholder - in a real system, you would implement
      // a proper shutdown sequence with authentication and authorization
      
      // For now, we'll just return a message
      res.json({
        success: true,
        message: 'System shutdown initiated',
        warning: 'This is a placeholder - no actual shutdown is performed'
      });
      
      // In a real implementation, you would do something like:
      // await agentManager.shutdown();
      // await messageBroker.shutdown();
      // await taskManager.shutdown();
      // process.exit(0);
    } catch (error) {
      logger.error('Error shutting down system:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to shutdown system',
        message: error.message
      });
    }
  });

  /**
   * POST /api/system/restart
   * Restart the system
   */
  systemRouter.post('/restart', async (req, res) => {
    try {
      // This is a placeholder - in a real system, you would implement
      // a proper restart sequence with authentication and authorization
      
      // For now, we'll just return a message
      res.json({
        success: true,
        message: 'System restart initiated',
        warning: 'This is a placeholder - no actual restart is performed'
      });
      
      // In a real implementation, you would do something like:
      // await agentManager.shutdown();
      // await messageBroker.shutdown();
      // await taskManager.shutdown();
      // process.on('exit', () => {
      //   require('child_process').spawn(process.argv[0], process.argv.slice(1), {
      //     detached: true,
      //     stdio: ['ignore', 'ignore', 'ignore']
      //   }).unref();
      // });
      // process.exit(0);
    } catch (error) {
      logger.error('Error restarting system:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to restart system',
        message: error.message
      });
    }
  });

  /**
   * GET /api/system/logs
   * Get system logs
   */
  systemRouter.get('/logs', async (req, res) => {
    try {
      // This is a placeholder - in a real system, you would implement
      // a proper log retrieval mechanism
      
      // For now, we'll just return a message
      res.json({
        success: true,
        message: 'Log retrieval not implemented',
        logs: []
      });
      
      // In a real implementation, you would do something like:
      // const logs = await logManager.getLogs({
      //   level: req.query.level,
      //   startTime: req.query.startTime ? new Date(req.query.startTime) : undefined,
      //   endTime: req.query.endTime ? new Date(req.query.endTime) : undefined,
      //   limit: parseInt(req.query.limit) || 100
      // });
      // res.json({ success: true, logs });
    } catch (error) {
      logger.error('Error getting system logs:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system logs',
        message: error.message
      });
    }
  });
}

/**
 * Calculate system health score (0-100)
 */
function calculateSystemHealth(
  agentHealthRatio: number,
  messageQueueSize: number,
  blockedTasks: number
): number {
  // This is a simplified health calculation - in a real system,
  // you would implement a more sophisticated algorithm
  
  // Agent health (0-40 points)
  const agentHealth = agentHealthRatio * 40;
  
  // Message queue health (0-30 points)
  // Lower score as queue size increases
  const queueHealth = Math.max(0, 30 - Math.min(30, messageQueueSize / 10));
  
  // Task health (0-30 points)
  // Lower score as blocked tasks increase
  const taskHealth = Math.max(0, 30 - Math.min(30, blockedTasks * 3));
  
  // Overall health score
  const healthScore = agentHealth + queueHealth + taskHealth;
  
  return Math.round(healthScore);
}