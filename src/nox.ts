#!/usr/bin/env node

import { Command } from 'commander';
import chalk from 'chalk';
import { NoxSystem } from './core/system';
import { AgentCommands } from './commands/agent';
import { SystemCommands } from './commands/system';
import { RegistryCommands } from './commands/registry';
import { TaskCommands } from './commands/task';
import { logger } from './utils/logger';

const program = new Command();

async function main(): Promise<void> {
  try {
    // Initialize program
    program
      .name('nox')
      .description('Autonomous AI Agent Ecosystem')
      .version('1.0.0')
      .option('-v, --verbose', 'Enable verbose logging')
      .option('--dev', 'Enable development mode')
      .option('--config <path>', 'Path to configuration file');

    // Global error handler
    program.exitOverride();

    // Initialize Nox system
    const noxSystem = new NoxSystem();

    // Register command groups
    AgentCommands.register(program, noxSystem);
    SystemCommands.register(program, noxSystem);
    RegistryCommands.register(program, noxSystem);
    TaskCommands.register(program, noxSystem);

    // Initialize command - sets up .nox-registry and bootstrap
    program
      .command('init')
      .description('Initialize Nox agent ecosystem')
      .option('--force', 'Force reinitialize if already exists')
      .action(async (options) => {
        try {
          console.log(chalk.blue('🚀 Initializing Nox Agent Ecosystem...'));
          await noxSystem.initialize(options.force);
          console.log(chalk.green('✅ Nox ecosystem initialized successfully!'));
          console.log(chalk.yellow('💡 Run "nox start" to begin the agent ecosystem'));
        } catch (error) {
          console.error(chalk.red('❌ Initialization failed:'), error);
          process.exit(1);
        }
      });

    // Start command - launches the ecosystem
    program
      .command('start')
      .description('Start the Nox agent ecosystem')
      .option('--background', 'Run in background mode')
      .action(async (options) => {
        try {
          console.log(chalk.blue('🌟 Starting Nox Agent Ecosystem...'));
          await noxSystem.start(options.background);
          
          if (!options.background) {
            console.log(chalk.green('✅ Nox ecosystem is running!'));
            console.log(chalk.yellow('🌐 Dashboard available at http://localhost:3000'));
            console.log(chalk.gray('Press Ctrl+C to stop'));
            
            // Keep process alive
            process.on('SIGINT', async () => {
              console.log(chalk.yellow('\n⏹️  Shutting down Nox ecosystem...'));
              await noxSystem.shutdown();
              console.log(chalk.green('✅ Shutdown complete'));
              process.exit(0);
            });
          }
        } catch (error) {
          console.error(chalk.red('❌ Failed to start ecosystem:'), error);
          process.exit(1);
        }
      });

    // Stop command
    program
      .command('stop')
      .description('Stop the Nox agent ecosystem')
      .action(async () => {
        try {
          console.log(chalk.yellow('⏹️  Stopping Nox ecosystem...'));
          await noxSystem.shutdown();
          console.log(chalk.green('✅ Ecosystem stopped successfully'));
        } catch (error) {
          console.error(chalk.red('❌ Failed to stop ecosystem:'), error);
          process.exit(1);
        }
      });

    // Status command
    program
      .command('status')
      .description('Show ecosystem status')
      .action(async () => {
        try {
          if (!noxSystem.isInitialized) {
            await noxSystem.initialize();
          }
          const status = await noxSystem.getStatus();
          console.log(chalk.blue('📊 Nox Ecosystem Status:'));
          console.log(`Status: ${status.running ? chalk.green('Running') : chalk.red('Stopped')}`);
          console.log(`Active Agents: ${chalk.cyan(status.activeAgents)}`);
          console.log(`Total Tasks: ${chalk.cyan(status.totalTasks)}`);
          console.log(`Registry Version: ${chalk.gray(status.registryVersion)}`);
          console.log(`Uptime: ${chalk.gray(status.uptime)}`);
        } catch (error) {
          console.error(chalk.red('❌ Failed to get status:'), error);
          process.exit(1);
        }
      });

    // Parse command line arguments
    await program.parseAsync(process.argv);

  } catch (error: any) {
    // Don't treat help display as an error
    if (error.code === 'commander.helpDisplayed') {
      return;
    }
    
    logger.error('Fatal error in main:', error);
    console.error(chalk.red('❌ Fatal error:'), error);
    process.exit(1);
  }
}

// Handle unhandled rejections
process.on('unhandledRejection', (reason, promise) => {
  logger.error('Unhandled Rejection at:', promise, 'reason:', reason);
  console.error(chalk.red('❌ Unhandled promise rejection:'), reason);
  process.exit(1);
});

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  logger.error('Uncaught Exception:', error);
  console.error(chalk.red('❌ Uncaught exception:'), error);
  process.exit(1);
});

// Entry point
if (require.main === module) {
  main().catch((error) => {
    console.error(chalk.red('❌ Application failed to start:'), error);
    process.exit(1);
  });
}

export { main };