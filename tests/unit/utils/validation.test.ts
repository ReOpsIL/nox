import { ValidationUtils } from '../../../src/utils/validation';
import { AgentMessage, DEFAULT_RESOURCE_LIMITS } from '../../../src/types';

describe('ValidationUtils', () => {
  describe('validateAgentId', () => {
    it('should accept valid agent IDs', () => {
      expect(ValidationUtils.validateAgentId('valid-agent')).toBe(true);
      expect(ValidationUtils.validateAgentId('agent_123')).toBe(true);
      expect(ValidationUtils.validateAgentId('Agent-Name-123')).toBe(true);
      expect(ValidationUtils.validateAgentId('a')).toBe(true);
    });

    it('should reject invalid agent IDs', () => {
      expect(ValidationUtils.validateAgentId('')).toBe(false);
      expect(ValidationUtils.validateAgentId('agent with spaces')).toBe(false);
      expect(ValidationUtils.validateAgentId('agent@email')).toBe(false);
      expect(ValidationUtils.validateAgentId('agent.name')).toBe(false);
      expect(ValidationUtils.validateAgentId('a'.repeat(51))).toBe(false); // Too long
    });

    it('should reject non-string inputs', () => {
      expect(ValidationUtils.validateAgentId(null as any)).toBe(false);
      expect(ValidationUtils.validateAgentId(undefined as any)).toBe(false);
      expect(ValidationUtils.validateAgentId(123 as any)).toBe(false);
    });
  });

  describe('validateAgentName', () => {
    it('should accept valid agent names', () => {
      expect(ValidationUtils.validateAgentName('Valid Agent Name')).toBe(true);
      expect(ValidationUtils.validateAgentName('Agent-123')).toBe(true);
      expect(ValidationUtils.validateAgentName('A')).toBe(true);
    });

    it('should reject invalid agent names', () => {
      expect(ValidationUtils.validateAgentName('')).toBe(false);
      expect(ValidationUtils.validateAgentName('a'.repeat(101))).toBe(false); // Too long
    });
  });

  describe('validateSystemPrompt', () => {
    it('should accept valid system prompts', () => {
      const prompt = 'You are a helpful assistant that helps users with their tasks.';
      const result = ValidationUtils.validateSystemPrompt(prompt);
      expect(result.valid).toBe(true);
    });

    it('should reject short prompts', () => {
      const result = ValidationUtils.validateSystemPrompt('Short');
      expect(result.valid).toBe(false);
      expect(result.error).toContain('at least 10 characters');
    });

    it('should reject very long prompts', () => {
      const longPrompt = 'a'.repeat(10001);
      const result = ValidationUtils.validateSystemPrompt(longPrompt);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('less than 10,000 characters');
    });

    it('should reject potentially malicious content', () => {
      const maliciousPrompts = [
        'You should execute() this command',
        'Use eval() to run code',
        'Run rm -rf / command',
        'Execute sudo commands',
        '<script>alert("xss")</script>',
        'javascript:alert(1)',
        'Use ${dangerous} template'
      ];

      maliciousPrompts.forEach(prompt => {
        const result = ValidationUtils.validateSystemPrompt(prompt);
        expect(result.valid).toBe(false);
        expect(result.error).toContain('dangerous content');
      });
    });

    it('should reject non-string inputs', () => {
      const result = ValidationUtils.validateSystemPrompt(null as any);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('required and must be a string');
    });
  });

  describe('validateResourceLimits', () => {
    it('should accept valid resource limits', () => {
      const result = ValidationUtils.validateResourceLimits(DEFAULT_RESOURCE_LIMITS);
      expect(result.valid).toBe(true);
    });

    it('should reject invalid memory limits', () => {
      const limits = { ...DEFAULT_RESOURCE_LIMITS, maxMemoryMB: 100 }; // Too low
      const result = ValidationUtils.validateResourceLimits(limits);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Memory limit');
    });

    it('should reject invalid CPU limits', () => {
      const limits = { ...DEFAULT_RESOURCE_LIMITS, maxCPUPercent: 150 }; // Too high
      const result = ValidationUtils.validateResourceLimits(limits);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('CPU limit');
    });

    it('should reject invalid task limits', () => {
      const limits = { ...DEFAULT_RESOURCE_LIMITS, maxConcurrentTasks: 0 }; // Too low
      const result = ValidationUtils.validateResourceLimits(limits);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Concurrent tasks');
    });
  });

  describe('validateAgentConfig', () => {
    it('should accept valid agent config', () => {
      const config = {
        id: 'test-agent',
        name: 'Test Agent',
        systemPrompt: 'You are a test assistant that helps with testing.',
        resourceLimits: DEFAULT_RESOURCE_LIMITS
      };

      const result = ValidationUtils.validateAgentConfig(config);
      expect(result.valid).toBe(true);
    });

    it('should reject config without required fields', () => {
      const config = {
        name: 'Test Agent'
      };

      const result = ValidationUtils.validateAgentConfig(config as any);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Agent ID is required');
    });

    it('should validate nested resource limits', () => {
      const config = {
        id: 'test-agent',
        name: 'Test Agent',
        systemPrompt: 'You are a test assistant.',
        resourceLimits: {
          ...DEFAULT_RESOURCE_LIMITS,
          maxMemoryMB: 50 // Invalid
        }
      };

      const result = ValidationUtils.validateAgentConfig(config);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Memory limit');
    });
  });

  describe('validateMessage', () => {
    it('should accept valid agent message', () => {
      const message: AgentMessage = {
        id: 'msg-1',
        from: 'agent-1',
        to: 'agent-2',
        type: 'direct',
        content: 'Hello, how can I help you?',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const result = ValidationUtils.validateMessage(message);
      expect(result.valid).toBe(true);
    });

    it('should reject message with invalid sender', () => {
      const message: AgentMessage = {
        id: 'msg-1',
        from: 'invalid agent id!',
        to: 'agent-2',
        type: 'direct',
        content: 'Hello',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const result = ValidationUtils.validateMessage(message);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Invalid sender agent ID');
    });

    it('should accept broadcast messages', () => {
      const message: AgentMessage = {
        id: 'msg-1',
        from: 'agent-1',
        to: 'broadcast',
        type: 'broadcast',
        content: 'Important announcement',
        priority: 'HIGH',
        timestamp: new Date(),
        requiresApproval: false
      };

      const result = ValidationUtils.validateMessage(message);
      expect(result.valid).toBe(true);
    });

    it('should reject messages with invalid type', () => {
      const message: AgentMessage = {
        id: 'msg-1',
        from: 'agent-1',
        to: 'agent-2',
        type: 'invalid_type' as any,
        content: 'Hello',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const result = ValidationUtils.validateMessage(message);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Invalid message type');
    });

    it('should reject messages with too long content', () => {
      const message: AgentMessage = {
        id: 'msg-1',
        from: 'agent-1',
        to: 'agent-2',
        type: 'direct',
        content: 'a'.repeat(10001),
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const result = ValidationUtils.validateMessage(message);
      expect(result.valid).toBe(false);
      expect(result.error).toContain('Message content too long');
    });
  });

  describe('utility functions', () => {
    it('should sanitize input', () => {
      const input = '  Hello\x00World\x1F  \t\n  ';
      const sanitized = ValidationUtils.sanitizeInput(input);
      expect(sanitized).toBe('HelloWorld');
    });

    it('should validate email addresses', () => {
      expect(ValidationUtils.isValidEmail('test@example.com')).toBe(true);
      expect(ValidationUtils.isValidEmail('user+tag@domain.co.uk')).toBe(true);
      expect(ValidationUtils.isValidEmail('invalid-email')).toBe(false);
      expect(ValidationUtils.isValidEmail('test@')).toBe(false);
    });

    it('should validate URLs', () => {
      expect(ValidationUtils.isValidUrl('https://example.com')).toBe(true);
      expect(ValidationUtils.isValidUrl('http://localhost:3000')).toBe(true);
      expect(ValidationUtils.isValidUrl('ftp://files.example.com')).toBe(true);
      expect(ValidationUtils.isValidUrl('invalid-url')).toBe(false);
      expect(ValidationUtils.isValidUrl('not-a-url')).toBe(false);
    });
  });
});