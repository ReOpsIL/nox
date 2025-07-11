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
import { DockerManager } from './docker-manager';
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
  async discoverServices(query?: string, options?: {
    category?: string;
    capabilities?: string[];
    limit?: number;
    offset?: number;
    sortBy?: 'name' | 'updated' | 'created' | 'popularity';
    includeDeprecated?: boolean;
  }): Promise<MCPService[]> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    try {
      logger.info(`Discovering MCP services${query ? ` matching "${query}"` : ''}`);

      // Fetch repositories from Docker Hub with pagination
      let allRepositories: any[] = [];
      let nextUrl: string | null = `https://hub.docker.com/v2/repositories/${this.dockerHubUsername}/?page_size=100`;

      while (nextUrl) {
        const response: any = await fetch(nextUrl);

        if (!response.ok) {
          if (response.status === 404) {
            logger.warn(`Docker Hub user ${this.dockerHubUsername} not found`);
            return [];
          }
          throw new Error(`Failed to fetch from Docker Hub: ${response.status} ${response.statusText}`);
        }

        const data: any = await response.json();
        allRepositories = [...allRepositories, ...(data.results || [])];
        nextUrl = data.next;

        // Limit to prevent infinite loops
        if (allRepositories.length > 1000) {
          logger.warn('Too many repositories found, limiting to first 1000');
          break;
        }
      }

      const repositories = allRepositories;

      // Process repositories
      const services: MCPService[] = [];

      // Process repositories in parallel for better performance
      const servicePromises = repositories
        .filter(repo => {
          // Skip repositories that don't have the MCP label or indicator
          return repo.labels && (
            repo.labels.some((label: any) => label.includes('mcp.service')) ||
            repo.description?.includes('[MCP]') ||
            repo.name?.includes('mcp-')
          );
        })
        .map(async (repo) => {
          try {
            // Get service details
            const service = await this.getServiceDetails(repo.name);

            // Add to cache
            this.serviceCache.set(service.id, service);

            // Check if matches query
            if (!query || 
                service.name.toLowerCase().includes(query.toLowerCase()) ||
                service.description.toLowerCase().includes(query.toLowerCase()) ||
                service.tags.some(tag => tag.toLowerCase().includes(query.toLowerCase()))) {
              return service;
            }
            return null;
          } catch (error) {
            logger.warn(`Failed to process service ${repo.name}: ${error}`);
            return null;
          }
        });

      const serviceResults = await Promise.allSettled(servicePromises);

      // Filter successful results
      for (const result of serviceResults) {
        if (result.status === 'fulfilled' && result.value) {
          services.push(result.value);
        }
      }

      // Apply additional filtering and sorting
      let filteredServices = services;

      if (options?.category) {
        filteredServices = filteredServices.filter(service => 
          service.tags.some(tag => tag.toLowerCase().includes(options.category!.toLowerCase()))
        );
      }

      if (options?.capabilities) {
        filteredServices = filteredServices.filter(service =>
          options.capabilities!.every(cap => 
            service.capabilities.some(serviceCap => 
              serviceCap.toLowerCase().includes(cap.toLowerCase())
            )
          )
        );
      }

      if (!options?.includeDeprecated) {
        filteredServices = filteredServices.filter(service =>
          !service.tags.includes('deprecated') && 
          !service.description.toLowerCase().includes('deprecated')
        );
      }

      // Sort services
      if (options?.sortBy) {
        filteredServices.sort((a, b) => {
          switch (options.sortBy) {
            case 'name':
              return a.name.localeCompare(b.name);
            case 'updated':
              return b.updated.getTime() - a.updated.getTime();
            case 'created':
              return b.created.getTime() - a.created.getTime();
            default:
              return 0;
          }
        });
      }

      // Apply pagination
      if (options?.offset || options?.limit) {
        const start = options.offset || 0;
        const end = options.limit ? start + options.limit : undefined;
        filteredServices = filteredServices.slice(start, end);
      }

      logger.info(`Discovered ${filteredServices.length} MCP services (${services.length} total found)`);
      return filteredServices;

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
      // Validate service name
      if (!serviceName || typeof serviceName !== 'string') {
        throw new Error('Invalid service name');
      }

      // Check cache first
      const cacheKey = `${this.dockerHubUsername}/${serviceName}`;
      const cachedService = this.serviceCache.get(cacheKey);
      if (cachedService) {
        logger.debug(`Using cached service details for ${serviceName}`);
        return cachedService;
      }

      // Fetch service details from Docker Hub
      const url = `https://hub.docker.com/v2/repositories/${this.dockerHubUsername}/${serviceName}/`;
      const response = await fetch(url);

      if (!response.ok) {
        if (response.status === 404) {
          throw new Error(`Service ${serviceName} not found`);
        }
        throw new Error(`Failed to fetch service details: ${response.status} ${response.statusText}`);
      }

      const data = await response.json();

      // Validate required fields
      if (!data.name) {
        throw new Error(`Invalid service data: missing name for ${serviceName}`);
      }

      // Parse MCP service metadata from labels and description
      const labels = data.labels || [];
      const mcpLabels = labels.filter((label: any) => label.startsWith('mcp.'));

      // Extract metadata from labels
      const metadata: Record<string, any> = {};
      for (const label of mcpLabels) {
        const [key, ...valueParts] = label.split('=');
        const value = valueParts.join('='); // Handle values with = in them
        const metaKey = key.replace('mcp.', '');
        metadata[metaKey] = value;
      }

      // Also check for metadata in description (JSON format)
      if (data.description) {
        try {
          const descriptionMatch = data.description.match(/\[MCP\]\s*({.*})/s);
          if (descriptionMatch) {
            const descriptionMetadata = JSON.parse(descriptionMatch[1]);
            Object.assign(metadata, descriptionMetadata);
          }
        } catch (error) {
          logger.debug(`Failed to parse description metadata for ${serviceName}: ${error}`);
        }
      }

      // Parse capabilities
      const capabilities = metadata.capabilities ? metadata.capabilities.split(',').map((c: string) => c.trim()) : [];

      // Parse ports
      const ports: { internal: number; external: number }[] = [];
      if (metadata.ports) {
        const portMappings = metadata.ports.split(',');
        for (const mapping of portMappings) {
          const [internal, external] = mapping.split(':').map((p: any) => parseInt(p.trim(), 10));
          ports.push({ internal, external });
        }
      }

      // Parse environment variables with better validation
      const environment: { name: string; required: boolean; default?: string; description: string }[] = [];
      if (metadata.environment) {
        try {
          const envVars = metadata.environment.split(',');
          for (const envVar of envVars) {
            const parts = envVar.split(':').map((e: any) => e.trim());
            if (parts.length >= 2) {
              const [name, required, defaultValue, ...descriptionParts] = parts;
              environment.push({
                name: name || '',
                required: required === 'true',
                default: defaultValue && defaultValue !== 'null' ? defaultValue : undefined,
                description: descriptionParts.join(':') || ''
              });
            }
          }
        } catch (error) {
          logger.warn(`Failed to parse environment variables for ${serviceName}: ${error}`);
        }
      }

      // Parse volumes with better validation
      const volumes: { host: string; container: string; description: string }[] = [];
      if (metadata.volumes) {
        try {
          const volumeMappings = metadata.volumes.split(',');
          for (const mapping of volumeMappings) {
            const parts = mapping.split(':').map((v: any) => v.trim());
            if (parts.length >= 2) {
              const [host, container, ...descriptionParts] = parts;
              volumes.push({ 
                host: host || '', 
                container: container || '', 
                description: descriptionParts.join(':') || '' 
              });
            }
          }
        } catch (error) {
          logger.warn(`Failed to parse volumes for ${serviceName}: ${error}`);
        }
      }

      // Validate and create service object
      const serviceId = `${this.dockerHubUsername}/${serviceName}`;
      const version = metadata.version || 'latest';
      const imageTag = `${this.dockerHubUsername}/${serviceName}:${version}`;

      // Validate required fields
      if (!capabilities || capabilities.length === 0) {
        logger.warn(`Service ${serviceName} has no capabilities defined`);
      }

      const service: MCPService = {
        id: serviceId,
        name: serviceName,
        description: data.description || metadata.description || '',
        version,
        image: imageTag,
        capabilities: capabilities || [],
        ports: ports || [],
        environment: environment || [],
        volumes: volumes || [],
        author: data.user || this.dockerHubUsername,
        website: metadata.website,
        documentation: metadata.documentation,
        tags: metadata.tags ? 
          metadata.tags.split(',').map((t: any) => t.trim()).filter((t: string) => t.length > 0) : 
          [],
        created: new Date(data.date_registered || Date.now()),
        updated: new Date(data.last_updated || Date.now())
      };

      // Additional validation
      this.validateService(service);

      // Add to cache
      this.serviceCache.set(service.id, service);

      logger.debug(`Service details retrieved for ${serviceName}: ${service.capabilities.length} capabilities, ${service.ports.length} ports`);
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
        const serviceName = serviceId.split('/')[1] || serviceId;
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
        .filter((e: any) => e.required && !env[e.name])
        .map((e: any) => e.name);

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
        agentId: agentId || null,
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
   * Search for services by capability or keyword
   */
  async searchServices(criteria: {
    capabilities?: string[];
    keyword?: string;
    category?: string;
    author?: string;
    limit?: number;
  }): Promise<MCPService[]> {
    if (!this.initialized) {
      throw new Error('ServiceManager not initialized');
    }

    try {
      // First discover services if cache is empty
      if (this.serviceCache.size === 0) {
        await this.discoverServices();
      }

      let results = Array.from(this.serviceCache.values());

      // Filter by capabilities
      if (criteria.capabilities && criteria.capabilities.length > 0) {
        results = results.filter(service =>
          criteria.capabilities!.every(cap =>
            service.capabilities.some(serviceCap =>
              serviceCap.toLowerCase().includes(cap.toLowerCase())
            )
          )
        );
      }

      // Filter by keyword
      if (criteria.keyword) {
        const keyword = criteria.keyword.toLowerCase();
        results = results.filter(service =>
          service.name.toLowerCase().includes(keyword) ||
          service.description.toLowerCase().includes(keyword) ||
          service.tags.some(tag => tag.toLowerCase().includes(keyword))
        );
      }

      // Filter by category
      if (criteria.category) {
        results = results.filter(service =>
          service.tags.some(tag =>
            tag.toLowerCase().includes(criteria.category!.toLowerCase())
          )
        );
      }

      // Filter by author
      if (criteria.author) {
        results = results.filter(service =>
          service.author.toLowerCase().includes(criteria.author!.toLowerCase())
        );
      }

      // Apply limit
      if (criteria.limit) {
        results = results.slice(0, criteria.limit);
      }

      logger.info(`Found ${results.length} services matching search criteria`);
      return results;

    } catch (error) {
      logger.error('Failed to search services:', error);
      throw error;
    }
  }

  /**
   * Check service compatibility with agent requirements
   */
  async checkServiceCompatibility(serviceId: string, requirements?: {
    maxMemory?: string;
    maxCpu?: string;
    requiredCapabilities?: string[];
    blockedCapabilities?: string[];
  }): Promise<{
    compatible: boolean;
    issues: string[];
    warnings: string[];
  }> {
    const issues: string[] = [];
    const warnings: string[] = [];

    try {
      const service = this.serviceCache.get(serviceId);
      if (!service) {
        const serviceName = serviceId.split('/')[1] || serviceId;
        const serviceDetails = await this.getServiceDetails(serviceName);
        if (!serviceDetails) {
          issues.push('Service not found');
          return { compatible: false, issues, warnings };
        }
      }

      const serviceToCheck = service || this.serviceCache.get(serviceId)!;

      // Check required capabilities
      if (requirements?.requiredCapabilities) {
        const missingCapabilities = requirements.requiredCapabilities.filter(cap =>
          !serviceToCheck.capabilities.includes(cap)
        );
        if (missingCapabilities.length > 0) {
          issues.push(`Missing required capabilities: ${missingCapabilities.join(', ')}`);
        }
      }

      // Check blocked capabilities
      if (requirements?.blockedCapabilities) {
        const blockedCapabilities = requirements.blockedCapabilities.filter(cap =>
          serviceToCheck.capabilities.includes(cap)
        );
        if (blockedCapabilities.length > 0) {
          issues.push(`Service has blocked capabilities: ${blockedCapabilities.join(', ')}`);
        }
      }

      // Check if service has any capabilities
      if (serviceToCheck.capabilities.length === 0) {
        warnings.push('Service has no defined capabilities');
      }

      // Check if service has required environment variables
      const requiredEnvVars = serviceToCheck.environment.filter(env => env.required);
      if (requiredEnvVars.length > 0) {
        warnings.push(`Service requires ${requiredEnvVars.length} environment variables`);
      }

      return {
        compatible: issues.length === 0,
        issues,
        warnings
      };

    } catch (error) {
      logger.error(`Failed to check service compatibility for ${serviceId}:`, error);
      issues.push('Failed to check compatibility');
      return { compatible: false, issues, warnings };
    }
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
          const serviceData = service as any;
          this.installedServices.set(containerId, {
            ...serviceData,
            installedAt: new Date(serviceData.installedAt),
            created: new Date(serviceData.created),
            updated: new Date(serviceData.updated)
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
   * Validate an MCP service definition
   */
  private validateService(service: MCPService): void {
    // Check for required fields
    if (!service.id || !service.name || !service.image) {
      throw new Error(`Invalid service: missing required fields`);
    }

    // Validate capabilities
    for (const capability of service.capabilities) {
      if (typeof capability !== 'string' || capability.length === 0) {
        throw new Error(`Invalid capability: ${capability}`);
      }
    }

    // Validate ports
    for (const port of service.ports) {
      if (!port.internal || !port.external || 
          port.internal < 1 || port.internal > 65535 ||
          port.external < 1 || port.external > 65535) {
        throw new Error(`Invalid port mapping: ${port.internal}:${port.external}`);
      }
    }

    // Validate environment variables
    for (const env of service.environment) {
      if (!env.name || typeof env.name !== 'string') {
        throw new Error(`Invalid environment variable: missing name`);
      }
    }

    // Validate volumes
    for (const volume of service.volumes) {
      if (!volume.container || typeof volume.container !== 'string') {
        throw new Error(`Invalid volume: missing container path`);
      }
    }

    logger.debug(`Service ${service.name} validation passed`);
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
