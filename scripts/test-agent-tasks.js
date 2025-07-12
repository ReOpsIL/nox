#!/usr/bin/env node

/**
 * NOX Agent Task Testing Suite
 * Creates comprehensive tasks for all test agents to validate system functionality
 * Usage: node scripts/test-agent-tasks.js
 */

const { agents, testEndpoint } = require('./test-agents.js');

const API_BASE = 'http://localhost:3001/api';

// Task templates for each agent based on their capabilities
const taskTemplates = {
  "NewsHunter": [
    {
      title: "AI Coding News Sweep",
      description: "Search for and analyze the latest news articles about AI coding tools, autonomous programming, and code generation. Focus on major tech companies' announcements and breakthrough research.",
      priority: "high",
      estimatedTime: "45 minutes"
    },
    {
      title: "Weekly Tech Trend Analysis", 
      description: "Compile a weekly summary of trending topics in AI development tools. Track sentiment and impact of new announcements.",
      priority: "medium",
      estimatedTime: "30 minutes"
    },
    {
      title: "Competitor Intelligence Report",
      description: "Monitor competitor releases in the AI coding space. Track GitHub Copilot, Cursor, Replit Agent, and other autonomous coding tools.",
      priority: "medium", 
      estimatedTime: "60 minutes"
    }
  ],
  "CodeScout": [
    {
      title: "GitHub Repository Discovery",
      description: "Scan GitHub for trending repositories related to autonomous coding, AI agents, and code generation tools. Evaluate their potential and document findings.",
      priority: "high",
      estimatedTime: "40 minutes"
    },
    {
      title: "Technology Stack Assessment",
      description: "Analyze and evaluate new coding frameworks and tools. Focus on their integration potential with existing agent ecosystems.",
      priority: "medium",
      estimatedTime: "35 minutes"
    },
    {
      title: "Documentation Quality Review",
      description: "Review documentation quality of discovered tools and create improvement recommendations for our own documentation.",
      priority: "low",
      estimatedTime: "25 minutes"
    }
  ],
  "TrendWatcher": [
    {
      title: "Social Media Sentiment Monitoring",
      description: "Monitor Twitter, Reddit, and HackerNews for discussions about agentic coding. Track sentiment and emerging concerns or excitement.",
      priority: "high",
      estimatedTime: "50 minutes"
    },
    {
      title: "Developer Community Analysis", 
      description: "Analyze conversations in developer communities about AI coding tools. Identify pain points and feature requests.",
      priority: "medium",
      estimatedTime: "40 minutes"
    },
    {
      title: "Real-time Alert Setup",
      description: "Configure monitoring for specific keywords and topics related to autonomous coding breakthroughs.",
      priority: "medium",
      estimatedTime: "30 minutes"
    }
  ],
  "ResearchBot": [
    {
      title: "ArXiv Paper Analysis", 
      description: "Crawl and analyze recent papers on arXiv related to code generation, program synthesis, and autonomous programming agents.",
      priority: "high",
      estimatedTime: "60 minutes"
    },
    {
      title: "Citation Network Mapping",
      description: "Create a citation network map of influential papers in the autonomous coding research space.",
      priority: "medium",
      estimatedTime: "45 minutes"
    },
    {
      title: "Research Trend Identification",
      description: "Identify emerging research trends and methodologies in AI-assisted programming from academic sources.",
      priority: "medium",
      estimatedTime: "40 minutes"
    }
  ],
  "IndustryIntel": [
    {
      title: "Corporate Announcement Tracking",
      description: "Monitor press releases and announcements from major tech companies regarding AI coding tools and autonomous development platforms.",
      priority: "high", 
      estimatedTime: "35 minutes"
    },
    {
      title: "Product Launch Analysis",
      description: "Analyze recent product launches in the AI coding space. Assess market positioning and competitive advantages.",
      priority: "medium",
      estimatedTime: "40 minutes"
    },
    {
      title: "Market Intelligence Report",
      description: "Compile intelligence report on market trends, funding rounds, and strategic partnerships in autonomous coding sector.",
      priority: "low",
      estimatedTime: "50 minutes"
    }
  ]
};

