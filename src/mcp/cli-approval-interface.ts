/**
 * CLI Approval Interface - Provides interactive command-line approval prompts
 * Handles user interaction for approval requests through the terminal
 */

import * as readline from 'readline';
import { ApprovalRequest } from './approval-manager';
import { logger } from '../utils/logger';

export interface CLIApprovalOptions {
  timeout?: number; // Timeout in seconds
  autoApprove?: boolean;
  skipInteractive?: boolean;
}

/**
 * CLI Approval Interface - Manages interactive approval prompts
 */
export class CLIApprovalInterface {
  private rl: readline.Interface | null = null;
  private pendingPrompts = new Map<string, {
    resolve: (approved: boolean) => void;
    reject: (error: Error) => void;
    timeout?: NodeJS.Timeout;
  }>();

  constructor() {
    // Create readline interface for user input
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout
    });
  }

  /**
   * Request approval through CLI interface
   */
  async requestApproval(
    request: ApprovalRequest, 
    options: CLIApprovalOptions = {}
  ): Promise<boolean> {
    if (options.autoApprove) {
      logger.info(`Auto-approving request: ${request.title}`);
      return true;
    }

    if (options.skipInteractive) {
      logger.warn(`Skipping interactive approval for: ${request.title}`);
      return false;
    }

    return new Promise<boolean>((resolve, reject) => {
      try {
        // Display approval request
        this.displayApprovalRequest(request);

        // Set up timeout if specified
        let timeoutHandle: NodeJS.Timeout | undefined;
        if (options.timeout) {
          timeoutHandle = setTimeout(() => {
            this.pendingPrompts.delete(request.id);
            logger.warn(`Approval request timed out: ${request.title}`);
            resolve(false);
          }, options.timeout * 1000);
        }

        // Store promise handlers
        this.pendingPrompts.set(request.id, {
          resolve,
          reject,
          ...(timeoutHandle && { timeout: timeoutHandle })
        });

        // Prompt for user input
        this.promptForApproval(request.id);

      } catch (error) {
        reject(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  /**
   * Display approval request details
   */
  private displayApprovalRequest(request: ApprovalRequest): void {
    console.log('\n' + '='.repeat(80));
    console.log(`🔐 APPROVAL REQUEST: ${request.title}`);
    console.log('='.repeat(80));
    console.log(`📋 Type: ${request.type}`);
    console.log(`⚠️  Risk Level: ${request.riskLevel}`);
    console.log(`👤 Requested by: ${request.requestedBy}`);
    console.log(`🕒 Requested at: ${request.requestedAt.toLocaleString()}`);
    
    if (request.expiresAt) {
      console.log(`⏰ Expires at: ${request.expiresAt.toLocaleString()}`);
    }
    
    console.log(`\n📄 Description:`);
    console.log(request.description);
    
    if (request.details && Object.keys(request.details).length > 0) {
      console.log(`\n🔍 Details:`);
      for (const [key, value] of Object.entries(request.details)) {
        console.log(`  • ${key}: ${this.formatDetailValue(value)}`);
      }
    }
    
    console.log('\n' + '='.repeat(80));
  }

  /**
   * Format detail values for display
   */
  private formatDetailValue(value: any): string {
    if (Array.isArray(value)) {
      return value.join(', ');
    }
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  }

  /**
   * Prompt user for approval decision
   */
  private promptForApproval(requestId: string): void {
    if (!this.rl) {
      throw new Error('Readline interface not initialized');
    }

    this.rl.question(
      '\n🤔 Do you approve this request? [y/N/details]: ',
      (answer) => {
        const normalized = answer.trim().toLowerCase();
        
        if (normalized === 'y' || normalized === 'yes') {
          this.handleApprovalResponse(requestId, true);
        } else if (normalized === 'n' || normalized === 'no' || normalized === '') {
          this.handleApprovalResponse(requestId, false);
        } else if (normalized === 'd' || normalized === 'details') {
          this.showDetailedView(requestId);
        } else {
          console.log('❓ Please enter "y" for yes, "n" for no, or "details" for more information.');
          this.promptForApproval(requestId);
        }
      }
    );
  }

  /**
   * Show detailed view and prompt again
   */
  private showDetailedView(requestId: string): void {
    console.log('\n📋 Additional Options:');
    console.log('  • Type "y" or "yes" to approve');
    console.log('  • Type "n" or "no" to reject');
    console.log('  • Type "reason" to add a rejection reason');
    console.log('  • Type "postpone" to postpone the decision');
    
    if (!this.rl) return;
    
    this.rl.question('\n🎯 Your choice: ', (answer) => {
      const normalized = answer.trim().toLowerCase();
      
      if (normalized === 'reason') {
        this.promptForReason(requestId);
      } else if (normalized === 'postpone') {
        console.log('⏸️  Request postponed. You can review it later.');
        this.handleApprovalResponse(requestId, false, 'Postponed by user');
      } else {
        // Handle as regular approval
        if (normalized === 'y' || normalized === 'yes') {
          this.handleApprovalResponse(requestId, true);
        } else {
          this.handleApprovalResponse(requestId, false);
        }
      }
    });
  }

  /**
   * Prompt for rejection reason
   */
  private promptForReason(requestId: string): void {
    if (!this.rl) return;
    
    this.rl.question('\n💬 Please provide a reason for rejection: ', (reason) => {
      const trimmedReason = reason.trim();
      if (trimmedReason) {
        this.handleApprovalResponse(requestId, false, trimmedReason);
      } else {
        console.log('⚠️  No reason provided.');
        this.handleApprovalResponse(requestId, false);
      }
    });
  }

  /**
   * Handle approval response
   */
  private handleApprovalResponse(requestId: string, approved: boolean, reason?: string): void {
    const pending = this.pendingPrompts.get(requestId);
    if (!pending) {
      logger.warn(`No pending approval found for request: ${requestId}`);
      return;
    }

    // Clear timeout
    if (pending.timeout) {
      clearTimeout(pending.timeout);
    }

    // Remove from pending
    this.pendingPrompts.delete(requestId);

    // Log the decision
    const decision = approved ? '✅ APPROVED' : '❌ REJECTED';
    console.log(`\n${decision}`);
    if (reason) {
      console.log(`📝 Reason: ${reason}`);
    }
    console.log('');

    // Resolve the promise
    pending.resolve(approved);
  }

  /**
   * Batch approval for multiple requests
   */
  async batchApproval(
    requests: ApprovalRequest[], 
    options: CLIApprovalOptions = {}
  ): Promise<Map<string, boolean>> {
    const results = new Map<string, boolean>();
    
    console.log(`\n📦 Processing ${requests.length} approval requests...`);
    
    if (options.autoApprove) {
      for (const request of requests) {
        results.set(request.id, true);
        console.log(`✅ Auto-approved: ${request.title}`);
      }
      return results;
    }

    for (let i = 0; i < requests.length; i++) {
      const request = requests[i];
      console.log(`\n📊 Request ${i + 1}/${requests.length}:`);
      
      try {
        if (request) {
          const approved = await this.requestApproval(request, options);
          results.set(request.id, approved);
        }
      } catch (error) {
        if (request) {
          logger.error(`Error processing approval request ${request.id}:`, error);
          results.set(request.id, false);
        }
      }
    }

    return results;
  }

  /**
   * Show approval summary
   */
  showApprovalSummary(results: Map<string, boolean>, requests: ApprovalRequest[]): void {
    console.log('\n' + '='.repeat(60));
    console.log('📊 APPROVAL SUMMARY');
    console.log('='.repeat(60));
    
    let approved = 0;
    let rejected = 0;
    
    for (const request of requests) {
      const result = results.get(request.id);
      const status = result ? '✅ APPROVED' : '❌ REJECTED';
      console.log(`${status}: ${request.title}`);
      
      if (result) {
        approved++;
      } else {
        rejected++;
      }
    }
    
    console.log('='.repeat(60));
    console.log(`✅ Approved: ${approved}`);
    console.log(`❌ Rejected: ${rejected}`);
    console.log(`📊 Total: ${requests.length}`);
    console.log('='.repeat(60));
  }

  /**
   * Cleanup resources
   */
  cleanup(): void {
    // Clear all pending timeouts
    for (const pending of this.pendingPrompts.values()) {
      if (pending.timeout) {
        clearTimeout(pending.timeout);
      }
      pending.resolve(false);
    }
    this.pendingPrompts.clear();

    // Close readline interface
    if (this.rl) {
      this.rl.close();
      this.rl = null;
    }
  }
}