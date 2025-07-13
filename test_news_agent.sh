#!/bin/bash

# Test script for Nox news search agent
# This script creates a news search agent and assigns it 3 tasks

set -e  # Exit on any error

echo "🚀 Starting Nox News Agent Test Script"
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Build the project
print_status "Building Nox project..."
cargo build --release
if [ $? -eq 0 ]; then
    print_success "Build completed successfully"
else
    print_error "Build failed"
    exit 1
fi

# Initialize the system (if not already done)
print_status "Initializing Nox system..."
./target/release/nox init 2>/dev/null || true

# Check if Claude CLI is available
print_status "Checking Claude CLI availability..."
if command -v claude &> /dev/null; then
    print_success "Claude CLI found: $(claude --version)"
else
    print_error "Claude CLI not found. Please install Claude CLI first."
    echo "Visit: https://docs.anthropic.com/claude/reference/claude-cli"
    exit 1
fi

# Create the news search agent
print_status "Creating News Search Agent..."
AGENT_OUTPUT=$(./target/release/nox agent add "News Search Agent" "You are a specialized news search and analysis agent. Your role is to search for recent news articles, analyze their content, and provide concise summaries with key insights. You should focus on accuracy, relevance, and providing actionable information. When given a topic, search for the most recent and credible news sources, then provide a structured summary with headlines, key points, and implications.")

if [ $? -eq 0 ]; then
    print_success "News Search Agent created successfully"
    echo "$AGENT_OUTPUT"
else
    print_error "Failed to create agent"
    exit 1
fi

# Get the agent ID from the list (we'll need to parse this)
print_status "Getting agent information..."
AGENT_LIST=$(./target/release/nox agent list)
echo "$AGENT_LIST"

# Extract agent ID (this is a simple approach, assumes the agent was just created)
AGENT_ID=$(echo "$AGENT_LIST" | grep -A1 "News Search Agent" | grep "ID:" | awk '{print $2}' | head -1)

if [ -z "$AGENT_ID" ]; then
    print_error "Could not find agent ID. Trying to parse differently..."
    # Alternative parsing method
    AGENT_ID=$(echo "$AGENT_LIST" | grep "News Search Agent" | sed -n 's/.*ID: \([a-f0-9\-]*\).*/\1/p' | head -1)
fi

if [ -z "$AGENT_ID" ]; then
    print_error "Could not determine agent ID. Please check agent list manually:"
    ./target/release/nox agent list
    print_warning "Please run: ./target/release/nox agent show 'News Search Agent' to get the ID"
    exit 1
fi

print_success "Found Agent ID: $AGENT_ID"

# Start the agent
print_status "Starting the News Search Agent..."
./target/release/nox agent start "News Search Agent"
if [ $? -eq 0 ]; then
    print_success "Agent started successfully"
else
    print_error "Failed to start agent"
    exit 1
fi

# Wait a moment for the agent to fully start
sleep 2

# Create 3 test tasks
print_status "Creating test tasks..."

# Task 1: Technology News
print_status "Creating Task 1: Technology News Search..."
TASK1_OUTPUT=$(./target/release/nox task create "$AGENT_ID" "Technology News Search" "Search for the latest technology news from the past 24 hours. Focus on developments in AI, machine learning, and software engineering. Provide a summary of the top 3 most significant stories with their implications for the tech industry.")

if [ $? -eq 0 ]; then
    print_success "Task 1 created"
    echo "$TASK1_OUTPUT"
else
    print_error "Failed to create Task 1"
fi

# Task 2: Climate News
print_status "Creating Task 2: Climate and Environment News..."
TASK2_OUTPUT=$(./target/release/nox task create "$AGENT_ID" "Climate News Analysis" "Find recent climate and environmental news from credible sources. Look for policy changes, scientific breakthroughs, and major environmental events. Summarize the most impactful stories and their potential long-term effects.")

if [ $? -eq 0 ]; then
    print_success "Task 2 created"
    echo "$TASK2_OUTPUT"
else
    print_error "Failed to create Task 2"
fi

# Task 3: Business and Economy
print_status "Creating Task 3: Business and Economy News..."
TASK3_OUTPUT=$(./target/release/nox task create "$AGENT_ID" "Business Market Analysis" "Research the latest business and economic news focusing on market trends, major corporate announcements, and economic indicators. Provide insights on how these developments might affect different market sectors.")

if [ $? -eq 0 ]; then
    print_success "Task 3 created"
    echo "$TASK3_OUTPUT"
else
    print_error "Failed to create Task 3"
fi

# Show all tasks
print_status "Showing all created tasks..."
./target/release/nox task list

# Check system status
print_status "Checking system status..."
./target/release/nox status

# Test Claude CLI integration by simulating task execution
print_status "Testing Claude CLI integration..."
print_warning "Note: The following will test Claude CLI directly to verify integration"

# Create a simple test message to Claude
TEST_MESSAGE="Hello, can you confirm you're working correctly? Please respond with a brief acknowledgment."

echo ""
print_status "Testing direct Claude CLI call..."
echo "Command: claude --print --model claude-sonnet-4-20250514 --output-format text \"$TEST_MESSAGE\""
echo ""

# Test Claude CLI directly
claude --print --model claude-sonnet-4-20250514 --output-format text "$TEST_MESSAGE"

if [ $? -eq 0 ]; then
    print_success "Claude CLI integration test passed!"
else
    print_error "Claude CLI integration test failed!"
fi

echo ""
print_status "System check completed!"
print_status "Checking final system health..."
./target/release/nox health

echo ""
print_success "🎉 News Agent Test Script Completed Successfully!"
echo ""
print_status "Next steps:"
echo "1. Check task status with: ./target/release/nox task list"
echo "2. Start tasks with: ./target/release/nox task update TASK_ID --status in_progress"
echo "3. Monitor system with: ./target/release/nox status"
echo "4. Stop the agent when done with: ./target/release/nox agent stop 'News Search Agent'"
echo ""
print_warning "Note: Actual task execution with Claude responses would require implementing"
print_warning "the task processing logic in the system. This script verified the setup."