/**
 * Capability Registry - Manages agent capabilities and their sources
 * Provides methods for registering, unregistering, and querying capabilities
 */

import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';

export interface CapabilitySource {
  source: 'mcp' | 'built-in' | 'learned' | 'delegated';
  serviceId?: string;
  containerId?: string;
  agentId?: string;
  details?: Record<string, any>;
}

export interface Capability {
  id: string;
  name: string;
  description?: string;
  sources: CapabilitySource[];
  addedAt: Date;
  updatedAt: Date;
}

export interface AgentCapabilities {
  agentId: string;
  capabilities: Map<string, Capability>;
  updatedAt: Date;
}

/**
 * Capability Registry - Manages agent capabilities
 */
export class CapabilityRegistry extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private capabilitiesDir: string;
  private agentCapabilities: Map<string, AgentCapabilities> = new Map();

  constructor(workingDir: string) {
    super();
    this.workingDir = workingDir;
    this.capabilitiesDir = path.join(workingDir, 'capabilities');
  }

  /**
   * Initialize the capability registry
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('CapabilityRegistry already initialized');
      return;
    }

    try {
      // Ensure capabilities directory exists
      await fs.mkdir(this.capabilitiesDir, { recursive: true });

      // Load agent capabilities
      await this.loadAgentCapabilities();

      this.initialized = true;
      logger.info('CapabilityRegistry initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize CapabilityRegistry:', error);
      throw error;
    }
  }

  /**
   * Start the capability registry
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    logger.info('CapabilityRegistry started');
    this.emit('started');
  }

  /**
   * Shutdown the capability registry
   */
  async shutdown(): Promise<void> {
    // Save agent capabilities
    await this.saveAgentCapabilities();

    logger.info('CapabilityRegistry shutdown');
    this.emit('shutdown');
  }

  /**
   * Register a capability for an agent
   */
  async registerCapability(
    agentId: string,
    capabilityName: string,
    source: CapabilitySource
  ): Promise<Capability> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    try {
      // Normalize capability name
      const normalizedName = this.normalizeCapabilityName(capabilityName);
      const capabilityId = `${normalizedName}`;

      // Get or create agent capabilities
      let agentCaps = this.agentCapabilities.get(agentId);
      if (!agentCaps) {
        agentCaps = {
          agentId,
          capabilities: new Map(),
          updatedAt: new Date()
        };
        this.agentCapabilities.set(agentId, agentCaps);
      }

      // Get or create capability
      let capability = agentCaps.capabilities.get(capabilityId);
      if (!capability) {
        capability = {
          id: capabilityId,
          name: capabilityName,
          sources: [],
          addedAt: new Date(),
          updatedAt: new Date()
        };
        agentCaps.capabilities.set(capabilityId, capability);
      }

      // Add source if not already present
      const sourceExists = capability.sources.some(s => 
        s.source === source.source && 
        s.serviceId === source.serviceId &&
        s.containerId === source.containerId &&
        s.agentId === source.agentId
      );

      if (!sourceExists) {
        capability.sources.push(source);
        capability.updatedAt = new Date();
        agentCaps.updatedAt = new Date();
      }

      // Save agent capabilities
      await this.saveAgentCapabilities(agentId);

      logger.info(`Capability registered: ${capabilityName} for agent ${agentId}`);
      this.emit('capability-registered', agentId, capability);

      return capability;

    } catch (error) {
      logger.error(`Failed to register capability ${capabilityName} for agent ${agentId}:`, error);
      throw error;
    }
  }

  /**
   * Unregister a capability source for an agent
   */
  async unregisterCapability(
    agentId: string,
    capabilityName: string,
    source: CapabilitySource
  ): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    try {
      // Normalize capability name
      const normalizedName = this.normalizeCapabilityName(capabilityName);
      const capabilityId = `${normalizedName}`;

      // Get agent capabilities
      const agentCaps = this.agentCapabilities.get(agentId);
      if (!agentCaps) {
        return false;
      }

      // Get capability
      const capability = agentCaps.capabilities.get(capabilityId);
      if (!capability) {
        return false;
      }

      // Find source index
      const sourceIndex = capability.sources.findIndex(s => 
        s.source === source.source && 
        s.serviceId === source.serviceId &&
        s.containerId === source.containerId &&
        s.agentId === source.agentId
      );

      if (sourceIndex === -1) {
        return false;
      }

      // Remove source
      capability.sources.splice(sourceIndex, 1);
      capability.updatedAt = new Date();
      agentCaps.updatedAt = new Date();

      // If no sources left, remove capability
      if (capability.sources.length === 0) {
        agentCaps.capabilities.delete(capabilityId);
      }

      // Save agent capabilities
      await this.saveAgentCapabilities(agentId);

      logger.info(`Capability source unregistered: ${capabilityName} for agent ${agentId}`);
      this.emit('capability-unregistered', agentId, capabilityName, source);

      return true;

    } catch (error) {
      logger.error(`Failed to unregister capability ${capabilityName} for agent ${agentId}:`, error);
      throw error;
    }
  }

  /**
   * Check if an agent has a capability
   */
  hasCapability(agentId: string, capabilityName: string): boolean {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    // Normalize capability name
    const normalizedName = this.normalizeCapabilityName(capabilityName);
    const capabilityId = `${normalizedName}`;

    // Get agent capabilities
    const agentCaps = this.agentCapabilities.get(agentId);
    if (!agentCaps) {
      return false;
    }

    // Check if capability exists
    return agentCaps.capabilities.has(capabilityId);
  }

  /**
   * Get a capability for an agent
   */
  getCapability(agentId: string, capabilityName: string): Capability | null {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    // Normalize capability name
    const normalizedName = this.normalizeCapabilityName(capabilityName);
    const capabilityId = `${normalizedName}`;

    // Get agent capabilities
    const agentCaps = this.agentCapabilities.get(agentId);
    if (!agentCaps) {
      return null;
    }

    // Get capability
    return agentCaps.capabilities.get(capabilityId) || null;
  }

  /**
   * Get all capabilities for an agent
   */
  getAgentCapabilities(agentId: string): Capability[] {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    // Get agent capabilities
    const agentCaps = this.agentCapabilities.get(agentId);
    if (!agentCaps) {
      return [];
    }

    // Return capabilities
    return Array.from(agentCaps.capabilities.values());
  }

  /**
   * Find agents with a specific capability
   */
  findAgentsWithCapability(capabilityName: string): string[] {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    // Normalize capability name
    const normalizedName = this.normalizeCapabilityName(capabilityName);
    const capabilityId = `${normalizedName}`;

    // Find agents with capability
    const agents: string[] = [];
    for (const [agentId, agentCaps] of this.agentCapabilities.entries()) {
      if (agentCaps.capabilities.has(capabilityId)) {
        agents.push(agentId);
      }
    }

    return agents;
  }

  /**
   * Get all registered capabilities across all agents
   */
  getAllCapabilities(): Map<string, Capability> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    // Collect all unique capabilities
    const allCapabilities = new Map<string, Capability>();
    for (const agentCaps of this.agentCapabilities.values()) {
      for (const [id, capability] of agentCaps.capabilities.entries()) {
        if (!allCapabilities.has(id)) {
          allCapabilities.set(id, { ...capability });
        }
      }
    }

    return allCapabilities;
  }

  /**
   * Get capabilities by source type
   */
  getCapabilitiesBySource(sourceType: 'mcp' | 'built-in' | 'learned' | 'delegated'): Capability[] {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    const capabilities: Capability[] = [];
    for (const agentCaps of this.agentCapabilities.values()) {
      for (const capability of agentCaps.capabilities.values()) {
        if (capability.sources.some(source => source.source === sourceType)) {
          capabilities.push(capability);
        }
      }
    }

    // Remove duplicates based on capability ID
    const uniqueCapabilities = new Map<string, Capability>();
    for (const capability of capabilities) {
      if (!uniqueCapabilities.has(capability.id)) {
        uniqueCapabilities.set(capability.id, capability);
      }
    }

    return Array.from(uniqueCapabilities.values());
  }

  /**
   * Get MCP service capabilities for an agent
   */
  getMCPCapabilities(agentId: string): Array<{
    capability: Capability;
    serviceId: string;
    containerId?: string;
  }> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    const agentCaps = this.agentCapabilities.get(agentId);
    if (!agentCaps) {
      return [];
    }

    const mcpCapabilities: Array<{
      capability: Capability;
      serviceId: string;
      containerId?: string;
    }> = [];

    for (const capability of agentCaps.capabilities.values()) {
      for (const source of capability.sources) {
        if (source.source === 'mcp' && source.serviceId) {
          mcpCapabilities.push({
            capability,
            serviceId: source.serviceId,
            containerId: source.containerId
          });
        }
      }
    }

    return mcpCapabilities;
  }

  /**
   * Remove all capabilities from a specific MCP service
   */
  async removeMCPServiceCapabilities(serviceId: string, containerId?: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    try {
      const agentsToUpdate = new Set<string>();

      for (const [agentId, agentCaps] of this.agentCapabilities.entries()) {
        for (const [capabilityId, capability] of agentCaps.capabilities.entries()) {
          // Remove sources that match the service
          const originalSourceCount = capability.sources.length;
          capability.sources = capability.sources.filter(source => {
            if (source.source !== 'mcp' || source.serviceId !== serviceId) {
              return true;
            }
            if (containerId && source.containerId !== containerId) {
              return true;
            }
            return false;
          });

          // If sources were removed, mark for update
          if (capability.sources.length !== originalSourceCount) {
            capability.updatedAt = new Date();
            agentCaps.updatedAt = new Date();
            agentsToUpdate.add(agentId);

            // If no sources left, remove capability
            if (capability.sources.length === 0) {
              agentCaps.capabilities.delete(capabilityId);
            }
          }
        }
      }

      // Save updated agent capabilities
      for (const agentId of agentsToUpdate) {
        await this.saveAgentCapabilities(agentId);
        logger.info(`Removed MCP service capabilities for agent ${agentId} (service: ${serviceId})`);
        this.emit('mcp-capabilities-removed', agentId, serviceId);
      }

    } catch (error) {
      logger.error(`Failed to remove MCP service capabilities for ${serviceId}:`, error);
      throw error;
    }
  }

  /**
   * Get capability statistics
   */
  getCapabilityStatistics(): {
    totalCapabilities: number;
    capabilitiesBySource: Record<string, number>;
    agentsWithCapabilities: number;
    mostCommonCapabilities: Array<{ name: string; count: number }>;
  } {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    const capabilityCount = new Map<string, number>();
    const sourceCount: Record<string, number> = {};
    let totalCapabilities = 0;

    for (const agentCaps of this.agentCapabilities.values()) {
      for (const capability of agentCaps.capabilities.values()) {
        totalCapabilities++;
        
        // Count capability occurrences
        const currentCount = capabilityCount.get(capability.name) || 0;
        capabilityCount.set(capability.name, currentCount + 1);

        // Count sources
        for (const source of capability.sources) {
          sourceCount[source.source] = (sourceCount[source.source] || 0) + 1;
        }
      }
    }

    // Get most common capabilities
    const mostCommon = Array.from(capabilityCount.entries())
      .sort((a, b) => b[1] - a[1])
      .slice(0, 10)
      .map(([name, count]) => ({ name, count }));

    return {
      totalCapabilities,
      capabilitiesBySource: sourceCount,
      agentsWithCapabilities: this.agentCapabilities.size,
      mostCommonCapabilities: mostCommon
    };
  }

  /**
   * Find capability conflicts (overlapping capabilities from different sources)
   */
  findCapabilityConflicts(agentId?: string): Array<{
    agentId: string;
    capability: string;
    conflictingSources: CapabilitySource[];
  }> {
    if (!this.initialized) {
      throw new Error('CapabilityRegistry not initialized');
    }

    const conflicts: Array<{
      agentId: string;
      capability: string;
      conflictingSources: CapabilitySource[];
    }> = [];

    const agentsToCheck = agentId ? 
      [this.agentCapabilities.get(agentId)].filter(Boolean) : 
      Array.from(this.agentCapabilities.values());

    for (const agentCaps of agentsToCheck) {
      for (const capability of agentCaps.capabilities.values()) {
        if (capability.sources.length > 1) {
          // Check for conflicts between different source types
          const sourceTypes = new Set(capability.sources.map(s => s.source));
          if (sourceTypes.size > 1) {
            conflicts.push({
              agentId: agentCaps.agentId,
              capability: capability.name,
              conflictingSources: capability.sources
            });
          }
        }
      }
    }

    return conflicts;
  }

  /**
   * Normalize a capability name for consistent lookup
   */
  private normalizeCapabilityName(name: string): string {
    return name.toLowerCase().replace(/[^a-z0-9_]/g, '_');
  }

  /**
   * Validate capability source data
   */
  private validateCapabilitySource(source: CapabilitySource): void {
    if (!source.source) {
      throw new Error('Capability source type is required');
    }

    if (source.source === 'mcp' && !source.serviceId) {
      throw new Error('MCP capability source requires serviceId');
    }

    if (source.source === 'delegated' && !source.agentId) {
      throw new Error('Delegated capability source requires agentId');
    }
  }

  /**
   * Load agent capabilities from disk
   */
  private async loadAgentCapabilities(): Promise<void> {
    try {
      const files = await fs.readdir(this.capabilitiesDir);
      const jsonFiles = files.filter(file => file.endsWith('.json'));

      for (const file of jsonFiles) {
        try {
          const agentId = path.basename(file, '.json');
          const filePath = path.join(this.capabilitiesDir, file);
          const data = await fs.readFile(filePath, 'utf-8');
          const saved = JSON.parse(data);

          // Create agent capabilities
          const agentCaps: AgentCapabilities = {
            agentId,
            capabilities: new Map(),
            updatedAt: new Date(saved.updatedAt)
          };

          // Load capabilities
          for (const [id, capability] of Object.entries(saved.capabilities)) {
            agentCaps.capabilities.set(id, {
              ...capability as any,
              addedAt: new Date(capability.addedAt),
              updatedAt: new Date(capability.updatedAt)
            });
          }

          this.agentCapabilities.set(agentId, agentCaps);
        } catch (error) {
          logger.error(`Failed to load capabilities from file ${file}:`, error);
        }
      }

      logger.info(`Loaded capabilities for ${this.agentCapabilities.size} agents`);

    } catch (error) {
      logger.error('Failed to load agent capabilities:', error);
    }
  }

  /**
   * Save agent capabilities to disk
   */
  private async saveAgentCapabilities(agentId?: string): Promise<void> {
    try {
      if (agentId) {
        // Save specific agent capabilities
        const agentCaps = this.agentCapabilities.get(agentId);
        if (agentCaps) {
          const filePath = path.join(this.capabilitiesDir, `${agentId}.json`);
          const data = {
            agentId,
            updatedAt: agentCaps.updatedAt.toISOString(),
            capabilities: Object.fromEntries(agentCaps.capabilities)
          };
          await fs.writeFile(filePath, JSON.stringify(data, null, 2));
        }
      } else {
        // Save all agent capabilities
        for (const [agentId, agentCaps] of this.agentCapabilities.entries()) {
          const filePath = path.join(this.capabilitiesDir, `${agentId}.json`);
          const data = {
            agentId,
            updatedAt: agentCaps.updatedAt.toISOString(),
            capabilities: Object.fromEntries(agentCaps.capabilities)
          };
          await fs.writeFile(filePath, JSON.stringify(data, null, 2));
        }
      }

      logger.debug('Agent capabilities saved to disk');

    } catch (error) {
      logger.error('Failed to save agent capabilities:', error);
    }
  }
}