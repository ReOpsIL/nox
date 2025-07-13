/**
 * Dashboard Server - Provides web interface for monitoring and managing the Nox system
 * Serves the dashboard UI and API endpoints
 */

import express from 'express';
import * as http from 'http';
import * as path from 'path';
import cors from 'cors';
import helmet from 'helmet';
import { EventEmitter } from 'events';
import { NoxConfig } from '../types';
import { logger } from '../utils/logger';
import { AgentManager } from '../core/agent-manager';
import { TaskManager } from '../core/task-manager';
import { MessageBroker } from '../core/message-broker';
import { MetricsManager } from '../monitoring/metrics';
import { WebSocketServer } from '../server/websocket';
import { setupAgentRoutes } from './routes/agent-routes';
import { setupTaskRoutes } from './routes/task-routes';
import { setupMetricsRoutes } from './routes/metrics-routes';
import { setupSystemRoutes } from './routes/system-routes';
import { RegistryManager } from '../core/registry-manager';

/**
 * Dashboard Server - Provides web interface for monitoring and managing the Nox system
 */
export class DashboardServer extends EventEmitter {
  private app: express.Application;
  private server: http.Server | null = null;
  private initialized = false;
  private port: number;
  private frontendPath: string;

  constructor(
    private agentManager: AgentManager,
    private taskManager: TaskManager,
    private messageBroker: MessageBroker,
    private metricsManager: MetricsManager,
    private websocketServer: WebSocketServer,
    private registryManager: RegistryManager,
    _workingDir: string
  ) {
    super();
    this.app = express();
    this.port = 3001; // Default port (different from WebSocket server)
    this.frontendPath = path.join(process.cwd(), 'frontend', 'build');
  }

  /**
   * Initialize the dashboard server
   */
  async initialize(config: NoxConfig): Promise<void> {
    if (this.initialized) {
      logger.warn('DashboardServer already initialized');
      return;
    }

    try {
      // Set dashboard port from config if available, otherwise use default 3001
      if (config.server?.dashboardPort) {
        this.port = config.server.dashboardPort;
      }

      // Use default frontend path (already set in constructor)

      // Configure Express middleware
      this.configureMiddleware();

      // Set up API routes
      this.setupRoutes();

      // Set up static file serving for frontend
      this.setupStaticServing();

      this.initialized = true;
      logger.info('DashboardServer initialized');
      this.emit('initialized');

    } catch (error) {
      logger.error('Failed to initialize DashboardServer:', error);
      throw error;
    }
  }

  /**
   * Start the dashboard server
   */
  async start(): Promise<void> {
    if (!this.initialized) {
      throw new Error('DashboardServer not initialized');
    }

    return new Promise<void>((resolve, reject) => {
      try {
        this.server = this.app.listen(this.port, () => {
          logger.info(`DashboardServer started on port ${this.port}`);
          this.emit('started');
          resolve();
        });

        this.server.on('error', (error) => {
          logger.error('DashboardServer error:', error);
          this.emit('error', error);
          reject(error);
        });

      } catch (error) {
        logger.error('Failed to start DashboardServer:', error);
        reject(error);
      }
    });
  }

  /**
   * Shutdown the dashboard server
   */
  async shutdown(): Promise<void> {
    if (!this.server) {
      return;
    }

    return new Promise<void>((resolve, reject) => {
      this.server!.close((err) => {
        if (err) {
          logger.error('Error shutting down DashboardServer:', err);
          reject(err);
        } else {
          logger.info('DashboardServer shutdown');
          this.server = null;
          this.emit('shutdown');
          resolve();
        }
      });
    });
  }

  /**
   * Configure Express middleware
   */
  private configureMiddleware(): void {
    // Enable CORS
    this.app.use(cors());

    // Security middleware
    this.app.use(helmet({
      contentSecurityPolicy: {
        directives: {
          defaultSrc: ["'self'"],
          scriptSrc: ["'self'", "'unsafe-inline'", "'unsafe-eval'"],
          styleSrc: ["'self'", "'unsafe-inline'"],
          imgSrc: ["'self'", 'data:'],
          connectSrc: ["'self'", 'ws:', 'wss:']
        }
      }
    }));

    // Parse JSON request body
    this.app.use(express.json());

    // Parse URL-encoded request body
    this.app.use(express.urlencoded({ extended: true }));

    // Request logging
    this.app.use((req, _res, next) => {
      logger.debug(`${req.method} ${req.url}`);
      next();
    });

    // Error handling
    this.app.use((err: any, _req: express.Request, res: express.Response, _next: express.NextFunction) => {
      logger.error('Express error:', err);
      res.status(500).json({
        error: 'Internal Server Error',
        message: err.message
      });
    });
  }

