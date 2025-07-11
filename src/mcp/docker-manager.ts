/**
 * Docker Manager - Handles Docker container lifecycle management
 * Provides methods for pulling, starting, stopping, and managing Docker containers
 */

import { EventEmitter } from 'events';
import Docker from 'dockerode';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';

export interface ContainerInfo {
  id: string;
  name: string;
  image: string;
  status: 'created' | 'running' | 'paused' | 'restarting' | 'removing' | 'exited' | 'dead';
  created: Date;
  ports: { internal: number; external: number }[];
  labels: Record<string, string>;
}

export interface ContainerStats {
  id: string;
  name: string;
  cpuUsage: number;
  memoryUsage: number;
  memoryLimit: number;
  networkRx: number;
  networkTx: number;
}

/**
 * Docker Manager - Manages Docker containers for MCP services
 */
export class DockerManager extends EventEmitter {
  private docker: Docker;
  private initialized = false;
  private containers: Map<string, ContainerInfo> = new Map();
  private statsInterval: NodeJS.Timeout | null = null;

  constructor() {
    super();
    // Initialize Docker client with default options
    this.docker = new Docker();
  }

  /**
   * Initialize the Docker manager
   */
  async initialize(_config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('DockerManager already initialized');
      return;
    }

    try {
      // Check Docker connection
      const info = await this.docker.info();
      logger.info(`Connected to Docker daemon: ${info.Name}, version ${info.ServerVersion}`);

      // Load existing MCP containers
      await this.loadExistingContainers();

      // Start stats monitoring
      this.startStatsMonitoring();

      this.initialized = true;
      logger.info('DockerManager initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize DockerManager:', error);
      if (error instanceof Error) {
        throw new Error(`Docker connection failed: ${error.message}`);
      } else {
        throw new Error(`Docker connection failed: ${String(error)}`);
      }
    }
  }

  /**
   * Start the Docker manager
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    logger.info('DockerManager started');
    this.emit('started');
  }

  /**
   * Shutdown the Docker manager
   */
  async shutdown(): Promise<void> {
    // Stop stats monitoring
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = null;
    }

    logger.info('DockerManager shutdown');
    this.emit('shutdown');
  }

  /**
   * Pull a Docker image
   */
  async pullImage(image: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    logger.info(`Pulling Docker image: ${image}`);

    try {
      await new Promise<void>((resolve, reject) => {
        this.docker.pull(image, {}, (err, stream) => {
          if (err) {
            reject(err);
            return;
          }

          this.docker.modem.followProgress(stream!, (err) => {
            if (err) {
              reject(err);
            } else {
              resolve();
            }
          }, (event) => {
            logger.debug(`Pull progress: ${JSON.stringify(event)}`);
          });
        });
      });

      logger.info(`Successfully pulled image: ${image}`);
      this.emit('image-pulled', image);

    } catch (error) {
      logger.error(`Failed to pull image ${image}:`, error);
      throw error;
    }
  }

  /**
   * Create and start a container
   */
  async createContainer(
    name: string,
    image: string,
    options: {
      ports?: { internal: number; external: number }[];
      env?: Record<string, string>;
      volumes?: { host: string; container: string }[];
      labels?: Record<string, string>;
    } = {}
  ): Promise<ContainerInfo> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      // Check if image exists, pull if not
      const images = await this.docker.listImages();
      const imageExists = images.some(img => 
        img.RepoTags && img.RepoTags.includes(image)
      );

      if (!imageExists) {
        await this.pullImage(image);
      }

      // Prepare port bindings
      const portBindings: Record<string, { HostPort: string }[]> = {};
      const exposedPorts: Record<string, {}> = {};

      if (options.ports) {
        for (const port of options.ports) {
          const containerPort = `${port.internal}/tcp`;
          exposedPorts[containerPort] = {};
          portBindings[containerPort] = [{ HostPort: port.external.toString() }];
        }
      }

      // Prepare volumes
      const volumes: Record<string, {}> = {};
      const binds: string[] = [];

      if (options.volumes) {
        for (const volume of options.volumes) {
          volumes[volume.container] = {};
          binds.push(`${volume.host}:${volume.container}`);
        }
      }

      // Prepare environment variables
      const env: string[] = [];
      if (options.env) {
        for (const [key, value] of Object.entries(options.env)) {
          env.push(`${key}=${value}`);
        }
      }

      // Add MCP labels
      const labels = {
        'com.nox.mcp': 'true',
        'com.nox.mcp.created': new Date().toISOString(),
        ...(options.labels || {})
      };

      // Create container
      const container = await this.docker.createContainer({
        name,
        Image: image,
        ExposedPorts: exposedPorts,
        HostConfig: {
          PortBindings: portBindings,
          Binds: binds,
          RestartPolicy: {
            Name: 'unless-stopped'
          }
        },
        Env: env,
        Labels: labels
      });

      // Start container
      await container.start();

      // Get container info
      const containerInfo = await this.getContainerInfo(container.id);
      this.containers.set(container.id, containerInfo);

      logger.info(`Container created and started: ${name} (${container.id})`);
      this.emit('container-created', containerInfo);

      return containerInfo;

    } catch (error) {
      logger.error(`Failed to create container ${name}:`, error);
      throw error;
    }
  }

  /**
   * Start a container
   */
  async startContainer(containerId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      await container.start();

      // Update container info
      const containerInfo = await this.getContainerInfo(containerId);
      this.containers.set(containerId, containerInfo);

      logger.info(`Container started: ${containerInfo.name} (${containerId})`);
      this.emit('container-started', containerInfo);

    } catch (error) {
      logger.error(`Failed to start container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Stop a container
   */
  async stopContainer(containerId: string, options?: { timeout?: number; force?: boolean }): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      const containerInfo = this.containers.get(containerId);

      if (options?.force) {
        await container.kill();
      } else {
        await container.stop({ t: options?.timeout || 30 });
      }

      // Update container info
      const updatedInfo = await this.getContainerInfo(containerId);
      this.containers.set(containerId, updatedInfo);

      logger.info(`Container stopped: ${containerInfo?.name || containerId}`);
      this.emit('container-stopped', updatedInfo);

    } catch (error) {
      logger.error(`Failed to stop container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Restart a container
   */
  async restartContainer(containerId: string, timeout = 30): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      await container.restart({ t: timeout });

      // Update container info
      const containerInfo = await this.getContainerInfo(containerId);
      this.containers.set(containerId, containerInfo);

      logger.info(`Container restarted: ${containerInfo.name} (${containerId})`);
      this.emit('container-restarted', containerInfo);

    } catch (error) {
      logger.error(`Failed to restart container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Pause a container
   */
  async pauseContainer(containerId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      await container.pause();

      // Update container info
      const containerInfo = await this.getContainerInfo(containerId);
      this.containers.set(containerId, containerInfo);

      logger.info(`Container paused: ${containerInfo.name} (${containerId})`);
      this.emit('container-paused', containerInfo);

    } catch (error) {
      logger.error(`Failed to pause container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Unpause a container
   */
  async unpauseContainer(containerId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      await container.unpause();

      // Update container info
      const containerInfo = await this.getContainerInfo(containerId);
      this.containers.set(containerId, containerInfo);

      logger.info(`Container unpaused: ${containerInfo.name} (${containerId})`);
      this.emit('container-unpaused', containerInfo);

    } catch (error) {
      logger.error(`Failed to unpause container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Stop and remove a container
   */
  async removeContainer(containerId: string, options?: { force?: boolean; removeVolumes?: boolean }): Promise<void> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);

      // Get container info before removal
      const containerInfo = this.containers.get(containerId);

      // Stop container if running
      const info = await container.inspect();
      if (info.State.Running) {
        if (options?.force) {
          await container.kill();
        } else {
          await container.stop({ t: 30 });
        }
      }

      // Remove container with options
      await container.remove({ 
        v: options?.removeVolumes || false,
        force: options?.force || false
      });

      // Remove from containers map
      this.containers.delete(containerId);

      logger.info(`Container removed: ${containerInfo?.name || containerId}`);
      this.emit('container-removed', containerInfo || { id: containerId });

    } catch (error) {
      logger.error(`Failed to remove container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Get information about a container
   */
  async getContainerInfo(containerId: string): Promise<ContainerInfo> {
    const container = this.docker.getContainer(containerId);
    const info = await container.inspect();

    // Extract port mappings
    const ports: { internal: number; external: number }[] = [];
    const portBindings = info.HostConfig.PortBindings || {};

    for (const [containerPort, hostBindings] of Object.entries(portBindings)) {
      const portParts = containerPort.split('/');
      const internalPort = parseInt(portParts[0] || '0', 10);

      if (hostBindings && Array.isArray(hostBindings) && hostBindings.length > 0) {
        const externalPort = parseInt(hostBindings[0]?.HostPort || '0', 10);
        ports.push({ internal: internalPort, external: externalPort });
      }
    }

    return {
      id: info.Id,
      name: info.Name.replace(/^\//, ''),
      image: info.Config.Image,
      status: info.State.Status as 'created' | 'running' | 'paused' | 'restarting' | 'removing' | 'exited' | 'dead',
      created: new Date(info.Created),
      ports,
      labels: info.Config.Labels || {}
    };
  }

  /**
   * Get statistics for a container
   */
  async getContainerStats(containerId: string): Promise<ContainerStats> {
    const container = this.docker.getContainer(containerId);
    const stats = await container.stats({ stream: false });
    const info = this.containers.get(containerId);

    // Calculate CPU usage percentage
    const cpuDelta = stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;
    const systemCpuDelta = stats.cpu_stats.system_cpu_usage - stats.precpu_stats.system_cpu_usage;
    const cpuUsage = (cpuDelta / systemCpuDelta) * stats.cpu_stats.online_cpus * 100;

    // Calculate memory usage
    const memoryUsage = stats.memory_stats.usage;
    const memoryLimit = stats.memory_stats.limit;

    // Calculate network usage
    let networkRx = 0;
    let networkTx = 0;

    if (stats.networks) {
      for (const [, network] of Object.entries(stats.networks)) {
        networkRx += network.rx_bytes;
        networkTx += network.tx_bytes;
      }
    }

    return {
      id: containerId,
      name: info?.name || containerId,
      cpuUsage,
      memoryUsage,
      memoryLimit,
      networkRx,
      networkTx
    };
  }

  /**
   * List all MCP containers
   */
  async listContainers(): Promise<ContainerInfo[]> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    return Array.from(this.containers.values());
  }

  /**
   * Check if a container exists
   */
  async containerExists(name: string): Promise<boolean> {
    try {
      const containers = await this.docker.listContainers({ all: true });
      return containers.some(container => 
        container.Names.some(containerName => containerName === `/${name}` || containerName === name)
      );
    } catch (error) {
      logger.error(`Failed to check if container ${name} exists:`, error);
      return false;
    }
  }

  /**
   * Get container logs
   */
  async getContainerLogs(containerId: string, options?: {
    tail?: number;
    since?: Date;
    follow?: boolean;
  }): Promise<string> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      const logOptions: any = {
        stdout: true,
        stderr: true,
        tail: options?.tail || 100
      };

      if (options?.since) {
        logOptions.since = Math.floor(options.since.getTime() / 1000);
      }

      const stream = await container.logs(logOptions);
      // Handle different return types from dockerode
      if (stream === null || stream === undefined) {
        return '';
      }
      if (typeof stream === 'string') {
        return stream;
      }
      // Cast to any to handle unknown return type
      const anyStream = stream as any;
      if (typeof anyStream.toString === 'function') {
        return anyStream.toString();
      }
      return '';

    } catch (error) {
      logger.error(`Failed to get logs for container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Check container health
   */
  async checkContainerHealth(containerId: string): Promise<{
    status: 'healthy' | 'unhealthy' | 'starting' | 'none';
    checks: { test: string; interval: string; timeout: string; retries: number }[];
    lastCheck?: Date;
    failingStreak: number;
  }> {
    if (!this.initialized) {
      throw new Error('DockerManager not initialized');
    }

    try {
      const container = this.docker.getContainer(containerId);
      const info = await container.inspect();

      const health = info.State.Health;
      if (!health) {
        return {
          status: 'none',
          checks: [],
          failingStreak: 0
        };
      }

      // Type assertion for Healthcheck
      const healthcheck = (info.Config as any).Healthcheck;

      // Create a variable for lastCheck to ensure type safety
      let lastCheck: Date | undefined = undefined;
      if (health.Log && health.Log.length > 0 && health.Log[health.Log.length - 1]?.Start) {
        // Safely access the Start property with optional chaining
        const startTime = health.Log[health.Log.length - 1]?.Start;
        if (startTime) {
          lastCheck = new Date(startTime);
        }
      }

      // Explicitly cast status to the expected type
      const status = (health.Status?.toLowerCase() as 'healthy' | 'unhealthy' | 'starting' | 'none') || 'none';

      // Create the result object with the correct types
      const result: {
        status: 'healthy' | 'unhealthy' | 'starting' | 'none';
        checks: { test: string; interval: string; timeout: string; retries: number }[];
        lastCheck?: Date;
        failingStreak: number;
      } = {
        status,
        checks: healthcheck ? [{
          test: healthcheck.Test?.join(' ') || '',
          interval: healthcheck.Interval || '30s',
          timeout: healthcheck.Timeout || '30s',
          retries: healthcheck.Retries || 3
        }] : [],
        failingStreak: health.FailingStreak || 0
      };

      // Only add lastCheck if it's defined
      if (lastCheck) {
        result.lastCheck = lastCheck;
      }

      return result;

    } catch (error) {
      logger.error(`Failed to check health for container ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Load existing MCP containers
   */
  private async loadExistingContainers(): Promise<void> {
    try {
      const containers = await this.docker.listContainers({ 
        all: true,
        filters: { label: ['com.nox.mcp=true'] }
      });

      for (const container of containers) {
        const containerInfo = await this.getContainerInfo(container.Id);
        this.containers.set(container.Id, containerInfo);
      }

      logger.info(`Loaded ${this.containers.size} existing MCP containers`);

    } catch (error) {
      logger.error('Failed to load existing containers:', error);
    }
  }

  /**
   * Start monitoring container stats
   */
  private startStatsMonitoring(): void {
    this.statsInterval = setInterval(async () => {
      try {
        for (const containerId of this.containers.keys()) {
          try {
            const stats = await this.getContainerStats(containerId);
            this.emit('container-stats', stats);
          } catch (error) {
            // Container might have been removed
            if ((error as any).statusCode === 404) {
              this.containers.delete(containerId);
            }
          }
        }
      } catch (error) {
        logger.error('Error monitoring container stats:', error);
      }
    }, 30000); // Every 30 seconds
  }
}