// Additional stress test tasks
const stressTestTasks = [
  {
    title: "Cross-Agent Collaboration Test",
    description: "Task that requires collaboration between NewsHunter and TrendWatcher to correlate news events with social sentiment.",
    priority: "critical",
    estimatedTime: "90 minutes"
  },
  {
    title: "High-Volume Data Processing",
    description: "Process large dataset of GitHub repositories and academic papers simultaneously.",
    priority: "low",
    estimatedTime: "120 minutes"
  },
  {
    title: "Real-time Monitoring Challenge",
    description: "Set up real-time monitoring across multiple data sources with immediate alert processing.",
    priority: "high",
    estimatedTime: "75 minutes"
  }
];

async function getAgentList() {
  console.log('🔍 Fetching current agent list from NOX...');
  const result = await testEndpoint('/agents');
  if (result && result.data && result.data.agents) {
    return result.data.agents;
  }
  return [];
}

async function createTaskForAgent(agentId, taskTemplate) {
  console.log(`📋 Creating task "${taskTemplate.title}" for agent ${agentId}`);
  
  const result = await testEndpoint('/tasks', 'POST', {
    agentId: agentId,
    title: taskTemplate.title,
    description: taskTemplate.description,
    priority: taskTemplate.priority || 'medium',
    estimatedTime: taskTemplate.estimatedTime || '30 minutes',
    metadata: {
      category: 'automated-test',
      createdBy: 'test-agent-tasks.js',
      timestamp: new Date().toISOString()
    }
  });
  
  return result;
}

async function runAgentTaskTests() {
  console.log('🚀 Starting NOX Agent Task Testing Suite\n');
  
  try {
    // Get current agents
    const currentAgents = await getAgentList();
    if (currentAgents.length === 0) {
      console.log('❌ No agents found in NOX. Run test-agents.js first to create agents.');
      return;
    }
    
    console.log(`✅ Found ${currentAgents.length} agents in NOX:\n`);
    currentAgents.forEach((agent, i) => {
      console.log(`${i + 1}. ${agent.name} (${agent.id}) - Status: ${agent.status}`);
    });
    console.log();
    
    // Create a map of agent names to IDs
    const agentMap = {};
    currentAgents.forEach(agent => {
      agentMap[agent.name] = agent.id;
    });
    
    let totalTasksCreated = 0;
    let successfulTasks = 0;
    let failedTasks = 0;
    
    // Create tasks for each agent based on their specialization
    console.log('📋 Creating specialized tasks for each agent...\n');
    
    for (const [agentName, tasks] of Object.entries(taskTemplates)) {
      const agentId = agentMap[agentName];
      
      if (!agentId) {
        console.log(`⚠️  Agent ${agentName} not found in current agent list, skipping...`);
        continue;
      }
      
      console.log(`🤖 Creating tasks for ${agentName} (${agentId}):`);
      
      for (const task of tasks) {
        totalTasksCreated++;
        const result = await createTaskForAgent(agentId, task);
        
        if (result && result.response && result.response.ok) {
          successfulTasks++;
          console.log(`  ✅ "${task.title}" - Created successfully`);
        } else {
          failedTasks++;
          console.log(`  ❌ "${task.title}" - Failed to create`);
          if (result && result.data) {
            console.log(`     Error: ${result.data.message || result.data.error}`);
          }
        }
        
        // Small delay to avoid overwhelming the system
        await new Promise(resolve => setTimeout(resolve, 500));
      }
      console.log();
    }
    
    // Create stress test tasks for random agents
    console.log('🔥 Creating stress test tasks...\n');
    
    for (const stressTask of stressTestTasks) {
      // Assign to a random agent
      const randomAgent = currentAgents[Math.floor(Math.random() * currentAgents.length)];
      totalTasksCreated++;
      
      const result = await createTaskForAgent(randomAgent.id, stressTask);
      
      if (result && result.response && result.response.ok) {
        successfulTasks++;
        console.log(`✅ Stress test "${stressTask.title}" assigned to ${randomAgent.name}`);
      } else {
        failedTasks++;
        console.log(`❌ Stress test "${stressTask.title}" failed`);
      }
      
      await new Promise(resolve => setTimeout(resolve, 1000));
    }
    
    // Test system monitoring endpoints
    console.log('\n📊 Testing system monitoring endpoints...\n');
    
    await testEndpoint('/system/status');
    await testEndpoint('/system/health');
    await testEndpoint('/tasks');
    await testEndpoint('/metrics/system');
    
    // Summary
    console.log('\n📈 Task Creation Summary:');
    console.log(`Total tasks attempted: ${totalTasksCreated}`);
    console.log(`Successful: ${successfulTasks}`);
    console.log(`Failed: ${failedTasks}`);
    console.log(`Success rate: ${((successfulTasks / totalTasksCreated) * 100).toFixed(1)}%\n`);
    
    // Verification steps
    console.log('🔍 Verification Steps:\n');
    console.log('1. Check dashboard: http://localhost:3001');
    console.log('2. View tasks: npx ts-node src/nox.ts list-tasks');
    console.log('3. Monitor agents: npx ts-node src/nox.ts list-agents');
    console.log('4. System status: npx ts-node src/nox.ts status');
    console.log('5. Task overview: npx ts-node src/nox.ts task-overview\n');
    
    if (successfulTasks > 0) {
      console.log('🎉 NOX is LIVE and KICKING! Agent ecosystem is operational.');
      console.log(`${successfulTasks} tasks successfully created and assigned to agents.`);
      console.log('Check the dashboard to see agents in action! 🚀');
    } else {
      console.log('⚠️  No tasks were successfully created. Check system status and logs.');
    }
    
  } catch (error) {
    console.error('❌ Test suite failed:', error.message);
    console.log('\nTroubleshooting:');
    console.log('1. Ensure NOX is running: ./scripts/run-dev.sh');
    console.log('2. Check if agents exist: npx ts-node src/nox.ts list-agents');
    console.log('3. Verify API is responding: curl http://localhost:3001/api/health');
  }
}

