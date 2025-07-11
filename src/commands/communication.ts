import { Command } from 'commander';
import { AgentManager } from '../core/agent-manager';
import { MessageBroker } from '../core/message-broker';
import { TaskManager } from '../core/task-manager';
import { WebSocketServer } from '../server/websocket';
import { protocolRegistry } from '../protocols/agent-protocols';
import { logger } from '../utils/logger';
import { ConfigManager } from '../core/config-manager';

/**
 * Communication Commands - CLI commands for testing inter-agent communication
 */

export function createCommunicationCommand(): Command {
  const cmd = new Command('comm')
    .description('Inter-agent communication commands');

  // Send message command
  cmd.command('send-message')
    .description('Send a message from one agent to another')
    .argument('<from>', 'Sender agent ID')
    .argument('<to>', 'Receiver agent ID')
    .argument('<type>', 'Message type (task_request, capability_query, direct, system)')
    .argument('<content>', 'Message content')
    .option('-p, --priority <priority>', 'Message priority (LOW, MEDIUM, HIGH, CRITICAL)', 'MEDIUM')
    .action(async (from: string, to: string, type: string, content: string, options) => {
      try {
        const configManager = new ConfigManager();
        const config = await configManager.loadConfig();
        
        const tempDir = config.storage.registryPath;
        const messageBroker = new MessageBroker(tempDir);
        await messageBroker.initialize(config);
        
        const message = protocolRegistry.createMessage(
          from,
          to,
          type as any,
          content,
          options.priority
        );
        
        await messageBroker.sendMessage(message);
        
        console.log(`✅ Message sent from ${from} to ${to}`);
        console.log(`📨 Type: ${type}, Priority: ${options.priority}`);
        console.log(`💬 Content: ${content}`);
        
        await messageBroker.shutdown();
      } catch (error) {
        logger.error('Failed to send message:', error);
        process.exit(1);
      }
    });

  // Delegate task command
  cmd.command('delegate-task')
    .description('Delegate a task from one agent to another')
    .argument('<from>', 'Source agent ID')
    .argument('<to>', 'Target agent ID')
    .argument('<title>', 'Task title')
    .argument('<description>', 'Task description')
    .option('-p, --priority <priority>', 'Task priority', 'MEDIUM')
    .action(async (from: string, to: string, title: string, description: string, options) => {
      try {
        const configManager = new ConfigManager();
        const config = await configManager.loadConfig();
        
        const tempDir = config.storage.registryPath;
        const agentManager = new AgentManager(tempDir);
        const taskManager = new TaskManager(tempDir);
        
        await agentManager.initialize(config);
        await taskManager.initialize(config);
        
        // Create the task
        const task = await taskManager.delegateTask(
          from,
          to,
          {
            title,
            description,
            status: 'todo',
            priority: options.priority,
            requestedBy: from,
            dependencies: [],
            progress: 0
          }
        );
        
        console.log(`✅ Task delegated from ${from} to ${to}`);
        console.log(`📋 Task ID: ${task.id}`);
        console.log(`📝 Title: ${title}`);
        console.log(`📄 Description: ${description}`);
        console.log(`⚡ Priority: ${options.priority}`);
        
        await agentManager.shutdown();
        await taskManager.shutdown();
      } catch (error) {
        logger.error('Failed to delegate task:', error);
        process.exit(1);
      }
    });

  // Start communication server
  cmd.command('start-server')
    .description('Start the WebSocket communication server')
    .option('-p, --port <port>', 'Server port', '3000')
    .action(async (options) => {
      try {
        const configManager = new ConfigManager();
        const config = await configManager.loadConfig();
        config.server.port = parseInt(options.port);
        
        const tempDir = config.storage.registryPath;
        
        // Initialize all components
        const agentManager = new AgentManager(tempDir);
        const messageBroker = new MessageBroker(tempDir);
        const taskManager = new TaskManager(tempDir);
        const websocketServer = new WebSocketServer(agentManager, messageBroker, taskManager);
        
        // Set up integrations
        messageBroker.setAgentManager(agentManager);
        
        // Initialize everything
        await agentManager.initialize(config);
        await messageBroker.initialize(config);
        await taskManager.initialize(config);
        await websocketServer.initialize(config);
        
        // Start services
        await messageBroker.start();
        await taskManager.start();
        await websocketServer.start();
        
        console.log(`🚀 Communication server started on port ${options.port}`);
        console.log(`📡 WebSocket server: ws://localhost:${options.port}`);
        console.log(`📊 Dashboard: http://localhost:${options.port}/dashboard`);
        console.log('Press Ctrl+C to stop...');
        
        // Handle graceful shutdown
        process.on('SIGINT', async () => {
          console.log('\n🛑 Shutting down communication server...');
          
          await websocketServer.shutdown();
          await taskManager.shutdown();
          await messageBroker.shutdown();
          await agentManager.shutdown();
          
          console.log('✅ Communication server stopped');
          process.exit(0);
        });
        
        // Keep the process alive
        await new Promise(() => {});
        
      } catch (error) {
        logger.error('Failed to start communication server:', error);
        process.exit(1);
      }
    });

  // Test communication flow
  cmd.command('test-flow')
    .description('Test the complete inter-agent communication flow')
    .option('-c, --count <count>', 'Number of test messages', '5')
    .action(async (options) => {
      try {
        const configManager = new ConfigManager();
        const config = await configManager.loadConfig();
        
        const tempDir = config.storage.registryPath;
        const messageBroker = new MessageBroker(tempDir);
        const taskManager = new TaskManager(tempDir);
        
        await messageBroker.initialize(config);
        await taskManager.initialize(config);
        await messageBroker.start();
        await taskManager.start();
        
        const count = parseInt(options.count);
        console.log(`🧪 Testing communication flow with ${count} messages...`);
        
        // Send test messages
        for (let i = 1; i <= count; i++) {
          const message = protocolRegistry.createTaskRequest(
            'test-agent-sender',
            'test-agent-receiver',
            `Test Task ${i}`,
            `This is test task number ${i} for communication flow testing`,
            i % 2 === 0 ? 'HIGH' : 'MEDIUM'
          );
          
          await messageBroker.sendMessage(message);
          console.log(`📨 Sent message ${i}/${count}: Task Request`);
          
          // Create corresponding task
          await taskManager.createTask({
            agentId: 'test-agent-receiver',
            title: `Test Task ${i}`,
            description: `Task created from message ${i}`,
            status: 'todo',
            priority: message.priority,
            requestedBy: 'test-agent-sender',
            dependencies: [],
            progress: 0
          });
          
          console.log(`📋 Created task ${i}/${count}`);
          
          // Small delay between messages
          await new Promise(resolve => setTimeout(resolve, 100));
        }
        
        // Show statistics
        const brokerStats = messageBroker.getStats();
        const taskDashboard = await taskManager.getTaskDashboard();
        
        console.log('\n📊 Communication Flow Test Results:');
        console.log(`📨 Message Queue Size: ${brokerStats.queueSize}`);
        console.log(`📬 Total History Messages: ${brokerStats.totalHistoryMessages}`);
        console.log(`📋 Total Tasks Created: ${taskDashboard.total}`);
        console.log(`📝 Tasks by Status:`, taskDashboard.byStatus);
        console.log(`⚡ Tasks by Priority:`, taskDashboard.byPriority);
        
        console.log('\n✅ Communication flow test completed successfully!');
        
        await messageBroker.shutdown();
        await taskManager.shutdown();
        
      } catch (error) {
        logger.error('Communication flow test failed:', error);
        process.exit(1);
      }
    });

  return cmd;
}