import { execSync } from 'child_process';
import * as path from 'path';
import * as fs from 'fs/promises';

const CLI_PATH = path.join(__dirname, '../../dist/nox.js');

describe('Complete Workflows E2E', () => {
  let tempDir: string;
  let originalCwd: string;

  beforeAll(async () => {
    // Ensure the CLI is built
    try {
      execSync('npm run build', { cwd: path.join(__dirname, '../..'), stdio: 'ignore' });
    } catch (error) {
      console.warn('Build failed, E2E tests may fail');
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
        timeout: 15000,
        env: { ...process.env, NOX_LOG_LEVEL: 'error' }
      });
    } catch (error: any) {
      if (error.stdout) return error.stdout;
      throw error;
    }
  };

  describe('Full Agent Lifecycle Workflow', () => {
    it('should complete full agent management workflow', () => {
      // 1. Initialize the system
      const initOutput = runCLI('init');
      expect(initOutput).toContain('Nox ecosystem initialized successfully');

      // 2. Check initial status
      const statusOutput = runCLI('status');
      expect(statusOutput).toContain('Active Agents: 0');

      // 3. Create multiple agents
      runCLI('add-agent researcher "You are a research assistant specializing in AI and machine learning"');
      runCLI('add-agent data-scientist "You are a data scientist expert in statistical analysis"');
      runCLI('add-agent web-developer "You are a web development expert with JavaScript and React skills"');

      // 4. Verify agents were created
      const listOutput = runCLI('list-agents');
      expect(listOutput).toContain('Found 3 agent(s)');
      expect(listOutput).toContain('researcher');
      expect(listOutput).toContain('data-scientist');
      expect(listOutput).toContain('web-developer');

      // 5. Test capability search
      const mlAgents = runCLI('query-registry "machine learning"');
      expect(mlAgents).toContain('researcher');

      const webAgents = runCLI('query-registry "JavaScript"');
      expect(webAgents).toContain('web-developer');

      // 6. Update an agent
      runCLI('update-agent researcher "You are an enhanced research assistant with quantum computing expertise"');
      
      const updatedAgent = runCLI('show-agent researcher');
      expect(updatedAgent).toContain('quantum computing');

      // 7. Check registry status
      const registryStatus = runCLI('registry-status');
      expect(registryStatus).toContain('Total Agents: 3');

      // 8. View history
      const history = runCLI('registry-history');
      expect(history).toContain('Created agent');
      expect(history).toContain('Updated agent');

      // 9. Delete an agent
      runCLI('delete-agent web-developer --force');
      
      const finalList = runCLI('list-agents');
      expect(finalList).toContain('Found 2 agent(s)');
      expect(finalList).not.toContain('web-developer');
    });
  });

  describe('Task Management Workflow', () => {
    beforeEach(() => {
      runCLI('init');
      runCLI('add-agent project-manager "You are a project management expert"');
      runCLI('add-agent developer "You are a software developer"');
      runCLI('add-agent tester "You are a QA testing expert"');
    });

    it('should complete full task management workflow', () => {
      // 1. Create tasks for different agents
      runCLI('create-task project-manager "Plan Sprint" "Plan the next development sprint" --priority HIGH');
      runCLI('create-task developer "Implement Feature" "Implement user authentication" --priority MEDIUM');
      runCLI('create-task tester "Test Feature" "Test the authentication feature" --priority LOW');

      // 2. List tasks for specific agent
      const pmTasks = runCLI('list-tasks project-manager');
      expect(pmTasks).toContain('Plan Sprint');
      expect(pmTasks).toContain('Priority: HIGH');

      // 3. Check overall task overview
      const overview = runCLI('task-overview');
      expect(overview).toContain('Total Tasks: 3');
      expect(overview).toContain('project-manager');
      expect(overview).toContain('developer');
      expect(overview).toContain('tester');

      // 4. List all tasks
      const allTasks = runCLI('list-tasks');
      expect(allTasks).toContain('No tasks found'); // Since we're not specifying an agent

      // 5. Filter tasks by priority
      const devTasks = runCLI('list-tasks developer --priority MEDIUM');
      expect(devTasks).toContain('Implement Feature');
    });
  });

  describe('Multi-Agent Collaboration Scenario', () => {
    it('should handle complex multi-agent setup', () => {
      // Initialize ecosystem
      runCLI('init');

      // Create research team
      runCLI('add-agent lead-researcher "You are a lead researcher coordinating AI research projects"');
      runCLI('add-agent ml-specialist "You are a machine learning specialist focusing on deep learning"');
      runCLI('add-agent data-engineer "You are a data engineer handling large-scale data processing"');
      runCLI('add-agent research-assistant "You are a research assistant supporting literature reviews"');

      // Verify team creation
      const team = runCLI('list-agents');
      expect(team).toContain('Found 4 agent(s)');

      // Test capability-based queries
      const mlExperts = runCLI('query-registry "machine learning"');
      expect(mlExperts).toContain('ml-specialist');
      expect(mlExperts).toContain('lead-researcher');

      const dataExperts = runCLI('query-registry "data"');
      expect(dataExperts).toContain('data-engineer');

      // Create coordinated tasks
      runCLI('create-task lead-researcher "Research Project Planning" "Plan the next AI research project" --priority CRITICAL');
      runCLI('create-task ml-specialist "Model Development" "Develop new neural network architecture" --priority HIGH');
      runCLI('create-task data-engineer "Data Pipeline" "Build data processing pipeline for research" --priority HIGH');
      runCLI('create-task research-assistant "Literature Review" "Conduct comprehensive literature review" --priority MEDIUM');

      // Verify task distribution
      const taskOverview = runCLI('task-overview');
      expect(taskOverview).toContain('Total Tasks: 4');

      // Check each agent's tasks
      const leadTasks = runCLI('list-tasks lead-researcher');
      expect(leadTasks).toContain('Research Project Planning');
      expect(leadTasks).toContain('Priority: CRITICAL');

      const mlTasks = runCLI('list-tasks ml-specialist');
      expect(mlTasks).toContain('Model Development');

      // Verify registry tracking
      const registryStatus = runCLI('registry-status');
      expect(registryStatus).toContain('Total Agents: 4');
      expect(registryStatus).toContain('Active: 0'); // Not started yet
    });
  });

  describe('Error Recovery Workflow', () => {
    it('should handle and recover from various error scenarios', () => {
      // Initialize system
      runCLI('init');

      // Test duplicate agent creation
      runCLI('add-agent test-agent "First agent"');
      
      try {
        runCLI('add-agent test-agent "Duplicate agent"');
        throw new Error('Should have failed with duplicate agent');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('already exists');
      }

      // Verify original agent still exists
      const agents = runCLI('list-agents');
      expect(agents).toContain('test-agent');
      expect(agents).toContain('Found 1 agent(s)');

      // Test invalid agent operations
      try {
        runCLI('show-agent non-existent-agent');
        throw new Error('Should have failed with non-existent agent');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('not found');
      }

      try {
        runCLI('update-agent non-existent-agent "New prompt"');
        throw new Error('Should have failed with non-existent agent');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('not found');
      }

      // Test task creation with invalid agent
      try {
        runCLI('create-task invalid-agent "Task" "Description"');
        throw new Error('Should have failed with invalid agent');
      } catch (error: any) {
        expect(error.stdout || error.message).toContain('not found');
      }

      // Verify system state is still consistent
      const finalStatus = runCLI('status');
      expect(finalStatus).toContain('Active Agents: 0');
      
      const finalRegistry = runCLI('registry-status');
      expect(finalRegistry).toContain('Total Agents: 1');
    });
  });

  describe('Registry Versioning Workflow', () => {
    it('should demonstrate complete git versioning workflow', () => {
      // Initialize and create initial agents
      runCLI('init');
      runCLI('add-agent v1-agent "Version 1 agent"');

      // Check initial history
      let history = runCLI('registry-history');
      expect(history).toContain('Created agent: v1-agent');

      // Create backup point
      runCLI('registry-backup "Before major changes"');

      // Make several changes
      runCLI('add-agent v2-agent "Version 2 agent"');
      runCLI('update-agent v1-agent "Updated version 1 agent with new capabilities"');
      runCLI('add-agent v3-agent "Version 3 agent"');

      // Check updated history
      history = runCLI('registry-history');
      expect(history).toContain('Before major changes');
      expect(history).toContain('Created agent: v2-agent');
      expect(history).toContain('Updated agent: v1-agent');

      // Verify current state
      const agents = runCLI('list-agents');
      expect(agents).toContain('Found 3 agent(s)');

      const registryStatus = runCLI('registry-status');
      expect(registryStatus).toContain('Total Agents: 3');
      expect(registryStatus).toContain('Git Status:');
    });
  });

  describe('Configuration and System Management', () => {
    it('should demonstrate system configuration and management', () => {
      // Initialize system
      runCLI('init');

      // Check configuration
      const config = runCLI('config');
      expect(config).toContain('Nox Configuration');
      expect(config).toContain('Security:');
      expect(config).toContain('Claude CLI:');
      expect(config).toContain('Storage:');

      // Check system health
      const health = runCLI('health');
      expect(health).toContain('System Health Check');
      expect(health).toContain('System Status:');
      expect(health).toContain('Agent Status:');

      // Create agents and check status changes
      runCLI('add-agent status-agent "Status test agent"');

      const status = runCLI('status');
      expect(status).toContain('Active Agents: 0'); // Not started
      expect(status).toContain('Registry Version:');

      // Check registry status
      const registryStatus = runCLI('registry-status');
      expect(registryStatus).toContain('Total Agents: 1');
      expect(registryStatus).toContain('Inactive: 1');
    });
  });

  describe('Data Persistence and Integrity', () => {
    it('should maintain data integrity across operations', async () => {
      // Initialize and create test data
      runCLI('init');
      runCLI('add-agent persist-agent "Persistence test agent"');
      runCLI('create-task persist-agent "Persistence Task" "Test data persistence"');

      // Verify files were created
      expect(await fs.access('.nox-registry/agents.json').then(() => true).catch(() => false)).toBe(true);
      expect(await fs.access('.nox-registry/.git').then(() => true).catch(() => false)).toBe(true);

      // Read and verify registry contents
      const agentsData = await fs.readFile('.nox-registry/agents.json', 'utf-8');
      const agents = JSON.parse(agentsData);
      expect(agents).toHaveProperty('persist-agent');
      expect(agents['persist-agent']).toHaveProperty('systemPrompt', 'Persistence test agent');

      // Verify data persists across CLI calls
      const listOutput = runCLI('list-agents');
      expect(listOutput).toContain('persist-agent');

      const showOutput = runCLI('show-agent persist-agent');
      expect(showOutput).toContain('Persistence test agent');

      // Verify git history exists
      const historyOutput = runCLI('registry-history');
      expect(historyOutput).toContain('Created agent: persist-agent');
    });
  });
});