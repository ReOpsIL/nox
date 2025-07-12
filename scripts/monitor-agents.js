#!/usr/bin/env node

/**
 * NOX Agent Activity Monitor
 * Real-time monitoring of agent status and task execution
 * Usage: node scripts/monitor-agents.js
 */

const API_BASE = 'http://localhost:3001/api';

async function fetchData(endpoint) {
  try {
    const response = await fetch(`${API_BASE}${endpoint}`);
    if (response.ok) {
      return await response.json();
    }
    return null;
  } catch (error) {
    return null;
  }
}

function formatUptime(seconds) {
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  const secs = Math.floor(seconds % 60);
  return `${hours}h ${minutes}m ${secs}s`;
}

function getStatusEmoji(status) {
  const statusEmojis = {
    'active': '🟢',
    'inactive': '⚪',
    'error': '🔴',
    'starting': '🟡',
    'stopping': '🟠',
    'idle': '🔵'
  };
  return statusEmojis[status] || '❓';
}

function getPriorityEmoji(priority) {
  const priorityEmojis = {
    'critical': '🔥',
    'high': '🔴',
    'medium': '🟡',
    'low': '🟢'
  };
  return priorityEmojis[priority] || '⚪';
}

async function displaySystemStatus() {
  console.clear();
  console.log('🚀 NOX Agent Ecosystem - Live Monitor');
  console.log('=====================================');
  console.log(`🕐 ${new Date().toLocaleTimeString()}\n`);

  // System health
  const health = await fetchData('/system/health');
  if (health) {
    console.log('🏥 System Health:');
    console.log(`   Status: ${health.status === 'ok' ? '✅ Healthy' : '❌ Issues'}`);
    console.log(`   Uptime: ${formatUptime(health.uptime || 0)}`);
    console.log('');
  }

  // Agent status
  const agentsResponse = await fetchData('/agents');
  if (agentsResponse && agentsResponse.agents) {
    const agents = agentsResponse.agents;
    console.log(`🤖 Agents (${agents.length}):`);
    console.log('┌─────────────────┬─────────┬──────────┬─────────┬────────────┐');
    console.log('│ Name            │ Status  │ Tasks    │ CPU %   │ Memory MB  │');
    console.log('├─────────────────┼─────────┼──────────┼─────────┼────────────┤');
    
    agents.forEach(agent => {
      const statusEmoji = getStatusEmoji(agent.status);
      const name = agent.name.padEnd(15).substring(0, 15);
      const status = `${statusEmoji} ${agent.status}`.padEnd(7);
      const tasks = `${agent.tasksCompleted || 0}`.padStart(8);
      const cpu = `${(agent.cpuUsage || 0).toFixed(1)}`.padStart(7);
      const memory = `${(agent.memoryUsage || 0).toFixed(1)}`.padStart(10);
      
      console.log(`│ ${name} │ ${status} │ ${tasks} │ ${cpu} │ ${memory} │`);
    });
    console.log('└─────────────────┴─────────┴──────────┴─────────┴────────────┘\n');
  }

  // Task status
  const tasksResponse = await fetchData('/tasks');
  if (tasksResponse && tasksResponse.tasks) {
    const tasks = tasksResponse.tasks;
    const taskStats = {
      total: tasks.length,
      pending: tasks.filter(t => t.status === 'pending').length,
      running: tasks.filter(t => t.status === 'running').length,
      completed: tasks.filter(t => t.status === 'completed').length,
      failed: tasks.filter(t => t.status === 'failed').length
    };

    console.log(`📋 Tasks Overview (${taskStats.total} total):`);
    console.log(`   ⏳ Pending: ${taskStats.pending}`);
    console.log(`   🏃 Running: ${taskStats.running}`);
    console.log(`   ✅ Completed: ${taskStats.completed}`);
    console.log(`   ❌ Failed: ${taskStats.failed}\n`);

    // Show recent tasks
    const recentTasks = tasks.slice(0, 5);
    if (recentTasks.length > 0) {
      console.log('📋 Recent Tasks:');
      recentTasks.forEach(task => {
        const priorityEmoji = getPriorityEmoji(task.priority);
        const statusEmoji = getStatusEmoji(task.status);
        console.log(`   ${priorityEmoji} ${statusEmoji} ${task.title}`);
        console.log(`      Agent: ${task.agentId || 'Unassigned'} | ${task.description?.substring(0, 60)}...`);
      });
      console.log('');
    }
  }

  // System metrics
  const metrics = await fetchData('/metrics/system');
  if (metrics) {
    console.log('📊 System Metrics:');
    console.log(`   🖥️  CPU Usage: ${(metrics.cpu || 0).toFixed(1)}%`);
    console.log(`   💾 Memory Usage: ${(metrics.memory || 0).toFixed(1)}%`);
    console.log(`   📡 API Requests: ${metrics.requests || 0}`);
    console.log('');
  }

  console.log('🔄 Auto-refreshing every 5 seconds... (Ctrl+C to exit)');
}

async function startMonitoring() {
  console.log('🚀 Starting NOX Agent Monitor...\n');

  // Check if NOX is running
  const health = await fetchData('/system/health');
  if (!health) {
    console.log('❌ Cannot connect to NOX. Make sure it\'s running:');
    console.log('   ./scripts/run-dev.sh\n');
    process.exit(1);
  }

  console.log('✅ Connected to NOX successfully!\n');

  // Display initial status
  await displaySystemStatus();

  // Set up auto-refresh
  const refreshInterval = setInterval(async () => {
    await displaySystemStatus();
  }, 5000);

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    console.log('\n👋 Stopping monitor...');
    clearInterval(refreshInterval);
    process.exit(0);
  });
}

// Command line options
if (require.main === module) {
  const args = process.argv.slice(2);
  
  if (args.includes('--help') || args.includes('-h')) {
    console.log('NOX Agent Activity Monitor');
    console.log('Usage: node scripts/monitor-agents.js [options]');
    console.log('');
    console.log('Options:');
    console.log('  --help, -h     Show this help message');
    console.log('  --once         Display status once and exit');
    console.log('');
    console.log('The monitor shows real-time status of:');
    console.log('  • System health and uptime');
    console.log('  • Agent status and resource usage');
    console.log('  • Task queue and execution status');
    console.log('  • System performance metrics');
    process.exit(0);
  }
  
  if (args.includes('--once')) {
    displaySystemStatus().then(() => process.exit(0));
  } else {
    startMonitoring().catch(console.error);
  }
}

module.exports = { displaySystemStatus, startMonitoring };