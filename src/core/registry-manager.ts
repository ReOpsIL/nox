import { EventEmitter } from 'events';
import * as path from 'path';
import { AgentConfig, AgentRelationship, DEFAULT_RESOURCE_LIMITS } from '../types/agent';
import { FileUtils } from '../utils/file-utils';
import { ValidationUtils } from '../utils/validation';
import { logger } from '../utils/logger';

export interface RegistryStats {
  totalAgents: number;
  activeAgents: number;
  inactiveAgents: number;
  errorAgents: number;
  lastModified: Date;
  registrySize: number;
}

export class RegistryManager extends EventEmitter {
  private registryPath = '';
  private agentsFilePath = '';
  private relationshipsFilePath = '';
  private mcpServicesFilePath = '';
  private agents: Map<string, AgentConfig> = new Map();
  private relationships: Map<string, AgentRelationship[]> = new Map();
  private initialized = false;

  constructor() {
    super();
  }

  async initialize(registryPath: string, force = false): Promise<void> {
    this.registryPath = registryPath;
    this.agentsFilePath = path.join(registryPath, 'agents.json');
    this.relationshipsFilePath = path.join(registryPath, 'agent-relationships.json');
    this.mcpServicesFilePath = path.join(registryPath, 'mcp-services.json');

    try {
      // Create registry directory
      await FileUtils.ensureDirectory(registryPath);

      // Create subdirectories
      await FileUtils.ensureDirectory(path.join(registryPath, 'agents'));
      await FileUtils.ensureDirectory(path.join(registryPath, 'tasks'));
      await FileUtils.ensureDirectory(path.join(registryPath, 'conversations'));

      // Initialize or load existing registry
      if (force || !(await FileUtils.exists(this.agentsFilePath))) {
        await this.createEmptyRegistry();
        logger.info('Created new agent registry');
      } else {
        await this.loadRegistry();
        logger.info(`Loaded existing registry with ${this.agents.size} agents`);
      }

      this.initialized = true;
      this.emit('registry-initialized');

    } catch (error) {
      logger.error('Failed to initialize registry:', error);
      throw error;
    }
  }

  private async createEmptyRegistry(): Promise<void> {
    const emptyAgents: Record<string, AgentConfig> = {};
    const emptyRelationships: Record<string, AgentRelationship[]> = {};
    const emptyMcpServices: Record<string, string[]> = {};

    await FileUtils.writeJson(this.agentsFilePath, emptyAgents);
    await FileUtils.writeJson(this.relationshipsFilePath, emptyRelationships);
    await FileUtils.writeJson(this.mcpServicesFilePath, emptyMcpServices);

    this.agents.clear();
    this.relationships.clear();
  }

  /**
   * Load registry data from disk
   */
  async loadRegistry(): Promise<void> {
    try {
      // Load agents
      const agentsData = await FileUtils.readJson<Record<string, AgentConfig>>(this.agentsFilePath);
      this.agents.clear();

      for (const [id, agent] of Object.entries(agentsData)) {
        // Convert date strings back to Date objects
        agent.createdAt = new Date(agent.createdAt);
        agent.lastModified = new Date(agent.lastModified);
        this.agents.set(id, agent);
      }

      // Load relationships
      if (await FileUtils.exists(this.relationshipsFilePath)) {
        const relationshipsData = await FileUtils.readJson<Record<string, AgentRelationship[]>>(this.relationshipsFilePath);
        this.relationships.clear();

        for (const [agentId, rels] of Object.entries(relationshipsData)) {
          const relationships = rels.map(rel => ({
            ...rel,
            createdAt: new Date(rel.createdAt)
          }));
          this.relationships.set(agentId, relationships);
        }
      }

      logger.info('Registry loaded from disk');
      this.emit('registry-loaded');

    } catch (error) {
      logger.error('Failed to load registry:', error);
      throw error;
    }
  }

  private async saveRegistry(): Promise<void> {
    try {
      // Convert Map to plain object for JSON serialization
      const agentsData: Record<string, AgentConfig> = {};
      for (const [id, agent] of Array.from(this.agents)) {
        agentsData[id] = agent;
      }

      const relationshipsData: Record<string, AgentRelationship[]> = {};
      for (const [agentId, rels] of Array.from(this.relationships)) {
        relationshipsData[agentId] = rels;
      }

      await FileUtils.writeJson(this.agentsFilePath, agentsData);
      await FileUtils.writeJson(this.relationshipsFilePath, relationshipsData);

      this.emit('registry-updated');
      logger.debug('Registry saved successfully');

    } catch (error) {
      logger.error('Failed to save registry:', error);
      throw error;
    }
  }

  async createAgent(config: Partial<AgentConfig>): Promise<AgentConfig> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    // Validate configuration
    const validation = ValidationUtils.validateAgentConfig(config);
    if (!validation.valid) {
      throw new Error(`Invalid agent configuration: ${validation.error}`);
    }

    const agentId = config.id!;

    // Check if agent already exists
    if (this.agents.has(agentId)) {
      throw new Error(`Agent with ID '${agentId}' already exists`);
    }

    // Create full agent configuration
    const now = new Date();
    const agent: AgentConfig = {
      id: agentId,
      name: config.name || agentId,
      systemPrompt: config.systemPrompt!,
      status: 'inactive',
      createdAt: now,
      lastModified: now,
      resourceLimits: config.resourceLimits || { ...DEFAULT_RESOURCE_LIMITS },
      capabilities: config.capabilities || [],
      relationships: [],
      mcpServices: []
    };

    // Add to registry
    this.agents.set(agentId, agent);
    await this.saveRegistry();

    logger.info(`Agent created: ${agentId}`);
    this.emit('agent-created', agent);