// Interactive mode functions
async function showTaskMenu() {
  console.log('\n🎛️  NOX Agent Task Testing Menu:\n');
  console.log('1. Run full task creation suite');
  console.log('2. Create tasks for specific agent');
  console.log('3. Create single stress test task');
  console.log('4. View current tasks');
  console.log('5. Test system endpoints');
  console.log('6. Exit\n');
}

async function interactiveMode() {
  const readline = require('readline');
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout
  });
  
  while (true) {
    await showTaskMenu();
    
    const choice = await new Promise(resolve => {
      rl.question('Choose an option (1-6): ', resolve);
    });
    
    switch (choice.trim()) {
      case '1':
        await runAgentTaskTests();
        break;
      case '2':
        console.log('Available agents:');
        const agents = await getAgentList();
        agents.forEach((agent, i) => console.log(`${i + 1}. ${agent.name}`));
        break;
      case '3':
        const randomStressTask = stressTestTasks[Math.floor(Math.random() * stressTestTasks.length)];
        const randomAgent = await getAgentList();
        if (randomAgent.length > 0) {
          await createTaskForAgent(randomAgent[0].id, randomStressTask);
        }
        break;
      case '4':
        await testEndpoint('/tasks');
        break;
      case '5':
        await testEndpoint('/system/status');
        await testEndpoint('/system/health');
        break;
      case '6':
        console.log('👋 Goodbye!');
        rl.close();
        return;
      default:
        console.log('❌ Invalid option. Please choose 1-6.');
    }
    
    console.log('\nPress Enter to continue...');
    await new Promise(resolve => {
      rl.question('', resolve);
    });
  }
}

// Main execution
if (require.main === module) {
  const args = process.argv.slice(2);
  
  if (args.includes('--interactive') || args.includes('-i')) {
    interactiveMode().catch(console.error);
  } else {
    runAgentTaskTests().catch(console.error);
  }
}

module.exports = { 
  runAgentTaskTests, 
  createTaskForAgent, 
  taskTemplates, 
  stressTestTasks 
};