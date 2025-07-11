import { Command } from 'commander';
import chalk from 'chalk';
import { NoxSystem } from '../core/system';
import { TaskPriority } from '../types';

export class TaskCommands {
  private static async ensureInitialized(nox: NoxSystem): Promise<void> {
    if (!nox.isInitialized) {
      await nox.initialize();
    }
  }

  static register(program: Command, nox: NoxSystem): void {
    
    // Create task command
    program
      .command('create-task <agentId> <title> <description>')
      .description('Create a new task for an agent')
      .option('--priority <level>', 'Task priority (LOW, MEDIUM, HIGH, CRITICAL)', 'MEDIUM')
      .option('--deadline <date>', 'Task deadline (ISO 8601 format)')
      .action(async (agentId: string, title: string, description: string, options) => {
        try {
          await TaskCommands.ensureInitialized(nox);

          // Validate agent exists
          const agent = await nox.registry.getAgent(agentId);
          if (!agent) {
            console.error(chalk.red(`❌ Agent '${agentId}' not found`));
            process.exit(1);
          }

          // Validate priority
          const validPriorities = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'];
          if (!validPriorities.includes(options.priority)) {
            console.error(chalk.red(`❌ Invalid priority. Use: ${validPriorities.join(', ')}`));
            process.exit(1);
          }

          let deadline: Date | undefined;
          if (options.deadline) {
            deadline = new Date(options.deadline);
            if (isNaN(deadline.getTime())) {
              console.error(chalk.red('❌ Invalid deadline format. Use ISO 8601 format (e.g., 2024-01-15T18:00:00Z)'));
              process.exit(1);
            }
          }

          console.log(chalk.blue(`📝 Creating task for agent: ${agentId}`));

          const taskData: any = {
            agentId,
            title,
            description,
            status: 'todo',
            priority: options.priority as TaskPriority,
            dependencies: [],
            requestedBy: 'user',
            progress: 0
          };

          if (deadline) {
            taskData.deadline = deadline;
          }

          const task = await nox.tasks.createTask(taskData);

          console.log(chalk.green('✅ Task created successfully!'));
          console.log(chalk.gray(`   ID: ${task.id}`));
          console.log(chalk.gray(`   Title: ${task.title}`));
          console.log(chalk.gray(`   Priority: ${task.priority}`));
          if (task.deadline) {
            console.log(chalk.gray(`   Deadline: ${task.deadline.toISOString()}`));
          }

        } catch (error) {
          console.error(chalk.red('❌ Failed to create task:'), error);
          process.exit(1);
        }
      });

    // List tasks command
    program
      .command('list-tasks [agentId]')
      .description('List tasks for a specific agent or all agents')
      .option('--status <status>', 'Filter by status (todo, inprogress, done, blocked)')
      .option('--priority <priority>', 'Filter by priority (LOW, MEDIUM, HIGH, CRITICAL)')
      .option('--format <format>', 'Output format (table, json)', 'table')
      .action(async (agentId: string | undefined, options) => {
        try {
          await TaskCommands.ensureInitialized(nox);

          let tasks: any[] = [];
          if (agentId) {
            // Validate agent exists
            const agent = await nox.registry.getAgent(agentId);
            if (!agent) {
              console.error(chalk.red(`❌ Agent '${agentId}' not found`));
              process.exit(1);
            }
            tasks = await nox.tasks.getAgentTasks(agentId);
          } else {
            // Get all tasks from all agents
            const allAgents = await nox.registry.listAgents();
            tasks = [];
            for (const agent of allAgents) {
              const agentTasks = await nox.tasks.getAgentTasks(agent.id);
              tasks.push(...agentTasks);
            }
          }

          // Apply filters
          let filteredTasks = tasks;
          if (options.status) {
            filteredTasks = tasks.filter(task => task.status === options.status);
          }
          if (options.priority) {
            filteredTasks = tasks.filter(task => task.priority === options.priority);
          }

          if (options.format === 'json') {
            console.log(JSON.stringify(filteredTasks, null, 2));
            return;
          }

          // Table format
          if (filteredTasks.length === 0) {
            console.log(chalk.yellow('📭 No tasks found'));
            if (agentId) {
              console.log(chalk.gray(`   Agent: ${agentId}`));
            }
            return;
          }

          const agentName = agentId ? agentId : 'All Agents';
          const taskWord = filteredTasks.length === 1 ? 'task' : 'tasks';
          console.log(chalk.blue(`📋 Tasks for ${agentName} (${filteredTasks.length} ${taskWord} found)\n`));

          filteredTasks.forEach((task, index) => {
            const statusColor = task.status === 'done' ? chalk.green : 
                               task.status === 'inprogress' ? chalk.blue :
                               task.status === 'blocked' ? chalk.red : 
                               chalk.yellow;

            const priorityColor = task.priority === 'CRITICAL' ? chalk.red.bold :
                                 task.priority === 'HIGH' ? chalk.red :
                                 task.priority === 'MEDIUM' ? chalk.yellow :
                                 chalk.gray;

            console.log(`${index + 1}. ${chalk.bold(task.title)}`);
            console.log(`   Status: ${statusColor(task.status)} | Priority: ${priorityColor(task.priority)}`);
            console.log(`   Agent: ${chalk.cyan(task.agentId)}`);
            console.log(`   Created: ${chalk.gray(task.createdAt.toISOString().split('T')[0])}`);
            
            if (task.deadline) {
              const isOverdue = task.deadline < new Date() && task.status !== 'done';
              const deadlineColor = isOverdue ? chalk.red : chalk.gray;
              console.log(`   Deadline: ${deadlineColor(task.deadline.toISOString().split('T')[0])}`);
            }
            
            if (task.progress > 0) {
              console.log(`   Progress: ${chalk.cyan(task.progress + '%')}`);
            }
            
            console.log(`   Description: ${chalk.gray(task.description.slice(0, 80) + (task.description.length > 80 ? '...' : ''))}`);
            console.log();
          });

        } catch (error) {
          console.error(chalk.red('❌ Failed to list tasks:'), error);
          process.exit(1);
        }
      });

    // Update task command
    program
      .command('update-task <taskId>')
      .description('Update task status or details')
      .option('--status <status>', 'New status (todo, inprogress, done, blocked)')
      .option('--priority <priority>', 'New priority (LOW, MEDIUM, HIGH, CRITICAL)')
      .option('--progress <percent>', 'Progress percentage (0-100)')
      .action(async (taskId: string, options) => {
        try {
          await TaskCommands.ensureInitialized(nox);

          const task = await nox.tasks.getTask(taskId);
          if (!task) {
            console.error(chalk.red(`❌ Task '${taskId}' not found`));
            process.exit(1);
          }

          const updates: any = {};

          if (options.status) {
            const validStatuses = ['todo', 'inprogress', 'done', 'blocked'];
            if (!validStatuses.includes(options.status)) {
              console.error(chalk.red(`❌ Invalid status. Use: ${validStatuses.join(', ')}`));
              process.exit(1);
            }
            updates.status = options.status;
          }

          if (options.priority) {
            const validPriorities = ['LOW', 'MEDIUM', 'HIGH', 'CRITICAL'];
            if (!validPriorities.includes(options.priority)) {
              console.error(chalk.red(`❌ Invalid priority. Use: ${validPriorities.join(', ')}`));
              process.exit(1);
            }
            updates.priority = options.priority;
          }

          if (options.progress) {
            const progress = parseInt(options.progress);
            if (isNaN(progress) || progress < 0 || progress > 100) {
              console.error(chalk.red('❌ Progress must be a number between 0 and 100'));
              process.exit(1);
            }
            updates.progress = progress;
          }

          if (Object.keys(updates).length === 0) {
            console.error(chalk.red('❌ No updates specified. Use --status, --priority, or --progress'));
            process.exit(1);
          }

          console.log(chalk.blue(`🔄 Updating task: ${taskId}`));

          const updatedTask = await nox.tasks.updateTask(taskId, updates);

          console.log(chalk.green('✅ Task updated successfully!'));
          console.log(chalk.gray(`   Title: ${updatedTask.title}`));
          console.log(chalk.gray(`   Status: ${updatedTask.status}`));
          console.log(chalk.gray(`   Priority: ${updatedTask.priority}`));
          console.log(chalk.gray(`   Progress: ${updatedTask.progress}%`));

        } catch (error) {
          console.error(chalk.red('❌ Failed to update task:'), error);
          process.exit(1);
        }
      });

    // Task overview command
    program
      .command('task-overview')
      .description('Show task overview across all agents')
      .action(async () => {
        try {
          await TaskCommands.ensureInitialized(nox);

          const totalTasks = await nox.tasks.getTotalTaskCount();
          const agents = await nox.registry.listAgents();

          console.log(chalk.blue('📊 Task Overview\n'));
          
          console.log(chalk.bold('System Summary:'));
          console.log(`   Total Tasks: ${chalk.cyan(totalTasks)}`);
          console.log(`   Active Agents: ${chalk.green(agents.filter(a => a.status === 'active').length)}`);
          console.log(`   Total Agents: ${chalk.cyan(agents.length)}`);

          // Per-agent breakdown (simplified)
          if (agents.length > 0) {
            console.log(chalk.bold('\nPer-Agent Breakdown:'));
            for (const agent of agents.slice(0, 10)) { // Show first 10 agents
              const agentTasks = await nox.tasks.getAgentTasks(agent.id);
              const todoCount = agentTasks.filter(t => t.status === 'todo').length;
              const inProgressCount = agentTasks.filter(t => t.status === 'inprogress').length;
              const doneCount = agentTasks.filter(t => t.status === 'done').length;
              
              console.log(`   ${agent.name} (${agent.id}):`);
              console.log(`     Todo: ${chalk.yellow(todoCount)} | In Progress: ${chalk.blue(inProgressCount)} | Done: ${chalk.green(doneCount)}`);
            }

            if (agents.length > 10) {
              console.log(chalk.gray(`   ... and ${agents.length - 10} more agents`));
            }
          }

        } catch (error) {
          console.error(chalk.red('❌ Failed to get task overview:'), error);
          process.exit(1);
        }
      });
  }
}