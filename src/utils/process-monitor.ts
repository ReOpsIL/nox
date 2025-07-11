import { EventEmitter } from 'events';
import * as os from 'os';
import { logger } from './logger';

export interface ProcessMetrics {
  pid: number;
  cpuUsage: number;
  memoryUsage: number;
  memoryUsageMB: number;
  uptime: number;
  status: 'running' | 'idle' | 'high_cpu' | 'high_memory' | 'unresponsive';
  lastCheck: Date;
}

export interface SystemMetrics {
  totalMemory: number;
  freeMemory: number;
  memoryUsagePercent: number;
  cpuCount: number;
  loadAverage: number[];
  uptime: number;
}

export interface MonitorConfig {
  checkInterval: number; // milliseconds
  cpuThreshold: number; // percentage
  memoryThreshold: number; // MB
  unresponsiveTimeout: number; // milliseconds
}

/**
 * Process Monitor - Monitors system and process health
 * Tracks CPU, memory usage, and process responsiveness
 */
export class ProcessMonitor extends EventEmitter {
  private processes = new Map<string, NodeJS.Timer>();
  private metrics = new Map<string, ProcessMetrics>();
  private config: MonitorConfig;
  private systemMetrics: SystemMetrics | null = null;
  private systemMonitorInterval: NodeJS.Timer | null = null;
  private isRunning = false;

  constructor(config: Partial<MonitorConfig> = {}) {
    super();
    
    this.config = {
      checkInterval: config.checkInterval || 5000, // 5 seconds
      cpuThreshold: config.cpuThreshold || 80, // 80%
      memoryThreshold: config.memoryThreshold || 500, // 500MB
      unresponsiveTimeout: config.unresponsiveTimeout || 30000, // 30 seconds
    };
  }

  /**
   * Start monitoring system and processes
   */
  start(): void {
    if (this.isRunning) {
      logger.warn('ProcessMonitor already running');
      return;
    }

    this.isRunning = true;
    
    // Start system monitoring
    this.startSystemMonitoring();
    
    logger.info('ProcessMonitor started');
    this.emit('started');
  }

  /**
   * Stop all monitoring
   */
  stop(): void {
    if (!this.isRunning) {
      return;
    }

    this.isRunning = false;

    // Stop system monitoring
    if (this.systemMonitorInterval) {
      clearInterval(this.systemMonitorInterval);
      this.systemMonitorInterval = null;
    }

    // Stop all process monitoring
    for (const [processId, interval] of this.processes) {
      clearInterval(interval);
      this.processes.delete(processId);
    }

    this.metrics.clear();
    
    logger.info('ProcessMonitor stopped');
    this.emit('stopped');
  }

  /**
   * Add a process to monitor
   */
  addProcess(processId: string, pid: number): void {
    if (this.processes.has(processId)) {
      logger.warn(`Process ${processId} is already being monitored`);
      return;
    }

    // Initialize metrics
    this.metrics.set(processId, {
      pid,
      cpuUsage: 0,
      memoryUsage: 0,
      memoryUsageMB: 0,
      uptime: 0,
      status: 'running',
      lastCheck: new Date()
    });

    // Start monitoring interval
    const interval = setInterval(() => {
      this.checkProcess(processId, pid);
    }, this.config.checkInterval);

    this.processes.set(processId, interval);
    
    logger.info(`Started monitoring process: ${processId} (PID: ${pid})`);
    this.emit('process-added', processId, pid);
  }

  /**
   * Remove a process from monitoring
   */
  removeProcess(processId: string): void {
    const interval = this.processes.get(processId);
    if (interval) {
      clearInterval(interval);
      this.processes.delete(processId);
    }

    this.metrics.delete(processId);
    
    logger.info(`Stopped monitoring process: ${processId}`);
    this.emit('process-removed', processId);
  }

  /**
   * Get metrics for a specific process
   */
  getProcessMetrics(processId: string): ProcessMetrics | null {
    return this.metrics.get(processId) || null;
  }

  /**
   * Get metrics for all monitored processes
   */
  getAllProcessMetrics(): Map<string, ProcessMetrics> {
    return new Map(this.metrics);
  }

  /**
   * Get current system metrics
   */
  getSystemMetrics(): SystemMetrics | null {
    return this.systemMetrics;
  }

  /**
   * Get monitoring configuration
   */
  getConfig(): MonitorConfig {
    return { ...this.config };
  }

  /**
   * Update monitoring configuration
   */
  updateConfig(newConfig: Partial<MonitorConfig>): void {
    this.config = { ...this.config, ...newConfig };
    logger.info('ProcessMonitor configuration updated');
    this.emit('config-updated', this.config);
  }

  /**
   * Start system-wide monitoring
   */
  private startSystemMonitoring(): void {
    // Initial system metrics collection
    this.collectSystemMetrics();

    // Set up interval for system monitoring
    this.systemMonitorInterval = setInterval(() => {
      this.collectSystemMetrics();
    }, this.config.checkInterval * 2); // Check system less frequently
  }

  /**
   * Collect system-wide metrics
   */
  private collectSystemMetrics(): void {
    try {
      const totalMemory = os.totalmem();
      const freeMemory = os.freemem();
      const memoryUsagePercent = ((totalMemory - freeMemory) / totalMemory) * 100;

      this.systemMetrics = {
        totalMemory: Math.round(totalMemory / (1024 * 1024)), // MB
        freeMemory: Math.round(freeMemory / (1024 * 1024)), // MB
        memoryUsagePercent: Math.round(memoryUsagePercent * 100) / 100,
        cpuCount: os.cpus().length,
        loadAverage: os.loadavg(),
        uptime: os.uptime()
      };

      this.emit('system-metrics', this.systemMetrics);

      // Check for system-wide issues
      if (memoryUsagePercent > 90) {
        this.emit('system-warning', 'high-memory', `System memory usage: ${memoryUsagePercent.toFixed(1)}%`);
      }

      if (this.systemMetrics.loadAverage[0] > this.systemMetrics.cpuCount * 2) {
        this.emit('system-warning', 'high-load', `System load: ${this.systemMetrics.loadAverage[0].toFixed(2)}`);
      }

    } catch (error) {
      logger.error('Failed to collect system metrics:', error);
    }
  }

