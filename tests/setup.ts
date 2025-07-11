// Jest setup file for Nox tests
import * as path from 'path';
import * as fs from 'fs/promises';

// Set up test environment
process.env.NODE_ENV = 'test';
process.env.NOX_LOG_LEVEL = 'error'; // Reduce log noise in tests

// Global test timeout
jest.setTimeout(30000);

// Mock console.log/error to reduce noise unless debugging
const originalConsole = global.console;
global.console = {
  ...originalConsole,
  log: jest.fn(),
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn(),
  debug: jest.fn()
};

// Global test helpers
declare global {
  namespace globalThis {
    var testUtils: {
      createTempDir: () => Promise<string>;
      cleanupTempDir: (dir: string) => Promise<void>;
      restoreConsole: () => void;
    };
  }
}

(global as any).testUtils = {
  async createTempDir(): Promise<string> {
    const tmpDir = await fs.mkdtemp(path.join(__dirname, '../temp-test-'));
    return tmpDir;
  },

  async cleanupTempDir(dir: string): Promise<void> {
    try {
      await fs.rm(dir, { recursive: true, force: true });
    } catch (error) {
      // Ignore cleanup errors
    }
  },

  restoreConsole(): void {
    global.console = originalConsole;
  }
};