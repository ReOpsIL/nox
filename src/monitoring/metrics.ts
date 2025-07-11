/**
 * Metrics Manager - Collects and manages system performance metrics
 * Provides methods for tracking, storing, and retrieving metrics
 */

import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as os from 'os';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';
import { AgentManager } from '../core/agent-manager';
import { MessageBroker } from '../core/message-broker';
import { TaskManager } from '../core/task-manager';

export interface SystemMetrics {
  timestamp: Date;
  system: {
    cpuUsage: number;
    memoryUsage: number;
    memoryTotal: number;
    uptime: number;
    loadAverage: number[];
  };
  agents: {
    total: number;
    active: number;
    inactive: number;
    error: number;
  };
  messages: {
    queueSize: number;
    processedPerMinute: number;
    totalProcessed: number;
  };
  tasks: {
    total: number;
    todo: number;
    inProgress: number;
    done: number;
    blocked: number;
  };
}

export interface AgentMetrics {
  agentId: string;
  timestamp: Date;
  status: string;
  cpuUsage: number;
  memoryUsage: number;
  messagesSent: number;
  messagesReceived: number;
  tasksCreated: number;
  tasksCompleted: number;
  uptime: number;
  lastActivity: Date;
}

/**
 * Metrics Manager - Collects and manages system performance metrics
 */
