/**
 * Agent Routes - API endpoints for agent management and monitoring
 */

import { Router } from 'express';
import { AgentManager } from '../../core/agent-manager';
import { logger } from '../../utils/logger';

/**
 * Set up agent routes
 */
export function setupAgentRoutes(router: Router, agentManager: AgentManager): void {
  const agentRouter = Router();
  router.use('/agents', agentRouter);

  /**
   * GET /api/agents
   * Get all agents
   */
  agentRouter.get('/', async (req, res) => {
    try {
      const agents = await agentManager.listRunningAgents();
      res.json({
        success: true,
        agents: agents.map(agent => ({
          id: agent.id,
          status: agent.status,
          startTime: agent.startTime,
          restartCount: agent.restartCount,
          memoryUsage: agent.memoryUsage,
          cpuUsage: agent.cpuUsage,
          lastHealthCheck: agent.lastHealthCheck
        }))
      });
    } catch (error) {
      logger.error('Error getting agents:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agents',
        message: error.message
      });
    }
  });

  /**
   * GET /api/agents/:agentId
   * Get a specific agent
   */
  agentRouter.get('/:agentId', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const agent = await agentManager.getAgentStatus(agentId);
      
      if (!agent) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found`
        });
      }
      
      res.json({
        success: true,
        agent: {
          id: agent.id,
          status: agent.status,
          startTime: agent.startTime,
          restartCount: agent.restartCount,
          memoryUsage: agent.memoryUsage,
          cpuUsage: agent.cpuUsage,
          lastHealthCheck: agent.lastHealthCheck
        }
      });
    } catch (error) {
      logger.error(`Error getting agent ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agent',
        message: error.message
      });
    }
  });

  /**
   * POST /api/agents/:agentId/message
   * Send a message to an agent
   */
  agentRouter.post('/:agentId/message', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const { message } = req.body;
      
      if (!message) {
        return res.status(400).json({
          success: false,
          error: 'Bad Request',
          message: 'Message content is required'
        });
      }
      
      const response = await agentManager.sendMessageToAgent(agentId, message);
      
      res.json({
        success: true,
        response
      });
    } catch (error) {
      logger.error(`Error sending message to agent ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to send message',
        message: error.message
      });
    }
  });

  /**
   * POST /api/agents/:agentId/restart
   * Restart an agent
   */
  agentRouter.post('/:agentId/restart', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const agent = await agentManager.getAgentStatus(agentId);
      
      if (!agent) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found`
        });
      }
      
      // Note: This is a simplified implementation. In a real system,
      // you would need to get the agent config from a registry before restarting.
      // For now, we'll just assume the agent can be restarted with its current config.
      await agentManager.restartAgent(agentId, { id: agentId } as any);
      
      res.json({
        success: true,
        message: `Agent ${agentId} restarted successfully`
      });
    } catch (error) {
      logger.error(`Error restarting agent ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to restart agent',
        message: error.message
      });
    }
  });

  /**
   * GET /api/agents/:agentId/health
   * Get agent health status
   */
  agentRouter.get('/:agentId/health', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const agent = await agentManager.getAgentStatus(agentId);
      
      if (!agent) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found`
        });
      }
      
      // Get Claude interface health
      const claudeInterface = agentManager.getClaudeInterface(agentId);
      const claudeHealth = claudeInterface ? claudeInterface.getHealthStatus() : null;
      
      // Get process monitor metrics
      const processMonitor = agentManager.getProcessMonitor();
      const processMetrics = processMonitor.getProcessMetrics(agentId);
      
      res.json({
        success: true,
        health: {
          agent: {
            id: agent.id,
            status: agent.status,
            lastHealthCheck: agent.lastHealthCheck
          },
          claude: claudeHealth,
          process: processMetrics
        }
      });
    } catch (error) {
      logger.error(`Error getting agent health ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agent health',
        message: error.message
      });
    }
  });

  /**
   * GET /api/agents/:agentId/conversation
   * Get agent conversation history
   */
  agentRouter.get('/:agentId/conversation', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const claudeInterface = agentManager.getClaudeInterface(agentId);
      
      if (!claudeInterface) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found or not running`
        });
      }
      
      const conversation = claudeInterface.getConversationHistory();
      
      res.json({
        success: true,
        conversation
      });
    } catch (error) {
      logger.error(`Error getting agent conversation ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agent conversation',
        message: error.message
      });
    }
  });
}