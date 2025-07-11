import { AgentConfig, AgentMessage, ResourceLimits } from '../types';

export class ValidationUtils {
  static validateAgentId(id: string): boolean {
    if (!id || typeof id !== 'string') return false;
    if (id.length < 1 || id.length > 50) return false;
    // Only allow alphanumeric, underscore, and dash
    return /^[a-zA-Z0-9_-]+$/.test(id);
  }

  static validateAgentName(name: string): boolean {
    if (!name || typeof name !== 'string') return false;
    if (name.length < 1 || name.length > 100) return false;
    return true;
  }

  static validateSystemPrompt(prompt: string): ValidationResult {
    if (!prompt || typeof prompt !== 'string') {
      return { valid: false, error: 'System prompt is required and must be a string' };
    }

    if (prompt.length < 10) {
      return { valid: false, error: 'System prompt must be at least 10 characters long' };
    }

    if (prompt.length > 10000) {
      return { valid: false, error: 'System prompt must be less than 10,000 characters' };
    }

    // Check for potentially malicious content
    const maliciousPatterns = [
      /execute\s*\(/i,
      /eval\s*\(/i,
      /function\s*\(/i,
      /<script/i,
      /javascript:/i,
      /vbscript:/i,
      /data:text\/html/i,
      /\$\{.*\}/,  // Template literals
      /`.*`/,      // Backticks
      /exec\s*\(/i,
      /spawn\s*\(/i,
      /system\s*\(/i,
      /rm\s+-rf/i,
      /sudo\s+/i,
      /chmod\s+/i
    ];

    for (const pattern of maliciousPatterns) {
      if (pattern.test(prompt)) {
        return { valid: false, error: 'System prompt contains potentially dangerous content' };
      }
    }

    return { valid: true };
  }

  static validateResourceLimits(limits: ResourceLimits): ValidationResult {
    if (!limits || typeof limits !== 'object') {
      return { valid: false, error: 'Resource limits must be an object' };
    }

    const { maxMemoryMB, maxCPUPercent, maxConcurrentTasks, maxSpawnRate, maxDepth } = limits;

    if (!Number.isInteger(maxMemoryMB) || maxMemoryMB < 128 || maxMemoryMB > 16384) {
      return { valid: false, error: 'Memory limit must be between 128MB and 16GB' };
    }

    if (!Number.isInteger(maxCPUPercent) || maxCPUPercent < 1 || maxCPUPercent > 100) {
      return { valid: false, error: 'CPU limit must be between 1% and 100%' };
    }

    if (!Number.isInteger(maxConcurrentTasks) || maxConcurrentTasks < 1 || maxConcurrentTasks > 100) {
      return { valid: false, error: 'Concurrent tasks limit must be between 1 and 100' };
    }

    if (!Number.isInteger(maxSpawnRate) || maxSpawnRate < 1 || maxSpawnRate > 50) {
      return { valid: false, error: 'Spawn rate limit must be between 1 and 50 per hour' };
    }

    if (!Number.isInteger(maxDepth) || maxDepth < 1 || maxDepth > 10) {
      return { valid: false, error: 'Hierarchy depth limit must be between 1 and 10' };
    }

    return { valid: true };
  }

  static validateAgentConfig(config: Partial<AgentConfig>): ValidationResult {
    if (!config.id) {
      return { valid: false, error: 'Agent ID is required' };
    }

    if (!this.validateAgentId(config.id)) {
      return { valid: false, error: 'Invalid agent ID format' };
    }

    if (!this.validateAgentName(config.name || '')) {
      return { valid: false, error: 'Invalid agent name' };
    }

    if (!config.systemPrompt) {
      return { valid: false, error: 'System prompt is required' };
    }

    const promptValidation = this.validateSystemPrompt(config.systemPrompt);
    if (!promptValidation.valid) {
      return promptValidation;
    }

    if (config.resourceLimits) {
      const limitsValidation = this.validateResourceLimits(config.resourceLimits);
      if (!limitsValidation.valid) {
        return limitsValidation;
      }
    }

    return { valid: true };
  }

  static validateTaskTitle(title: string): boolean {
    if (!title || typeof title !== 'string') return false;
    if (title.length < 1 || title.length > 200) return false;
    return true;
  }

  static validateTaskDescription(description: string): boolean {
    if (!description || typeof description !== 'string') return false;
    if (description.length < 1 || description.length > 5000) return false;
    return true;
  }

  static validateMessage(message: AgentMessage): ValidationResult {
    if (!message.from || !this.validateAgentId(message.from)) {
      return { valid: false, error: 'Invalid sender agent ID' };
    }

    if (message.to !== 'broadcast' && !this.validateAgentId(message.to)) {
      return { valid: false, error: 'Invalid recipient agent ID' };
    }

    if (!message.content || typeof message.content !== 'string') {
      return { valid: false, error: 'Message content is required' };
    }

    if (message.content.length > 10000) {
      return { valid: false, error: 'Message content too long (max 10,000 characters)' };
    }

    const validTypes = ['task_request', 'task_response', 'broadcast', 'direct', 'system', 'approval_request', 'capability_query', 'collaboration_invite'];
    if (!validTypes.includes(message.type)) {
      return { valid: false, error: 'Invalid message type' };
    }

    const validPriorities = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'];
    if (!validPriorities.includes(message.priority)) {
      return { valid: false, error: 'Invalid message priority' };
    }

    return { valid: true };
  }

  static sanitizeInput(input: string): string {
    return input
      .trim()
      .replace(/[\x00-\x1f\x7f-\x9f]/g, '') // Remove control characters
      .replace(/\s+/g, ' '); // Normalize whitespace
  }

  static isValidEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }

  static isValidUrl(url: string): boolean {
    try {
      new URL(url);
      return true;
    } catch {
      return false;
    }
  }
}

export interface ValidationResult {
  valid: boolean;
  error?: string;
}