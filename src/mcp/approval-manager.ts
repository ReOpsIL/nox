/**
 * Approval Manager - Handles user approval workflows for sensitive operations
 * Provides methods for requesting, tracking, and managing approvals
 */

import { EventEmitter } from 'events';
import * as path from 'path';
import * as fs from 'fs/promises';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';\nimport { CLIApprovalInterface } from './cli-approval-interface';\nimport { ApprovalNotificationManager } from './approval-notifications';

export type ApprovalType = 'mcp_installation' | 'agent_creation' | 'external_communication' | 'resource_allocation';
export type RiskLevel = 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface ApprovalRequest {
  id: string;
  type: ApprovalType;
  title: string;
  description: string;
  details: Record<string, any>;
  requestedBy: string;
  requestedAt: Date;
  riskLevel: RiskLevel;
  expiresAt?: Date;
  autoApproveThreshold?: RiskLevel;
}

export interface ApprovalResponse {
  requestId: string;
  approved: boolean;
  respondedBy: string;
  respondedAt: Date;
  reason?: string;
}

export interface ApprovalRecord {
  request: ApprovalRequest;
  response?: ApprovalResponse;
  status: 'pending' | 'approved' | 'rejected' | 'expired';
}

/**
 * Approval Manager - Manages approval workflows for sensitive operations
 */
export class ApprovalManager extends EventEmitter {
  private initialized = false;
  private workingDir: string;
  private approvalsDir: string;
  private pendingApprovals: Map<string, ApprovalRecord> = new Map();
  private approvalHistory: ApprovalRecord[] = [];
  private autoApproveThresholds: Record<ApprovalType, RiskLevel | null> = {
    mcp_installation: null,
    agent_creation: null,
    external_communication: null,
    resource_allocation: null
  };
  private defaultExpirationMinutes = 60; // 1 hour
  private maxHistorySize = 1000;
  private userInteractionCallback?: (request: ApprovalRequest) => Promise<boolean>;\n  private cliInterface?: CLIApprovalInterface;\n  private notificationManager?: ApprovalNotificationManager;

  constructor(workingDir: string) {
    super();
    this.workingDir = workingDir;
    this.approvalsDir = path.join(workingDir, 'approvals');
  }

  /**
   * Initialize the approval manager
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('ApprovalManager already initialized');
      return;
    }

    try {
      // Ensure approvals directory exists
      await fs.mkdir(this.approvalsDir, { recursive: true });

      // Set auto-approve thresholds from config
      if (config.security?.autoApproveThresholds) {
        for (const [type, level] of Object.entries(config.security.autoApproveThresholds)) {
          if (this.autoApproveThresholds.hasOwnProperty(type)) {
            this.autoApproveThresholds[type as ApprovalType] = level as RiskLevel | null;
          }
        }
      }

      // Set default expiration time
      if (config.security?.approvalExpirationMinutes) {
        this.defaultExpirationMinutes = config.security.approvalExpirationMinutes;
      }

      // Set max history size
      if (config.security?.approvalHistorySize) {
        this.maxHistorySize = config.security.approvalHistorySize;
      }

      // Load approval history
      await this.loadApprovalHistory();

      // Clean up expired approvals
      this.cleanupExpiredApprovals();

      this.initialized = true;
      logger.info('ApprovalManager initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize ApprovalManager:', error);
      throw error;
    }
  }

  /**
   * Start the approval manager
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('ApprovalManager not initialized');
    }

    logger.info('ApprovalManager started');
    this.emit('started');
  }

  /**
   * Shutdown the approval manager
   */
  async shutdown(): Promise<void> {
    // Save approval history
    await this.saveApprovalHistory();

    logger.info('ApprovalManager shutdown');
    this.emit('shutdown');
  }

  /**
   * Set the user interaction callback
   * This callback will be called when user approval is needed
   */
  setUserInteractionCallback(callback: (request: ApprovalRequest) => Promise<boolean>): void {
    this.userInteractionCallback = callback;
  }

