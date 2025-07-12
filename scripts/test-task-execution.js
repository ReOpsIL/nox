#!/usr/bin/env node

/**
 * Test Task Execution - Verifies that tasks are properly executed by agents
 * Usage: node scripts/test-task-execution.js
 */

const API_BASE = 'http://localhost:3001/api';

async function fetchData(endpoint, method = 'GET', body = null) {
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

async function testTaskExecution() {
  console.log('🧪 Testing NOX Task Execution Flow\n');
  
  // Step 1: Get current agents
  console.log('📋 Step 1: Getting current agents...');
  const agentsResult = await fetchData('/agents');
  if (!agentsResult || !agentsResult.data.agents || agentsResult.data.agents.length === 0) {
    console.log('❌ No agents found. Run ./scripts/run-full-test.sh first');
    return;
  }
  
  const agents = agentsResult.data.agents;
  console.log(`✅ Found ${agents.length} agents`);
  
  // Step 2: Get current tasks
  console.log('\n📋 Step 2: Getting current tasks...');
  const tasksResult = await fetchData('/tasks');
  if (!tasksResult || !tasksResult.data.tasks || tasksResult.data.tasks.length === 0) {
    console.log('❌ No tasks found. Run ./scripts/run-full-test.sh first');
    return;
  }
  
  const tasks = tasksResult.data.tasks;
  console.log(`✅ Found ${tasks.length} tasks`);
  
  // Step 3: Show task status
  console.log('\n📊 Step 3: Current task status...');
  const statusCounts = {};
  tasks.forEach(task => {
    statusCounts[task.status] = (statusCounts[task.status] || 0) + 1;
  });
  
  console.log('Task Status Distribution:');
  Object.entries(statusCounts).forEach(([status, count]) => {
    console.log(`  ${status}: ${count}`);
  });
  
  // Step 4: Test automatic task processing
  console.log('\n🔄 Step 4: Testing automatic task processing...');
  console.log('Tasks should automatically start processing...');
  
  // Wait a few seconds for the task processor to run
  console.log('⏳ Waiting 10 seconds for task processor...');
  await new Promise(resolve => setTimeout(resolve, 10000));
  
  // Check task status again
  const updatedTasksResult = await fetchData('/tasks');
  if (updatedTasksResult && updatedTasksResult.data.tasks) {
    const updatedTasks = updatedTasksResult.data.tasks;
    const updatedStatusCounts = {};
    updatedTasks.forEach(task => {
      updatedStatusCounts[task.status] = (updatedStatusCounts[task.status] || 0) + 1;
    });
    
    console.log('\nUpdated Task Status Distribution:');
    Object.entries(updatedStatusCounts).forEach(([status, count]) => {
      console.log(`  ${status}: ${count}`);
    });
    
    // Check if any tasks started
    const inProgressTasks = updatedTasks.filter(t => t.status === 'inprogress');
    if (inProgressTasks.length > 0) {
      console.log(`\n✅ SUCCESS: ${inProgressTasks.length} tasks are now in progress!`);
      console.log('Tasks being executed:');
      inProgressTasks.forEach(task => {
        console.log(`  - ${task.title} (Agent: ${task.agentId})`);
      });
    } else {
      console.log('\n⚠️  No tasks started automatically. Testing manual execution...');
      
      // Step 5: Manually start a task
      const todoTasks = updatedTasks.filter(t => t.status === 'todo');
      if (todoTasks.length > 0) {
        const testTask = todoTasks[0];
        console.log(`\n🚀 Step 5: Manually starting task: ${testTask.title}`);
        
        const startResult = await fetchData(`/tasks/${testTask.id}/start`, 'POST');
        if (startResult && startResult.response.ok) {
          console.log('✅ Task started successfully');
          
          // Execute the task
          console.log('📤 Sending task to agent...');
          const executeResult = await fetchData(`/tasks/${testTask.id}/execute`, 'POST');
          if (executeResult && executeResult.response.ok) {
            console.log('✅ Task sent to agent for execution');
          } else {
            console.log('❌ Failed to send task to agent');
          }
        } else {
          console.log('❌ Failed to start task manually');
        }
      }
    }
  }
  
  // Step 6: Final status check
  console.log('\n📊 Step 6: Final system status...');
  await fetchData('/system/status');
  
  console.log('\n🎉 Task execution test completed!');
  console.log('\n💡 Next steps:');
  console.log('1. Check the dashboard: http://localhost:3001');
  console.log('2. Monitor live: node scripts/monitor-agents.js');
  console.log('3. Check logs for Claude CLI activity');
  console.log('4. Run: npx ts-node src/nox.ts list-tasks');
}

if (require.main === module) {
  testTaskExecution().catch(console.error);
}

module.exports = { testTaskExecution };