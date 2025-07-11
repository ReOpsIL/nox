import { RegistryManager } from '../../../src/core/registry-manager';
import { DEFAULT_RESOURCE_LIMITS } from '../../../src/types/agent';
import { FileUtils } from '../../../src/utils/file-utils';
import * as path from 'path';

describe('RegistryManager', () => {
  let registryManager: RegistryManager;
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    registryManager = new RegistryManager();
  });

  afterEach(async () => {
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('initialization', () => {
    it('should initialize empty registry', async () => {
      await registryManager.initialize(tempDir);
      
      expect(registryManager.isInitialized).toBe(true);
      expect(await FileUtils.exists(path.join(tempDir, 'agents.json'))).toBe(true);
      expect(await FileUtils.exists(path.join(tempDir, 'agent-relationships.json'))).toBe(true);
    });

    it('should load existing registry', async () => {
      // Create an existing registry
      const agentsData = {
        'test-agent': {
          id: 'test-agent',
          name: 'Test Agent',
          systemPrompt: 'Test prompt',
          status: 'inactive',
          createdAt: new Date().toISOString(),
          lastModified: new Date().toISOString(),
          resourceLimits: DEFAULT_RESOURCE_LIMITS,
          capabilities: [],
          relationships: [],
          mcpServices: []
        }
      };
      
      await FileUtils.ensureDirectory(tempDir);
      await FileUtils.writeJson(path.join(tempDir, 'agents.json'), agentsData);
      await FileUtils.writeJson(path.join(tempDir, 'agent-relationships.json'), {});
      await FileUtils.writeJson(path.join(tempDir, 'mcp-services.json'), {});
      
      await registryManager.initialize(tempDir);
      
      const agents = await registryManager.listAgents();
      expect(agents).toHaveLength(1);
      expect(agents[0]!.id).toBe('test-agent');
    });
  });

  describe('agent management', () => {
    beforeEach(async () => {
      await registryManager.initialize(tempDir);
    });

    it('should create a new agent', async () => {
      const agentConfig = {
        id: 'new-agent',
        name: 'New Agent',
        systemPrompt: 'You are a helpful assistant'
      };

      const agent = await registryManager.createAgent(agentConfig);

      expect(agent.id).toBe('new-agent');
      expect(agent.name).toBe('New Agent');
      expect(agent.status).toBe('inactive');
      expect(agent.resourceLimits).toEqual(DEFAULT_RESOURCE_LIMITS);
    });

    it('should reject duplicate agent IDs', async () => {
      const agentConfig = {
        id: 'duplicate-agent',
        name: 'First Agent',
        systemPrompt: 'First prompt'
      };

      await registryManager.createAgent(agentConfig);

      const duplicateConfig = {
        id: 'duplicate-agent',
        name: 'Second Agent',
        systemPrompt: 'Second prompt'
      };

      await expect(registryManager.createAgent(duplicateConfig))
        .rejects.toThrow("Agent with ID 'duplicate-agent' already exists");
    });

    it('should get agent by ID', async () => {
      const agentConfig = {
        id: 'get-test',
        name: 'Get Test',
        systemPrompt: 'Test prompt'
      };

      await registryManager.createAgent(agentConfig);
      const agent = await registryManager.getAgent('get-test');

      expect(agent).toBeTruthy();
      expect(agent!.id).toBe('get-test');
    });

    it('should return null for non-existent agent', async () => {
      const agent = await registryManager.getAgent('non-existent');
      expect(agent).toBeNull();
    });

    it('should update agent', async () => {
      const agentConfig = {
        id: 'update-test',
        name: 'Update Test',
        systemPrompt: 'Original prompt'
      };

      await registryManager.createAgent(agentConfig);
      
      const updatedAgent = await registryManager.updateAgent('update-test', {
        systemPrompt: 'Updated prompt',
        name: 'Updated Name'
      });

      expect(updatedAgent.systemPrompt).toBe('Updated prompt');
      expect(updatedAgent.name).toBe('Updated Name');
      expect(updatedAgent.id).toBe('update-test'); // ID should not change
    });

    it('should delete agent', async () => {
      const agentConfig = {
        id: 'delete-test',
        name: 'Delete Test',
        systemPrompt: 'Test prompt'
      };

      await registryManager.createAgent(agentConfig);
      await registryManager.deleteAgent('delete-test');

      const agent = await registryManager.getAgent('delete-test');
      expect(agent).toBeNull();
    });

    it('should list all agents', async () => {
      const configs = [
        { id: 'agent-1', name: 'Agent 1', systemPrompt: 'You are agent 1, a helpful assistant' },
        { id: 'agent-2', name: 'Agent 2', systemPrompt: 'You are agent 2, a helpful assistant' }
      ];

      for (const config of configs) {
        await registryManager.createAgent(config);
      }

      const agents = await registryManager.listAgents();
      expect(agents).toHaveLength(2);
      expect(agents.map(a => a.id)).toContain('agent-1');
      expect(agents.map(a => a.id)).toContain('agent-2');
    });
  });

  describe('capability search', () => {
    beforeEach(async () => {
      await registryManager.initialize(tempDir);
    });

    it('should find agents by capability', async () => {
      const configs = [
        { 
          id: 'ml-agent', 
          name: 'ML Agent', 
          systemPrompt: 'You are a machine learning expert',
          capabilities: ['machine-learning', 'data-analysis'] 
        },
        { 
          id: 'web-agent', 
          name: 'Web Agent', 
          systemPrompt: 'You are a web development expert',
          capabilities: ['web-development', 'javascript'] 
        }
      ];

      for (const config of configs) {
        await registryManager.createAgent(config);
      }

      const mlAgents = await registryManager.findAgentsByCapability('machine learning');
      expect(mlAgents).toHaveLength(1);
      expect(mlAgents[0]!.id).toBe('ml-agent');

      const webAgents = await registryManager.findAgentsByCapability('web');
      expect(webAgents).toHaveLength(1);
      expect(webAgents[0]!.id).toBe('web-agent');
    });
  });

  describe('status management', () => {
    beforeEach(async () => {
      await registryManager.initialize(tempDir);
    });

    it('should update agent status', async () => {
      const agentConfig = {
        id: 'status-test',
        name: 'Status Test',
        systemPrompt: 'Test prompt'
      };

      await registryManager.createAgent(agentConfig);
      await registryManager.updateAgentStatus('status-test', 'active');

      const agent = await registryManager.getAgent('status-test');
      expect(agent!.status).toBe('active');
    });
  });

  describe('relationships', () => {
    beforeEach(async () => {
      await registryManager.initialize(tempDir);
    });

    it('should add agent relationships', async () => {
      const configs = [
        { id: 'agent-a', name: 'Agent A', systemPrompt: 'You are agent A, a helpful assistant' },
        { id: 'agent-b', name: 'Agent B', systemPrompt: 'You are agent B, a helpful assistant' }
      ];

      for (const config of configs) {
        await registryManager.createAgent(config);
      }

      await registryManager.addRelationship('agent-a', {
        targetAgentId: 'agent-b',
        relationshipType: 'collaborator'
      });

      const relationships = await registryManager.getRelationships('agent-a');
      expect(relationships).toHaveLength(1);
      expect(relationships[0]!.targetAgentId).toBe('agent-b');
      expect(relationships[0]!.relationshipType).toBe('collaborator');
    });
  });

  describe('statistics', () => {
    beforeEach(async () => {
      await registryManager.initialize(tempDir);
    });

    it('should provide registry statistics', async () => {
      const configs = [
        { id: 'active-agent', name: 'Active', systemPrompt: 'You are an active agent' },
        { id: 'inactive-agent', name: 'Inactive', systemPrompt: 'You are an inactive agent' }
      ];

      for (const config of configs) {
        await registryManager.createAgent(config);
      }

      await registryManager.updateAgentStatus('active-agent', 'active');

      const stats = await registryManager.getStats();
      expect(stats.totalAgents).toBe(2);
      expect(stats.activeAgents).toBe(1);
      expect(stats.inactiveAgents).toBe(1);
      expect(stats.errorAgents).toBe(0);
    });
  });
});