/**
 * Service Manager - Handles MCP service discovery and management
 * Provides methods for discovering, installing, and managing MCP services
 */

import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import fetch from 'node-fetch';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';
import { DockerManager, ContainerInfo } from './docker-manager';
import { ApprovalManager, ApprovalRequest } from './approval-manager';
import { CapabilityRegistry } from './capability-registry';

export interface MCPService {
  id: string;
  name: string;
  description: string;
  version: string;
  image: string;
  capabilities: string[];
  ports: { internal: number; external: number }[];
  environment: { name: string; required: boolean; default?: string; description: string }[];
  volumes: { host: string; container: string; description: string }[];
  author: string;
  website?: string;
  documentation?: string;
  tags: string[];
  created: Date;
  updated: Date;
}

export interface InstalledService extends MCPService {
  containerId: string;
  agentId?: string;
  status: 'running' | 'stopped' | 'error';
  installedAt: Date;
}

/**
 * Service Manager - Manages MCP services
 */
export class ServiceManager extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private servicesDir: string;
  private installedServices: Map<string, InstalledService> = new Map();
  private serviceCache: Map<string, MCPService> = new Map();
  private dockerHubUsername = 'mcp';

  constructor(
    private dockerManager: DockerManager,
    private approvalManager: ApprovalManager,
    private capabilityRegistry: CapabilityRegistry,
    workingDir: string
  ) {
    super();
    this.workingDir = workingDir;
    this.servicesDir = path.join(workingDir, 'mcp-services');
  }

  /**
   * Initialize the service manager
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('ServiceManager already initialized');
      return;
    }

    try {
      // Ensure services directory exists
      await fs.mkdir(this.servicesDir, { recursive: true });

      // Set Docker Hub username from config if available
      if (config.mcp?.dockerHubUsername) {
        this.dockerHubUsername = config.mcp.dockerHubUsername;
      }

      // Load installed services
      await this.loadInstalledServices();

      // Register event listeners
      this.setupEventListeners();

      this.initialized = true;
      logger.info('ServiceManager initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize ServiceManager:', error);
      throw error;
    }
  }

  /**
   * Start the service manager
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    logger.info('ServiceManager started');
    this.emit('started');
  }

  /**
   * Shutdown the service manager
   */
  async shutdown(): Promise<void> {
    // Save installed services
    await this.saveInstalledServices();

    logger.info('ServiceManager shutdown');
    this.emit('shutdown');
  }

  /**
   * Discover available MCP services from Docker Hub
   */
  async discoverServices(query?: string): Promise<MCPService[]> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    try {
      logger.info(`Discovering MCP services${query ? ` matching "${query}"` : ''}`);

      // Fetch repositories from Docker Hub
      const url = `https://hub.docker.com/v2/repositories/${this.dockerHubUsername}/?page_size=100`;
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch from Docker Hub: ${response.statusText}`);
      }
      
      const data = await response.json();
      const repositories = data.results || [];
      
      // Process repositories
      const services: MCPService[] = [];
      
      for (const repo of repositories) {
        // Skip repositories that don't have the MCP label
        if (!repo.labels || !repo.labels.some(label => label.includes('mcp.service'))) {
          continue;
        }
        
        // Get service details
        const service = await this.getServiceDetails(repo.name);
        
        // Add to cache
        this.serviceCache.set(service.id, service);
        
        // Add to results if matches query
        if (!query || 
            service.name.toLowerCase().includes(query.toLowerCase()) ||
            service.description.toLowerCase().includes(query.toLowerCase()) ||
            service.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase()))) {
          services.push(service);
        }
      }
      
      logger.info(`Discovered ${services.length} MCP services`);
      return services;
      
    } catch (error) {
      logger.error('Failed to discover MCP services:', error);
      throw error;
    }
  }

  /**
   * Get details for a specific MCP service
   */
  async getServiceDetails(serviceName: string): Promise<MCPService> {
    try {
      // Check cache first
      const cachedService = Array.from(this.serviceCache.values()).find(s => s.name === serviceName);
      if (cachedService) {
        return cachedService;
      }
      
      // Fetch service details from Docker Hub
      const url = `https://hub.docker.com/v2/repositories/${this.dockerHubUsername}/${serviceName}/`;
      const response = await fetch(url);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch service details: ${response.statusText}`);
      }
      
      const data = await response.json();
      
      // Parse MCP service metadata from labels
      const labels = data.labels || [];
      const mcpLabels = labels.filter(label => label.startsWith('mcp.'));
      
      // Extract metadata
      const metadata: Record<string, any> = {};
      for (const label of mcpLabels) {
        const [key, value] = label.split('=');
        const metaKey = key.replace('mcp.', '');
        metadata[metaKey] = value;
      }
      
      // Parse capabilities
      const capabilities = metadata.capabilities ? metadata.capabilities.split(',').map(c => c.trim()) : [];
      
      // Parse ports
      const ports: { internal: number; external: number }[] = [];
      if (metadata.ports) {
        const portMappings = metadata.ports.split(',');
        for (const mapping of portMappings) {
          const [internal, external] = mapping.split(':').map(p => parseInt(p.trim(), 10));
          ports.push({ internal, external });
        }
      }
      
      // Parse environment variables
      const environment: { name: string; required: boolean; default?: string; description: string }[] = [];
      if (metadata.environment) {
        const envVars = metadata.environment.split(',');
        for (const envVar of envVars) {
          const [name, required, defaultValue, description] = envVar.split(':').map(e => e.trim());
          environment.push({
            name,
            required: required === 'true',
            default: defaultValue !== 'null' ? defaultValue : undefined,
            description: description || ''
          });
        }
      }
      
      // Parse volumes
      const volumes: { host: string; container: string; description: string }[] = [];
      if (metadata.volumes) {
        const volumeMappings = metadata.volumes.split(',');
        for (const mapping of volumeMappings) {
          const [host, container, description] = mapping.split(':').map(v => v.trim());
          volumes.push({ host, container, description });
        }
      }
      
      // Create service object
      const service: MCPService = {
        id: `${this.dockerHubUsername}/${serviceName}`,
        name: serviceName,
        description: data.description || '',
        version: metadata.version || 'latest',
        image: `${this.dockerHubUsername}/${serviceName}:${metadata.version || 'latest'}`,
        capabilities,
        ports,
        environment,
        volumes,
        author: data.user || this.dockerHubUsername,
        website: metadata.website,
        documentation: metadata.documentation,
        tags: metadata.tags ? metadata.tags.split(',').map(t => t.trim()) : [],
        created: new Date(data.date_registered),
        updated: new Date(data.last_updated)
      };
      
      // Add to cache
      this.serviceCache.set(service.id, service);
      
      return service;
      
    } catch (error) {
      logger.error(`Failed to get details for service ${serviceName}:`, error);
      throw error;
    }
  }

  /**
   * Install an MCP service
   */
  async installService(
    serviceId: string,
    agentId?: string,
    options: {
      autoApprove?: boolean;
      env?: Record<string, string>;
      volumes?: { host: string; container: string }[];
    } = {}
  ): Promise<InstalledService> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    try {
      // Get service details
      let service: MCPService;
      
      if (this.serviceCache.has(serviceId)) {
        service = this.serviceCache.get(serviceId)!;
      } else {
        const serviceName = serviceId.split('/')[1];
        service = await this.getServiceDetails(serviceName);
      }
      
      logger.info(`Installing MCP service: ${service.name}`);
      
      // Request approval if not auto-approved
      if (!options.autoApprove) {
        const approved = await this.requestApproval(service, agentId);
        if (!approved) {
          throw new Error(`Installation of service ${service.name} was not approved`);
        }
      }
      
      // Prepare container options
      const containerName = `mcp-${service.name}-${Date.now()}`;
      
      // Merge environment variables
      const env: Record<string, string> = {};
      for (const envVar of service.environment) {
        if (envVar.default) {
          env[envVar.name] = envVar.default;
        }
      }
      
      // Override with provided env vars
      if (options.env) {
        Object.assign(env, options.env);
      }
      
      // Check for missing required env vars
      const missingEnvVars = service.environment
        .filter(e => e.required && !env[e.name])
        .map(e => e.name);
        
      if (missingEnvVars.length > 0) {
        throw new Error(`Missing required environment variables: ${missingEnvVars.join(', ')}`);
      }
      
      // Prepare volumes
      const volumes = [...(options.volumes || [])];
      
      // Add default volumes
      for (const volume of service.volumes) {
        // Skip if already defined
        if (volumes.some(v => v.container === volume.container)) {
          continue;
        }
        
        // Create host path
        const hostPath = path.join(this.servicesDir, service.name, volume.host);
        await fs.mkdir(path.dirname(hostPath), { recursive: true });
        
        volumes.push({
          host: hostPath,
          container: volume.container
        });
      }
      
      // Create container
      const containerInfo = await this.dockerManager.createContainer(
        containerName,
        service.image,
        {
          ports: service.ports,
          env,
          volumes,
          labels: {
            'com.nox.mcp.service': service.name,
            'com.nox.mcp.version': service.version,
            'com.nox.mcp.agent': agentId || 'system'
          }
        }
      );
      
      // Create installed service record
      const installedService: InstalledService = {
        ...service,
        containerId: containerInfo.id,
        agentId,
        status: containerInfo.status === 'running' ? 'running' : 'error',
        installedAt: new Date()
      };
      
      // Add to installed services
      this.installedServices.set(containerInfo.id, installedService);
      
      // Save installed services
      await this.saveInstalledServices();
      
      // Register capabilities
      if (agentId) {
        for (const capability of service.capabilities) {
          await this.capabilityRegistry.registerCapability(agentId, capability, {
            source: 'mcp',
            serviceId: service.id,
            containerId: containerInfo.id
          });
        }
      }
      
      logger.info(`MCP service installed: ${service.name} (${containerInfo.id})`);
      this.emit('service-installed', installedService);
      
      return installedService;
      
    } catch (error) {
      logger.error(`Failed to install service ${serviceId}:`, error);
      throw error;
    }
  }

  /**
   * Uninstall an MCP service
   */
  async uninstallService(containerId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    try {
      // Get installed service
      const service = this.installedServices.get(containerId);
      if (!service) {
        throw new Error(`Service with container ID ${containerId} not found`);
      }
      
      logger.info(`Uninstalling MCP service: ${service.name} (${containerId})`);
      
      // Remove container
      await this.dockerManager.removeContainer(containerId);
      
      // Remove capabilities
      if (service.agentId) {
        for (const capability of service.capabilities) {
          await this.capabilityRegistry.unregisterCapability(service.agentId, capability, {
            source: 'mcp',
            serviceId: service.id,
            containerId
          });
        }
      }
      
      // Remove from installed services
      this.installedServices.delete(containerId);
      
      // Save installed services
      await this.saveInstalledServices();
      
      logger.info(`MCP service uninstalled: ${service.name} (${containerId})`);
      this.emit('service-uninstalled', service);
      
    } catch (error) {
      logger.error(`Failed to uninstall service ${containerId}:`, error);
      throw error;
    }
  }

  /**
   * Get an installed service by container ID
   */
  getInstalledService(containerId: string): InstalledService | undefined {
    return this.installedServices.get(containerId);
  }

  /**
   * List all installed services
   */
  listInstalledServices(agentId?: string): InstalledService[] {
    const services = Array.from(this.installedServices.values());
    
    if (agentId) {
      return services.filter(service => service.agentId === agentId);
    }
    
    return services;
  }

  /**
   * Request approval for service installation
   */
  private async requestApproval(service: MCPService, agentId?: string): Promise<boolean> {
    const request: ApprovalRequest = {
      id: `mcp-install-${Date.now()}`,
      type: 'mcp_installation',
      title: `Install MCP Service: ${service.name}`,
      description: service.description,
      details: {
        serviceId: service.id,
        serviceName: service.name,
        image: service.image,
        capabilities: service.capabilities,
        agentId: agentId || 'system'
      },
      requestedBy: agentId || 'system',
      requestedAt: new Date(),
      riskLevel: 'MEDIUM'
    };
    
    return this.approvalManager.requestApproval(request);
  }

  /**
   * Load installed services from disk
   */
  private async loadInstalledServices(): Promise<void> {
    try {
      const filePath = path.join(this.servicesDir, 'installed-services.json');
      const exists = await fs.access(filePath).then(() => true).catch(() => false);
      
      if (exists) {
        const data = await fs.readFile(filePath, 'utf-8');
        const saved = JSON.parse(data);
        
        for (const [containerId, service] of Object.entries(saved.services || {})) {
          this.installedServices.set(containerId, {
            ...service as any,
            installedAt: new Date(service.installedAt),
            created: new Date(service.created),
            updated: new Date(service.updated)
          });
        }
        
        logger.info(`Loaded ${this.installedServices.size} installed MCP services`);
      }
      
      // Verify containers still exist
      const containers = await this.dockerManager.listContainers();
      const containerIds = new Set(containers.map(c => c.id));
      
      for (const [containerId, service] of this.installedServices.entries()) {
        if (!containerIds.has(containerId)) {
          logger.warn(`Container for service ${service.name} (${containerId}) no longer exists, removing from installed services`);
          this.installedServices.delete(containerId);
        }
      }
      
    } catch (error) {
      logger.error('Failed to load installed services:', error);
    }
  }

  /**
   * Save installed services to disk
   */
  private async saveInstalledServices(): Promise<void> {
    try {
      const filePath = path.join(this.servicesDir, 'installed-services.json');
      const data = {
        lastUpdated: new Date().toISOString(),
        services: Object.fromEntries(this.installedServices)
      };
      
      await fs.writeFile(filePath, JSON.stringify(data, null, 2));
      logger.debug('Installed services saved to disk');
      
    } catch (error) {
      logger.error('Failed to save installed services:', error);
    }
  }

  /**
   * Set up event listeners
   */
  private setupEventListeners(): void {
    // Listen for container events
    this.dockerManager.on('container-removed', (container) => {
      if (this.installedServices.has(container.id)) {
        const service = this.installedServices.get(container.id)!;
        this.installedServices.delete(container.id);
        this.emit('service-uninstalled', service);
        this.saveInstalledServices().catch(error => {
          logger.error('Failed to save installed services after container removal:', error);
        });
      }
    });
  }
}