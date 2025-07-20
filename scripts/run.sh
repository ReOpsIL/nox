#!/bin/bash

# Nox Production Run Script
# This script runs the Nox server in production mode

set -e

echo "🚀 Starting Nox Agent Ecosystem"
echo "==============================="

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

# Get script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Change to script directory
#cd "$SCRIPT_DIR"


# Check if nox executable exists
#if [[ ! -f "nox" ]]; then
#    print_error "Nox executable not found! Please run build.sh first."
#    exit 1
#fi

# Check if frontend build exists
if [[ ! -d "frontend" ]]; then
    print_warning "Frontend build not found. Server will show fallback page."
fi

# Set default environment variables for production
export RUST_LOG="${RUST_LOG:-debug}"
export NOX_SERVER__HOST="${NOX_SERVER__HOST:-0.0.0.0}"
export NOX_SERVER__PORT="${NOX_SERVER__PORT:-8080}"

print_status "Environment Configuration:"
echo "  RUST_LOG: $RUST_LOG"
echo "  NOX_SERVER__HOST: $NOX_SERVER__HOST"
echo "  NOX_SERVER__PORT: $NOX_SERVER__PORT"

# Initialize if needed
if [[ ! -d ".nox-registry" ]]; then
    print_status "Initializing Nox ecosystem..."
    ./nox init
fi

# Function to handle graceful shutdown
graceful_shutdown() {
    print_status "Received shutdown signal..."
    if [[ -n "$NOX_PID" ]]; then
        print_status "Stopping Nox server (PID: $NOX_PID)..."
        kill -TERM "$NOX_PID" 2>/dev/null || true
        wait "$NOX_PID" 2>/dev/null || true
    fi
    print_success "Nox server stopped gracefully."
    exit 0
}

# Set up signal handlers
trap graceful_shutdown SIGTERM SIGINT

# Start the server
print_success "Starting Nox server..."
print_status "Access the web interface at: http://${NOX_SERVER__HOST:-localhost}:${NOX_SERVER__PORT:-8080}"
print_status "API documentation at: http://${NOX_SERVER__HOST:-localhost}:${NOX_SERVER__PORT:-8080}/api"
print_status "Health check at: http://${NOX_SERVER__HOST:-localhost}:${NOX_SERVER__PORT:-8080}/health"

echo ""
echo "Press Ctrl+C to stop the server"
echo "================================"

# Start the server in the background to allow signal handling
./nox serve &
NOX_PID=$!

# Wait for the server process
wait $NOX_PID
