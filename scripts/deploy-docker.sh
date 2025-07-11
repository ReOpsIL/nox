#!/bin/bash
set -e

# Nox Docker Deployment Script
echo "🐳 Deploying Nox with Docker..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Create Dockerfile if it doesn't exist
if [ ! -f "Dockerfile" ]; then
    echo "📝 Creating Dockerfile..."
    cat > Dockerfile << 'EOF'
FROM node:18-alpine

WORKDIR /app

# Install PM2 globally
RUN npm install -g pm2

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY . .

# Build TypeScript
RUN npm run build

# Expose ports
EXPOSE 3000

# Set environment variables
ENV NODE_ENV=production

# Start the application with PM2
CMD ["pm2-runtime", "dist/nox.js", "--", "start"]
EOF
fi

# Create docker-compose.yml if it doesn't exist
if [ ! -f "docker-compose.yml" ]; then
    echo "📝 Creating docker-compose.yml..."
    cat > docker-compose.yml << 'EOF'
version: '3.8'

services:
  nox:
    build: .
    container_name: nox
    restart: unless-stopped
    ports:
      - "3000:3000"
    volumes:
      - nox-data:/app/.nox-registry
    environment:
      - NODE_ENV=production

volumes:
  nox-data:
EOF
fi

# Build and start Docker containers
echo "🔨 Building Docker image..."
docker-compose build

echo "🚀 Starting Docker containers..."
docker-compose up -d

echo "✅ Docker deployment completed successfully!"
echo "📊 View logs with: docker-compose logs -f"
echo "⏹️  Stop with: docker-compose down"
echo "🌐 Access dashboard at: http://localhost:3000"