    return agent;
  }

  async getAgent(agentId: string): Promise<AgentConfig | null> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    return this.agents.get(agentId) || null;
  }

  async updateAgent(agentId: string, updates: Partial<AgentConfig>): Promise<AgentConfig> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new Error(`Agent '${agentId}' not found`);
    }

    // Validate updates
    if (updates.systemPrompt) {
      const validation = ValidationUtils.validateSystemPrompt(updates.systemPrompt);
      if (!validation.valid) {
        throw new Error(`Invalid system prompt: ${validation.error}`);
      }
    }

    if (updates.resourceLimits) {
      const validation = ValidationUtils.validateResourceLimits(updates.resourceLimits);
      if (!validation.valid) {
        throw new Error(`Invalid resource limits: ${validation.error}`);
      }
    }

    // Apply updates
    const updatedAgent: AgentConfig = {
      ...agent,
      ...updates,
      id: agentId, // Prevent ID changes
      lastModified: new Date()
    };

    this.agents.set(agentId, updatedAgent);
    await this.saveRegistry();

    logger.info(`Agent updated: ${agentId}`);
    this.emit('agent-updated', updatedAgent);

    return updatedAgent;
  }

  async deleteAgent(agentId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    if (!this.agents.has(agentId)) {
      throw new Error(`Agent '${agentId}' not found`);
    }

    // Remove agent and its relationships
    this.agents.delete(agentId);
    this.relationships.delete(agentId);

    // Remove relationships where this agent is the target
    for (const [otherId, rels] of Array.from(this.relationships)) {
      const filtered = rels.filter(rel => rel.targetAgentId !== agentId);
      if (filtered.length !== rels.length) {
        this.relationships.set(otherId, filtered);
      }
    }

    await this.saveRegistry();

    // Clean up agent-specific files
    await this.cleanupAgentFiles(agentId);

    logger.info(`Agent deleted: ${agentId}`);
    this.emit('agent-deleted', agentId);
  }

  private async cleanupAgentFiles(agentId: string): Promise<void> {
    try {
      const agentDir = path.join(this.registryPath, 'agents', agentId);
      const taskFile = path.join(this.registryPath, 'tasks', `${agentId}.md`);
      const conversationDir = path.join(this.registryPath, 'conversations', agentId);

      // Remove agent directory
      if (await FileUtils.exists(agentDir)) {
        logger.debug(`Removing agent directory: ${agentDir}`);
        await FileUtils.deleteDirectory(agentDir);
      }

      // Remove conversation directory if it exists
      if (await FileUtils.exists(conversationDir)) {
        logger.debug(`Removing agent conversation directory: ${conversationDir}`);
        await FileUtils.deleteDirectory(conversationDir);
      }

      // Remove task file
      await FileUtils.deleteFile(taskFile);

      logger.info(`Cleaned up all files for agent ${agentId}`);

    } catch (error) {
      logger.warn(`Failed to cleanup files for agent ${agentId}:`, error);
    }
  }

  async listAgents(): Promise<AgentConfig[]> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    return Array.from(this.agents.values());
  }

  async findAgentsByCapability(capability: string): Promise<AgentConfig[]> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    const agents = Array.from(this.agents.values());
    return agents.filter(agent => 
      agent.capabilities.some(cap => 
        cap.toLowerCase().includes(capability.toLowerCase())
      ) ||
      agent.systemPrompt.toLowerCase().includes(capability.toLowerCase())
    );
  }

  async updateAgentStatus(agentId: string, status: AgentConfig['status']): Promise<void> {
    const agent = this.agents.get(agentId);
    if (!agent) {
      throw new Error(`Agent '${agentId}' not found`);
    }

    agent.status = status;
    agent.lastModified = new Date();

    this.agents.set(agentId, agent);
    await this.saveRegistry();

    logger.debug(`Agent status updated: ${agentId} -> ${status}`);
    this.emit('agent-status-changed', agentId, status);
  }

  async addRelationship(fromAgentId: string, relationship: Omit<AgentRelationship, 'createdAt'>): Promise<void> {
    if (!this.agents.has(fromAgentId)) {
      throw new Error(`Agent '${fromAgentId}' not found`);
    }

    if (!this.agents.has(relationship.targetAgentId)) {
      throw new Error(`Target agent '${relationship.targetAgentId}' not found`);
    }

    const relationships = this.relationships.get(fromAgentId) || [];
    const fullRelationship: AgentRelationship = {
      ...relationship,
      createdAt: new Date()
    };

    relationships.push(fullRelationship);
    this.relationships.set(fromAgentId, relationships);

    await this.saveRegistry();
    logger.info(`Relationship added: ${fromAgentId} -> ${relationship.targetAgentId} (${relationship.relationshipType})`);
  }

  async getRelationships(agentId: string): Promise<AgentRelationship[]> {
    return this.relationships.get(agentId) || [];
  }

  async getStats(): Promise<RegistryStats> {
    if (!this.initialized) {
      throw new Error('Registry not initialized');
    }

    const agents = Array.from(this.agents.values());
    const activeAgents = agents.filter(a => a.status === 'active').length;
    const inactiveAgents = agents.filter(a => a.status === 'inactive').length;
    const errorAgents = agents.filter(a => a.status === 'error' || a.status === 'crashed').length;

    let lastModified = new Date(0);
    agents.forEach(agent => {
      if (agent.lastModified > lastModified) {
        lastModified = agent.lastModified;
      }
    });

    const stats = await FileUtils.getFileStats(this.agentsFilePath);
    const registrySize = Number(stats?.size || 0);

    return {
      totalAgents: agents.length,
      activeAgents,
      inactiveAgents,
      errorAgents,
      lastModified,
      registrySize
    };
  }

  get isInitialized(): boolean {
    return this.initialized;
  }

  get path(): string {
    return this.registryPath;
  }
}
