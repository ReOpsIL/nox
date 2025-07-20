#!/bin/bash

# Nox Production Deployment Script
# This script builds and deploys Nox to a production server

set -e

echo "📦 Nox Production Deployment"
echo "==========================="

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

# Configuration
DEPLOY_USER="${DEPLOY_USER:-ubuntu}"
DEPLOY_HOST="${DEPLOY_HOST:-}"
DEPLOY_PATH="${DEPLOY_PATH:-/opt/nox}"
SYSTEMD_SERVICE="${SYSTEMD_SERVICE:-nox}"

# Check if deployment host is provided
if [[ -z "$DEPLOY_HOST" ]]; then
    print_error "DEPLOY_HOST environment variable is required"
    echo "Usage: DEPLOY_HOST=your-server.com ./scripts/deploy.sh"
    echo "Optional: DEPLOY_USER=ubuntu DEPLOY_PATH=/opt/nox ./scripts/deploy.sh"
    exit 1
fi

print_status "Deployment Configuration:"
echo "  Host: $DEPLOY_HOST"
echo "  User: $DEPLOY_USER"
echo "  Path: $DEPLOY_PATH"
echo "  Service: $SYSTEMD_SERVICE"

# Step 1: Build for production
print_status "Step 1: Building for production..."
./scripts/build.sh

# Step 2: Create deployment package
print_status "Step 2: Creating deployment package..."
cd deploy
tar -czf nox-deployment.tar.gz *
cd ..

# Step 3: Upload to server
print_status "Step 3: Uploading to server..."
scp deploy/nox-deployment.tar.gz "$DEPLOY_USER@$DEPLOY_HOST:/tmp/"

# Step 4: Deploy on server
print_status "Step 4: Deploying on server..."
ssh "$DEPLOY_USER@$DEPLOY_HOST" << EOF
set -e

# Create deployment directory
sudo mkdir -p $DEPLOY_PATH
sudo chown $DEPLOY_USER:$DEPLOY_USER $DEPLOY_PATH

# Extract deployment package
cd $DEPLOY_PATH
tar -xzf /tmp/nox-deployment.tar.gz
rm /tmp/nox-deployment.tar.gz

# Make executable
chmod +x nox run.sh

# Create systemd service if it doesn't exist
if [[ ! -f /etc/systemd/system/$SYSTEMD_SERVICE.service ]]; then
    sudo tee /etc/systemd/system/$SYSTEMD_SERVICE.service > /dev/null << 'EOL'
[Unit]
Description=Nox Agent Ecosystem
After=network.target

[Service]
Type=simple
User=$DEPLOY_USER
WorkingDirectory=$DEPLOY_PATH
ExecStart=$DEPLOY_PATH/nox serve
Restart=always
RestartSec=10
Environment=RUST_LOG=info
Environment=NOX_SERVER__HOST=0.0.0.0
Environment=NOX_SERVER__PORT=8080

[Install]
WantedBy=multi-user.target
EOL
    
    sudo systemctl daemon-reload
    sudo systemctl enable $SYSTEMD_SERVICE
    echo "Created systemd service: $SYSTEMD_SERVICE"
fi

# Restart service
sudo systemctl restart $SYSTEMD_SERVICE

# Check status
sudo systemctl status $SYSTEMD_SERVICE --no-pager
EOF

# Step 5: Verify deployment
print_status "Step 5: Verifying deployment..."
sleep 5

# Check if service is running
if ssh "$DEPLOY_USER@$DEPLOY_HOST" "sudo systemctl is-active $SYSTEMD_SERVICE" | grep -q "active"; then
    print_success "Deployment successful!"
    print_status "Service is running on: http://$DEPLOY_HOST:8080"
else
    print_error "Deployment failed - service is not running"
    exit 1
fi

# Cleanup
rm -f deploy/nox-deployment.tar.gz

print_success "🚀 Deployment completed successfully!"
echo ""
echo "🌐 Access your application:"
echo "  Web Interface: http://$DEPLOY_HOST:8080"
echo "  API Endpoints: http://$DEPLOY_HOST:8080/api"
echo "  Health Check:  http://$DEPLOY_HOST:8080/health"
echo ""
echo "🛠️  Management commands:"
echo "  Check status:  ssh $DEPLOY_USER@$DEPLOY_HOST 'sudo systemctl status $SYSTEMD_SERVICE'"
echo "  View logs:     ssh $DEPLOY_USER@$DEPLOY_HOST 'sudo journalctl -u $SYSTEMD_SERVICE -f'"
echo "  Restart:       ssh $DEPLOY_USER@$DEPLOY_HOST 'sudo systemctl restart $SYSTEMD_SERVICE'"