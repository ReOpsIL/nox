/**
 * Approval Notifications - Handles various notification channels for approval requests
 * Supports email, webhook, and system notifications
 */

import { EventEmitter } from 'events';
import { ApprovalRequest, ApprovalResponse } from './approval-manager';
import { logger } from '../utils/logger';

export interface NotificationChannel {
  type: 'email' | 'webhook' | 'system' | 'desktop';
  config: Record<string, any>;
  enabled: boolean;
}

export interface NotificationConfig {
  channels: NotificationChannel[];
  templates: {
    requestSubject?: string;
    requestBody?: string;
    responseSubject?: string;
    responseBody?: string;
  };
  filters: {
    riskLevels?: Array<'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'>;
    types?: string[];
    agents?: string[];
  };
}

/**
 * Approval Notification Manager
 */
export class ApprovalNotificationManager extends EventEmitter {
  private channels: Map<string, NotificationChannel> = new Map();
  private templates: Record<string, string> = {};
  private filters: NotificationConfig['filters'] = {};

  constructor(config?: NotificationConfig) {
    super();
    
    if (config) {
      this.configure(config);
    }

    // Set default templates
    this.templates = {
      requestSubject: 'Approval Required: {{title}}',
      requestBody: `
An approval request requires your attention:

Title: {{title}}
Type: {{type}}
Risk Level: {{riskLevel}}
Requested by: {{requestedBy}}
Requested at: {{requestedAt}}

Description:
{{description}}

Please review and approve or reject this request.
      `.trim(),
      responseSubject: 'Approval {{status}}: {{title}}',
      responseBody: `
The approval request has been {{status}}:

Title: {{title}}
Type: {{type}}
Responded by: {{respondedBy}}
Responded at: {{respondedAt}}
{{#reason}}
Reason: {{reason}}
{{/reason}}
      `.trim(),
      ...config?.templates
    };
  }

  /**
   * Configure notification settings
   */
  configure(config: NotificationConfig): void {
    // Clear existing channels
    this.channels.clear();

    // Add new channels
    for (const channel of config.channels) {
      this.channels.set(`${channel.type}_${Date.now()}`, channel);
    }

    // Update templates
    if (config.templates) {
      Object.assign(this.templates, config.templates);
    }

    // Update filters
    this.filters = { ...config.filters };

    logger.info(`Configured ${this.channels.size} notification channels`);
  }

  /**
   * Send notification for approval request
   */
  async notifyApprovalRequest(request: ApprovalRequest): Promise<void> {
    if (!this.shouldNotify(request)) {
      logger.debug(`Skipping notification for request ${request.id} due to filters`);
      return;
    }

    const subject = this.renderTemplate(this.templates.requestSubject || '', request);
    const body = this.renderTemplate(this.templates.requestBody || '', request);

    const notifications = Array.from(this.channels.values())
      .filter(channel => channel.enabled)
      .map(channel => this.sendNotification(channel, {
        type: 'request',
        subject,
        body,
        request
      }));

    try {
      await Promise.allSettled(notifications);
      logger.info(`Sent approval request notifications for: ${request.title}`);
      this.emit('notification-sent', 'request', request);
    } catch (error) {
      logger.error('Failed to send approval request notifications:', error);
      this.emit('notification-error', 'request', request, error);
    }
  }

  /**
   * Send notification for approval response
   */
  async notifyApprovalResponse(
    request: ApprovalRequest, 
    response: ApprovalResponse
  ): Promise<void> {
    const context = {
      ...request,
      ...response,
      status: response.approved ? 'approved' : 'rejected'
    };

    const subject = this.renderTemplate(this.templates.responseSubject || '', context);
    const body = this.renderTemplate(this.templates.responseBody || '', context);

    const notifications = Array.from(this.channels.values())
      .filter(channel => channel.enabled)
      .map(channel => this.sendNotification(channel, {
        type: 'response',
        subject,
        body,
        request,
        response
      }));

    try {
      await Promise.allSettled(notifications);
      logger.info(`Sent approval response notifications for: ${request.title}`);
      this.emit('notification-sent', 'response', request, response);
    } catch (error) {
      logger.error('Failed to send approval response notifications:', error);
      this.emit('notification-error', 'response', request, response, error);
    }
  }

  /**
   * Send notification through specific channel
   */
  private async sendNotification(
    channel: NotificationChannel,
    notification: {
      type: 'request' | 'response';
      subject: string;
      body: string;
      request: ApprovalRequest;
      response?: ApprovalResponse;
    }
  ): Promise<void> {
    try {
      switch (channel.type) {
        case 'webhook':
          await this.sendWebhookNotification(channel, notification);
          break;
        case 'system':
          await this.sendSystemNotification(channel, notification);
          break;
        case 'desktop':
          await this.sendDesktopNotification(channel, notification);
          break;
        case 'email':
          await this.sendEmailNotification(channel, notification);
          break;
        default:
          logger.warn(`Unknown notification channel type: ${channel.type}`);
      }
    } catch (error) {
      logger.error(`Failed to send ${channel.type} notification:`, error);
      throw error;
    }
  }

