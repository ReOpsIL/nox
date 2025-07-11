/**
 * Dashboard Server - Provides web interface for monitoring and managing the Nox system
 * Serves the dashboard UI and API endpoints
 */

import * as express from 'express';
import * as http from 'http';
import * as path from 'path';
import * as cors from 'cors';
import * as helmet from 'helmet';
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
    private workingDir: string
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
      // Set port from config if available
      if (config.dashboard?.port) {
        this.port = config.dashboard.port;
      }

      // Set frontend path from config if available
      if (config.dashboard?.frontendPath) {
        this.frontendPath = config.dashboard.frontendPath;
      }

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
    this.app.use((req, res, next) => {
      logger.debug(`${req.method} ${req.url}`);
      next();
    });

    // Error handling
    this.app.use((err: any, req: express.Request, res: express.Response, next: express.NextFunction) => {
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
    setupAgentRoutes(apiRouter, this.agentManager);
    setupTaskRoutes(apiRouter, this.taskManager);
    setupMetricsRoutes(apiRouter, this.metricsManager);
    setupSystemRoutes(apiRouter, this.agentManager, this.messageBroker, this.taskManager);

    // Health check endpoint
    apiRouter.get('/health', (req, res) => {
      res.json({
        status: 'ok',
        uptime: process.uptime(),
        timestamp: new Date().toISOString()
      });
    });

    // WebSocket info endpoint
    apiRouter.get('/websocket-info', (req, res) => {
      res.json({
        url: `ws://${req.headers.host?.split(':')[0] || 'localhost'}:${this.websocketServer.getPort()}`
      });
    });
  }

  /**
   * Set up static file serving for frontend
   */
  private setupStaticServing(): void {
    // Serve static files from frontend build directory
    this.app.use(express.static(this.frontendPath));

    // Serve index.html for all routes not handled by API
    this.app.get('*', (req, res) => {
      // Skip API routes
      if (req.url.startsWith('/api/')) {
        return res.status(404).json({ error: 'Not Found' });
      }

      // Serve index.html for all other routes (SPA support)
      res.sendFile(path.join(this.frontendPath, 'index.html'));
    });
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