export class MetricsManager extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private metricsDir: string;
  private systemMetricsHistory: SystemMetrics[] = [];
  private agentMetricsHistory: Map<string, AgentMetrics[]> = new Map();
  private collectionInterval: NodeJS.Timeout | null = null;
  private retentionPeriodDays = 7; // Default: keep metrics for 7 days
  private collectionIntervalMs = 60000; // Default: collect metrics every minute
  private maxDataPoints = 10080; // Default: 7 days of minute-by-minute data (7*24*60)
  private messageCountsByMinute: Map<string, number> = new Map();
  private lastMessageCount = 0;
  private totalMessagesProcessed = 0;

  constructor(
    private agentManager: AgentManager,
    private messageBroker: MessageBroker,
    private taskManager: TaskManager,
    workingDir: string
  ) {
    super();
    this.workingDir = workingDir;
    this.metricsDir = path.join(workingDir, 'metrics');
  }

  /**
   * Initialize the metrics manager
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('MetricsManager already initialized');
      return;
    }

    try {
      // Ensure metrics directory exists
      await fs.mkdir(this.metricsDir, { recursive: true });

      // Set configuration
      if (config.metrics?.retentionPeriodDays) {
        this.retentionPeriodDays = config.metrics.retentionPeriodDays;
      }

      if (config.metrics?.collectionIntervalMs) {
        this.collectionIntervalMs = config.metrics.collectionIntervalMs;
      }

      if (config.metrics?.maxDataPoints) {
        this.maxDataPoints = config.metrics.maxDataPoints;
      }

      // Load existing metrics
      await this.loadMetrics();

      // Set up event listeners
      this.setupEventListeners();

      this.initialized = true;
      logger.info('MetricsManager initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize MetricsManager:', error);
      throw error;
    }
  }

  /**
   * Start the metrics manager
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('MetricsManager not initialized');
    }

    // Start metrics collection
    this.collectionInterval = setInterval(() => this.collectMetrics(), this.collectionIntervalMs);

    logger.info('MetricsManager started');
    this.emit('started');
  }

  /**
   * Shutdown the metrics manager
   */
  async shutdown(): Promise<void> {
    // Stop metrics collection
    if (this.collectionInterval) {
      clearInterval(this.collectionInterval);
      this.collectionInterval = null;
    }

    // Save metrics
    await this.saveMetrics();

    logger.info('MetricsManager shutdown');
    this.emit('shutdown');
  }

  /**
   * Collect system and agent metrics
   */
  async collectMetrics(): Promise<void> {
    if (!this.initialized) {
      return;
    }

    try {
      // Collect system metrics
      const systemMetrics = await this.collectSystemMetrics();
      this.systemMetricsHistory.push(systemMetrics);

      // Trim system metrics history if needed
      if (this.systemMetricsHistory.length > this.maxDataPoints) {
        this.systemMetricsHistory = this.systemMetricsHistory.slice(-this.maxDataPoints);
      }

      // Collect agent metrics
      const agents = await this.agentManager.listRunningAgents();
      for (const agent of agents) {
        const agentMetrics = await this.collectAgentMetrics(agent.id);
        
        // Get or create agent metrics history
        if (!this.agentMetricsHistory.has(agent.id)) {
          this.agentMetricsHistory.set(agent.id, []);
        }
        
        const history = this.agentMetricsHistory.get(agent.id)!;
        history.push(agentMetrics);
        
        // Trim agent metrics history if needed
        if (history.length > this.maxDataPoints) {
          this.agentMetricsHistory.set(agent.id, history.slice(-this.maxDataPoints));
        }
      }

      // Save metrics periodically (every 10 collections)
      if (this.systemMetricsHistory.length % 10 === 0) {
        await this.saveMetrics();
      }

      // Emit metrics collected event
      this.emit('metrics-collected', systemMetrics);

    } catch (error) {
      logger.error('Failed to collect metrics:', error);
    }
  }

  /**
   * Collect system metrics
   */
  private async collectSystemMetrics(): Promise<SystemMetrics> {
    // Get CPU usage
    const cpuUsage = await this.getCpuUsage();

    // Get memory usage
    const memTotal = os.totalmem();
    const memFree = os.freemem();
    const memUsed = memTotal - memFree;
    const memUsage = (memUsed / memTotal) * 100;

    // Get agent stats
    const agents = await this.agentManager.listRunningAgents();
    const agentStats = {
      total: agents.length,
      active: agents.filter(a => a.status === 'running').length,
      inactive: agents.filter(a => a.status === 'stopped').length,
      error: agents.filter(a => a.status !== 'running' && a.status !== 'stopped').length
    };

    // Get message stats
    const messageStats = this.messageBroker.getStats();
    const messagesProcessedPerMinute = this.calculateMessagesPerMinute();

    // Get task stats
    const taskDashboard = await this.taskManager.getTaskDashboard();

    return {
      timestamp: new Date(),
      system: {
        cpuUsage,
        memoryUsage: memUsage,
        memoryTotal: memTotal,
        uptime: os.uptime(),
        loadAverage: os.loadavg()
      },
      agents: agentStats,
      messages: {
        queueSize: messageStats.queueSize,
        processedPerMinute: messagesProcessedPerMinute,
        totalProcessed: this.totalMessagesProcessed
      },
      tasks: {
        total: taskDashboard.total,
        todo: taskDashboard.byStatus.todo,
        inProgress: taskDashboard.byStatus.inprogress,
        done: taskDashboard.byStatus.done,
        blocked: taskDashboard.blocked
      }
    };
  }

  /**
   * Collect metrics for a specific agent
   */
  private async collectAgentMetrics(agentId: string): Promise<AgentMetrics> {
    // Get agent status
    const agentProcess = await this.agentManager.getAgentStatus(agentId);
    
    // Get agent message history
    const messageHistory = await this.messageBroker.getMessageHistory(agentId);
    const messagesSent = messageHistory.filter(m => m.from === agentId).length;
    const messagesReceived = messageHistory.filter(m => m.to === agentId).length;
    
    // Get agent tasks
    const tasks = await this.taskManager.getAgentTasks(agentId);
    const tasksCreated = tasks.length;
    const tasksCompleted = tasks.filter(t => t.status === 'done').length;
    
    // Calculate uptime
    const uptime = agentProcess ? 
      (Date.now() - agentProcess.startTime.getTime()) / 1000 : 0;
    
    // Get last activity
    const lastActivity = agentProcess ? 
      agentProcess.lastHealthCheck : new Date();
    
    return {
      agentId,
      timestamp: new Date(),
      status: agentProcess ? agentProcess.status : 'unknown',
      cpuUsage: agentProcess ? agentProcess.cpuUsage : 0,
      memoryUsage: agentProcess ? agentProcess.memoryUsage : 0,
      messagesSent,
      messagesReceived,
      tasksCreated,
      tasksCompleted,
      uptime,
      lastActivity
    };
  }

  /**
   * Get CPU usage percentage
   */
  private async getCpuUsage(): Promise<number> {
    return new Promise<number>((resolve) => {
      const startMeasure = process.cpuUsage();
      
      // Measure CPU usage over 100ms
      setTimeout(() => {
        const endMeasure = process.cpuUsage(startMeasure);
        const userCPUUsage = endMeasure.user;
        const sysCPUUsage = endMeasure.system;
        const totalCPUUsage = userCPUUsage + sysCPUUsage;
        
        // Convert to percentage (based on measurement time and number of cores)
        const cpuCount = os.cpus().length;
        const percentage = (totalCPUUsage / 1000 / 100 / cpuCount) * 100;
        
        resolve(Math.min(percentage, 100)); // Cap at 100%
      }, 100);
    });
  }

  /**
   * Calculate messages processed per minute
   */
  private calculateMessagesPerMinute(): number {
    const now = new Date();
    const currentMinute = `${now.getHours()}:${now.getMinutes()}`;
    
    // Get message broker stats
    const messageStats = this.messageBroker.getStats();
    const currentTotal = this.totalMessagesProcessed + messageStats.queueSize;
    
    // Calculate difference from last count
    const diff = currentTotal - this.lastMessageCount;
    this.lastMessageCount = currentTotal;
    
    // Store in minute buckets
    this.messageCountsByMinute.set(currentMinute, diff);
    
    // Keep only the last 60 minutes
    if (this.messageCountsByMinute.size > 60) {
      const oldestKey = Array.from(this.messageCountsByMinute.keys())[0];
      this.messageCountsByMinute.delete(oldestKey);
    }
    
    // Calculate average per minute
    const sum = Array.from(this.messageCountsByMinute.values())
      .reduce((acc, count) => acc + count, 0);
    
    return Math.round(sum / this.messageCountsByMinute.size);
  }

  /**
   * Get system metrics for a specific time range
   */
  getSystemMetrics(
    startTime?: Date,
    endTime?: Date,
    interval?: 'minute' | 'hour' | 'day'
  ): SystemMetrics[] {
    if (!this.initialized) {
      throw new Error('MetricsManager not initialized');
    }
    
    // Filter by time range if specified
    let metrics = this.systemMetricsHistory;
    
    if (startTime) {
      metrics = metrics.filter(m => m.timestamp >= startTime);
    }
    
    if (endTime) {
      metrics = metrics.filter(m => m.timestamp <= endTime);
    }
    
    // Apply interval sampling if specified
    if (interval) {
      metrics = this.sampleMetrics(metrics, interval);
    }
    
    return metrics;
  }

  /**
   * Get agent metrics for a specific time range
   */
  getAgentMetrics(
    agentId: string,
    startTime?: Date,
    endTime?: Date,
    interval?: 'minute' | 'hour' | 'day'
  ): AgentMetrics[] {
    if (!this.initialized) {
      throw new Error('MetricsManager not initialized');
    }
    
    // Get agent metrics
    const metrics = this.agentMetricsHistory.get(agentId) || [];
    
    // Filter by time range if specified
    let filteredMetrics = metrics;
    
    if (startTime) {
      filteredMetrics = filteredMetrics.filter(m => m.timestamp >= startTime);
    }
    
    if (endTime) {
      filteredMetrics = filteredMetrics.filter(m => m.timestamp <= endTime);
    }
    
    // Apply interval sampling if specified
    if (interval) {
      filteredMetrics = this.sampleMetrics(filteredMetrics, interval);
    }
    
    return filteredMetrics;
  }

  /**
   * Get the latest system metrics
   */
  getLatestSystemMetrics(): SystemMetrics | null {
    if (!this.initialized || this.systemMetricsHistory.length === 0) {
      return null;
    }
    
    return this.systemMetricsHistory[this.systemMetricsHistory.length - 1];
  }

  /**
   * Get the latest metrics for a specific agent
   */
  getLatestAgentMetrics(agentId: string): AgentMetrics | null {
    if (!this.initialized) {
      return null;
    }
    
    const metrics = this.agentMetricsHistory.get(agentId) || [];
    
    if (metrics.length === 0) {
      return null;
    }
    
    return metrics[metrics.length - 1];
  }

  /**
   * Sample metrics at a specific interval
   */
  private sampleMetrics<T extends { timestamp: Date }>(
    metrics: T[],
    interval: 'minute' | 'hour' | 'day'
  ): T[] {
    if (metrics.length === 0) {
      return [];
    }
    
    const result: T[] = [];
    let currentBucket: Date | null = null;
    let currentMetric: T | null = null;
    
    for (const metric of metrics) {
      const bucketTime = this.getBucketTime(metric.timestamp, interval);
      
      if (!currentBucket || bucketTime.getTime() !== currentBucket.getTime()) {
        if (currentMetric) {
          result.push(currentMetric);
        }
        currentBucket = bucketTime;
        currentMetric = metric;
      }
    }
    
    // Add the last bucket
    if (currentMetric) {
      result.push(currentMetric);
    }
    
    return result;
  }

  /**
   * Get the bucket time for a timestamp based on the interval
   */
  private getBucketTime(timestamp: Date, interval: 'minute' | 'hour' | 'day'): Date {
    const date = new Date(timestamp);
    
    if (interval === 'minute') {
      date.setSeconds(0, 0);
    } else if (interval === 'hour') {
      date.setMinutes(0, 0, 0);
    } else if (interval === 'day') {
      date.setHours(0, 0, 0, 0);
    }
    
    return date;
  }

  /**
   * Load metrics from disk
   */
  private async loadMetrics(): Promise<void> {
    try {
      // Load system metrics
      const systemMetricsPath = path.join(this.metricsDir, 'system-metrics.json');
      const systemMetricsExists = await fs.access(systemMetricsPath).then(() => true).catch(() => false);
      
      if (systemMetricsExists) {
        const data = await fs.readFile(systemMetricsPath, 'utf-8');
        const parsed = JSON.parse(data);
        
        this.systemMetricsHistory = parsed.metrics.map((m: any) => ({
          ...m,
          timestamp: new Date(m.timestamp)
        }));
        
        this.totalMessagesProcessed = parsed.totalMessagesProcessed || 0;
        
        logger.info(`Loaded ${this.systemMetricsHistory.length} system metrics`);
      }
      
      // Load agent metrics
      const agentMetricsDir = path.join(this.metricsDir, 'agents');
      const agentMetricsDirExists = await fs.access(agentMetricsDir).then(() => true).catch(() => false);
      
      if (agentMetricsDirExists) {
        const files = await fs.readdir(agentMetricsDir);
        const jsonFiles = files.filter(f => f.endsWith('.json'));
        
        for (const file of jsonFiles) {
          const agentId = path.basename(file, '.json');
          const filePath = path.join(agentMetricsDir, file);
          
          const data = await fs.readFile(filePath, 'utf-8');
          const parsed = JSON.parse(data);
          
          const metrics = parsed.metrics.map((m: any) => ({
            ...m,
            timestamp: new Date(m.timestamp),
            lastActivity: new Date(m.lastActivity)
          }));
          
          this.agentMetricsHistory.set(agentId, metrics);
        }
        
        logger.info(`Loaded metrics for ${this.agentMetricsHistory.size} agents`);
      }
      
    } catch (error) {
      logger.error('Failed to load metrics:', error);
    }
  }

  /**
   * Save metrics to disk
   */
  private async saveMetrics(): Promise<void> {
    try {
      // Save system metrics
      const systemMetricsPath = path.join(this.metricsDir, 'system-metrics.json');
      const systemMetricsData = {
        lastUpdated: new Date().toISOString(),
        totalMessagesProcessed: this.totalMessagesProcessed,
        metrics: this.systemMetricsHistory
      };
      
      await fs.writeFile(systemMetricsPath, JSON.stringify(systemMetricsData, null, 2));
      
      // Save agent metrics
      const agentMetricsDir = path.join(this.metricsDir, 'agents');
      await fs.mkdir(agentMetricsDir, { recursive: true });
      
      for (const [agentId, metrics] of this.agentMetricsHistory.entries()) {
        const filePath = path.join(agentMetricsDir, `${agentId}.json`);
        const data = {
          agentId,
          lastUpdated: new Date().toISOString(),
          metrics
        };
        
        await fs.writeFile(filePath, JSON.stringify(data, null, 2));
      }
      
      logger.debug('Metrics saved to disk');
      
    } catch (error) {
      logger.error('Failed to save metrics:', error);
    }
  }

  /**
   * Set up event listeners
   */
  private setupEventListeners(): void {
    // Listen for message broker events
    this.messageBroker.on('message-sent', () => {
      this.totalMessagesProcessed++;
    });
    
    // Listen for agent manager events
    this.agentManager.on('agent-created', () => {
      // Force metrics collection on agent creation
      this.collectMetrics().catch(error => {
        logger.error('Failed to collect metrics after agent creation:', error);
      });
    });
    
    this.agentManager.on('agent-deleted', (agentId) => {
      // Remove agent metrics when agent is deleted
      this.agentMetricsHistory.delete(agentId);
    });
  }
}