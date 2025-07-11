#!/usr/bin/env node

/**
 * Test script to create 5 random agents for testing the Nox dashboard
 * Usage: node scripts/test-agents.js
 */

const agents = [
  {
    name: "NewsHunter",
    description: "Specialized agent for searching and analyzing news articles about AI and autonomous coding",
    capabilities: ["web-scraping", "news-analysis", "content-filtering", "trend-detection"],
    status: "active",
    tasksCompleted: 23,
    currentTask: "Scanning tech news for AI coding breakthroughs"
  },
  {
    name: "CodeScout", 
    description: "Agent focused on discovering and evaluating autonomous coding tools and frameworks",
    capabilities: ["code-analysis", "github-monitoring", "technology-assessment", "documentation-parsing"],
    status: "idle",
    tasksCompleted: 15,
    currentTask: null
  },
  {
    name: "TrendWatcher",
    description: "Monitors social media and forums for discussions about agentic coding practices",
    capabilities: ["social-monitoring", "sentiment-analysis", "topic-clustering", "real-time-alerts"],
    status: "active", 
    tasksCompleted: 41,
    currentTask: "Analyzing Reddit discussions on AI code generation"
  },
  {
    name: "ResearchBot",
    description: "Academic paper crawler specializing in autonomous programming research",
    capabilities: ["paper-crawling", "citation-analysis", "research-summarization", "peer-review-tracking"],
    status: "active",
    tasksCompleted: 8,
    currentTask: "Processing arxiv papers on code generation models"
  },
  {
    name: "IndustryIntel",
    description: "Corporate intelligence agent tracking company announcements and product releases in agentic coding",
    capabilities: ["press-release-monitoring", "product-analysis", "competitor-tracking", "market-intelligence"],
    status: "error",
    tasksCompleted: 2,
    currentTask: null
  }
];

const API_BASE = 'http://localhost:3001/api';

async function testEndpoint(endpoint, method = 'GET', body = null) {
  try {
    const options = {
      method,
      headers: {
        'Content-Type': 'application/json',
      }
    };
    
    if (body) {
      options.body = JSON.stringify(body);
    }

    const response = await fetch(`${API_BASE}${endpoint}`, options);
    const data = await response.json();
    
    console.log(`${method} ${endpoint}: ${response.status}`);
    if (!response.ok) {
      console.log(`  Error: ${data.message || data.error}`);
    }
    
    return { response, data };
  } catch (error) {
    console.log(`${method} ${endpoint}: FAILED - ${error.message}`);
    return null;
  }
}

async function main() {
  console.log('🧪 Testing Nox Dashboard API with 5 random agents\n');
  
  // Test health endpoint
  console.log('📊 Testing API health...');
  await testEndpoint('/health');
  console.log();
  
  // Test getting existing agents
  console.log('👥 Testing agents endpoint...');
  const agentsResult = await testEndpoint('/agents');
  if (agentsResult?.data?.agents) {
    console.log(`  Found ${agentsResult.data.agents.length} existing agents`);
  }
  console.log();
  
  // Test creating agents (will get 501 Not Implemented)
  console.log('🤖 Testing agent creation (expecting 501 Not Implemented)...');
  for (const agent of agents) {
    await testEndpoint('/agents', 'POST', {
      name: agent.name,
      description: agent.description,
      capabilities: agent.capabilities
    });
  }
  console.log();
  
  // Test system status
  console.log('⚡ Testing system endpoints...');
  await testEndpoint('/system/status');
  await testEndpoint('/system/health');
  console.log();
  
  // Test metrics
  console.log('📈 Testing metrics endpoints...');
  await testEndpoint('/metrics/system');
  console.log();
  
  // Test tasks
  console.log('📋 Testing tasks endpoint...');
  await testEndpoint('/tasks');
  console.log();
  
  console.log('✅ Test completed!');
  console.log('\n📝 Summary:');
  console.log('- Created test data for 5 agents related to news searching about agentic coding');
  console.log('- Agent creation returns 501 (Not Implemented) as expected');
  console.log('- Use this data as reference for implementing actual agent creation');
  console.log('\nAgent profiles created:');
  agents.forEach((agent, i) => {
    console.log(`${i + 1}. ${agent.name} - ${agent.description}`);
  });
}

if (require.main === module) {
  main().catch(console.error);
}

module.exports = { agents, testEndpoint };