  /**
   * Set up API routes
   */
  private setupRoutes(): void {
    // API routes
    const apiRouter = express.Router();
    this.app.use('/api', apiRouter);

    // Set up route handlers
    setupAgentRoutes(apiRouter, this.agentManager, this.registryManager, this.taskManager, this.websocketServer);
    setupTaskRoutes(apiRouter, this.taskManager, this.websocketServer);
    setupMetricsRoutes(apiRouter, this.metricsManager);
    setupSystemRoutes(apiRouter, this.agentManager, this.messageBroker, this.taskManager);

    // Health check endpoint
    apiRouter.get('/health', (_req: express.Request, res: express.Response) => {
      res.json({
        status: 'ok',
        uptime: process.uptime(),
        timestamp: new Date().toISOString()
      });
    });

    // WebSocket info endpoint
    apiRouter.get('/websocket-info', (req: express.Request, res: express.Response) => {
      res.json({
        url: `ws://${req.headers.host?.split(':')[0] || 'localhost'}:3000` // WebSocket server port
      });
    });
  }

  /**
   * Set up static file serving for frontend
   */
  private setupStaticServing(): void {
    // Check if frontend build directory exists
    const fs = require('fs');
    const frontendExists = fs.existsSync(this.frontendPath) && 
                          fs.existsSync(path.join(this.frontendPath, 'index.html'));

    if (frontendExists) {
      // Serve static files from frontend build directory
      this.app.use(express.static(this.frontendPath));
      logger.info(`Serving frontend from ${this.frontendPath}`);

      // Serve index.html for all routes not handled by API
      this.app.get('*', (req: express.Request, res: express.Response) => {
        // Skip API routes
        if (req.url.startsWith('/api/')) {
          return res.status(404).json({ error: 'Not Found' });
        }

        // Serve index.html for all other routes (SPA support)
        return res.sendFile(path.join(this.frontendPath, 'index.html'));
      });
    } else {
      logger.warn(`Frontend build not found at ${this.frontendPath}. Serving API-only mode.`);
      
      // Serve a simple message for non-API routes
      this.app.get('*', (req: express.Request, res: express.Response) => {
        // Skip API routes
        if (req.url.startsWith('/api/')) {
          return res.status(404).json({ error: 'Not Found' });
        }

        // Serve a simple HTML page indicating API-only mode
        return res.send(`
          <!DOCTYPE html>
          <html>
          <head>
            <title>NOX Dashboard - API Mode</title>
            <style>
              body { font-family: Arial, sans-serif; margin: 40px; background: #f5f5f5; }
              .container { background: white; padding: 30px; border-radius: 8px; max-width: 600px; margin: 0 auto; }
              h1 { color: #333; }
              .api-links { margin: 20px 0; }
              .api-links a { display: block; margin: 5px 0; color: #007acc; text-decoration: none; }
              .api-links a:hover { text-decoration: underline; }
              .status { color: #28a745; font-weight: bold; }
            </style>
          </head>
          <body>
            <div class="container">
              <h1>🚀 NOX Agent Ecosystem</h1>
              <p class="status">✅ Dashboard Server Running (API Mode)</p>
              <p>The NOX dashboard is running in API-only mode. The frontend UI build is not available.</p>
              
              <h3>📡 Available API Endpoints:</h3>
              <div class="api-links">
                <a href="/api/system/status">/api/system/status</a>
                <a href="/api/agents">/api/agents</a>
                <a href="/api/tasks">/api/tasks</a>
                <a href="/api/system/health">/api/system/health</a>
                <a href="/api/metrics/system">/api/metrics/system</a>
              </div>
              
              <h3>🔧 To Enable Full Dashboard:</h3>
              <p>Build the frontend by running:</p>
              <pre style="background: #f8f9fa; padding: 10px; border-radius: 4px;">
cd frontend && npm install && npm run build
              </pre>
              
              <p><em>Port 3001 • WebSocket on Port 3000</em></p>
            </div>
          </body>
          </html>
        `);
      });
    }
  }

  /**
   * Get the Express application instance
   */
  getApp(): express.Application {
    return this.app;
  }

  /**
   * Get the server port
   */
  getPort(): number {
    return this.port;
  }
}
