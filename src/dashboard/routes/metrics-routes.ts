/**
 * Metrics Routes - API endpoints for system and agent metrics
 */

import { Router } from 'express';
import { MetricsManager } from '../../monitoring/metrics';
import { logger } from '../../utils/logger';

/**
 * Set up metrics routes
 */
export function setupMetricsRoutes(router: Router, metricsManager: MetricsManager): void {
  const metricsRouter = Router();
  router.use('/metrics', metricsRouter);

  /**
   * GET /api/metrics/system
   * Get system metrics with optional time range and interval
   */
  metricsRouter.get('/system', async (req, res) => {
    try {
      // Parse query parameters
      const startTime = req.query.startTime ? new Date(req.query.startTime as string) : undefined;
      const endTime = req.query.endTime ? new Date(req.query.endTime as string) : undefined;
      const interval = req.query.interval as 'minute' | 'hour' | 'day' | undefined;
      
      // Get system metrics
      const metrics = metricsManager.getSystemMetrics(startTime, endTime, interval);
      
      res.json({
        success: true,
        metrics
      });
    } catch (error) {
      logger.error('Error getting system metrics:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get system metrics',
        message: error.message
      });
    }
  });

  /**
   * GET /api/metrics/system/latest
   * Get the latest system metrics
   */
  metricsRouter.get('/system/latest', async (req, res) => {
    try {
      const metrics = metricsManager.getLatestSystemMetrics();
      
      if (!metrics) {
        return res.status(404).json({
          success: false,
          error: 'No metrics available',
          message: 'No system metrics have been collected yet'
        });
      }
      
      res.json({
        success: true,
        metrics
      });
    } catch (error) {
      logger.error('Error getting latest system metrics:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get latest system metrics',
        message: error.message
      });
    }
  });

  /**
   * GET /api/metrics/agents/:agentId
   * Get metrics for a specific agent with optional time range and interval
   */
  metricsRouter.get('/agents/:agentId', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      
      // Parse query parameters
      const startTime = req.query.startTime ? new Date(req.query.startTime as string) : undefined;
      const endTime = req.query.endTime ? new Date(req.query.endTime as string) : undefined;
      const interval = req.query.interval as 'minute' | 'hour' | 'day' | undefined;
      
      // Get agent metrics
      const metrics = metricsManager.getAgentMetrics(agentId, startTime, endTime, interval);
      
      res.json({
        success: true,
        metrics
      });
    } catch (error) {
      logger.error(`Error getting metrics for agent ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agent metrics',
        message: error.message
      });
    }
  });

  /**
   * GET /api/metrics/agents/:agentId/latest
   * Get the latest metrics for a specific agent
   */
  metricsRouter.get('/agents/:agentId/latest', async (req, res) => {
    try {
      const agentId = req.params.agentId;
      const metrics = metricsManager.getLatestAgentMetrics(agentId);
      
      if (!metrics) {
        return res.status(404).json({
          success: false,
          error: 'No metrics available',
          message: `No metrics have been collected for agent ${agentId} yet`
        });
      }
      
      res.json({
        success: true,
        metrics
      });
    } catch (error) {
      logger.error(`Error getting latest metrics for agent ${req.params.agentId}:`, error);
      res.status(500).json({
        success: false,
        error: 'Failed to get latest agent metrics',
        message: error.message
      });
    }
  });

  /**
   * GET /api/metrics/agents
   * Get metrics for all agents (latest only)
   */
  metricsRouter.get('/agents', async (req, res) => {
    try {
      // This is a simplified implementation - in a real system,
      // you would implement a more efficient way to get metrics for all agents
      
      // Get all agent IDs from the metrics manager
      // For now, we'll just use a workaround by checking the latest system metrics
      const latestSystemMetrics = metricsManager.getLatestSystemMetrics();
      
      if (!latestSystemMetrics) {
        return res.status(404).json({
          success: false,
          error: 'No metrics available',
          message: 'No system metrics have been collected yet'
        });
      }
      
      // Get latest metrics for each agent
      const agentMetrics = {};
      
      // This is a placeholder - in a real implementation, you would have a way
      // to get all agent IDs from the metrics manager
      const agentIds: string[] = []; // This should be populated with actual agent IDs
      
      for (const agentId of agentIds) {
        const metrics = metricsManager.getLatestAgentMetrics(agentId);
        if (metrics) {
          agentMetrics[agentId] = metrics;
        }
      }
      
      res.json({
        success: true,
        metrics: agentMetrics
      });
    } catch (error) {
      logger.error('Error getting metrics for all agents:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to get agent metrics',
        message: error.message
      });
    }
  });

  /**
   * POST /api/metrics/collect
   * Trigger metrics collection manually
   */
  metricsRouter.post('/collect', async (req, res) => {
    try {
      await metricsManager.collectMetrics();
      
      const latestMetrics = metricsManager.getLatestSystemMetrics();
      
      res.json({
        success: true,
        message: 'Metrics collection triggered successfully',
        metrics: latestMetrics
      });
    } catch (error) {
      logger.error('Error triggering metrics collection:', error);
      res.status(500).json({
        success: false,
        error: 'Failed to trigger metrics collection',
        message: error.message
      });
    }
  });
}