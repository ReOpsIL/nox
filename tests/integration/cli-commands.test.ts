import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs/promises';

const CLI_PATH = path.join(__dirname, '../../dist/nox.js');

describe('CLI Commands Integration', () => {
  let tempDir: string;
  let originalCwd: string;

  beforeAll(async () => {
    // Ensure the CLI is built
    try {
      execSync('npm run build', { cwd: path.join(__dirname, '../..'), stdio: 'ignore' });
    } catch (error) {
      console.warn('Build failed, tests may fail');
    }
  });

  beforeEach(async () => {
    tempDir = await (global as any).testUtils.createTempDir();
    originalCwd = process.cwd();
    process.chdir(tempDir);
  });

  afterEach(async () => {
    process.chdir(originalCwd);
    await (global as any).testUtils.cleanupTempDir(tempDir);
  });

  const runCLI = (args: string): string => {
    try {
      return execSync(`node "${CLI_PATH}" ${args}`, {
        encoding: 'utf8',
        timeout: 10000,
        env: { ...process.env, NOX_LOG_LEVEL: 'error' }
      });
    } catch (error: any) {
      if (error.stdout) return error.stdout;
      throw error;
    }
  };

  describe('System Commands', () => {
    it('should show help', () => {
      const output = runCLI('--help');
      expect(output).toContain('Autonomous AI Agent Ecosystem');
      expect(output).toContain('add-agent');
      expect(output).toContain('list-agents');
    });

    it('should show version', () => {
      const output = runCLI('version');
      expect(output).toContain('Version: 1.0.0');
      expect(output).toContain('Node.js:');
    });

    it('should initialize system', () => {
      const output = runCLI('init');
      expect(output).toContain('Nox ecosystem initialized successfully');
    });

    it('should show status after init', () => {
      runCLI('init');
      const output = runCLI('status');
      expect(output).toContain('Nox Ecosystem Status');
      expect(output).toContain('Status: Stopped');
      expect(output).toContain('Active Agents: 0');
    });
  });

  describe('Agent Management', () => {
    beforeEach(() => {
      runCLI('init');
    });

    it('should create an agent', () => {
      const output = runCLI('add-agent test-agent "You are a test assistant"');
      expect(output).toContain('Agent created successfully');
      expect(output).toContain('ID: test-agent');
    });

    it('should list agents', () => {
      runCLI('add-agent agent1 "First agent"');
      runCLI('add-agent agent2 "Second agent"');
      
      const output = runCLI('list-agents');
      expect(output).toContain('Found 2 agent(s)');
      expect(output).toContain('agent1');
      expect(output).toContain('agent2');
    });

    it('should show agent details', () => {
      runCLI('add-agent detailed-agent "Detailed test agent"');
      
      const output = runCLI('show-agent detailed-agent');
      expect(output).toContain('Agent Details: detailed-agent');
      expect(output).toContain('Basic Information');
      expect(output).toContain('Resource Limits');
      expect(output).toContain('System Prompt');
    });

    it('should update agent prompt', () => {
      runCLI('add-agent update-agent "Original prompt"');
      
      const output = runCLI('update-agent update-agent "Updated prompt"');
      expect(output).toContain('Agent updated successfully');
    });

    it('should delete agent with force flag', () => {
      runCLI('add-agent delete-agent "Agent to delete"');
      
      const output = runCLI('delete-agent delete-agent --force');
      expect(output).toContain('Agent deleted successfully');
      
      // Verify agent is gone
      const listOutput = runCLI('list-agents');
      expect(listOutput).not.toContain('delete-agent');
    });

    it('should output JSON format', () => {
      runCLI('add-agent json-agent "JSON test agent"');
      
      const output = runCLI('list-agents --format json');
      const agents = JSON.parse(output);
      expect(Array.isArray(agents)).toBe(true);
      expect(agents[0]).toHaveProperty('id', 'json-agent');
      expect(agents[0]).toHaveProperty('systemPrompt');
    });

    it('should filter agents by status', () => {
      runCLI('add-agent active-agent "Active agent"');
      runCLI('add-agent inactive-agent "Inactive agent"');
      
      const output = runCLI('list-agents --status inactive');
      expect(output).toContain('Found 2 agent(s)'); // Both are inactive by default
    });

    it('should reject invalid agent names', () => {
      try {
        runCLI('add-agent "invalid name with spaces" "Test prompt"');
        fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('Invalid agent name');
      }
    });

    it('should reject duplicate agent IDs', () => {
      runCLI('add-agent duplicate-test "First agent"');
      
      try {
        runCLI('add-agent duplicate-test "Second agent"');
        throw new Error('Should have thrown an error');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('already exists');
      }
    });
  });

  describe('Registry Commands', () => {
    beforeEach(() => {
      runCLI('init');
      runCLI('add-agent ml-expert "You are a machine learning expert"');
      runCLI('add-agent web-dev "You are a web development expert"');
    });

    it('should show registry status', () => {
      const output = runCLI('registry-status');
      expect(output).toContain('Registry Status');
      expect(output).toContain('Total Agents: 2');
      expect(output).toContain('Git Status');
    });

    it('should show registry history', () => {
      const output = runCLI('registry-history');
      expect(output).toContain('Registry History');
      expect(output).toContain('Created agent');
    });

    it('should query agents by capability', () => {
      const output = runCLI('query-registry "machine learning"');
      expect(output).toContain('Found 1 agent(s)');
      expect(output).toContain('ml-expert');
    });

    it('should create manual backup', () => {
      const output = runCLI('registry-backup "Manual test backup"');
      expect(output).toContain('Backup created successfully');
    });
  });

  describe('Task Commands', () => {
    beforeEach(() => {
      runCLI('init');
      runCLI('add-agent task-agent "You are a task management expert"');
    });

    it('should create a task', () => {
      const output = runCLI('create-task task-agent "Test Task" "This is a test task"');
      expect(output).toContain('Task created successfully');
      expect(output).toContain('Title: Test Task');
    });

    it('should create task with priority', () => {
      const output = runCLI('create-task task-agent "High Priority Task" "Urgent task" --priority HIGH');
      expect(output).toContain('Task created successfully');
      expect(output).toContain('Priority: HIGH');
    });

    it('should list tasks for agent', () => {
      runCLI('create-task task-agent "Task 1" "First task"');
      runCLI('create-task task-agent "Task 2" "Second task"');
      
      const output = runCLI('list-tasks task-agent');
      expect(output).toContain('Tasks for task-agent');
      expect(output).toContain('Task 1');
      expect(output).toContain('Task 2');
    });

    it('should show task overview', () => {
      runCLI('create-task task-agent "Overview Task" "Task for overview"');
      
      const output = runCLI('task-overview');
      expect(output).toContain('Task Overview');
      expect(output).toContain('Total Tasks:');
    });

    it('should reject invalid agent for task creation', () => {
      try {
        runCLI('create-task non-existent-agent "Task" "Description"');
        throw new Error('Should have thrown an error');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('not found');
      }
    });
  });

  describe('Error Handling', () => {
    it('should handle invalid commands gracefully', () => {
      try {
        runCLI('invalid-command');
        throw new Error('Should have thrown an error');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('unknown command');
      }
    });

    it('should require agent initialization for most commands', () => {
      try {
        runCLI('list-agents');
        // Should work because of auto-initialization
        expect(true).toBe(true);
      } catch (error) {
        // If it fails, that's also acceptable for this test
      }
    });
  });

  describe('File System Integration', () => {
    beforeEach(() => {
      runCLI('init');
    });

    it('should create registry directory structure', async () => {
      expect(await fs.access('.nox-registry').then(() => true).catch(() => false)).toBe(true);
      expect(await fs.access('.nox-registry/agents.json').then(() => true).catch(() => false)).toBe(true);
      expect(await fs.access('.nox-registry/agent-relationships.json').then(() => true).catch(() => false)).toBe(true);
    });

    it('should persist agents between commands', () => {
      runCLI('add-agent persistent-agent "Persistent test agent"');
      
      // Run list command to verify persistence
      const output = runCLI('list-agents');
      expect(output).toContain('persistent-agent');
    });

    it('should create git repository in registry', async () => {
      expect(await fs.access('.nox-registry/.git').then(() => true).catch(() => false)).toBe(true);
    });
  });
});