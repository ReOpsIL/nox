#!/bin/bash

# NOX Full System Test Runner
# Creates agents and assigns comprehensive tasks to validate the entire ecosystem

echo "🚀 NOX Full System Test - Agent Creation & Task Assignment"
echo "========================================================="

# Check if NOX is running
echo "🔍 Checking if NOX is running..."
if ! curl -s http://localhost:3001/api/health > /dev/null 2>&1; then
    echo "❌ NOX is not running. Please start it first:"
    echo "   ./scripts/run-dev.sh"
    exit 1
fi

echo "✅ NOX is running and healthy!"

# Step 1: Create test agents (if they don't exist)
echo ""
echo "👥 Step 1: Creating test agents..."
node scripts/test-agents.js

# Wait a moment for agents to be fully registered
echo "⏳ Waiting for agents to be registered..."
sleep 3

# Step 2: Create comprehensive tasks for all agents
echo ""
echo "📋 Step 2: Creating tasks for all agents..."
node scripts/test-agent-tasks.js

# Step 3: Display results
echo ""
echo "📊 Step 3: Displaying system status..."
echo ""

# Show agent list
echo "🤖 Current Agents:"
npx ts-node src/nox.ts list-agents

echo ""
echo "📋 Current Tasks:"
npx ts-node src/nox.ts list-tasks

echo ""
echo "🏥 System Health:"
npx ts-node src/nox.ts health

echo ""
echo "📈 Task Overview:"
npx ts-node src/nox.ts task-overview

echo ""
echo "🎉 FULL SYSTEM TEST COMPLETE!"
echo ""
echo "🌐 Dashboard: http://localhost:3001"
echo "📡 WebSocket: ws://localhost:3000"
echo ""
echo "💡 Try these commands to monitor the system:"
echo "   npx ts-node src/nox.ts status"
echo "   npx ts-node src/nox.ts list-tasks"
echo "   npx ts-node src/nox.ts task-overview"
echo ""
echo "🚀 NOX is LIVE and KICKING!"