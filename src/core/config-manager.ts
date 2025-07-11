import * as path from 'path';
import { NoxConfig, DEFAULT_CONFIG } from '../types/config';
import { FileUtils } from '../utils/file-utils';
import { logger } from '../utils/logger';

export class ConfigManager {
  private config: NoxConfig | null = null;
  private configPath: string;

  constructor(configPath?: string) {
    this.configPath = configPath || this.getDefaultConfigPath();
  }

  private getDefaultConfigPath(): string {
    // Check for config file in order of preference
    const possiblePaths = [
      process.env.NOX_CONFIG_PATH,
      path.join(process.cwd(), 'nox.config.json'),
      path.join(process.cwd(), 'config', 'nox.json'),
      path.join(process.cwd(), '.nox', 'config.json')
    ].filter(Boolean) as string[];

    return possiblePaths[0] || path.join(process.cwd(), 'nox.config.json');
  }

  async loadConfig(): Promise<NoxConfig> {
    try {
      // Try to load existing config
      if (await FileUtils.exists(this.configPath)) {
        logger.info(`Loading configuration from ${this.configPath}`);
        const loadedConfig = await FileUtils.readJson<Partial<NoxConfig>>(this.configPath);
        this.config = this.mergeWithDefaults(loadedConfig);
      } else {
        logger.info('No configuration file found, using defaults');
        this.config = { ...DEFAULT_CONFIG };
        await this.saveConfig();
      }

      // Apply environment variable overrides
      this.applyEnvironmentOverrides();

      logger.debug('Configuration loaded:', this.config);
      return this.config;

    } catch (error) {
      logger.error('Failed to load configuration:', error);
      logger.info('Using default configuration');
      this.config = { ...DEFAULT_CONFIG };
      return this.config;
    }
  }

  async saveConfig(config?: NoxConfig): Promise<void> {
    if (config) {
      this.config = config;
    }

    if (!this.config) {
      throw new Error('No configuration to save');
    }

    try {
      await FileUtils.writeJson(this.configPath, this.config);
      logger.info(`Configuration saved to ${this.configPath}`);
    } catch (error) {
      logger.error('Failed to save configuration:', error);
      throw error;
    }
  }

  private mergeWithDefaults(partial: Partial<NoxConfig>): NoxConfig {
    const merged: NoxConfig = {
      security: { ...DEFAULT_CONFIG.security, ...partial.security },
      claudeCli: { ...DEFAULT_CONFIG.claudeCli, ...partial.claudeCli },
      git: { ...DEFAULT_CONFIG.git, ...partial.git },
      server: { ...DEFAULT_CONFIG.server, ...partial.server },
      storage: { ...DEFAULT_CONFIG.storage, ...partial.storage },
      mcp: { ...DEFAULT_CONFIG.mcp, ...partial.mcp },
      logging: { ...DEFAULT_CONFIG.logging, ...partial.logging }
    };

    // Deep merge nested objects
    if (partial.security?.resourceLimits) {
      merged.security.resourceLimits = {
        ...DEFAULT_CONFIG.security.resourceLimits,
        ...partial.security.resourceLimits
      };
    }

    if (partial.server?.rateLimiting) {
      merged.server.rateLimiting = {
        ...DEFAULT_CONFIG.server.rateLimiting,
        ...partial.server.rateLimiting
      };
    }

    if (partial.mcp?.resourceLimits) {
      merged.mcp.resourceLimits = {
        ...DEFAULT_CONFIG.mcp.resourceLimits,
        ...partial.mcp.resourceLimits
      };
    }

    if (partial.logging?.retention) {
      merged.logging.retention = {
        ...DEFAULT_CONFIG.logging.retention,
        ...partial.logging.retention
      };
    }

    return merged;
  }

  private applyEnvironmentOverrides(): void {
    if (!this.config) return;

    // Override with environment variables
    if (process.env.NOX_PORT) {
      this.config.server.port = parseInt(process.env.NOX_PORT, 10);
    }

    if (process.env.NOX_HOST) {
      this.config.server.host = process.env.NOX_HOST;
    }

    if (process.env.NOX_REGISTRY_PATH) {
      this.config.storage.registryPath = process.env.NOX_REGISTRY_PATH;
    }

    if (process.env.NOX_LOG_LEVEL) {
      const level = process.env.NOX_LOG_LEVEL as NoxConfig['logging']['level'];
      if (['debug', 'info', 'warn', 'error', 'fatal'].includes(level)) {
        this.config.logging.level = level;
      }
    }

    if (process.env.NOX_MAX_AGENTS) {
      this.config.security.maxAgents = parseInt(process.env.NOX_MAX_AGENTS, 10);
    }

    if (process.env.NOX_SPAWN_RATE_LIMIT) {
      this.config.security.spawnRateLimit = parseInt(process.env.NOX_SPAWN_RATE_LIMIT, 10);
    }

    if (process.env.NOX_CLAUDE_CLI_PATH) {
      this.config.claudeCli.cliPath = process.env.NOX_CLAUDE_CLI_PATH;
    }

    if (process.env.NOX_GIT_REMOTE) {
      this.config.git.remoteBackup = process.env.NOX_GIT_REMOTE;
    }

    if (process.env.NOX_SANDBOX_MODE === 'false') {
      this.config.security.sandboxMode = false;
    }

    if (process.env.NOX_EXTERNAL_COMMUNICATION === 'true') {
      this.config.security.allowExternalCommunication = true;
    }

    logger.debug('Applied environment overrides to configuration');
  }

  getConfig(): NoxConfig {
    if (!this.config) {
      throw new Error('Configuration not loaded. Call loadConfig() first.');
    }
    return this.config;
  }

  updateConfig(updates: Partial<NoxConfig>): NoxConfig {
    if (!this.config) {
      throw new Error('Configuration not loaded. Call loadConfig() first.');
    }

    this.config = this.mergeWithDefaults({ ...this.config, ...updates });
    return this.config;
  }

  getRegistryPath(): string {
    return this.getConfig().storage.registryPath;
  }

  isSecurityEnabled(): boolean {
    return this.getConfig().security.sandboxMode;
  }

  getMaxAgents(): number {
    return this.getConfig().security.maxAgents;
  }

  getSpawnRateLimit(): number {
    return this.getConfig().security.spawnRateLimit;
  }

  requiresApproval(action: string): boolean {
    return this.getConfig().security.requireApprovalFor.includes(action as any);
  }
}