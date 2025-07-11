/**
 * Unit Tests for MCP Approval Manager
 * Tests approval workflows, risk assessment, and notification integration
 */

import { ApprovalManager, ApprovalRequest } from '../../../src/mcp/approval-manager';
import { DEFAULT_CONFIG } from '../../../src/types/config';
import * as path from 'path';
import * as os from 'os';
import * as fs from 'fs/promises';

describe('ApprovalManager', () => {
  let approvalManager: ApprovalManager;
  let tempDir: string;

  beforeEach(async () => {
    tempDir = await fs.mkdtemp(path.join(os.tmpdir(), 'approval-manager-test-'));
    approvalManager = new ApprovalManager(tempDir);
  });

  afterEach(async () => {
    try {
      await approvalManager.shutdown();
      await fs.rmdir(tempDir, { recursive: true });
    } catch (error) {
      // Ignore cleanup errors
    }
  });

  describe('Initialization', () => {
    it('should initialize successfully', async () => {
      await expect(approvalManager.initialize(DEFAULT_CONFIG)).resolves.not.toThrow();
      expect(approvalManager['initialized']).toBe(true);
    });

    it('should create approvals directory', async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
      const approvalsDir = path.join(tempDir, 'approvals');
      const stats = await fs.stat(approvalsDir);
      expect(stats.isDirectory()).toBe(true);
    });

    it('should not initialize twice', async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
      // Second initialization should warn but not throw
      await expect(approvalManager.initialize(DEFAULT_CONFIG)).resolves.not.toThrow();
    });
  });

  describe('Approval Requests', () => {
    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
    });

    it('should create approval request with auto-generated ID', async () => {
      const request: ApprovalRequest = {
        id: '',
        type: 'mcp_installation',
        title: 'Test Installation',
        description: 'Test MCP service installation',
        details: { serviceId: 'test/service' },
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'LOW'
      };

      // Set up auto-approval for testing
      approvalManager.setUserInteractionCallback(async () => true);

      const approved = await approvalManager.requestApproval(request);
      expect(approved).toBe(true);
      expect(request.id).toBeTruthy();
    });

    it('should auto-approve low risk requests', async () => {
      const lowRiskRequest: ApprovalRequest = {
        id: 'test-auto-approval',
        type: 'mcp_installation',
        title: 'Low Risk Installation',
        description: 'Low risk service installation',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'LOW',
        autoApproveThreshold: 'MEDIUM'
      };

      const approved = await approvalManager.requestApproval(lowRiskRequest);
      expect(approved).toBe(true);
    });

    it('should not auto-approve high risk requests', async () => {
      const highRiskRequest: ApprovalRequest = {
        id: 'test-high-risk',
        type: 'mcp_installation',
        title: 'High Risk Installation',
        description: 'High risk service installation',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'HIGH',
        autoApproveThreshold: 'MEDIUM'
      };

      // Set up rejection callback
      approvalManager.setUserInteractionCallback(async () => false);

      const approved = await approvalManager.requestApproval(highRiskRequest);
      expect(approved).toBe(false);
    });

    it('should handle approval expiration', async () => {
      const expiringRequest: ApprovalRequest = {
        id: 'test-expiring',
        type: 'mcp_installation',
        title: 'Expiring Request',
        description: 'Request that will expire',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM',
        expiresAt: new Date(Date.now() + 100) // Expire in 100ms
      };

      const approved = await approvalManager.requestApproval(expiringRequest);
      expect(approved).toBe(false);
    }, 10000);

    it('should generate proper expiration time', async () => {
      const request: ApprovalRequest = {
        id: 'test-expiration',
        type: 'mcp_installation',
        title: 'Test Expiration',
        description: 'Test expiration handling',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM'
      };

      // Set callback to avoid hanging
      approvalManager.setUserInteractionCallback(async () => true);

      await approvalManager.requestApproval(request);
      expect(request.expiresAt).toBeDefined();
      expect(request.expiresAt!.getTime()).toBeGreaterThan(Date.now());
    });
  });

  describe('Risk Level Assessment', () => {
    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
    });

    it('should correctly assess risk levels', () => {
      // Test risk level comparison
      
      // Test risk level comparison
      expect(approvalManager['isRiskLevelBelowThreshold']('LOW', 'MEDIUM')).toBe(true);
      expect(approvalManager['isRiskLevelBelowThreshold']('MEDIUM', 'MEDIUM')).toBe(true);
      expect(approvalManager['isRiskLevelBelowThreshold']('HIGH', 'MEDIUM')).toBe(false);
      expect(approvalManager['isRiskLevelBelowThreshold']('CRITICAL', 'HIGH')).toBe(false);
    });
  });

  describe('Approval Responses', () => {
    let pendingRequestId: string;

    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
      
      // Create a pending request
      const request: ApprovalRequest = {
        id: 'test-pending',
        type: 'mcp_installation',
        title: 'Pending Request',
        description: 'A request that will be responded to manually',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM'
      };

      // Add to pending manually
      const record = {
        request,
        status: 'pending' as const
      };
      approvalManager['pendingApprovals'].set(request.id, record);
      pendingRequestId = request.id;
    });

    it('should respond to approval request', async () => {
      const success = await approvalManager.respondToApproval(
        pendingRequestId,
        true,
        'test-user',
        'Approved for testing'
      );

      expect(success).toBe(true);
      expect(approvalManager.getPendingApproval(pendingRequestId)).toBeUndefined();
    });

    it('should handle rejection with reason', async () => {
      const success = await approvalManager.respondToApproval(
        pendingRequestId,
        false,
        'test-user',
        'Rejected due to security concerns'
      );

      expect(success).toBe(true);
      
      const history = approvalManager.getApprovalHistory(1);
      expect(history).toHaveLength(1);
      expect(history.length).toBeGreaterThan(0);
      const lastRecord = history[history.length - 1];
      expect(lastRecord).toBeDefined();
      expect(lastRecord!.status).toBe('rejected');
      expect(lastRecord!.response?.reason).toBe('Rejected due to security concerns');
    });

    it('should handle non-existent approval response', async () => {
      const success = await approvalManager.respondToApproval(
        'non-existent-id',
        true,
        'test-user'
      );

      expect(success).toBe(false);
    });
  });

  describe('Approval History', () => {
    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
    });

    it('should track approval history', async () => {
      const request: ApprovalRequest = {
        id: 'test-history',
        type: 'mcp_installation',
        title: 'History Test',
        description: 'Test approval history tracking',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'LOW',
        autoApproveThreshold: 'MEDIUM'
      };

      await approvalManager.requestApproval(request);
      
      const history = approvalManager.getApprovalHistory();
      expect(history.length).toBeGreaterThan(0);
      
      const lastEntry = history[history.length - 1];
      expect(lastEntry).toBeDefined();
      expect(lastEntry!.request.id).toBe('test-history');
      expect(lastEntry!.status).toBe('approved');
    });

    it('should filter history by agent', async () => {
      // Create multiple requests from different agents
      const requests = [
        {
          id: 'agent1-request',
          type: 'mcp_installation' as const,
          title: 'Agent 1 Request',
          description: 'Request from agent 1',
          details: {},
          requestedBy: 'agent-1',
          requestedAt: new Date(),
          riskLevel: 'LOW' as const,
          autoApproveThreshold: 'MEDIUM' as const
        },
        {
          id: 'agent2-request',
          type: 'mcp_installation' as const,
          title: 'Agent 2 Request',
          description: 'Request from agent 2',
          details: {},
          requestedBy: 'agent-2',
          requestedAt: new Date(),
          riskLevel: 'LOW' as const,
          autoApproveThreshold: 'MEDIUM' as const
        }
      ];

      for (const request of requests) {
        await approvalManager.requestApproval(request);
      }

      const agent1History = approvalManager.getAgentApprovalHistory('agent-1');
      const agent2History = approvalManager.getAgentApprovalHistory('agent-2');

      expect(agent1History.length).toBeGreaterThan(0);
      expect(agent2History.length).toBeGreaterThan(0);
      expect(agent1History.every(record => record.request.requestedBy === 'agent-1')).toBe(true);
      expect(agent2History.every(record => record.request.requestedBy === 'agent-2')).toBe(true);
    });

    it('should limit history results', () => {
      const history = approvalManager.getApprovalHistory(5);
      expect(history.length).toBeLessThanOrEqual(5);
    });
  });

  describe('Pending Approvals', () => {
    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
    });

    it('should list pending approvals', () => {
      const pendingApprovals = approvalManager.getPendingApprovals();
      expect(Array.isArray(pendingApprovals)).toBe(true);
    });

    it('should get specific pending approval', () => {
      const approval = approvalManager.getPendingApproval('non-existent-id');
      expect(approval).toBeUndefined();
    });

    it('should clean up expired approvals', async () => {
      // Add an expired approval manually
      const expiredRequest: ApprovalRequest = {
        id: 'expired-request',
        type: 'mcp_installation',
        title: 'Expired Request',
        description: 'This request has expired',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(Date.now() - 3600000), // 1 hour ago
        riskLevel: 'MEDIUM',
        expiresAt: new Date(Date.now() - 1800000) // Expired 30 minutes ago
      };

      const record = {
        request: expiredRequest,
        status: 'pending' as const
      };

      approvalManager['pendingApprovals'].set(expiredRequest.id, record);

      // Trigger cleanup
      approvalManager['cleanupExpiredApprovals']();

      expect(approvalManager.getPendingApproval(expiredRequest.id)).toBeUndefined();
    });
  });

  describe('User Interaction Callback', () => {
    beforeEach(async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);
    });

    it('should use user interaction callback when set', async () => {
      const mockCallback = jest.fn().mockResolvedValue(true);
      approvalManager.setUserInteractionCallback(mockCallback);

      const request: ApprovalRequest = {
        id: 'callback-test',
        type: 'mcp_installation',
        title: 'Callback Test',
        description: 'Test user callback',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM'
      };

      const approved = await approvalManager.requestApproval(request);
      
      expect(approved).toBe(true);
      expect(mockCallback).toHaveBeenCalledWith(request);
    });

    it('should handle callback rejection', async () => {
      const mockCallback = jest.fn().mockResolvedValue(false);
      approvalManager.setUserInteractionCallback(mockCallback);

      const request: ApprovalRequest = {
        id: 'callback-reject-test',
        type: 'mcp_installation',
        title: 'Callback Reject Test',
        description: 'Test user callback rejection',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'MEDIUM'
      };

      const approved = await approvalManager.requestApproval(request);
      
      expect(approved).toBe(false);
      expect(mockCallback).toHaveBeenCalledWith(request);
    });
  });

  describe('Persistence', () => {
    it('should save and load approval history', async () => {
      await approvalManager.initialize(DEFAULT_CONFIG);

      // Create an approval
      const request: ApprovalRequest = {
        id: 'persistence-test',
        type: 'mcp_installation',
        title: 'Persistence Test',
        description: 'Test approval persistence',
        details: {},
        requestedBy: 'test-agent',
        requestedAt: new Date(),
        riskLevel: 'LOW',
        autoApproveThreshold: 'MEDIUM'
      };

      await approvalManager.requestApproval(request);
      await approvalManager.shutdown();

      // Create new instance and load
      const newApprovalManager = new ApprovalManager(tempDir);
      await newApprovalManager.initialize(DEFAULT_CONFIG);

      const history = newApprovalManager.getApprovalHistory();
      expect(history.length).toBeGreaterThan(0);
      expect(history.some(record => record.request.id === 'persistence-test')).toBe(true);

      await newApprovalManager.shutdown();
    });
  });
});