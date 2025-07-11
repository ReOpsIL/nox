import { ConfigManager } from '../../../src/core/config-manager';
import { DEFAULT_CONFIG } from '../../../src/types/config';
import { FileUtils } from '../../../src/utils/file-utils';
import * as path from 'path';

describe('ConfigManager', () => {
  let configManager: ConfigManager;
  let tempDir: string;
  let configPath: string;

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    configPath = path.join(tempDir, 'test-config.json');
    configManager = new ConfigManager(configPath);
  });

  afterEach(async () => {
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  describe('loadConfig', () => {
    it('should load default config when no file exists', async () => {
      const config = await configManager.loadConfig();
      expect(config).toEqual(DEFAULT_CONFIG);
    });

    it('should load existing config file', async () => {
      const customConfig = {
        ...DEFAULT_CONFIG,
        server: { ...DEFAULT_CONFIG.server, port: 8080 }
      };
      
      await FileUtils.writeJson(configPath, customConfig);
      const config = await configManager.loadConfig();
      
      expect(config.server.port).toBe(8080);
    });

    it('should merge partial config with defaults', async () => {
      const partialConfig = {
        server: { port: 9000 },
        security: { maxAgents: 100 }
      };
      
      await FileUtils.writeJson(configPath, partialConfig);
      const config = await configManager.loadConfig();
      
      expect(config.server.port).toBe(9000);
      expect(config.security.maxAgents).toBe(100);
      expect(config.security.spawnRateLimit).toBe(DEFAULT_CONFIG.security.spawnRateLimit);
    });
  });

  describe('saveConfig', () => {
    it('should save config to file', async () => {
      const config = await configManager.loadConfig();
      config.server.port = 7000;
      
      await configManager.saveConfig(config);
      
      const savedConfig = await FileUtils.readJson<any>(configPath);
      expect(savedConfig.server.port).toBe(7000);
    });
  });

  describe('environment overrides', () => {
    it('should apply environment variable overrides', async () => {
      process.env.NOX_PORT = '5555';
      process.env.NOX_MAX_AGENTS = '25';
      
      const config = await configManager.loadConfig();
      
      expect(config.server.port).toBe(5555);
      expect(config.security.maxAgents).toBe(25);
      
      // Cleanup
      delete process.env.NOX_PORT;
      delete process.env.NOX_MAX_AGENTS;
    });
  });

  describe('getters', () => {
    it('should provide config getters', async () => {
      await configManager.loadConfig();
      
      expect(configManager.getRegistryPath()).toBe(DEFAULT_CONFIG.storage.registryPath);
      expect(configManager.getMaxAgents()).toBe(DEFAULT_CONFIG.security.maxAgents);
      expect(configManager.isSecurityEnabled()).toBe(DEFAULT_CONFIG.security.sandboxMode);
    });

    it('should check approval requirements', async () => {
      await configManager.loadConfig();
      
      expect(configManager.requiresApproval('agent_creation')).toBe(true);
      expect(configManager.requiresApproval('unknown_action')).toBe(false);
    });
  });
});