import { simpleGit, SimpleGit, CommitResult } from 'simple-git';
import * as path from 'path';
import { FileUtils } from '../utils/file-utils';
import { logger } from '../utils/logger';

export interface GitCommit {
  hash: string;
  message: string;
  author: string;
  date: Date;
}

export class GitManager {
  private git: SimpleGit;
  private repositoryPath: string;
  private initialized = false;

  constructor() {
    this.repositoryPath = '';
    this.git = simpleGit();
  }

  async initialize(registryPath: string): Promise<void> {
    this.repositoryPath = registryPath;
    this.git = simpleGit(this.repositoryPath);

    try {
      // Ensure registry directory exists
      await FileUtils.ensureDirectory(this.repositoryPath);

      // Check if git repository exists
      const isRepo = await this.isGitRepository();
      
      if (!isRepo) {
        logger.info('Initializing Git repository for registry');
        await this.git.init();
        
        // Create initial .gitkeep file
        const gitkeepPath = path.join(this.repositoryPath, '.gitkeep');
        await FileUtils.writeJson(gitkeepPath, {});
        
        // Create .gitignore for the registry repo
        const gitignorePath = path.join(this.repositoryPath, '.gitignore');
        const gitignoreContent = `# Runtime files
conversations/
backups/
*.log
*.tmp
`;
        await FileUtils.ensureDirectory(path.dirname(gitignorePath));
        await require('fs/promises').writeFile(gitignorePath, gitignoreContent, 'utf-8');
        
        // Configure git if not already configured
        await this.configureGit();
        
        // Initial commit
        await this.git.add('.gitkeep');
        await this.git.commit('Initial commit - Nox registry initialized');
        
        logger.info('Git repository initialized');
      } else {
        logger.info('Existing Git repository found');
      }

      this.initialized = true;

    } catch (error) {
      logger.error('Failed to initialize Git repository:', error);
      throw error;
    }
  }

  private async isGitRepository(): Promise<boolean> {
    try {
      await this.git.status();
      return true;
    } catch {
      return false;
    }
  }

  private async configureGit(): Promise<void> {
    try {
      // Check if user is already configured globally
      const globalConfig = await this.git.getConfig('user.name', 'global').catch(() => null);
      
      if (!globalConfig) {
        // Set default user for this repository
        await this.git.addConfig('user.name', 'Nox System', false);
        await this.git.addConfig('user.email', 'nox@localhost', false);
        logger.info('Git user configuration set for repository');
      }
    } catch (error) {
      logger.warn('Could not configure git user:', error);
    }
  }

  async commit(message: string, files?: string[]): Promise<string> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      // Add files to staging area
      if (files && files.length > 0) {
        await this.git.add(files);
      } else {
        // Add all changes
        await this.git.add('.');
      }

      // Check if there are changes to commit
      const status = await this.git.status();
      if (status.staged.length === 0) {
        logger.debug('No changes to commit');
        return await this.getCurrentCommit();
      }

      // Commit with timestamp and prefix
      const timestamp = new Date().toISOString();
      const commitMessage = `[nox] ${message} - ${timestamp}`;
      
      const result: CommitResult = await this.git.commit(commitMessage);
      logger.info(`Git commit created: ${result.commit} - ${message}`);
      