  /**
   * Check metrics for a specific process
   */
  private async checkProcess(processId: string, pid: number): Promise<void> {
    try {
      const metrics = await this.getProcessMetricsFromPid(pid);
      if (!metrics) {
        // Process might have died
        this.handleProcessDeath(processId, pid);
        return;
      }

      const currentMetrics = this.metrics.get(processId);
      if (!currentMetrics) {
        return;
      }

      // Update metrics
      const updatedMetrics: ProcessMetrics = {
        ...currentMetrics,
        cpuUsage: metrics.cpuUsage,
        memoryUsage: metrics.memoryUsage,
        memoryUsageMB: Math.round(metrics.memoryUsage / (1024 * 1024)),
        uptime: metrics.uptime,
        lastCheck: new Date()
      };

      // Determine status
      updatedMetrics.status = this.determineProcessStatus(updatedMetrics);

      this.metrics.set(processId, updatedMetrics);
      this.emit('process-metrics', processId, updatedMetrics);

      // Check for alerts
      this.checkProcessAlerts(processId, updatedMetrics);

    } catch (error) {
      logger.error(`Failed to check process ${processId} (PID: ${pid}):`, error);
      this.handleProcessError(processId, pid, error);
    }
  }

  /**
   * Get process metrics from PID using Node.js process information
   */
  private async getProcessMetricsFromPid(pid: number): Promise<{ cpuUsage: number; memoryUsage: number; uptime: number } | null> {
    try {
      // Note: This is a simplified implementation
      // In a real implementation, you might use tools like ps, pidusage, or system APIs
      
      if (pid === process.pid) {
        // Current Node.js process
        const memUsage = process.memoryUsage();
        const cpuUsage = process.cpuUsage();
        
        return {
          cpuUsage: (cpuUsage.user + cpuUsage.system) / 1000000, // Convert to percentage (simplified)
          memoryUsage: memUsage.rss,
          uptime: process.uptime()
        };
      }

      // For other processes, we'd need to use external tools or libraries
      // This is a placeholder implementation
      return {
        cpuUsage: Math.random() * 100, // Mock data
        memoryUsage: Math.random() * 1024 * 1024 * 100, // Mock data: 0-100MB
        uptime: Date.now() / 1000 // Mock data
      };

    } catch (error) {
      return null;
    }
  }

  /**
   * Determine process status based on metrics
   */
  private determineProcessStatus(metrics: ProcessMetrics): ProcessMetrics['status'] {
    const now = Date.now();
    const lastCheckAge = now - metrics.lastCheck.getTime();

    if (lastCheckAge > this.config.unresponsiveTimeout) {
      return 'unresponsive';
    }

    if (metrics.cpuUsage > this.config.cpuThreshold) {
      return 'high_cpu';
    }

    if (metrics.memoryUsageMB > this.config.memoryThreshold) {
      return 'high_memory';
    }

    if (metrics.cpuUsage < 5 && metrics.memoryUsageMB < 50) {
      return 'idle';
    }

    return 'running';
  }

  /**
   * Check for process alerts and emit warnings
   */
  private checkProcessAlerts(processId: string, metrics: ProcessMetrics): void {
    if (metrics.status === 'high_cpu') {
      this.emit('process-warning', processId, 'high-cpu', `CPU usage: ${metrics.cpuUsage.toFixed(1)}%`);
    }

    if (metrics.status === 'high_memory') {
      this.emit('process-warning', processId, 'high-memory', `Memory usage: ${metrics.memoryUsageMB}MB`);
    }

    if (metrics.status === 'unresponsive') {
      this.emit('process-warning', processId, 'unresponsive', `Process unresponsive for ${this.config.unresponsiveTimeout}ms`);
    }
  }

  /**
   * Handle process death
   */
  private handleProcessDeath(processId: string, pid: number): void {
    logger.warn(`Process ${processId} (PID: ${pid}) appears to have died`);
    this.emit('process-died', processId, pid);
    this.removeProcess(processId);
  }

  /**
   * Handle process monitoring errors
   */
  private handleProcessError(processId: string, pid: number, error: any): void {
    logger.error(`Process monitoring error for ${processId} (PID: ${pid}):`, error);
    this.emit('process-error', processId, pid, error);
  }

  /**
   * Get summary of all monitored processes
   */
  getSummary(): {
    totalProcesses: number;
    runningProcesses: number;
    highCpuProcesses: number;
    highMemoryProcesses: number;
    unresponsiveProcesses: number;
    totalMemoryUsage: number;
  } {
    const allMetrics = Array.from(this.metrics.values());
    
    return {
      totalProcesses: allMetrics.length,
      runningProcesses: allMetrics.filter(m => m.status === 'running').length,
      highCpuProcesses: allMetrics.filter(m => m.status === 'high_cpu').length,
      highMemoryProcesses: allMetrics.filter(m => m.status === 'high_memory').length,
      unresponsiveProcesses: allMetrics.filter(m => m.status === 'unresponsive').length,
      totalMemoryUsage: allMetrics.reduce((sum, m) => sum + m.memoryUsageMB, 0)
    };
  }
}