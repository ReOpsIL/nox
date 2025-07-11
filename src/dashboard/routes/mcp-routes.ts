/**
 * MCP Routes - Dashboard routes for MCP service monitoring and management
 * Provides REST API endpoints for MCP service operations
 */

import { Router, Request, Response } from 'express';
import { ServiceManager } from '../../mcp/service-manager';
import { DockerManager } from '../../mcp/docker-manager';
import { ApprovalManager } from '../../mcp/approval-manager';
import { CapabilityRegistry } from '../../mcp/capability-registry';
import { logger } from '../../utils/logger';

export function setupMCPRoutes(
  serviceManager: ServiceManager,
  dockerManager: DockerManager,
  approvalManager: ApprovalManager,
  capabilityRegistry: CapabilityRegistry
): Router {
  const router = Router();

  // Get all MCP services
  router.get('/services', async (req: Request, res: Response) => {
    try {
      const { query, category, capabilities, limit, sortBy } = req.query;
      
      const services = await serviceManager.discoverServices(
        query as string,
        {
          category: category as string,
          capabilities: capabilities ? (capabilities as string).split(',') : undefined,
          limit: limit ? parseInt(limit as string) : undefined,
          sortBy: sortBy as 'name' | 'updated' | 'created' | 'popularity'
        }
      );

      res.json({
        success: true,
        data: services,
        total: services.length
      });
    } catch (error) {
      logger.error('Failed to get MCP services:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve MCP services'
      });
    }
  });

  // Get service details
  router.get('/services/:serviceName', async (req: Request, res: Response) => {
    try {
      const { serviceName } = req.params;
      const service = await serviceManager.getServiceDetails(serviceName);

      res.json({
        success: true,
        data: service
      });
    } catch (error) {
      logger.error(`Failed to get service details for ${req.params.serviceName}:`, error);
      res.status(404).json({
        success: false,
        error: 'Service not found'
      });
    }
  });

  // Search services
  router.post('/services/search', async (req: Request, res: Response) => {
    try {
      const { capabilities, keyword, category, author, limit } = req.body;
      
      const services = await serviceManager.searchServices({
        capabilities,
        keyword,
        category,
        author,
        limit
      });

      res.json({
        success: true,
        data: services,
        total: services.length
      });
    } catch (error) {
      logger.error('Failed to search services:', error);
      res.status(500).json({
        success: false,
        error: 'Service search failed'
      });
    }
  });

  // Check service compatibility
  router.post('/services/:serviceId/compatibility', async (req: Request, res: Response) => {
    try {
      const { serviceId } = req.params;
      const { requirements } = req.body;
      
      const compatibility = await serviceManager.checkServiceCompatibility(serviceId, requirements);

      res.json({
        success: true,
        data: compatibility
      });
    } catch (error) {
      logger.error(`Failed to check compatibility for ${req.params.serviceId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Compatibility check failed'
      });
    }
  });

  // Install service
  router.post('/services/:serviceId/install', async (req: Request, res: Response) => {
    try {
      const { serviceId } = req.params;
      const { agentId, autoApprove, env, volumes } = req.body;

      const installedService = await serviceManager.installService(serviceId, agentId, {
        autoApprove,
        env,
        volumes
      });

      res.json({
        success: true,
        data: installedService
      });
    } catch (error) {
      logger.error(`Failed to install service ${req.params.serviceId}:`, error);
      res.status(500).json({
        success: false,
        error: (error as Error).message
      });
    }
  });

  // Uninstall service
  router.delete('/services/installed/:containerId', async (req: Request, res: Response) => {
    try {
      const { containerId } = req.params;
      await serviceManager.uninstallService(containerId);

      res.json({
        success: true,
        message: 'Service uninstalled successfully'
      });
    } catch (error) {
      logger.error(`Failed to uninstall service ${req.params.containerId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to uninstall service'
      });
    }
  });

  // Get installed services
  router.get('/services/installed', async (req: Request, res: Response) => {
    try {
      const { agentId } = req.query;
      const services = serviceManager.listInstalledServices(agentId as string);

      res.json({
        success: true,
        data: services,
        total: services.length
      });
    } catch (error) {
      logger.error('Failed to get installed services:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve installed services'
      });
    }
  });

  // Get container status
  router.get('/containers', async (req: Request, res: Response) => {
    try {
      const containers = await dockerManager.listContainers();

      res.json({
        success: true,
        data: containers,
        total: containers.length
      });
    } catch (error) {
      logger.error('Failed to get containers:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve container status'
      });
    }
  });

  // Container operations
  router.post('/containers/:containerId/:action', async (req: Request, res: Response) => {
    try {
      const { containerId, action } = req.params;
      const { timeout, force } = req.body;

      switch (action) {
        case 'start':
          await dockerManager.startContainer(containerId);
          break;
        case 'stop':
          await dockerManager.stopContainer(containerId, { timeout, force });
          break;
        case 'restart':
          await dockerManager.restartContainer(containerId, timeout);
          break;
        case 'pause':
          await dockerManager.pauseContainer(containerId);
          break;
        case 'unpause':
          await dockerManager.unpauseContainer(containerId);
          break;
        default:
          return res.status(400).json({
            success: false,
            error: 'Invalid action'
          });
      }

      res.json({
        success: true,
        message: `Container ${action} completed`
      });
    } catch (error) {
      logger.error(`Failed to ${req.params.action} container ${req.params.containerId}:`, error);
      res.status(500).json({
        success: false,
        error: `Failed to ${req.params.action} container`
      });
    }
  });

  // Get container logs
  router.get('/containers/:containerId/logs', async (req: Request, res: Response) => {
    try {
      const { containerId } = req.params;
      const { tail, since } = req.query;

      const logs = await dockerManager.getContainerLogs(containerId, {
        tail: tail ? parseInt(tail as string) : undefined,
        since: since ? new Date(since as string) : undefined
      });

      res.json({
        success: true,
        data: { logs }
      });
    } catch (error) {
      logger.error(`Failed to get logs for container ${req.params.containerId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve container logs'
      });
    }
  });

  // Get container health
  router.get('/containers/:containerId/health', async (req: Request, res: Response) => {
    try {
      const { containerId } = req.params;
      const health = await dockerManager.checkContainerHealth(containerId);

      res.json({
        success: true,
        data: health
      });
    } catch (error) {
      logger.error(`Failed to check health for container ${req.params.containerId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to check container health'
      });
    }
  });

  // Get approval requests
  router.get('/approvals', async (req: Request, res: Response) => {
    try {
      const { status, limit } = req.query;
      
      let approvals;
      if (status === 'pending') {
        approvals = approvalManager.getPendingApprovals();
      } else {
        approvals = approvalManager.getApprovalHistory(
          limit ? parseInt(limit as string) : undefined
        );
      }

      res.json({
        success: true,
        data: approvals,
        total: approvals.length
      });
    } catch (error) {
      logger.error('Failed to get approval requests:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve approval requests'
      });
    }
  });

  // Respond to approval
  router.post('/approvals/:requestId/respond', async (req: Request, res: Response) => {
    try {
      const { requestId } = req.params;
      const { approved, respondedBy, reason } = req.body;

      const success = await approvalManager.respondToApproval(
        requestId,
        approved,
        respondedBy,
        reason
      );

      if (success) {
        res.json({
          success: true,
          message: 'Response recorded successfully'
        });
      } else {
        res.status(404).json({
          success: false,
          error: 'Approval request not found'
        });
      }
    } catch (error) {
      logger.error(`Failed to respond to approval ${req.params.requestId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to record approval response'
      });
    }
  });

  // Get capabilities
  router.get('/capabilities', async (req: Request, res: Response) => {
    try {
      const { agentId, source } = req.query;

      let capabilities;
      if (agentId) {
        if (source === 'mcp') {
          capabilities = capabilityRegistry.getMCPCapabilities(agentId as string);
        } else {
          capabilities = capabilityRegistry.getAgentCapabilities(agentId as string);
        }
      } else if (source) {
        capabilities = capabilityRegistry.getCapabilitiesBySource(
          source as 'mcp' | 'built-in' | 'learned' | 'delegated'
        );
      } else {
        capabilities = Array.from(capabilityRegistry.getAllCapabilities().values());
      }

      res.json({
        success: true,
        data: capabilities,
        total: Array.isArray(capabilities) ? capabilities.length : 0
      });
    } catch (error) {
      logger.error('Failed to get capabilities:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve capabilities'
      });
    }
  });

  // Get capability statistics
  router.get('/capabilities/stats', async (req: Request, res: Response) => {
    try {
      const stats = capabilityRegistry.getCapabilityStatistics();

      res.json({
        success: true,
        data: stats
      });
    } catch (error) {
      logger.error('Failed to get capability statistics:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve capability statistics'
      });
    }
  });

  // Get capability conflicts
  router.get('/capabilities/conflicts', async (req: Request, res: Response) => {
    try {
      const { agentId } = req.query;
      const conflicts = capabilityRegistry.findCapabilityConflicts(agentId as string);

      res.json({
        success: true,
        data: conflicts,
        total: conflicts.length
      });
    } catch (error) {
      logger.error('Failed to get capability conflicts:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to retrieve capability conflicts'
      });
    }
  });

  return router;
}