  /**
   * Request approval for an operation
   */
  async requestApproval(request: ApprovalRequest): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('ApprovalManager not initialized');
    }

    try {
      // Ensure request has an ID
      if (!request.id) {
        request.id = `approval_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      }

      // Set expiration time if not provided
      if (!request.expiresAt) {
        request.expiresAt = new Date(Date.now() + this.defaultExpirationMinutes * 60 * 1000);
      }

      logger.info(`Approval requested: ${request.title} (${request.id})`);

      // Check if auto-approve is possible
      const autoApproveThreshold = request.autoApproveThreshold || this.autoApproveThresholds[request.type];
      const canAutoApprove = autoApproveThreshold !== null && this.isRiskLevelBelowThreshold(request.riskLevel, autoApproveThreshold);

      if (canAutoApprove) {
        logger.info(`Auto-approving request: ${request.id} (risk level ${request.riskLevel} below threshold ${autoApproveThreshold})`);
        
        // Create approval record
        const record: ApprovalRecord = {
          request,
          response: {
            requestId: request.id,
            approved: true,
            respondedBy: 'system',
            respondedAt: new Date(),
            reason: 'Auto-approved based on risk level'
          },
          status: 'approved'
        };

        // Add to history
        this.approvalHistory.push(record);
        this.trimApprovalHistory();

        // Save history
        await this.saveApprovalHistory();

        this.emit('approval-auto-approved', record);
        return true;
      }

      // Create approval record
      const record: ApprovalRecord = {
        request,
        status: 'pending'
      };

      // Add to pending approvals
      this.pendingApprovals.set(request.id, record);

      // Emit event
      this.emit('approval-requested', record);

      // If user interaction callback is set, use it
      if (this.userInteractionCallback) {
        const approved = await this.userInteractionCallback(request);
        await this.respondToApproval(request.id, approved, 'user');
        return approved;
      }

      // Otherwise, wait for manual approval
      return new Promise<boolean>((resolve) => {
        const listener = (response: ApprovalResponse) => {
          if (response.requestId === request.id) {
            this.off('approval-responded', listener);
            resolve(response.approved);
          }
        };

        this.on('approval-responded', listener);

        // Set timeout for expiration
        const expirationTime = request.expiresAt!.getTime() - Date.now();
        setTimeout(() => {
          // If still pending, expire it
          const record = this.pendingApprovals.get(request.id);
          if (record && record.status === 'pending') {
            this.off('approval-responded', listener);
            this.pendingApprovals.delete(request.id);
            
            record.status = 'expired';
            this.approvalHistory.push(record);
            
            this.emit('approval-expired', record);
            resolve(false);
          }
        }, expirationTime);
      });

    } catch (error) {
      logger.error(`Failed to request approval: ${error}`);
      return false;
    }
  }

  /**
   * Respond to an approval request
   */
  async respondToApproval(
    requestId: string,
    approved: boolean,
    respondedBy: string,
    reason?: string
  ): Promise<boolean> {
    if (!this.initialized) {
      throw new Error('ApprovalManager not initialized');
    }

    try {
      // Get approval record
      const record = this.pendingApprovals.get(requestId);
      if (!record) {
        logger.warn(`Approval request not found: ${requestId}`);
        return false;
      }

      // Create response
      const response: ApprovalResponse = {
        requestId,
        approved,
        respondedBy,
        respondedAt: new Date(),
        ...(reason && { reason })
      };

      // Update record
      record.response = response;
      record.status = approved ? 'approved' : 'rejected';

      // Remove from pending approvals
      this.pendingApprovals.delete(requestId);

      // Add to history
      this.approvalHistory.push(record);
      this.trimApprovalHistory();

      // Save history
      await this.saveApprovalHistory();

      // Emit event
      this.emit('approval-responded', response);
      this.emit(approved ? 'approval-approved' : 'approval-rejected', record);

      logger.info(`Approval ${approved ? 'approved' : 'rejected'}: ${record.request.title} (${requestId})`);
      return true;

    } catch (error) {
      logger.error(`Failed to respond to approval: ${error}`);
      return false;
    }
  }

  /**
   * Get a pending approval request
   */
  getPendingApproval(requestId: string): ApprovalRecord | undefined {
    return this.pendingApprovals.get(requestId);
  }

  /**
   * Get all pending approval requests
   */
  getPendingApprovals(): ApprovalRecord[] {
    return Array.from(this.pendingApprovals.values());
  }

  /**
   * Get approval history
   */
  getApprovalHistory(limit = 100): ApprovalRecord[] {
    return this.approvalHistory.slice(-limit);
  }

  /**
   * Get approval history for a specific agent
   */
  getAgentApprovalHistory(agentId: string, limit = 100): ApprovalRecord[] {
    return this.approvalHistory
      .filter(record => 
        record.request.requestedBy === agentId || 
        record.request.details.agentId === agentId
      )
      .slice(-limit);
  }

  /**
   * Check if a risk level is below a threshold
   */
  private isRiskLevelBelowThreshold(level: RiskLevel, threshold: RiskLevel): boolean {
    const levels: Record<RiskLevel, number> = {
      'LOW': 0,
      'MEDIUM': 1,
      'HIGH': 2,
      'CRITICAL': 3
    };
    
    return levels[level] <= levels[threshold];
  }

  /**
   * Clean up expired approvals
   */
  private cleanupExpiredApprovals(): void {
    const now = new Date();
    
    for (const [requestId, record] of this.pendingApprovals.entries()) {
      if (record.request.expiresAt && record.request.expiresAt < now) {
        // Mark as expired
        record.status = 'expired';
        
        // Remove from pending approvals
        this.pendingApprovals.delete(requestId);
        
        // Add to history
        this.approvalHistory.push(record);
        
        logger.info(`Approval request expired: ${record.request.title} (${requestId})`);
        this.emit('approval-expired', record);
      }
    }
    
    this.trimApprovalHistory();
  }

  /**
   * Trim approval history to max size
   */
  private trimApprovalHistory(): void {
    if (this.approvalHistory.length > this.maxHistorySize) {
      this.approvalHistory = this.approvalHistory.slice(-this.maxHistorySize);
    }
  }

  /**
   * Load approval history from disk
   */
  private async loadApprovalHistory(): Promise<void> {
    try {
      const filePath = path.join(this.approvalsDir, 'approval-history.json');
      const exists = await fs.access(filePath).then(() => true).catch(() => false);
      
      if (exists) {
        const data = await fs.readFile(filePath, 'utf-8');
        const saved = JSON.parse(data);
        
        // Parse dates in approval records
        this.approvalHistory = saved.history.map((record: any) => ({
          ...record,
          request: {
            ...record.request,
            requestedAt: new Date(record.request.requestedAt),
            expiresAt: record.request.expiresAt ? new Date(record.request.expiresAt) : undefined
          },
          response: record.response ? {
            ...record.response,
            respondedAt: new Date(record.response.respondedAt)
          } : undefined
        }));
        
        // Load pending approvals
        for (const record of saved.pending || []) {
          const parsedRecord = {
            ...record,
            request: {
              ...record.request,
              requestedAt: new Date(record.request.requestedAt),
              expiresAt: record.request.expiresAt ? new Date(record.request.expiresAt) : undefined
            }
          };
          
          // Only add if not expired
          if (parsedRecord.request.expiresAt && parsedRecord.request.expiresAt > new Date()) {
            this.pendingApprovals.set(parsedRecord.request.id, parsedRecord);
          } else {
            // Add to history as expired
            parsedRecord.status = 'expired';
            this.approvalHistory.push(parsedRecord);
          }
        }
        
        logger.info(`Loaded ${this.approvalHistory.length} approval records and ${this.pendingApprovals.size} pending approvals`);
      }
      
    } catch (error) {
      logger.error('Failed to load approval history:', error);
    }
  }

  /**
   * Save approval history to disk
   */
  private async saveApprovalHistory(): Promise<void> {
    try {
      const filePath = path.join(this.approvalsDir, 'approval-history.json');
      const data = {
        lastUpdated: new Date().toISOString(),
        history: this.approvalHistory,
        pending: Array.from(this.pendingApprovals.values())
      };
      
      await fs.writeFile(filePath, JSON.stringify(data, null, 2));
      logger.debug('Approval history saved to disk');
      
    } catch (error) {
      logger.error('Failed to save approval history:', error);
    }
  }
}