      return result.commit;

    } catch (error) {
      logger.error('Failed to create git commit:', error);
      throw error;
    }
  }

  async getCurrentCommit(): Promise<string> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const log = await this.git.log({ maxCount: 1 });
      return log.latest?.hash || 'no-commits';
    } catch (error) {
      logger.error('Failed to get current commit:', error);
      return 'unknown';
    }
  }

  async getCommitHistory(limit = 10): Promise<GitCommit[]> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const log = await this.git.log({ maxCount: limit });
      
      return log.all.map(commit => ({
        hash: commit.hash,
        message: commit.message,
        author: commit.author_name,
        date: new Date(commit.date)
      }));

    } catch (error) {
      logger.error('Failed to get commit history:', error);
      return [];
    }
  }

  async rollback(commitHash: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      logger.info(`Rolling back to commit: ${commitHash}`);
      
      // Create backup commit before rollback
      await this.commit('Backup before rollback');
      
      // Reset to specified commit
      await this.git.reset(['--hard', commitHash]);
      
      logger.info(`Rollback completed to commit: ${commitHash}`);

    } catch (error) {
      logger.error('Failed to rollback:', error);
      throw error;
    }
  }

  async rollbackToTime(hoursAgo: number): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const targetTime = new Date(Date.now() - (hoursAgo * 60 * 60 * 1000));
      const history = await this.getCommitHistory(100); // Get more history to find the right commit
      
      // Find the first commit before the target time
      const targetCommit = history.find(commit => commit.date <= targetTime);
      
      if (!targetCommit) {
        throw new Error(`No commit found ${hoursAgo} hours ago`);
      }

      logger.info(`Rolling back to commit from ${targetCommit.date.toISOString()}: ${targetCommit.hash}`);
      await this.rollback(targetCommit.hash);

    } catch (error) {
      logger.error('Failed to rollback to time:', error);
      throw error;
    }
  }

  async createBranch(branchName: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      await this.git.checkoutLocalBranch(branchName);
      logger.info(`Created and switched to branch: ${branchName}`);
    } catch (error) {
      logger.error(`Failed to create branch ${branchName}:`, error);
      throw error;
    }
  }

  async switchBranch(branchName: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      await this.git.checkout(branchName);
      logger.info(`Switched to branch: ${branchName}`);
    } catch (error) {
      logger.error(`Failed to switch to branch ${branchName}:`, error);
      throw error;
    }
  }

  async listBranches(): Promise<string[]> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const branches = await this.git.branchLocal();
      return branches.all;
    } catch (error) {
      logger.error('Failed to list branches:', error);
      return [];
    }
  }

  async getCurrentBranch(): Promise<string> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const branches = await this.git.branchLocal();
      return branches.current;
    } catch (error) {
      logger.error('Failed to get current branch:', error);
      return 'unknown';
    }
  }

  async mergeBranch(branchName: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      await this.git.merge([branchName]);
      logger.info(`Merged branch: ${branchName}`);
    } catch (error) {
      logger.error(`Failed to merge branch ${branchName}:`, error);
      throw error;
    }
  }

  async getStatus(): Promise<{ clean: boolean; staged: number; modified: number; untracked: number }> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const status = await this.git.status();
      
      return {
        clean: status.isClean(),
        staged: status.staged.length,
        modified: status.modified.length,
        untracked: status.not_added.length
      };
    } catch (error) {
      logger.error('Failed to get git status:', error);
      return { clean: false, staged: 0, modified: 0, untracked: 0 };
    }
  }

  async addRemote(name: string, url: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      await this.git.addRemote(name, url);
      logger.info(`Added remote: ${name} -> ${url}`);
    } catch (error) {
      logger.error(`Failed to add remote ${name}:`, error);
      throw error;
    }
  }

  async push(remote = 'origin', branch?: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const currentBranch = branch || await this.getCurrentBranch();
      await this.git.push(remote, currentBranch);
      logger.info(`Pushed to ${remote}/${currentBranch}`);
    } catch (error) {
      logger.error('Failed to push:', error);
      throw error;
    }
  }

  async pull(remote = 'origin', branch?: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('Git manager not initialized');
    }

    try {
      const currentBranch = branch || await this.getCurrentBranch();
      await this.git.pull(remote, currentBranch);
      logger.info(`Pulled from ${remote}/${currentBranch}`);
    } catch (error) {
      logger.error('Failed to pull:', error);
      throw error;
    }
  }

  get isInitialized(): boolean {
    return this.initialized;
  }
}