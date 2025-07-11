import { 
  ProtocolRegistry, 
  TaskRequestProtocol, 
  InformationRequestProtocol, 
  CollaborationProtocol, 
  StatusUpdateProtocol,
  protocolRegistry 
} from '../../../src/protocols/agent-protocols';
import { AgentMessage } from '../../../src/types';

describe('Agent Protocols', () => {
  let registry: ProtocolRegistry;

  beforeEach(() => {
    registry = new ProtocolRegistry();
  });

  describe('TaskRequestProtocol', () => {
    let protocol: TaskRequestProtocol;
    let testMessage: AgentMessage;

    beforeEach(() => {
      protocol = new TaskRequestProtocol();
      testMessage = {
        id: 'test-msg-1',
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'Please analyze this data',
        priority: 'HIGH',
        timestamp: new Date(),
        requiresApproval: false,
        metadata: {
          taskId: 'task-123',
          replyTo: 'original-msg'
        }
      };
    });

    it('should handle task request messages', () => {
      expect(protocol.canHandle(testMessage)).toBe(true);
    });

    it('should not handle other message types', () => {
      const otherMessage = { ...testMessage, type: 'direct' as const };
      expect(protocol.canHandle(otherMessage)).toBe(false);
    });

    it('should generate appropriate response', async () => {
      const response = await protocol.handle(testMessage);
      
      expect(response).not.toBeNull();
      expect(response!.type).toBe('task_response');
      expect(response!.from).toBe('agent2');
      expect(response!.to).toBe('agent1');
      expect(response!.priority).toBe('HIGH');
      expect(response!.metadata?.replyTo).toBe('test-msg-1');
      expect(response!.metadata?.taskId).toBe('task-123');
    });
  });

  describe('InformationRequestProtocol', () => {
    let protocol: InformationRequestProtocol;
    let testMessage: AgentMessage;

    beforeEach(() => {
      protocol = new InformationRequestProtocol();
      testMessage = {
        id: 'info-msg-1',
        from: 'agent1',
        to: 'agent2',
        type: 'capability_query',
        content: 'What are your capabilities?',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };
    });

    it('should handle capability query messages', () => {
      expect(protocol.canHandle(testMessage)).toBe(true);
    });

    it('should generate appropriate response', async () => {
      const response = await protocol.handle(testMessage);
      
      expect(response).not.toBeNull();
      expect(response!.type).toBe('direct');
      expect(response!.from).toBe('agent2');
      expect(response!.to).toBe('agent1');
      expect(response!.content).toContain('Information request received');
    });
  });

  describe('CollaborationProtocol', () => {
    let protocol: CollaborationProtocol;
    let testMessage: AgentMessage;

    beforeEach(() => {
      protocol = new CollaborationProtocol();
      testMessage = {
        id: 'collab-msg-1',
        from: 'agent1',
        to: 'agent2',
        type: 'direct',
        content: 'Let us work together on this project',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false,
        metadata: {
          replyTo: 'collab-request'
        }
      };
    });

    it('should handle direct messages for collaboration', () => {
      expect(protocol.canHandle(testMessage)).toBe(true);
    });

    it('should generate collaboration response', async () => {
      const response = await protocol.handle(testMessage);
      
      expect(response).not.toBeNull();
      expect(response!.type).toBe('direct');
      expect(response!.from).toBe('agent2');
      expect(response!.to).toBe('agent1');
      expect(response!.content).toContain('Collaboration response');
    });
  });

  describe('StatusUpdateProtocol', () => {
    let protocol: StatusUpdateProtocol;
    let testMessage: AgentMessage;

    beforeEach(() => {
      protocol = new StatusUpdateProtocol();
      testMessage = {
        id: 'status-msg-1',
        from: 'agent1',
        to: 'agent2',
        type: 'system',
        content: 'Task completed successfully',
        priority: 'LOW',
        timestamp: new Date(),
        requiresApproval: false,
        metadata: {
          taskId: 'task-456'
        }
      };
    });

    it('should handle system status messages', () => {
      expect(protocol.canHandle(testMessage)).toBe(true);
    });

    it('should not generate response for status updates', async () => {
      const response = await protocol.handle(testMessage);
      expect(response).toBeNull();
    });
  });

  describe('ProtocolRegistry', () => {
    it('should register default protocols', () => {
      expect(registry).toBeDefined();
    });

    it('should find appropriate protocol handler', () => {
      const taskMessage: AgentMessage = {
        id: 'test',
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'test',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const handler = registry.findProtocolHandler(taskMessage);
      expect(handler).toBeInstanceOf(TaskRequestProtocol);
    });

    it('should return null for unknown message types', () => {
      const unknownMessage: AgentMessage = {
        id: 'test',
        from: 'agent1',
        to: 'agent2',
        type: 'approval_request',
        content: 'test',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const handler = registry.findProtocolHandler(unknownMessage);
      expect(handler).toBeNull();
    });

    it('should process messages through appropriate protocols', async () => {
      const taskMessage: AgentMessage = {
        id: 'test',
        from: 'agent1',
        to: 'agent2',
        type: 'task_request',
        content: 'test task',
        priority: 'MEDIUM',
        timestamp: new Date(),
        requiresApproval: false
      };

      const response = await registry.processMessage(taskMessage);
      expect(response).not.toBeNull();
      expect(response!.type).toBe('task_response');
    });

    it('should handle processing errors gracefully', async () => {
      // Create a message that will cause an error
      const invalidMessage = {} as AgentMessage;
      
      const response = await registry.processMessage(invalidMessage);
      expect(response).toBeNull();
    });

    it('should create standard messages correctly', () => {
      const message = registry.createMessage(
        'agent1',
        'agent2',
        'task_request',
        'Test message',
        'HIGH',
        { testData: 'value' }
      );

      expect(message.id).toBeDefined();
      expect(message.from).toBe('agent1');
      expect(message.to).toBe('agent2');
      expect(message.type).toBe('task_request');
      expect(message.content).toBe('Test message');
      expect(message.priority).toBe('HIGH');
      expect(message.requiresApproval).toBe(false);
      expect(message.timestamp).toBeInstanceOf(Date);
      expect(message.metadata).toEqual({ testData: 'value' });
    });

    it('should create task request messages', () => {
      const message = registry.createTaskRequest(
        'agent1',
        'agent2',
        'Analyze Data',
        'Please analyze the sales data',
        'HIGH',
        new Date('2024-12-31')
      );

      expect(message.type).toBe('task_request');
      expect(message.content).toBe('Analyze Data');
      expect(message.priority).toBe('HIGH');
      expect(message.metadata).toBeDefined();
    });

    it('should create information request messages', () => {
      const message = registry.createInfoRequest(
        'agent1',
        'agent2',
        'What capabilities do you have?',
        'HIGH'
      );

      expect(message.type).toBe('capability_query');
      expect(message.content).toBe('What capabilities do you have?');
      expect(message.priority).toBe('HIGH');
    });

    it('should create collaboration request messages', () => {
      const message = registry.createCollaborationRequest(
        'agent1',
        'agent2',
        'Joint Analysis',
        'Let us work together on the data analysis'
      );

      expect(message.type).toBe('direct');
      expect(message.content).toBe('Joint Analysis');
      expect(message.metadata).toBeDefined();
    });

    it('should create status update messages', () => {
      const message = registry.createStatusUpdate(
        'agent1',
        'agent2',
        'Task Completed',
        'All analysis is finished',
        'LOW'
      );

      expect(message.type).toBe('system');
      expect(message.content).toBe('Task Completed');
      expect(message.priority).toBe('LOW');
      expect(message.metadata).toBeDefined();
    });
  });

  describe('Global Protocol Registry', () => {
    it('should be available as singleton', () => {
      expect(protocolRegistry).toBeInstanceOf(ProtocolRegistry);
    });

    it('should handle messages consistently', async () => {
      const message = protocolRegistry.createTaskRequest(
        'agent1',
        'agent2',
        'Test Task',
        'Test Description'
      );

      const response = await protocolRegistry.processMessage(message);
      expect(response).not.toBeNull();
      expect(response!.type).toBe('task_response');
    });
  });
});