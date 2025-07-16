#!/bin/bash

# Nox Production Build Script
# This script builds both the frontend and backend for production deployment

set -e  # Exit on any error

echo "🏗️  Building Nox Agent Ecosystem for Production"
echo "=============================================="

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

# Step 1: Build Frontend
print_status "Step 1: Building React frontend..."
echo "-----------------------------------"

if [[ ! -d "frontend" ]]; then
    print_error "Frontend directory not found!"
    exit 1
fi

cd frontend

# Check if package.json exists
if [[ ! -f "package.json" ]]; then
    print_error "package.json not found in frontend directory!"
    exit 1
fi

# Install dependencies if node_modules doesn't exist
if [[ ! -d "node_modules" ]]; then
    print_status "Installing frontend dependencies..."
    npm install
fi

# Build frontend
print_status "Building frontend for production..."
npm run build

# Check if build was successful
if [[ ! -d "dist" ]]; then
    print_error "Frontend build failed - dist directory not found!"
    exit 1
fi

print_success "Frontend build completed successfully!"

# Step 2: Build Rust Backend
print_status "Step 2: Building Rust backend..."
echo "-----------------------------------"

cd "$PROJECT_ROOT"

# Build Rust backend in release mode
print_status "Building Rust backend in release mode..."
cargo build --release

# Check if build was successful
if [[ ! -f "target/release/nox" ]]; then
    print_error "Backend build failed - executable not found!"
    exit 1
fi

print_success "Backend build completed successfully!"

# Step 3: Create deployment structure
print_status "Step 3: Creating deployment structure..."
echo "-------------------------------------------"

# Create deployment directory
DEPLOY_DIR="$PROJECT_ROOT/deploy"
mkdir -p "$DEPLOY_DIR"

# Copy backend executable
cp "target/release/nox" "$DEPLOY_DIR/"

# Copy frontend build
rm -rf "$DEPLOY_DIR/frontend"
cp -r "frontend/dist" "$DEPLOY_DIR/frontend"

# Copy configuration template
mkdir -p "$DEPLOY_DIR/config"
if [[ -f "config/default.toml" ]]; then
    cp "config/default.toml" "$DEPLOY_DIR/config/"
else
    # Create default config if it doesn't exist
    cat > "$DEPLOY_DIR/config/default.toml" << EOF
[server]
port = 8080
frontend_port = 5173
host = "0.0.0.0"
websocket_enabled = true
api_enabled = true
cors_origins = ["*"]

[storage]
registry_path = ".nox-registry"

[claude_cli]
session_timeout = 3600
auto_restart_on_crash = true

[logging]
level = "info"
format = "json"
EOF
fi

# Copy run script
cp "$SCRIPT_DIR/run.sh" "$DEPLOY_DIR/" 2>/dev/null || true

# Create README for deployment
cat > "$DEPLOY_DIR/README.md" << 'EOF'
# Nox Agent Ecosystem - Production Deployment

This directory contains the production build of the Nox Agent Ecosystem.

## Files

- `nox` - Main executable (Rust backend)
- `frontend/` - Built React frontend static files
- `config/default.toml` - Default configuration
- `run.sh` - Production run script

## Running

1. **Simple run:**
   ```bash
   ./nox serve
   ```

2. **Using run script:**
   ```bash
   ./run.sh
   ```

3. **Custom configuration:**
   ```bash
   # Edit config/default.toml first
   ./nox serve --port 3000
   ```

## Environment Variables

You can override configuration using environment variables:

```bash
export NOX_SERVER__PORT=3000
export NOX_SERVER__HOST=0.0.0.0
./nox serve
```

## Production Checklist

- [ ] Configure firewall for port 8080
- [ ] Set up reverse proxy (nginx/Apache) if needed
- [ ] Configure TLS/SSL certificates
- [ ] Set up monitoring and logging
- [ ] Create systemd service file
- [ ] Set up backup strategy for `.nox-registry`

## Access

- **Web Interface:** http://localhost:8080
- **API Endpoints:** http://localhost:8080/api/*
- **Health Check:** http://localhost:8080/health
- **WebSocket:** ws://localhost:8080/ws

Built with ❤️ by the Nox Team
EOF

# Display file sizes
print_status "Deployment Summary:"
echo "-------------------"
echo "Backend executable: $(ls -lh "$DEPLOY_DIR/nox" | awk '{print $5}')"
echo "Frontend size: $(du -sh "$DEPLOY_DIR/frontend" | awk '{print $1}')"
echo "Total deployment size: $(du -sh "$DEPLOY_DIR" | awk '{print $1}')"

print_success "Production build completed successfully!"
print_status "Deployment ready in: $DEPLOY_DIR"

echo ""
echo "🚀 Next Steps:"
echo "1. Test the build: cd deploy && ./nox serve"
echo "2. Access the app: http://localhost:8080"
echo "3. Deploy to production server"