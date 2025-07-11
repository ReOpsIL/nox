import { Command } from 'commander';
import chalk from 'chalk';
import { NoxSystem } from '../core/system';
import * as fs from 'fs';
import * as path from 'path';

export class SystemCommands {
  static register(program: Command, nox: NoxSystem): void {
    
    // Health check command
    program
      .command('health')
      .description('Check system health and status')
      .action(async () => {
        try {
          const status = await nox.getStatus();
          
          console.log(chalk.blue('🏥 Nox System Health Check\n'));
          
          console.log(chalk.bold('System Status:'));
          console.log(`   Running: ${status.running ? chalk.green('Yes') : chalk.red('No')}`);
          console.log(`   Uptime: ${chalk.cyan(status.uptime)}`);
          console.log(`   Memory Usage: ${chalk.cyan(status.memoryUsage + 'MB')}`);
          
          console.log(chalk.bold('\nAgent Status:'));
          console.log(`   Active Agents: ${chalk.green(status.activeAgents)}`);
          console.log(`   Total Tasks: ${chalk.cyan(status.totalTasks)}`);
          
          console.log(chalk.bold('\nRegistry:'));
          console.log(`   Version: ${chalk.gray(status.registryVersion)}`);
          
          if (status.errors.length > 0) {
            console.log(chalk.bold('\nErrors:'));
            status.errors.forEach(error => {
              console.log(`   ${chalk.red('•')} ${error}`);
            });
          } else {
            console.log(chalk.green('\n✅ No errors detected'));
          }

        } catch (error) {
          console.error(chalk.red('❌ Failed to check system health:'), error);
          process.exit(1);
        }
      });

    // Config command
    program
      .command('config')
      .description('Show current configuration')
      .option('--json', 'Output as JSON')
      .action(async (options) => {
        try {
          const config = nox.config.getConfig();
          
          if (options.json) {
            console.log(JSON.stringify(config, null, 2));
            return;
          }

          console.log(chalk.blue('⚙️  Nox Configuration\n'));
          
          console.log(chalk.bold('Security:'));
          console.log(`   Max Agents: ${chalk.cyan(config.security.maxAgents)}`);
          console.log(`   Spawn Rate Limit: ${chalk.cyan(config.security.spawnRateLimit)}/hour`);
          console.log(`   Sandbox Mode: ${config.security.sandboxMode ? chalk.green('Enabled') : chalk.red('Disabled')}`);
          
          console.log(chalk.bold('\nClaude CLI:'));
          console.log(`   CLI Path: ${chalk.gray(config.claudeCli.cliPath)}`);
          console.log(`   Session Timeout: ${chalk.cyan(config.claudeCli.sessionTimeout + 's')}`);
          console.log(`   Auto Restart: ${config.claudeCli.autoRestartOnCrash ? chalk.green('Yes') : chalk.red('No')}`);
          
          console.log(chalk.bold('\nStorage:'));
          console.log(`   Registry Path: ${chalk.gray(config.storage.registryPath)}`);
          console.log(`   Format: ${chalk.cyan(config.storage.format)}`);
          console.log(`   Compression: ${config.storage.compression ? chalk.green('Enabled') : chalk.red('Disabled')}`);
          
          console.log(chalk.bold('\nServer:'));
          console.log(`   Port: ${chalk.cyan(config.server.port)}`);
          console.log(`   Host: ${chalk.gray(config.server.host)}`);
          console.log(`   WebSocket: ${config.server.websocketEnabled ? chalk.green('Enabled') : chalk.red('Disabled')}`);
          console.log(`   Dashboard: ${config.server.dashboardEnabled ? chalk.green('Enabled') : chalk.red('Disabled')}`);

        } catch (error) {
          console.error(chalk.red('❌ Failed to show configuration:'), error);
          process.exit(1);
        }
      });

    // Logs command
    program
      .command('logs')
      .description('Show system logs')
      .option('--tail <lines>', 'Number of lines to show', '50')
      .option('--follow', 'Follow log output')
      .action(async (options) => {
        console.log(chalk.blue('📋 System Logs\n'));
        // ⚠️  MOCK/DUMMY IMPLEMENTATION - NO ACTUAL LOG READING ⚠️
        console.log(chalk.yellow('⚠️  PLACEHOLDER IMPLEMENTATION - NO ACTUAL LOG FUNCTIONALITY'));
        console.log(chalk.red('TODO: Implement actual log file reading from system log files'));
        console.log(chalk.gray('This would show the last ' + options.tail + ' lines of system logs'));
        
        if (options.follow) {
          console.log(chalk.gray('Follow mode would tail the logs in real-time'));
        }
      });

    // Version command
    program
      .command('version')
      .description('Show version information')
      .action(() => {
        let version = '1.0.0'; // fallback
        try {
          const packagePath = path.join(__dirname, '../..', 'package.json');
          const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf-8'));
          version = packageJson.version;
        } catch (error) {
          // Use fallback version if package.json not found
        }
        
        console.log(chalk.blue('📦 Nox Version Information\n'));
        console.log(`   Version: ${chalk.cyan(version)}`);
        console.log(`   Node.js: ${chalk.gray(process.version)}`);
        console.log(`   Platform: ${chalk.gray(process.platform)}`);
        console.log(`   Architecture: ${chalk.gray(process.arch)}`);
      });
  }
}