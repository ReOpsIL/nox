import { Command } from 'commander';
import chalk from 'chalk';
import { NoxSystem } from '../core/system';

export class RegistryCommands {
  private static async ensureInitialized(nox: NoxSystem): Promise<void> {
    if (!nox.isInitialized) {
      await nox.initialize();
    }
  }

  static register(program: Command, nox: NoxSystem): void {

    // Registry status command
    program
      .command('registry-status')
      .description('Show registry information and statistics')
      .action(async () => {
        try {
          await RegistryCommands.ensureInitialized(nox);

          const stats = await nox.registry.getStats();
          const gitStatus = await nox.git.getStatus();
          const currentCommit = await nox.git.getCurrentCommit();
          const currentBranch = await nox.git.getCurrentBranch();

          console.log(chalk.blue('📂 Registry Status\n'));

          console.log(chalk.bold('Registry Statistics:'));
          console.log(`   Total Agents: ${chalk.cyan(stats.totalAgents)}`);
          console.log(`   Active: ${chalk.green(stats.activeAgents)}`);
          console.log(`   Inactive: ${chalk.yellow(stats.inactiveAgents)}`);
          console.log(`   Errors: ${chalk.red(stats.errorAgents)}`);
          console.log(`   Last Modified: ${chalk.gray(stats.lastModified.toISOString())}`);
          console.log(`   Registry Size: ${chalk.cyan(Math.round(stats.registrySize / 1024) + 'KB')}`);

          console.log(chalk.bold('\nGit Status:'));
          console.log(`   Branch: ${chalk.cyan(currentBranch)}`);
          console.log(`   Commit: ${chalk.gray(currentCommit.slice(0, 8))}`);
          console.log(`   Clean: ${gitStatus.clean ? chalk.green('Yes') : chalk.yellow('No')}`);
          console.log(`   Staged: ${chalk.cyan(gitStatus.staged)}`);
          console.log(`   Modified: ${chalk.yellow(gitStatus.modified)}`);
          console.log(`   Untracked: ${chalk.red(gitStatus.untracked)}`);

        } catch (error) {
          console.error(chalk.red('❌ Failed to get registry status:'), error);
          process.exit(1);
        }
      });

    // Registry history command
    program
      .command('registry-history')
      .description('Show registry change history')
      .option('--limit <count>', 'Number of commits to show', '10')
      .action(async (options) => {
        try {
          await RegistryCommands.ensureInitialized(nox);

          const limit = parseInt(options.limit);
          const history = await nox.git.getCommitHistory(limit);

          if (history.length === 0) {
            console.log(chalk.yellow('📭 No commit history found'));
            return;
          }

          console.log(chalk.blue(`📚 Registry History (last ${history.length} commits)\n`));

          history.forEach((commit, index) => {
            const timeAgo = this.getTimeAgo(commit.date);
            console.log(`${index + 1}. ${chalk.yellow(commit.hash.slice(0, 8))} - ${commit.message}`);
            console.log(`   ${chalk.gray(`by ${commit.author} • ${timeAgo}`)}\n`);
          });

        } catch (error) {
          console.error(chalk.red('❌ Failed to get registry history:'), error);
          process.exit(1);
        }
      });

    // Registry backup command
    program
      .command('registry-backup [message]')
      .description('Create a manual registry backup')
      .action(async (message: string = 'Manual backup') => {
        try {
          await RegistryCommands.ensureInitialized(nox);

          console.log(chalk.blue('💾 Creating registry backup...'));
          const commitHash = await nox.git.commit(message);

          console.log(chalk.green('✅ Backup created successfully!'));
          console.log(chalk.gray(`   Commit: ${commitHash.slice(0, 8)}`));
          console.log(chalk.gray(`   Message: ${message}`));

        } catch (error) {
          console.error(chalk.red('❌ Failed to create backup:'), error);
          process.exit(1);
        }
      });

    // Registry rollback command
    program
      .command('registry-rollback <target>')
      .description('Rollback registry to a previous state')
      .option('--force', 'Skip confirmation prompt')
      .action(async (target: string, options) => {
        try {
          await RegistryCommands.ensureInitialized(nox);

          // Parse target (commit hash or "last" or "hours-ago:N")
          let commitHash: string;

          if (target === 'last') {
            const history = await nox.git.getCommitHistory(2);
            if (history.length < 2) {
              console.error(chalk.red('❌ No previous commit found'));
              process.exit(1);
            }
            commitHash = history[1]!.hash;
          } else if (target.startsWith('hours-ago:')) {
            const hours = parseInt(target.split(':')[1]!);
            if (isNaN(hours)) {
              console.error(chalk.red('❌ Invalid hours format. Use: hours-ago:N'));
              process.exit(1);
            }
            await nox.git.rollbackToTime(hours);
            console.log(chalk.green(`✅ Rolled back to ${hours} hours ago`));
            return;
          } else {
            commitHash = target;
          }

          // Confirmation
          if (!options.force) {
            console.log(chalk.yellow(`⚠️  This will rollback the registry to commit: ${commitHash.slice(0, 8)}`));
            console.log(chalk.yellow('   All changes since then will be lost!'));
            console.log(chalk.yellow('   Use --force flag to proceed'));
            return;
          }

          console.log(chalk.blue(`🔄 Rolling back to commit: ${commitHash.slice(0, 8)}`));
          await nox.git.rollback(commitHash);

          // Reload the registry after rollback
          if (nox.isRunning) {
            console.log(chalk.blue('🔄 Reloading registry...'));

            // Reload the registry
            await nox.registry.loadRegistry();

            // Get all agents
            const agents = await nox.registry.listAgents();

            // Stop all currently running agents
            console.log(chalk.blue('🛑 Stopping current agents...'));
            await nox.agents.shutdown();

            // Start agents that should be active according to the rolled-back registry
            const activeAgents = agents.filter(agent => agent.status === 'active');
            console.log(chalk.blue(`🚀 Starting ${activeAgents.length} active agents...`));

            for (const agent of activeAgents) {
              try {
                await nox.agents.spawnAgent(agent);
                console.log(chalk.green(`✅ Started agent: ${agent.name} (${agent.id})`));
              } catch (error) {
                console.error(chalk.red(`❌ Failed to start agent ${agent.id}:`), error);
              }
            }
          }

          console.log(chalk.green('✅ Rollback completed successfully!'));

        } catch (error) {
          console.error(chalk.red('❌ Failed to rollback registry:'), error);
          process.exit(1);
        }
      });

    // Query registry command  
    program
      .command('query-registry <capability>')
      .description('Find agents with specific capabilities')
      .action(async (capability: string) => {
        try {
          await RegistryCommands.ensureInitialized(nox);

          const agents = await nox.registry.findAgentsByCapability(capability);

          if (agents.length === 0) {
            console.log(chalk.yellow(`📭 No agents found with capability: "${capability}"`));
            return;
          }

          console.log(chalk.blue(`🔍 Found ${agents.length} agent(s) with capability: "${capability}"\n`));

          agents.forEach((agent, index) => {
            console.log(`${index + 1}. ${chalk.bold(agent.name)} (${agent.id})`);
            console.log(`   Status: ${agent.status === 'active' ? chalk.green(agent.status) : chalk.yellow(agent.status)}`);

            if (agent.capabilities.length > 0) {
              console.log(`   Capabilities: ${chalk.cyan(agent.capabilities.join(', '))}`);
            }

            // Show relevant part of system prompt
            const promptSnippet = agent.systemPrompt.slice(0, 100) + (agent.systemPrompt.length > 100 ? '...' : '');
            console.log(`   Prompt: ${chalk.gray('"' + promptSnippet + '"')}`);
            console.log();
          });

        } catch (error) {
          console.error(chalk.red('❌ Failed to query registry:'), error);
          process.exit(1);
        }
      });
  }

  private static getTimeAgo(date: Date): string {
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffDays > 0) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    if (diffHours > 0) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    if (diffMins > 0) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    return 'just now';
  }
}
