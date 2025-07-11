import { Command } from 'commander';
import chalk from 'chalk';
import { NoxSystem } from '../core/system';
import { ValidationUtils } from '../utils/validation';
import { DEFAULT_RESOURCE_LIMITS } from '../types/agent';

export class AgentCommands {
  private static async ensureInitialized(nox: NoxSystem): Promise<void> {
    if (!nox.isInitialized) {
      await nox.initialize();
    }
  }

  static register(program: Command, nox: NoxSystem): void {
    
    // Add agent command
    program
      .command('add-agent <name> <systemPrompt>')
      .description('Create a new agent with specified configuration')
      .option('--memory <mb>', 'Memory limit in MB', '2048')
      .option('--cpu <percent>', 'CPU limit percentage', '50')
      .option('--tasks <count>', 'Max concurrent tasks', '10')
      .action(async (name: string, systemPrompt: string, options) => {
        try {
          await AgentCommands.ensureInitialized(nox);

          // Validate inputs
          if (!ValidationUtils.validateAgentId(name)) {
            console.error(chalk.red('❌ Invalid agent name. Use only letters, numbers, underscore, and dash.'));
            process.exit(1);
          }

          const promptValidation = ValidationUtils.validateSystemPrompt(systemPrompt);
          if (!promptValidation.valid) {
            console.error(chalk.red(`❌ ${promptValidation.error}`));
            process.exit(1);
          }

          // Create agent configuration
          const resourceLimits = {
            ...DEFAULT_RESOURCE_LIMITS,
            maxMemoryMB: parseInt(options.memory),
            maxCPUPercent: parseInt(options.cpu),
            maxConcurrentTasks: parseInt(options.tasks)
          };

          console.log(chalk.blue(`🤖 Creating agent: ${name}`));
          
          // Backup registry before modification
          await nox.git.commit(`Before creating agent: ${name}`);

          const agent = await nox.registry.createAgent({
            id: name,
            name: name,
            systemPrompt: systemPrompt,
            resourceLimits
          });

          // Commit the new agent
          await nox.git.commit(`Created agent: ${name}`);

          console.log(chalk.green('✅ Agent created successfully!'));
          console.log(chalk.gray(`   ID: ${agent.id}`));
          console.log(chalk.gray(`   Status: ${agent.status}`));
          console.log(chalk.gray(`   Memory Limit: ${agent.resourceLimits.maxMemoryMB}MB`));
          console.log(chalk.gray(`   CPU Limit: ${agent.resourceLimits.maxCPUPercent}%`));

          // If system is running, spawn the agent
          if (nox.isRunning) {
            console.log(chalk.blue('🚀 Starting agent...'));
            await nox.agents.spawnAgent(agent);
            await nox.registry.updateAgentStatus(agent.id, 'active');
            console.log(chalk.green('✅ Agent is now running!'));
          } else {
            console.log(chalk.yellow('💡 Run "nox start" to activate this agent'));
          }

        } catch (error) {
          console.error(chalk.red('❌ Failed to create agent:'), error);
          process.exit(1);
        }
      });

    // List agents command
    program
      .command('list-agents')
      .alias('ls')
      .description('Display all registered agents and their status')
      .option('--status <status>', 'Filter by status (active, inactive, error, crashed)')
      .option('--format <format>', 'Output format (table, json)', 'table')
      .action(async (options) => {
        try {
          await AgentCommands.ensureInitialized(nox);

          const agents = await nox.registry.listAgents();
          let filteredAgents = agents;

          if (options.status) {
            filteredAgents = agents.filter(agent => agent.status === options.status);
          }

          if (options.format === 'json') {
            console.log(JSON.stringify(filteredAgents, null, 2));
            return;
          }

          // Table format
          if (filteredAgents.length === 0) {
            console.log(chalk.yellow('📭 No agents found'));
            if (options.status) {
              console.log(chalk.gray(`   Filtered by status: ${options.status}`));
            }
            return;
          }

          console.log(chalk.blue(`🤖 Found ${filteredAgents.length} agent(s):\n`));

          filteredAgents.forEach((agent, index) => {
            const statusColor = agent.status === 'active' ? chalk.green : 
                               agent.status === 'error' || agent.status === 'crashed' ? chalk.red : 
                               chalk.yellow;

            console.log(`${index + 1}. ${chalk.bold(agent.name)} (${agent.id})`);
            console.log(`   Status: ${statusColor(agent.status)}`);
            console.log(`   Created: ${chalk.gray(agent.createdAt.toISOString().split('T')[0])}`);
            console.log(`   Memory: ${chalk.gray(agent.resourceLimits.maxMemoryMB + 'MB')}`);
            console.log(`   CPU: ${chalk.gray(agent.resourceLimits.maxCPUPercent + '%')}`);
            
            if (agent.capabilities.length > 0) {
              console.log(`   Capabilities: ${chalk.cyan(agent.capabilities.join(', '))}`);
            }
            
            if (agent.mcpServices.length > 0) {
              console.log(`   MCP Services: ${chalk.magenta(agent.mcpServices.join(', '))}`);
            }
            
            console.log(); // Empty line
          });

          // Summary
          const stats = await nox.registry.getStats();
          console.log(chalk.blue('📊 Registry Summary:'));
          console.log(`   Total: ${stats.totalAgents}`);
          console.log(`   Active: ${chalk.green(stats.activeAgents)}`);
          console.log(`   Inactive: ${chalk.yellow(stats.inactiveAgents)}`);
          console.log(`   Errors: ${chalk.red(stats.errorAgents)}`);

        } catch (error) {
          console.error(chalk.red('❌ Failed to list agents:'), error);
          process.exit(1);
        }
      });

    // Update agent command
    program
      .command('update-agent <name> <newPrompt>')
      .description('Modify an existing agent\'s system prompt')
      .action(async (name: string, newPrompt: string) => {
        try {
          await AgentCommands.ensureInitialized(nox);

          const agent = await nox.registry.getAgent(name);
          if (!agent) {
            console.error(chalk.red(`❌ Agent '${name}' not found`));
            process.exit(1);
          }

          const promptValidation = ValidationUtils.validateSystemPrompt(newPrompt);
          if (!promptValidation.valid) {
            console.error(chalk.red(`❌ ${promptValidation.error}`));
            process.exit(1);
          }

          console.log(chalk.blue(`🔄 Updating agent: ${name}`));
          
          // Backup before update
          await nox.git.commit(`Before updating agent: ${name}`);

          await nox.registry.updateAgent(name, {
            systemPrompt: newPrompt
          });

          // Commit the update
          await nox.git.commit(`Updated agent: ${name}`);

          console.log(chalk.green('✅ Agent updated successfully!'));
          console.log(chalk.gray(`   New prompt length: ${newPrompt.length} characters`));
          console.log(chalk.yellow('💡 Restart the agent to apply changes'));

        } catch (error) {
          console.error(chalk.red('❌ Failed to update agent:'), error);
          process.exit(1);
        }
      });

    // Delete agent command
    program
      .command('delete-agent <name>')
      .description('Remove an agent from the registry')
      .option('--force', 'Skip confirmation prompt')
      .action(async (name: string, options) => {
        try {
          await AgentCommands.ensureInitialized(nox);

          const agent = await nox.registry.getAgent(name);
          if (!agent) {
            console.error(chalk.red(`❌ Agent '${name}' not found`));
            process.exit(1);
          }

          // Confirmation (simplified for now)
          if (!options.force) {
            console.log(chalk.yellow(`⚠️  This will permanently delete agent '${name}'`));
            console.log(chalk.yellow('   Use --force flag to skip this confirmation'));
            return;
          }

          console.log(chalk.blue(`🗑️  Deleting agent: ${name}`));
          
          // Stop agent if running
          if (agent.status === 'active') {
            await nox.agents.killAgent(name);
          }

          // Backup before deletion
          await nox.git.commit(`Before deleting agent: ${name}`);

          await nox.registry.deleteAgent(name);

          // Commit the deletion
          await nox.git.commit(`Deleted agent: ${name}`);

          console.log(chalk.green('✅ Agent deleted successfully!'));

        } catch (error) {
          console.error(chalk.red('❌ Failed to delete agent:'), error);
          process.exit(1);
        }
      });

    // Show agent details command
    program
      .command('show-agent <name>')
      .description('Display detailed information about an agent')
      .action(async (name: string) => {
        try {
          await AgentCommands.ensureInitialized(nox);

          const agent = await nox.registry.getAgent(name);
          if (!agent) {
            console.error(chalk.red(`❌ Agent '${name}' not found`));
            process.exit(1);
          }

          const agentProcess = await nox.agents.getAgentStatus(name);
          const relationships = await nox.registry.getRelationships(name);

          console.log(chalk.blue(`🤖 Agent Details: ${agent.name}\n`));
          
          console.log(chalk.bold('Basic Information:'));
          console.log(`   ID: ${agent.id}`);
          console.log(`   Name: ${agent.name}`);
          console.log(`   Status: ${agent.status === 'active' ? chalk.green(agent.status) : chalk.yellow(agent.status)}`);
          console.log(`   Created: ${agent.createdAt.toISOString()}`);
          console.log(`   Modified: ${agent.lastModified.toISOString()}`);
          
          console.log(chalk.bold('\nResource Limits:'));
          console.log(`   Memory: ${agent.resourceLimits.maxMemoryMB}MB`);
          console.log(`   CPU: ${agent.resourceLimits.maxCPUPercent}%`);
          console.log(`   Concurrent Tasks: ${agent.resourceLimits.maxConcurrentTasks}`);
          console.log(`   Spawn Rate: ${agent.resourceLimits.maxSpawnRate}/hour`);
          console.log(`   Max Depth: ${agent.resourceLimits.maxDepth}`);

          if (agentProcess) {
            console.log(chalk.bold('\nProcess Information:'));
            console.log(`   PID: ${agentProcess.pid || 'N/A'}`);
            console.log(`   Start Time: ${agentProcess.startTime.toISOString()}`);
            console.log(`   Restart Count: ${agentProcess.restartCount}`);
            console.log(`   Memory Usage: ${agentProcess.memoryUsage}MB`);
            console.log(`   CPU Usage: ${agentProcess.cpuUsage}%`);
          }

          if (agent.capabilities.length > 0) {
            console.log(chalk.bold('\nCapabilities:'));
            agent.capabilities.forEach(cap => console.log(`   • ${cap}`));
          }

          if (agent.mcpServices.length > 0) {
            console.log(chalk.bold('\nMCP Services:'));
            agent.mcpServices.forEach(service => console.log(`   • ${service}`));
          }

          if (relationships.length > 0) {
            console.log(chalk.bold('\nRelationships:'));
            relationships.forEach(rel => {
              console.log(`   • ${rel.relationshipType} with ${rel.targetAgentId}`);
            });
          }

          console.log(chalk.bold('\nSystem Prompt:'));
          console.log(chalk.gray(`"${agent.systemPrompt}"`));

        } catch (error) {
          console.error(chalk.red('❌ Failed to show agent details:'), error);
          process.exit(1);
        }
      });
  }
}