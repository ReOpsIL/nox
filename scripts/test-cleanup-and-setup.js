#!/usr/bin/env node

const fs = require('fs').promises;
const path = require('path');

/**
 * Testing script that:
 * 1. Deletes all agents and tasks
 * 2. Creates a single test agent
 * 3. Creates a web search task for Google news
 */

const workingDir = process.cwd();
const tasksDir = path.join(workingDir, 'tasks');
const metricsDir = path.join(workingDir, 'metrics');
const conversationsDir = path.join(workingDir, 'conversations');
const registryDir = path.join(workingDir, '.nox-registry');

async function cleanup() {
  console.log('🧹 Starting cleanup...');
  
  // Clean up directories
  const dirsToClean = [tasksDir, metricsDir, conversationsDir];
  
  for (const dir of dirsToClean) {
    try {
      await fs.access(dir);
      const files = await fs.readdir(dir);
      
      for (const file of files) {
        const filePath = path.join(dir, file);
        const stat = await fs.stat(filePath);
        
        if (stat.isDirectory()) {
          // Remove subdirectories
          await fs.rmdir(filePath, { recursive: true });
        } else {
          // Remove files
          await fs.unlink(filePath);
        }
      }
      
      console.log(`  ✅ Cleaned ${dir}`);
    } catch (error) {
      if (error.code !== 'ENOENT') {
        console.error(`  ❌ Error cleaning ${dir}:`, error.message);
      }
    }
  }
  
  // Clean up registry
  try {
    await fs.access(registryDir);
    
    // Reset registry files to empty objects/arrays
    const agentsJsonPath = path.join(registryDir, 'agents.json');
    const relationshipsJsonPath = path.join(registryDir, 'agent-relationships.json');
    const mcpServicesJsonPath = path.join(registryDir, 'mcp-services.json');
    
    await fs.writeFile(agentsJsonPath, '{}');
    await fs.writeFile(relationshipsJsonPath, '[]');
    await fs.writeFile(mcpServicesJsonPath, '{}');
    
    console.log(`  ✅ Cleaned registry files`);
  } catch (error) {
    if (error.code !== 'ENOENT') {
      console.error(`  ❌ Error cleaning registry:`, error.message);
    }
  }
  
  console.log('✅ Cleanup completed\n');
}

async function createTestAgent() {
  console.log('🤖 Creating test agent...');
  
  const testAgent = {
    id: 'test-agent-001',
    name: 'Test Web Search Agent',
    systemPrompt: 'You are a helpful AI agent that specializes in web searches and information gathering. When given a task to search for information, use web search capabilities to find the most current and relevant information.',
    status: 'active',
    createdAt: new Date(),
    lastModified: new Date(),
    resourceLimits: {
      maxMemoryMB: 1024,
      maxCPUPercent: 30,
      maxConcurrentTasks: 5,
      maxSpawnRate: 5,
      maxDepth: 3
    },
    capabilities: ['web_search', 'information_gathering', 'text_analysis'],
    relationships: [],
    mcpServices: ['web-search'],
    model: 'sonnet'
  };
  
  // Register agent in the NOX registry
  const agentsJsonPath = path.join(registryDir, 'agents.json');
  
  try {
    // Read existing agents
    let agents = {};
    try {
      const existingContent = await fs.readFile(agentsJsonPath, 'utf-8');
      agents = JSON.parse(existingContent);
    } catch (error) {
      // File doesn't exist or is empty, start with empty object
      agents = {};
    }
    
    // Add test agent
    agents[testAgent.id] = testAgent;
    
    // Write back to registry
    await fs.writeFile(agentsJsonPath, JSON.stringify(agents, null, 2));
    
    console.log(`  ✅ Registered agent in NOX registry: ${testAgent.id}`);
    
  } catch (error) {
    console.error(`  ❌ Failed to register agent in registry:`, error);
    throw error;
  }
  
  // Ensure metrics directory exists
  const agentsMetricsDir = path.join(metricsDir, 'agents');
  await fs.mkdir(agentsMetricsDir, { recursive: true });
  
  // Create agent metrics file
  const agentMetrics = {
    id: testAgent.id,
    uptime: 0,
    totalTasks: 0,
    completedTasks: 0,
    failedTasks: 0,
    averageResponseTime: 0,
    memoryUsage: 0,
    cpuUsage: 0,
    lastActivity: new Date()
  };
  
  const metricsPath = path.join(agentsMetricsDir, `${testAgent.id}.json`);
  await fs.writeFile(metricsPath, JSON.stringify(agentMetrics, null, 2));
  
  console.log(`  ✅ Created test agent: ${testAgent.id}`);
  console.log(`  📊 Agent metrics file: ${metricsPath}`);
  
  return testAgent;
}

