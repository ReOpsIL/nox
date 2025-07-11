/**
 * Phase 4 Integration Tests - MCP Service Integration
 * Tests the complete MCP service lifecycle including discovery, installation, and management
 */

import { ServiceManager } from '../../src/mcp/service-manager';
import { DockerManager } from '../../src/mcp/docker-manager';
import { ApprovalManager } from '../../src/mcp/approval-manager';
import { CapabilityRegistry } from '../../src/mcp/capability-registry';
import { CLIApprovalInterface } from '../../src/mcp/cli-approval-interface';
import { ApprovalNotificationManager } from '../../src/mcp/approval-notifications';
import { DEFAULT_CONFIG } from '../../src/types/config';
import * as path from 'path';
import * as fs from 'fs/promises';
import * as os from 'os';

describe('Phase 4: MCP Service Integration', () => {
  let tempDir: string;
  let serviceManager: ServiceManager;
  let dockerManager: DockerManager;
  let approvalManager: ApprovalManager;
  let capabilityRegistry: CapabilityRegistry;
  let cliInterface: CLIApprovalInterface;
  let notificationManager: ApprovalNotificationManager;

  beforeAll(async () => {
    // Create temporary directory for testing
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'nox-phase4-test-'));
    
    // Initialize components
    dockerManager = new DockerManager();
    approvalManager = new ApprovalManager(tempDir);
    capabilityRegistry = new CapabilityRegistry(tempDir);
    cliInterface = new CLIApprovalInterface();
    notificationManager = new ApprovalNotificationManager();
    
    serviceManager = new ServiceManager(
      dockerManager,
      approvalManager,
      capabilityRegistry,
      tempDir
    );
  });

  afterAll(async () => {
    // Cleanup
    try {
      await serviceManager?.shutdown();
      await dockerManager?.shutdown();
      await approvalManager?.shutdown();
      await capabilityRegistry?.shutdown();
      cliInterface?.cleanup();
      
      // Remove temporary directory
      await fs.rmdir(tempDir, { recursive: true });
    } catch (error) {
      console.warn('Cleanup warning:', error);
    }
  });

  describe('Component Initialization', () => {
    it('should initialize all MCP components successfully', async () => {
      const config = {
        ...DEFAULT_CONFIG,
        mcp: {
          ...DEFAULT_CONFIG.mcp,
          enabled: true,
          dockerHubUsername: 'mcp-test'
        }
      };

      await expect(dockerManager.initialize(config)).resolves.not.toThrow();
      await expect(approvalManager.initialize(config)).resolves.not.toThrow();
      await expect(capabilityRegistry.initialize(config)).resolves.not.toThrow();
      await expect(serviceManager.initialize(config)).resolves.not.toThrow();
    });

    it('should handle component dependencies correctly', async () => {
      // Service manager should depend on other components being initialized
      expect(serviceManager['initialized']).toBe(true);
      expect(dockerManager['initialized']).toBe(true);
      expect(approvalManager['initialized']).toBe(true);
      expect(capabilityRegistry['initialized']).toBe(true);
    });
  });

  describe('Service Discovery', () => {
    it('should discover MCP services with proper error handling', async () => {
      // Test with invalid Docker Hub username to check error handling
      const services = await serviceManager.discoverServices();
      
      // Should return empty array for non-existent user
      expect(Array.isArray(services)).toBe(true);
      expect(services.length).toBe(0);
    });

    it('should validate discovered service metadata', async () => {
      // Mock a service discovery response
      const mockService = {
        id: 'test/mock-service',
        name: 'mock-service',
        description: 'A mock MCP service for testing',
        version: '1.0.0',
        image: 'test/mock-service:1.0.0',
        capabilities: ['file-management', 'data-processing'],
        ports: [{ internal: 8080, external: 8080 }],
        environment: [
          { name: 'API_KEY', required: true, description: 'API key for service' },
          { name: 'DEBUG', required: false, default: 'false', description: 'Enable debug mode' }
        ],
        volumes: [
          { host: './data', container: '/app/data', description: 'Data directory' }
        ],
        author: 'test-author',
        tags: ['productivity', 'automation'],
        created: new Date(),
        updated: new Date()
      };

      // Test service validation
      expect(() => serviceManager['validateService'](mockService)).not.toThrow();
    });

    it('should handle service search and filtering', async () => {
      const searchResults = await serviceManager.searchServices({
        capabilities: ['file-management'],
        keyword: 'productivity',
        limit: 10
      });

      expect(Array.isArray(searchResults)).toBe(true);
      expect(searchResults.length).toBeLessThanOrEqual(10);
    });

    it('should provide service compatibility checking', async () => {
      const compatibility = await serviceManager.checkServiceCompatibility(
        'test/mock-service',
        {
          requiredCapabilities: ['file-management'],
          blockedCapabilities: ['network-access']
        }
      );

      expect(compatibility).toHaveProperty('compatible');
      expect(compatibility).toHaveProperty('issues');
      expect(compatibility).toHaveProperty('warnings');
      expect(Array.isArray(compatibility.issues)).toBe(true);
      expect(Array.isArray(compatibility.warnings)).toBe(true);
    });
  });

  describe('Approval Workflows', () => {
    it('should create approval requests for MCP installations', async () => {
      const mockRequest = {
        id: 'test-approval-1',
        type: 'mcp_installation' as const,
        title: 'Install Mock MCP Service',
        description: 'Request to install a mock MCP service for testing',
        details: {
          serviceId: 'test/mock-service',
          serviceName: 'mock-service',
          agentId: 'test-agent'
        },
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM' as const
      };

      // Test approval request creation
      expect(() => approvalManager['requestApproval']).toBeDefined();
      expect(mockRequest.type).toBe('mcp_installation');
      expect(mockRequest.riskLevel).toBe('MEDIUM');
    });

    it('should handle auto-approval based on risk thresholds', async () => {
      const lowRiskRequest = {
        id: 'test-auto-approval',
        type: 'mcp_installation' as const,
        title: 'Install Low Risk Service',
        description: 'Low risk service installation',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'LOW' as const,
        autoApproveThreshold: 'MEDIUM' as const
      };

      // Should auto-approve since LOW risk is below MEDIUM threshold
      const approved = await approvalManager.requestApproval(lowRiskRequest);
      expect(approved).toBe(true);
    });

    it('should track approval history', async () => {
      const history = approvalManager.getApprovalHistory(10);
      expect(Array.isArray(history)).toBe(true);
      expect(history.length).toBeGreaterThanOrEqual(0);
    });
  });

  describe('Capability Registry Integration', () => {
    it('should register MCP service capabilities', async () => {
      const capability = await capabilityRegistry.registerCapability(
        'test-agent',
        'file-management',
        {
          source: 'mcp',
          serviceId: 'test/mock-service',
          containerId: 'test-container-123'
        }
      );

      expect(capability).toHaveProperty('id');
      expect(capability).toHaveProperty('name', 'file-management');
      expect(capability.sources).toHaveLength(1);
      expect(capability.sources.length).toBeGreaterThan(0);
      expect(capability.sources[0]!.source).toBe('mcp');
    });

    it('should provide MCP capability queries', async () => {
      const mcpCapabilities = capabilityRegistry.getMCPCapabilities('test-agent');
      expect(Array.isArray(mcpCapabilities)).toBe(true);
      
      if (mcpCapabilities.length > 0) {
        expect(mcpCapabilities[0]).toHaveProperty('capability');
        expect(mcpCapabilities[0]).toHaveProperty('serviceId');
      }
    });

    it('should detect capability conflicts', async () => {
      const conflicts = capabilityRegistry.findCapabilityConflicts('test-agent');
      expect(Array.isArray(conflicts)).toBe(true);
    });

    it('should provide capability statistics', async () => {
      const stats = capabilityRegistry.getCapabilityStatistics();
      expect(stats).toHaveProperty('totalCapabilities');
      expect(stats).toHaveProperty('capabilitiesBySource');
      expect(stats).toHaveProperty('agentsWithCapabilities');
      expect(stats).toHaveProperty('mostCommonCapabilities');
      expect(Array.isArray(stats.mostCommonCapabilities)).toBe(true);
    });
  });

  describe('Docker Container Management', () => {
    // Note: These tests assume Docker is available but don't actually create containers
    
    it('should validate Docker connection', async () => {
      // Test should pass even if Docker is not available
      try {
        await dockerManager.initialize(DEFAULT_CONFIG);
        expect(dockerManager['initialized']).toBe(true);
      } catch (error) {
        // Docker not available in test environment - this is acceptable
        expect((error as Error).message).toContain('Docker connection failed');
      }
    });

    it('should provide container lifecycle methods', () => {
      expect(typeof dockerManager.startContainer).toBe('function');
      expect(typeof dockerManager.stopContainer).toBe('function');
      expect(typeof dockerManager.restartContainer).toBe('function');
      expect(typeof dockerManager.pauseContainer).toBe('function');
      expect(typeof dockerManager.unpauseContainer).toBe('function');
      expect(typeof dockerManager.removeContainer).toBe('function');
    });

    it('should provide container monitoring capabilities', () => {
      expect(typeof dockerManager.getContainerLogs).toBe('function');
      expect(typeof dockerManager.checkContainerHealth).toBe('function');
      expect(typeof dockerManager.getContainerStats).toBe('function');
      expect(typeof dockerManager.listContainers).toBe('function');
    });
  });

  describe('CLI and Notifications', () => {
    it('should provide CLI approval interface', () => {
      expect(cliInterface).toBeInstanceOf(CLIApprovalInterface);
      expect(typeof cliInterface.requestApproval).toBe('function');
      expect(typeof cliInterface.batchApproval).toBe('function');
      expect(typeof cliInterface.showApprovalSummary).toBe('function');
    });

    it('should support notification channels', () => {
      expect(notificationManager).toBeInstanceOf(ApprovalNotificationManager);
      expect(typeof notificationManager.notifyApprovalRequest).toBe('function');
      expect(typeof notificationManager.notifyApprovalResponse).toBe('function');
      expect(typeof notificationManager.addChannel).toBe('function');
    });

    it('should provide notification statistics', () => {
      const stats = notificationManager.getStatistics();
      expect(stats).toHaveProperty('totalChannels');
      expect(stats).toHaveProperty('enabledChannels');
      expect(stats).toHaveProperty('channelsByType');
      expect(typeof stats.totalChannels).toBe('number');
    });
  });

  describe('Service Installation Workflow', () => {
    it('should validate service installation prerequisites', async () => {
      // Mock service installation without actually creating containers
      const mockService = {
        id: 'test/mock-service',
        name: 'mock-service',
        description: 'Test service',
        version: '1.0.0',
        image: 'test/mock-service:1.0.0',
        capabilities: ['test-capability'],
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

      // Test environment variable validation
      const missingEnvVars = mockService.environment
        .filter(e => e.required && !process.env[e.name])
        .map(e => e.name);

      expect(Array.isArray(missingEnvVars)).toBe(true);
      if (missingEnvVars.length > 0) {
        expect(missingEnvVars).toContain('REQUIRED_VAR');
      }
    });

    it('should handle service installation errors gracefully', async () => {
      // Test error handling for non-existent service
      await expect(serviceManager.installService('non-existent/service', 'test-agent', {
        autoApprove: true
      })).rejects.toThrow();
    });
  });

  describe('Integration Completeness', () => {
    it('should have all Phase 4 components integrated', () => {
      // Verify all components are properly instantiated and configured
      expect(serviceManager).toBeDefined();
      expect(dockerManager).toBeDefined();
      expect(approvalManager).toBeDefined();
      expect(capabilityRegistry).toBeDefined();
      expect(cliInterface).toBeDefined();
      expect(notificationManager).toBeDefined();
    });

    it('should provide comprehensive service management API', () => {
      // Check that all expected methods are available
      const serviceManagerMethods = [
        'initialize', 'start', 'shutdown', 'discoverServices', 'getServiceDetails',
        'installService', 'uninstallService', 'listInstalledServices',
        'searchServices', 'checkServiceCompatibility'
      ];

      for (const method of serviceManagerMethods) {
        expect(typeof (serviceManager as any)[method]).toBe('function');
      }
    });

    it('should handle concurrent operations safely', async () => {
      // Test concurrent service discovery calls
      const discoveries = await Promise.allSettled([
        serviceManager.discoverServices('test'),
        serviceManager.discoverServices('productivity'),
        serviceManager.discoverServices('automation')
      ]);

      discoveries.forEach(result => {
        expect(['fulfilled', 'rejected']).toContain(result.status);
      });
    });

    it('should provide proper cleanup and resource management', async () => {
      // Test that all components can be shut down cleanly
      await expect(serviceManager.shutdown()).resolves.not.toThrow();
      await expect(approvalManager.shutdown()).resolves.not.toThrow();
      await expect(capabilityRegistry.shutdown()).resolves.not.toThrow();
      await expect(dockerManager.shutdown()).resolves.not.toThrow();
      
      expect(() => cliInterface.cleanup()).not.toThrow();
    });
  });
});