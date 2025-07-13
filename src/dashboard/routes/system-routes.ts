/**
 * System Routes - API endpoints for system information and management
 */

import { Router, Request, Response } from 'express';
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
  systemRouter.get('/info', async (_req: Request, res: Response) => {
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
    } catch (error: any) {
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
  systemRouter.get('/status', async (_req: Request, res: Response) => {
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
    } catch (error: any) {
      logger.error('Error getting system status:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system status',
        message: error.message
      });
    }
  });

  /**
   * GET /api/system/config
   * Get system configuration
   */
  systemRouter.get('/config', async (_req: Request, res: Response) => {
    try {
      const config = {
        server: {
          dashboardPort: 3001,
          websocketPort: 3000
        },
        agents: {
          maxConcurrent: 10,
          defaultResourceLimits: {
            memory: '512MB',
            cpu: '1 core'
          }
        },
        tasks: {
          maxRetries: 3,
          defaultTimeout: 300000,
          cleanupInterval: 60000
        },
        logging: {
          level: 'info',
          enableFileLogging: true
        },
        features: {
          autoRefresh: true,
          taskOutputDisplay: true,
          cascadeDelete: true
        }
      };

      res.json({
        success: true,
        config
      });
    } catch (error: any) {
      logger.error('Error getting system config:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system config',
        message: error.message
      });
    }
  });

  /**
   * POST /api/system/shutdown
   * Shutdown the system
   */
  systemRouter.post('/shutdown', async (_req: Request, res: Response) => {
    try {
      logger.info('System shutdown initiated via API');
      
      // Send response immediately before shutdown
      res.json({
        success: true,
        message: 'System shutdown initiated - all components will be stopped'
      });

      // Perform graceful shutdown with proper cleanup sequence
      setTimeout(async () => {
        try {
          logger.info('Starting graceful shutdown sequence...');
          
          // Stop all agents
          await agentManager.shutdown();
          logger.info('AgentManager shutdown complete');
          
          // Stop message broker
          await messageBroker.shutdown();
          logger.info('MessageBroker shutdown complete');
          
          // Stop task manager
          await taskManager.shutdown();
          logger.info('TaskManager shutdown complete');
          
          logger.info('Graceful shutdown complete');
          process.exit(0);
        } catch (shutdownError) {
          logger.error('Error during shutdown:', shutdownError);
          process.exit(1);
        }
      }, 1000); // Give response time to send
      
    } catch (error: any) {
      logger.error('Error initiating system shutdown:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to initiate system shutdown',
        message: error.message
      });
    }
  });

  /**
   * POST /api/system/restart
   * Restart the system
   */
  systemRouter.post('/restart', async (_req: Request, res: Response) => {
    try {
      logger.info('System restart initiated via API');
      
      // Send response immediately before restart
      res.json({
        success: true,
        message: 'System restart initiated - shutting down and restarting'
      });

      // Perform graceful restart with proper cleanup and respawn
      setTimeout(async () => {
        try {
          logger.info('Starting graceful restart sequence...');
          
          // Set up process respawn before shutdown
          const { spawn } = require('child_process');
          process.on('exit', () => {
            logger.info('Respawning process...');
            spawn(process.argv[0], process.argv.slice(1), {
              detached: true,
              stdio: ['ignore', 'ignore', 'ignore']
            }).unref();
          });
          
          // Stop all components
          await agentManager.shutdown();
          logger.info('AgentManager shutdown complete');
          
          await messageBroker.shutdown();
          logger.info('MessageBroker shutdown complete');
          
          await taskManager.shutdown();
          logger.info('TaskManager shutdown complete');
          
          logger.info('Restart shutdown complete, process will respawn');
          process.exit(0);
        } catch (restartError) {
          logger.error('Error during restart:', restartError);
          process.exit(1);
        }
      }, 1000); // Give response time to send
      
    } catch (error: any) {
      logger.error('Error initiating system restart:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to initiate system restart',
        message: error.message
      });
    }
  });

  /**
   * GET /api/system/logs
   * Get system logs
   */
  systemRouter.get('/logs', async (req: Request, res: Response) => {
    try {
      const fs = require('fs').promises;
      const path = require('path');
      
      // Parse query parameters
      const level = req.query.level as string;
      const startTime = req.query.startTime ? new Date(req.query.startTime as string) : undefined;
      const endTime = req.query.endTime ? new Date(req.query.endTime as string) : undefined;
      const limit = parseInt(req.query.limit as string) || 100;
      
      // Read log file
      const logFilePath = '/tmp/nox.log';
      let logContent = '';
      
      try {
        logContent = await fs.readFile(logFilePath, 'utf-8');
      } catch (readError) {
        // If main log file doesn't exist, try alternative locations
        const altPaths = [
          path.join(process.cwd(), 'nox.log'),
          path.join(process.cwd(), 'logs', 'nox.log'),
          path.join(__dirname, '../../../nox.log')
        ];
        
        for (const altPath of altPaths) {
          try {
            logContent = await fs.readFile(altPath, 'utf-8');
            break;
          } catch {
            continue;
          }
        }
      }
      
      // Parse log lines
      const logLines = logContent.split('\n').filter(line => line.trim().length > 0);
      const logs = [];
      
      for (const line of logLines) {
        // Parse log entry (assuming format: [timestamp] LEVEL message)
        const match = line.match(/^\[(.*?)\]\s+(\w+)\s+(.*)$/);
        if (match) {
          const [, timestamp, logLevel, message] = match;
          if (timestamp && logLevel && message) {
            const logTime = new Date(timestamp);
            
            // Filter by level if specified
            if (level && logLevel.toLowerCase() !== level.toLowerCase()) {
              continue;
            }
            
            // Filter by time range if specified
            if (startTime && logTime < startTime) continue;
            if (endTime && logTime > endTime) continue;
            
            logs.push({
              timestamp: logTime,
              level: logLevel,
              message: message.trim()
            });
          }
        } else {
          // Handle lines that don't match the expected format
          logs.push({
            timestamp: new Date(),
            level: 'INFO',
            message: line.trim()
          });
        }
      }
      
      // Sort by timestamp (newest first) and limit
      const sortedLogs = logs
        .sort((a, b) => b.timestamp.getTime() - a.timestamp.getTime())
        .slice(0, limit);
      
      res.json({
        success: true,
        logs: sortedLogs,
        totalCount: logs.length,
        displayedCount: sortedLogs.length
      });
      
    } catch (error: any) {
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
