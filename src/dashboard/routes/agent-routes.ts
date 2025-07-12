/**
 * Agent Routes - API endpoints for agent management and monitoring
 */

import { Router, Request, Response } from 'express';
import { AgentManager } from '../../core/agent-manager';
import { RegistryManager } from '../../core/registry-manager';
import { DEFAULT_RESOURCE_LIMITS } from '../../types/agent';
import { logger } from '../../utils/logger';

/**
 * Set up agent routes
 */
export function setupAgentRoutes(router: Router, agentManager: AgentManager, registryManager: RegistryManager): void {
  const agentRouter = Router();
  router.use('/agents', agentRouter);

  /**
   * GET /api/agents
   * Get all agents
   */
  agentRouter.get('/', async (_req: Request, res: Response) => {
    try {
      // Get agent configurations from registry
      const registeredAgents = await registryManager.listAgents();
      
      // Get running agent processes
      const runningAgents = await agentManager.listRunningAgents();
      
      // Create a map of running agents for quick lookup
      const runningAgentsMap = new Map(runningAgents.map(agent => [agent.id, agent]));
      
      // Merge registered agent configs with runtime data
      const mergedAgents = registeredAgents.map(config => {
        const runtimeData = runningAgentsMap.get(config.id);
        
        return {
          id: config.id,
          name: config.name,
          description: config.systemPrompt || `Agent ${config.name}`,
          status: runtimeData ? runtimeData.status : config.status,
          capabilities: config.capabilities || [],
          createdAt: config.createdAt.toISOString(),
          lastActiveAt: runtimeData ? runtimeData.lastHealthCheck.toISOString() : config.lastModified.toISOString(),
          tasksCompleted: 0, // TODO: Get from task manager
          currentTask: null, // TODO: Get current task from task manager
          startTime: runtimeData ? runtimeData.startTime.toISOString() : config.createdAt.toISOString(),
          restartCount: runtimeData ? runtimeData.restartCount : 0,
          memoryUsage: runtimeData ? runtimeData.memoryUsage : 0,
          cpuUsage: runtimeData ? runtimeData.cpuUsage : 0,
          lastHealthCheck: runtimeData ? runtimeData.lastHealthCheck.toISOString() : config.lastModified.toISOString()
        };
      });
      
      return res.json({
        success: true,
        agents: mergedAgents
      });
    } catch (error: any) {
      logger.error('Error getting agents:', error);
      return res.status(500).json({
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
  agentRouter.get('/:agentId', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
      const agent = await agentManager.getAgentStatus(agentId);

      if (!agent) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found`
        });
      }

      return res.json({
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
    } catch (error: any) {
      logger.error(`Error getting agent ${req.params.agentId}:`, error);
      return res.status(500).json({
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
  agentRouter.post('/:agentId/message', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
      const { message } = req.body;

      if (!message) {
        return res.status(400).json({
          success: false,
          error: 'Bad Request',
          message: 'Message content is required'
        });
      }

      const response = await agentManager.sendMessageToAgent(agentId, message);

      return res.json({
        success: true,
        response
      });
    } catch (error: any) {
      logger.error(`Error sending message to agent ${req.params.agentId}:`, error);
      return res.status(500).json({
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
  agentRouter.post('/:agentId/restart', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
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

      return res.json({
        success: true,
        message: `Agent ${agentId} restarted successfully`
      });
    } catch (error: any) {
      logger.error(`Error restarting agent ${req.params.agentId}:`, error);
      return res.status(500).json({
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
  agentRouter.get('/:agentId/health', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
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

      return res.json({
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
    } catch (error: any) {
      logger.error(`Error getting agent health ${req.params.agentId}:`, error);
      return res.status(500).json({
        success: false,
        error: 'Failed to get agent health',
        message: error.message
      });
    }
  });

  /**
   * POST /api/agents
   * Create a new agent
   */
  agentRouter.post('/', async (req: Request, res: Response) => {
    try {
      const { name, description, capabilities } = req.body;

      if (!name) {
        return res.status(400).json({
          success: false,
          error: 'Bad Request',
          message: 'Agent name is required'
        });
      }

      // Create agent configuration
      const agentConfig = {
        id: `agent-${Date.now()}`, // Generate unique ID
        name: name,
        systemPrompt: description || `Agent ${name}`,
        status: 'inactive' as const,
        createdAt: new Date(),
        lastModified: new Date(),
        resourceLimits: DEFAULT_RESOURCE_LIMITS,
        capabilities: Array.isArray(capabilities) ? capabilities : 
                     (typeof capabilities === 'string' ? capabilities.split(',').map(c => c.trim()) : []),
        relationships: [],
        mcpServices: []
      };

      // Add agent to registry
      const createdAgent = await registryManager.createAgent(agentConfig);

      return res.json({
        success: true,
        message: `Agent ${name} created successfully`,
        agent: {
          id: createdAgent.id,
          name: createdAgent.name,
          description: createdAgent.systemPrompt,
          status: createdAgent.status,
          capabilities: createdAgent.capabilities,
          createdAt: createdAgent.createdAt.toISOString()
        }
      });
    } catch (error: any) {
      logger.error('Error creating agent:', error);
      return res.status(500).json({
        success: false,
        error: 'Failed to create agent',
        message: error.message
      });
    }
  });

  /**
   * DELETE /api/agents/:agentId
   * Delete an agent
   */
  agentRouter.delete('/:agentId', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
      
      // Check if agent exists in registry
      const agents = await registryManager.listAgents();
      const agent = agents.find(a => a.id === agentId);
      
      if (!agent) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found`
        });
      }

      // Stop the agent if it's running
      try {
        await agentManager.killAgent(agentId);
        logger.info(`Stopped running agent: ${agentId}`);
      } catch (error) {
        // Agent might not be running, continue with deletion
        logger.debug(`Agent ${agentId} was not running: ${error}`);
      }

      // Remove agent from registry
      await registryManager.deleteAgent(agentId);
      logger.info(`Deleted agent from registry: ${agentId}`);

      return res.json({
        success: true,
        message: `Agent ${agentId} deleted successfully`
      });
    } catch (error: any) {
      logger.error(`Error deleting agent ${req.params.agentId}:`, error);
      return res.status(500).json({
        success: false,
        error: 'Failed to delete agent',
        message: error.message
      });
    }
  });

  /**
   * GET /api/agents/:agentId/conversation
   * Get agent conversation history
   */
  agentRouter.get('/:agentId/conversation', async (req: Request, res: Response) => {
    try {
      const agentId = req.params.agentId as string;
      const claudeInterface = agentManager.getClaudeInterface(agentId);

      if (!claudeInterface) {
        return res.status(404).json({
          success: false,
          error: 'Agent not found',
          message: `Agent ${agentId} not found or not running`
        });
      }

      const conversation = claudeInterface.getConversationHistory();

      return res.json({
        success: true,
        conversation
      });
    } catch (error: any) {
      logger.error(`Error getting agent conversation ${req.params.agentId}:`, error);
      return res.status(500).json({
        success: false,
        error: 'Failed to get agent conversation',
        message: error.message
      });
    }
  });
}
