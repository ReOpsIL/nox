#!/bin/bash
set -e

# Nox Deployment Script
echo "🚀 Deploying Nox..."

# Check if target directory is provided
if [ -z "$1" ]; then
    echo "❌ Target directory not specified."
    echo "Usage: ./scripts/deploy.sh <target_directory>"
    exit 1
fi

TARGET_DIR="$1"

# Create target directory if it doesn't exist
if [ ! -d "$TARGET_DIR" ]; then
    echo "📁 Creating target directory: $TARGET_DIR"
    mkdir -p "$TARGET_DIR"
fi

# Check if git is installed
if command -v git &> /dev/null; then
    # Get current git commit hash
    GIT_COMMIT=$(git rev-parse --short HEAD)
    echo "📌 Deploying commit: $GIT_COMMIT"
else
    echo "⚠️ Git not found, skipping commit information"
    GIT_COMMIT="unknown"
fi

# Create a timestamp for the deployment
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
DEPLOY_ID="${TIMESTAMP}_${GIT_COMMIT}"
echo "🔖 Deployment ID: $DEPLOY_ID"

# Create a directory for this deployment
DEPLOY_DIR="${TARGET_DIR}/${DEPLOY_ID}"
mkdir -p "$DEPLOY_DIR"

# Copy necessary files
echo "📋 Copying files to deployment directory..."
cp -r package.json package-lock.json tsconfig.json "$DEPLOY_DIR/"
cp -r src "$DEPLOY_DIR/"
cp -r scripts "$DEPLOY_DIR/"

# Create necessary directories
mkdir -p "$DEPLOY_DIR/dist"

# Create a deployment info file
echo "Creating deployment info file..."
cat > "$DEPLOY_DIR/deployment.json" << EOF
{
  "id": "$DEPLOY_ID",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "git_commit": "$GIT_COMMIT",
  "deployed_by": "$(whoami)"
}
EOF

# Navigate to deployment directory
cd "$DEPLOY_DIR"

# Install dependencies
echo "📦 Installing dependencies..."
npm ci

# Build the application
echo "🔨 Building application..."
npm run build

# Create symbolic link for current deployment
echo "🔗 Creating symbolic link for current deployment..."
cd "$TARGET_DIR"
rm -f current
ln -s "$DEPLOY_ID" current

echo "✅ Deployment completed successfully!"
echo "📂 Deployed to: $DEPLOY_DIR"
echo "🚀 To start the application, run: cd $TARGET_DIR/current && ./scripts/run-prod.sh"