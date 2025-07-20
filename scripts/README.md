# Nox Build and Deployment Scripts

This directory contains scripts for building, running, and deploying the Nox Agent Ecosystem.

## Scripts Overview

### 🛠️ `build.sh` - Production Build
Builds both frontend and backend for production deployment.

```bash
./scripts/build.sh
```

**What it does:**
- Builds React frontend (`npm run build`)
- Builds Rust backend in release mode (`cargo build --release`)
- Creates a `deploy/` directory with all necessary files
- Copies built files to deployment structure

**Output:**
- `deploy/nox` - Production executable
- `deploy/frontend/` - Built frontend static files
- `deploy/config/` - Configuration files
- `deploy/README.md` - Deployment instructions

### 🚀 `run.sh` - Production Server
Runs the Nox server in production mode.

```bash
./scripts/run.sh
```

**Features:**
- Graceful shutdown handling
- Environment variable configuration
- Automatic initialization
- Production-ready logging

**Environment Variables:**
```bash
export NOX_SERVER__HOST=0.0.0.0
export NOX_SERVER__PORT=8080
export RUST_LOG=info
./scripts/run.sh
```

### 🛠️ `dev.sh` - Development Environment
Runs both frontend and backend in development mode.

```bash
./scripts/dev.sh
```

**What it does:**
- Starts Rust backend on port 8080
- Starts React frontend dev server on port 5173
- Enables hot reload for frontend
- Handles graceful shutdown of both servers

**Access Points:**
- Frontend: http://localhost:5173
- Backend API: http://localhost:8080/api
- Health Check: http://localhost:8080/health

### 📦 `deploy.sh` - Production Deployment
Builds and deploys to a production server.

```bash
DEPLOY_HOST=your-server.com ./scripts/deploy.sh
```

**Environment Variables:**
- `DEPLOY_HOST` - Target server hostname (required)
- `DEPLOY_USER` - SSH user (default: ubuntu)
- `DEPLOY_PATH` - Deployment path (default: /opt/nox)
- `SYSTEMD_SERVICE` - Service name (default: nox)

**Example:**
```bash
DEPLOY_HOST=nox.example.com \
DEPLOY_USER=ubuntu \
DEPLOY_PATH=/opt/nox \
./scripts/deploy.sh
```

**What it does:**
- Runs production build
- Creates deployment package
- Uploads to server via SSH
- Creates systemd service
- Starts the service
- Verifies deployment

## Usage Examples

### Quick Start (Development)
```bash
# Start development environment
./scripts/dev.sh

# Access frontend at http://localhost:5173
# Access backend at http://localhost:8080
```

### Production Build and Run
```bash
# Build for production
./scripts/build.sh

# Run production server
cd deploy
./run.sh
```

### Production Deployment
```bash
# Deploy to production server
DEPLOY_HOST=your-server.com ./scripts/deploy.sh

# The application will be available at:
# http://your-server.com:8080
```

## Requirements

### Development
- Rust (latest stable)
- Node.js (18+)
- npm or yarn

### Production Deployment
- SSH access to target server
- sudo privileges on target server
- systemd (for service management)

## Configuration

### Backend Configuration
The backend is configured via `config/default.toml`:

```toml
[server]
port = 8080
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
```

### Environment Variables
Override configuration using environment variables:

```bash
export NOX_SERVER__PORT=3000
export NOX_SERVER__HOST=127.0.0.1
export NOX_STORAGE__REGISTRY_PATH=/data/nox-registry
```

## Deployment Architecture

### Development Mode
```
┌─────────────────┐    ┌─────────────────┐
│   Frontend      │    │   Backend       │
│   (Vite Dev)    │    │   (Cargo Run)   │
│   Port: 5173    │────│   Port: 8080    │
└─────────────────┘    └─────────────────┘
```

### Production Mode
```
┌─────────────────────────────────────────┐
│           Nox Server                    │
│                                         │
│  ┌─────────────┐  ┌─────────────────┐  │
│  │  Static     │  │  API Server     │  │
│  │  Files      │  │  /api/*         │  │
│  │  (React)    │  │  /health        │  │
│  └─────────────┘  └─────────────────┘  │
│                                         │
│           Port: 8080                    │
└─────────────────────────────────────────┘
```

## Troubleshooting

### Common Issues

1. **Frontend build fails:**
   ```bash
   cd frontend
   npm install
   npm run build
   ```

2. **Backend build fails:**
   ```bash
   cargo clean
   cargo build --release
   ```

3. **Port already in use:**
   ```bash
   # Use different port
   NOX_SERVER__PORT=9090 ./scripts/run.sh
   ```

4. **Permission denied on deployment:**
   ```bash
   # Ensure SSH key is set up
   ssh-copy-id ubuntu@your-server.com
   ```

### Logs

- **Development:** Check terminal output
- **Production:** `journalctl -u nox -f`
- **Build logs:** Check script output

## Security Considerations

### Production Deployment
- Use reverse proxy (nginx/Apache) for HTTPS
- Configure firewall rules
- Use non-root user for service
- Set up monitoring and alerting
- Regular security updates

### Configuration
- Use environment variables for secrets
- Restrict CORS origins in production
- Enable authentication/authorization
- Use secure storage for registry data

## Support

For issues and questions:
- Check the logs: `journalctl -u nox -f`
- Review configuration: `cat config/default.toml`
- Test health endpoint: `curl http://localhost:8080/health`

Built with ❤️ by the Nox Team