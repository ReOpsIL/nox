#!/bin/bash

# Nox Development Script
# This script runs both frontend and backend in development mode

set -e

echo "🛠️  Starting Nox Development Environment"
echo "======================================="

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
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_ROOT"

# Check if we're in the right directory
if [[ ! -f "Cargo.toml" ]]; then
    print_error "Cargo.toml not found. Please run this script from the Nox project root."
    exit 1
fi

# PIDs for cleanup
BACKEND_PID=""
FRONTEND_PID=""

# Function to handle graceful shutdown
graceful_shutdown() {
    print_status "Received shutdown signal..."
    
    if [[ -n "$FRONTEND_PID" ]]; then
        print_status "Stopping frontend development server..."
        kill -TERM "$FRONTEND_PID" 2>/dev/null || true
        wait "$FRONTEND_PID" 2>/dev/null || true
    fi
    
    if [[ -n "$BACKEND_PID" ]]; then
        print_status "Stopping backend server..."
        kill -TERM "$BACKEND_PID" 2>/dev/null || true
        wait "$BACKEND_PID" 2>/dev/null || true
    fi
    
    print_success "Development environment stopped."
    exit 0
}

# Set up signal handlers
trap graceful_shutdown SIGTERM SIGINT

# Initialize if needed
if [[ ! -d ".nox-registry" ]]; then
    print_status "Initializing Nox ecosystem..."
    cargo run init
fi

# Start backend server
print_status "Starting Rust backend server on port 8080..."
cargo run serve --port 8080 &
BACKEND_PID=$!

# Wait a moment for backend to start
sleep 2

# Check if frontend directory exists
if [[ ! -d "frontend" ]]; then
    print_error "Frontend directory not found!"
    graceful_shutdown
fi

# Start frontend development server
print_status "Starting React frontend development server on port 5173..."
cd frontend

# Install dependencies if needed
if [[ ! -d "node_modules" ]]; then
    print_status "Installing frontend dependencies..."
    npm install
fi

# Start frontend dev server
npm run dev &
FRONTEND_PID=$!

cd "$PROJECT_ROOT"

# Print access information
print_success "Development environment started successfully!"
echo ""
echo "🌐 Access Points:"
echo "  Frontend (Dev): http://localhost:5173"
echo "  Backend API:    http://localhost:8080/api"
echo "  Health Check:   http://localhost:8080/health"
echo "  WebSocket:      ws://localhost:8080/ws"
echo ""
echo "📝 Development Notes:"
echo "  - Frontend has hot reload enabled"
echo "  - Backend will restart on code changes (use cargo-watch)"
echo "  - API calls from frontend proxy to backend"
echo ""
echo "Press Ctrl+C to stop both servers"
echo "=================================="

# Wait for both processes
wait $BACKEND_PID $FRONTEND_PID