async function createWebSearchTask(agentId) {
  console.log('📋 Creating web search task...');
  
  const task = {
    id: `task_${Date.now()}_websearch`,
    agentId: agentId,
    title: 'Search for latest Google news',
    description: 'Search the web for the latest news about Google company. Focus on recent developments, product launches, business news, or any significant announcements from the past week.',
    status: 'todo',
    priority: 'HIGH',
    createdAt: new Date(),
    updatedAt: new Date(),
    dependencies: [],
    requestedBy: 'test-script',
    progress: 0,
    metadata: {
      searchQuery: 'Google news latest developments',
      source: 'test-script',
      expectedDeliverable: 'Summary of latest Google news'
    }
  };
  
  // Ensure tasks directory exists
  await fs.mkdir(tasksDir, { recursive: true });
  
  // Create task markdown file
  const taskContent = `# Tasks for ${agentId}

## 🟢 In Progress

## 📋 Todo
- [ ] ${task.title} (Priority: ${task.priority})
  - Description: ${task.description}
  - Requested by: ${task.requestedBy}
  - Progress: ${task.progress}%

## ✅ Done

`;
  
  const taskFilePath = path.join(tasksDir, `${agentId}.md`);
  await fs.writeFile(taskFilePath, taskContent);
  
  console.log(`  ✅ Created web search task: ${task.id}`);
  console.log(`  📄 Task file: ${taskFilePath}`);
  console.log(`  🎯 Task: "${task.title}"`);
  
  return task;
}

async function createConversationFile(agentId) {
  console.log('💬 Creating conversation file...');
  
  const conversation = {
    agentId: agentId,
    messages: [
      {
        timestamp: new Date(),
        role: 'system',
        content: 'Agent initialized and ready for web search tasks.'
      },
      {
        timestamp: new Date(),
        role: 'user',
        content: 'Please search for the latest news about Google and provide a comprehensive summary.'
      }
    ],
    metadata: {
      sessionId: `session_${Date.now()}`,
      startTime: new Date(),
      source: 'test-script'
    }
  };
  
  // Ensure conversations directory exists
  await fs.mkdir(conversationsDir, { recursive: true });
  
  const conversationPath = path.join(conversationsDir, `${agentId}.json`);
  await fs.writeFile(conversationPath, JSON.stringify(conversation, null, 2));
  
  console.log(`  ✅ Created conversation file: ${conversationPath}`);
  
  return conversation;
}

async function main() {
  try {
    console.log('🚀 Starting test cleanup and setup script\n');
    
    // Step 1: Cleanup
    await cleanup();
    
    // Step 2: Create test agent
    const testAgent = await createTestAgent();
    
    // Step 3: Create web search task
    const task = await createWebSearchTask(testAgent.id);
    
    // Step 4: Create conversation file
    const conversation = await createConversationFile(testAgent.id);
    
    console.log('\n🎉 Test setup completed successfully!');
    console.log('\n📋 Summary:');
    console.log(`  🤖 Agent: ${testAgent.name} (${testAgent.id})`);
    console.log(`  📋 Task: ${task.title}`);
    console.log(`  🎯 Objective: Search for latest Google news`);
    console.log(`  📁 Files created:`);
    console.log(`    - ${path.join(tasksDir, testAgent.id + '.md')}`);
    console.log(`    - ${path.join(metricsDir, 'agents', testAgent.id + '.json')}`);
    console.log(`    - ${path.join(conversationsDir, testAgent.id + '.json')}`);
    
    console.log('\n🔧 Next steps:');
    console.log('  1. Start the NOX system');
    console.log('  2. The test agent should pick up the web search task');
    console.log('  3. Check the task file for progress updates');
    
  } catch (error) {
    console.error('❌ Error during setup:', error);
    process.exit(1);
  }
}

// Run the script
if (require.main === module) {
  main();
}

module.exports = { cleanup, createTestAgent, createWebSearchTask, createConversationFile };