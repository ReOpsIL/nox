/**
 * Unit Tests for MCP Service Manager
 * Tests service discovery, validation, and management functionality
 */

import { ServiceManager, MCPService } from '../../../src/mcp/service-manager';
import { DockerManager } from '../../../src/mcp/docker-manager';
import { ApprovalManager } from '../../../src/mcp/approval-manager';
import { CapabilityRegistry } from '../../../src/mcp/capability-registry';
import { DEFAULT_CONFIG } from '../../../src/types/config';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs/promises';

// Mock fetch for testing
global.fetch = jest.fn();

describe('ServiceManager', () => {
  let serviceManager: ServiceManager;
  let dockerManager: DockerManager;
  let approvalManager: ApprovalManager;
  let capabilityRegistry: CapabilityRegistry;
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'service-manager-test-'));
    
    dockerManager = new DockerManager();
    approvalManager = new ApprovalManager(tempDir);
    capabilityRegistry = new CapabilityRegistry(tempDir);
    
    serviceManager = new ServiceManager(
      dockerManager,
      approvalManager,
      capabilityRegistry,
      tempDir
    );

    // Mock Docker manager methods
    jest.spyOn(dockerManager, 'initialize').mockResolvedValue();
    jest.spyOn(dockerManager, 'createContainer').mockResolvedValue({
      id: 'mock-container-id',
      name: 'mock-container',
      image: 'mock-image',
      status: 'running',
      created: new Date(),
      ports: [],
      labels: {}
    });
  });

  afterEach(async () => {
    try {
      await fs.rmdir(tempDir, { recursive: true });
    } catch (error) {
      // Ignore cleanup errors
    }
    jest.resetAllMocks();
  });

  describe('Initialization', () => {
    it('should initialize successfully', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        mcp: {
          ...DEFAULT_CONFIG.mcp,
          enabled: true,
          dockerHubUsername: 'test-user'
        }
      };

      await expect(serviceManager.initialize(config)).resolves.not.toThrow();
      expect(serviceManager['initialized']).toBe(true);
    });

    it('should not initialize twice', async () => {
      const config = DEFAULT_CONFIG;
      await serviceManager.initialize(config);
      
      // Second initialization should not throw but warn
      await expect(serviceManager.initialize(config)).resolves.not.toThrow();
    });

    it('should set Docker Hub username from config', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        mcp: {
          ...DEFAULT_CONFIG.mcp,
          dockerHubUsername: 'custom-user'
        }
      };

      await serviceManager.initialize(config);
      expect(serviceManager['dockerHubUsername']).toBe('custom-user');
    });
  });

  describe('Service Discovery', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
    });

    it('should handle empty repository response', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ results: [] })
      });

      const services = await serviceManager.discoverServices();
      expect(services).toEqual([]);
    });

    it('should handle Docker Hub API errors', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: false,
        status: 404,
        statusText: 'Not Found'
      });

      const services = await serviceManager.discoverServices();
      expect(services).toEqual([]);
    });

    it('should filter services by MCP labels', async () => {
      const mockRepos = [
        {
          name: 'mcp-service-1',
          labels: ['mcp.service=true'],
          description: 'MCP service'
        },
        {
          name: 'regular-service',
          labels: [],
          description: 'Regular service'
        }
      ];

      (global.fetch as jest.Mock)
        .mockResolvedValueOnce({
          ok: true,
          json: () => Promise.resolve({ results: mockRepos })
        })
        .mockResolvedValueOnce({
          ok: true,
          json: () => Promise.resolve({
            name: 'mcp-service-1',
            description: 'MCP service',
            labels: ['mcp.service=true', 'mcp.capabilities=file-management'],
            date_registered: new Date().toISOString(),
            last_updated: new Date().toISOString(),
            user: 'test-user'
          })
        });

      const services = await serviceManager.discoverServices();
      expect(services).toHaveLength(1);
      expect(services.length).toBeGreaterThan(0);
      expect(services[0]!.name).toBe('mcp-service-1');
    });

    it('should parse service metadata correctly', async () => {
      const mockServiceData = {
        name: 'test-service',
        description: 'Test MCP service',
        labels: [
          'mcp.service=true',
          'mcp.capabilities=file-management,data-processing',
          'mcp.ports=8080:8080,9090:9090',
          'mcp.environment=API_KEY:true:null:API key,DEBUG:false:false:Debug mode',
          'mcp.volumes=./data:/app/data:Data directory',
          'mcp.version=1.0.0',
          'mcp.tags=productivity,automation'
        ],
        date_registered: new Date().toISOString(),
        last_updated: new Date().toISOString(),
        user: 'test-user'
      };

      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockServiceData)
      });

      const service = await serviceManager.getServiceDetails('test-service');

      expect(service.capabilities).toEqual(['file-management', 'data-processing']);
      expect(service.ports).toHaveLength(2);
      expect(service.ports[0]).toEqual({ internal: 8080, external: 8080 });
      expect(service.environment).toHaveLength(2);
      expect(service.environment[0]).toEqual({
        name: 'API_KEY',
        required: true,
        default: undefined,
        description: 'API key'
      });
      expect(service.volumes).toHaveLength(1);
      expect(service.tags).toEqual(['productivity', 'automation']);
    });

    it('should handle service discovery with query filtering', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ results: [] })
      });

      const services = await serviceManager.discoverServices('productivity');
      expect(services).toEqual([]);
    });

    it('should support advanced filtering options', async () => {
      (global.fetch as jest.Mock).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ results: [] })
      });

      const services = await serviceManager.discoverServices('test', {
        category: 'productivity',
        capabilities: ['file-management'],
        limit: 5,
        sortBy: 'updated'
      });

      expect(services).toEqual([]);
    });
  });

  describe('Service Validation', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
    });

    it('should validate valid service', () => {
      const validService: MCPService = {
        id: 'test/valid-service',
        name: 'valid-service',
        description: 'A valid test service',
        version: '1.0.0',
        image: 'test/valid-service:1.0.0',
        capabilities: ['file-management'],
        ports: [{ internal: 8080, external: 8080 }],
        environment: [{ name: 'API_KEY', required: true, description: 'API key' }],
        volumes: [{ host: './data', container: '/app/data', description: 'Data dir' }],
        author: 'test-author',
        tags: ['test'],
        created: new Date(),
        updated: new Date()
      };

      expect(() => serviceManager['validateService'](validService)).not.toThrow();
    });

    it('should reject service with invalid fields', () => {
      const invalidService = {
        id: '',
        name: '',
        image: '',
        capabilities: [''],
        ports: [{ internal: 0, external: 70000 }],
        environment: [{ name: '', required: true, description: '' }],
        volumes: [{ host: '', container: '', description: '' }]
      } as MCPService;

      expect(() => serviceManager['validateService'](invalidService)).toThrow();
    });

    it('should validate port ranges', () => {
      const serviceWithInvalidPorts: MCPService = {
        id: 'test/service',
        name: 'service',
        description: 'Test',
        version: '1.0.0',
        image: 'test/service:1.0.0',
        capabilities: ['test'],
        ports: [{ internal: -1, external: 70000 }],
        environment: [],
        volumes: [],
        author: 'test',
        tags: [],
        created: new Date(),
        updated: new Date()
      };

      expect(() => serviceManager['validateService'](serviceWithInvalidPorts)).toThrow();
    });
  });

  describe('Service Search', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
      
      // Mock some cached services
      const mockService: MCPService = {
        id: 'test/search-service',
        name: 'search-service',
        description: 'A searchable test service',
        version: '1.0.0',
        image: 'test/search-service:1.0.0',
        capabilities: ['file-management', 'data-processing'],
        ports: [],
        environment: [],
        volumes: [],
        author: 'test-author',
        tags: ['productivity', 'automation'],
        created: new Date(),
        updated: new Date()
      };
      
      serviceManager['serviceCache'].set(mockService.id, mockService);
    });

    it('should search by capability', async () => {
      const results = await serviceManager.searchServices({
        capabilities: ['file-management']
      });

      expect(results).toHaveLength(1);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]!.capabilities).toContain('file-management');
    });

    it('should search by keyword', async () => {
      const results = await serviceManager.searchServices({
        keyword: 'searchable'
      });

      expect(results).toHaveLength(1);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]!.description).toContain('searchable');
    });

    it('should search by category (tags)', async () => {
      const results = await serviceManager.searchServices({
        category: 'productivity'
      });

      expect(results).toHaveLength(1);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]!.tags).toContain('productivity');
    });

    it('should search by author', async () => {
      const results = await serviceManager.searchServices({
        author: 'test-author'
      });

      expect(results).toHaveLength(1);
      expect(results.length).toBeGreaterThan(0);
      expect(results[0]!.author).toBe('test-author');
    });

    it('should limit search results', async () => {
      const results = await serviceManager.searchServices({
        limit: 0
      });

      expect(results).toHaveLength(0);
    });
  });

  describe('Service Compatibility', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
      
      const mockService: MCPService = {
        id: 'test/compat-service',
        name: 'compat-service',
        description: 'Compatibility test service',
        version: '1.0.0',
        image: 'test/compat-service:1.0.0',
        capabilities: ['file-management', 'network-access'],
        ports: [],
        environment: [
          { name: 'REQUIRED_VAR', required: true, description: 'Required variable' }
        ],
        volumes: [],
        author: 'test',
        tags: [],
        created: new Date(),
        updated: new Date()
      };
      
      serviceManager['serviceCache'].set(mockService.id, mockService);
    });

    it('should check required capabilities', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'test/compat-service',
        { requiredCapabilities: ['file-management'] }
      );

      expect(compatibility.compatible).toBe(true);
      expect(compatibility.issues).toHaveLength(0);
    });

    it('should detect missing required capabilities', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'test/compat-service',
        { requiredCapabilities: ['database-access'] }
      );

      expect(compatibility.compatible).toBe(false);
      expect(compatibility.issues).toContain('Missing required capabilities: database-access');
    });

    it('should detect blocked capabilities', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'test/compat-service',
        { blockedCapabilities: ['network-access'] }
      );

      expect(compatibility.compatible).toBe(false);
      expect(compatibility.issues).toContain('Service has blocked capabilities: network-access');
    });

    it('should warn about required environment variables', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'test/compat-service'
      );

      expect(compatibility.warnings).toContain('Service requires 1 environment variables');
    });

    it('should handle non-existent services', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'non-existent/service'
      );

      expect(compatibility.compatible).toBe(false);
      expect(compatibility.issues).toContain('Service not found');
    });
  });

  describe('Service Installation', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
      
      // Mock approval manager
      jest.spyOn(approvalManager, 'requestApproval').mockResolvedValue(true);
      jest.spyOn(capabilityRegistry, 'registerCapability').mockResolvedValue({
        id: 'test-capability',
        name: 'test-capability',
        sources: [],
        addedAt: new Date(),
        updatedAt: new Date()
      });
    });

    it('should handle service installation with auto-approval', async () => {
      const mockService: MCPService = {
        id: 'test/install-service',
        name: 'install-service',
        description: 'Installation test service',
        version: '1.0.0',
        image: 'test/install-service:1.0.0',
        capabilities: ['test-capability'],
        ports: [],
        environment: [],
        volumes: [],
        author: 'test',
        tags: [],
        created: new Date(),
        updated: new Date()
      };
      
      serviceManager['serviceCache'].set(mockService.id, mockService);

      const installedService = await serviceManager.installService(
        'test/install-service',
        'test-agent',
        { autoApprove: true }
      );

      expect(installedService).toBeDefined();
      expect(installedService.containerId).toBe('mock-container-id');
      expect(installedService.agentId).toBe('test-agent');
    });

    it('should validate required environment variables', async () => {
      const mockService: MCPService = {
        id: 'test/env-service',
        name: 'env-service',
        description: 'Service with required env vars',
        version: '1.0.0',
        image: 'test/env-service:1.0.0',
        capabilities: [],
        ports: [],
        environment: [
          { name: 'REQUIRED_VAR', required: true, description: 'Required variable' }
        ],
        volumes: [],
        author: 'test',
        tags: [],
        created: new Date(),
        updated: new Date()
      };
      
      serviceManager['serviceCache'].set(mockService.id, mockService);

      await expect(serviceManager.installService(
        'test/env-service',
        'test-agent',
        { autoApprove: true }
      )).rejects.toThrow('Missing required environment variables: REQUIRED_VAR');
    });

    it('should handle service not found error', async () => {
      await expect(serviceManager.installService(
        'non-existent/service',
        'test-agent',
        { autoApprove: true }
      )).rejects.toThrow();
    });
  });

  describe('Installed Services Management', () => {
    beforeEach(async () => {
      await serviceManager.initialize(DEFAULT_CONFIG);
    });

    it('should list installed services', () => {
      const services = serviceManager.listInstalledServices();
      expect(Array.isArray(services)).toBe(true);
    });

    it('should filter installed services by agent', () => {
      const services = serviceManager.listInstalledServices('test-agent');
      expect(Array.isArray(services)).toBe(true);
    });

    it('should get installed service by container ID', () => {
      const service = serviceManager.getInstalledService('non-existent-container');
      expect(service).toBeUndefined();
    });
  });
});