  /**
   * Send webhook notification
   */
  private async sendWebhookNotification(
    channel: NotificationChannel,
    notification: any
  ): Promise<void> {
    const { url, method = 'POST', headers = {} } = channel.config;
    
    if (!url) {
      throw new Error('Webhook URL is required');
    }

    const payload = {
      type: notification.type,
      subject: notification.subject,
      body: notification.body,
      request: notification.request,
      response: notification.response,
      timestamp: new Date().toISOString()
    };

    const response = await fetch(url, {
      method,
      headers: {
        'Content-Type': 'application/json',
        ...headers
      },
      body: JSON.stringify(payload)
    });

    if (!response.ok) {
      throw new Error(`Webhook notification failed: ${response.status} ${response.statusText}`);
    }

    logger.debug(`Webhook notification sent to: ${url}`);
  }

  /**
   * Send system notification (console/log)
   */
  private async sendSystemNotification(
    channel: NotificationChannel,
    notification: any
  ): Promise<void> {
    const { level = 'info' } = channel.config;
    
    const message = `${notification.subject}\n${notification.body}`;
    
    switch (level) {
      case 'error':
        logger.error(message);
        break;
      case 'warn':
        logger.warn(message);
        break;
      case 'debug':
        logger.debug(message);
        break;
      default:
        logger.info(message);
    }
  }

  /**
   * Send desktop notification
   */
  private async sendDesktopNotification(
    _channel: NotificationChannel,
    notification: any
  ): Promise<void> {
    // This would integrate with system notification APIs
    // For now, we'll use console output
    console.log('\n🔔 APPROVAL NOTIFICATION 🔔');
    console.log('Subject:', notification.subject);
    console.log('Message:', notification.body);
    console.log('');
  }

  /**
   * Send email notification (placeholder)
   */
  private async sendEmailNotification(
    channel: NotificationChannel,
    notification: any
  ): Promise<void> {
    const { to, from: _from, smtp: _smtp } = channel.config;
    
    // This would integrate with email service
    logger.info(`Email notification would be sent to: ${to}`);
    logger.debug(`Subject: ${notification.subject}`);
    logger.debug(`Body: ${notification.body}`);
  }

  /**
   * Check if request should trigger notifications
   */
  private shouldNotify(request: ApprovalRequest): boolean {
    // Check risk level filter
    if (this.filters.riskLevels && this.filters.riskLevels.length > 0) {
      if (!this.filters.riskLevels.includes(request.riskLevel)) {
        return false;
      }
    }

    // Check type filter
    if (this.filters.types && this.filters.types.length > 0) {
      if (!this.filters.types.includes(request.type)) {
        return false;
      }
    }

    // Check agent filter
    if (this.filters.agents && this.filters.agents.length > 0) {
      if (!this.filters.agents.includes(request.requestedBy)) {
        return false;
      }
    }

    return true;
  }

  /**
   * Render template with context data
   */
  private renderTemplate(template: string, context: any): string {
    let rendered = template;

    // Simple template replacement
    for (const [key, value] of Object.entries(context)) {
      const regex = new RegExp(`{{${key}}}`, 'g');
      rendered = rendered.replace(regex, String(value));
    }

    // Handle conditional blocks (basic implementation)
    rendered = rendered.replace(/{{#(\w+)}}(.*?){{\/\1}}/gs, (_match, key, content) => {
      return context[key] ? content : '';
    });

    return rendered;
  }

  /**
   * Add notification channel
   */
  addChannel(channel: NotificationChannel): string {
    const id = `${channel.type}_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    this.channels.set(id, channel);
    logger.info(`Added notification channel: ${channel.type}`);
    return id;
  }

  /**
   * Remove notification channel
   */
  removeChannel(channelId: string): boolean {
    const removed = this.channels.delete(channelId);
    if (removed) {
      logger.info(`Removed notification channel: ${channelId}`);
    }
    return removed;
  }

  /**
   * Get notification statistics
   */
  getStatistics(): {
    totalChannels: number;
    enabledChannels: number;
    channelsByType: Record<string, number>;
  } {
    const stats = {
      totalChannels: this.channels.size,
      enabledChannels: 0,
      channelsByType: {} as Record<string, number>
    };

    for (const channel of this.channels.values()) {
      if (channel.enabled) {
        stats.enabledChannels++;
      }
      
      stats.channelsByType[channel.type] = (stats.channelsByType[channel.type] || 0) + 1;
    }

    return stats;
  }
}