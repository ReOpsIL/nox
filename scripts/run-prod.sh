#!/bin/bash
set -e

# Nox Production Run Script
echo "🚀 Starting Nox in production mode..."

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18 or higher."
    exit 1
fi

# Check Node.js version
NODE_VERSION=$(node -v | cut -d 'v' -f 2 | cut -d '.' -f 1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo "❌ Node.js version 18 or higher is required. Current version: $(node -v)"
    exit 1
fi

# Check if PM2 is installed
if ! command -v pm2 &> /dev/null; then
    echo "❌ PM2 is not installed. Installing PM2..."
    npm install -g pm2
fi

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm ci
fi

# Build TypeScript if dist/nox.js doesn't exist or is older than source files
if [ ! -f "dist/nox.js" ] || [ -n "$(find src -type f -newer dist/nox.js)" ]; then
    echo "🔨 Building TypeScript..."
    npm run build || {
        echo "⚠️ TypeScript build failed, but continuing with existing build..."
    }
else
    echo "🔄 Using existing TypeScript build..."
fi

# Initialize Nox if not already initialized
if [ ! -d ".nox-registry" ]; then
    echo "🔧 Initializing Nox ecosystem..."
    npm run init
fi

# Start the application with PM2
echo "🌟 Starting Nox ecosystem in production mode..."
pm2 start dist/nox.js --name "nox" -- start --background

echo "✅ Production environment started"
echo "📊 Monitor with: pm2 monit"
echo "📋 View logs with: pm2 logs nox"
echo "⏹️  Stop with: pm2 stop nox"
