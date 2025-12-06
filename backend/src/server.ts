/**
 * Sentinelle-MCP Backend Server
 * Main entry point
 */

import express, { Request, Response, NextFunction } from 'express';
import config from './config';
import logger from './utils/logger';
import {
  helmetMiddleware,
  corsMiddleware,
  rateLimiter,
  sanitizeInput,
  requestLogger,
} from './middleware/security';
import authRoutes from './routes/auth';
import adminRoutes from './routes/admin';
import clientRoutes from './routes/client';
import portalRoutes from './routes/portal';
import { getMCPClient } from './services/mcpClient';

const app = express();

// Security middleware
app.use(helmetMiddleware);
app.use(corsMiddleware);
app.use(rateLimiter);

// Body parsing
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Input sanitization
app.use(sanitizeInput);

// Request logging
app.use(requestLogger);

// Health check
app.get('/health', (req: Request, res: Response) => {
  res.json({
    status: 'healthy',
    timestamp: new Date().toISOString(),
    service: 'sentinelle-mcp-backend',
    version: '1.0.0',
  });
});

// API routes
const apiPrefix = config.apiPrefix;
app.use(`${apiPrefix}/auth`, authRoutes);
app.use(`${apiPrefix}/admin`, adminRoutes);
app.use(`${apiPrefix}/client`, clientRoutes);
app.use(`${apiPrefix}/portal`, portalRoutes);

// 404 handler
app.use((req: Request, res: Response) => {
  res.status(404).json({ error: 'Endpoint not found' });
});

// Error handler
app.use((err: Error, req: Request, res: Response, next: NextFunction) => {
  logger.error('Unhandled error:', err);
  res.status(500).json({
    error: config.env === 'production' ? 'Internal server error' : err.message,
  });
});

// Initialize MCP client on startup
let mcpClient: any;
try {
  mcpClient = getMCPClient();
  logger.info('MCP Client initialized successfully');
} catch (error) {
  logger.error('Failed to initialize MCP Client:', error);
  logger.warn('Server will start but Wazuh functionality will not be available');
}

// Start server
const server = app.listen(config.port, () => {
  logger.info(`
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║   🛡️  SENTINELLE-MCP Backend Server                      ║
║                                                           ║
║   Environment: ${config.env.padEnd(43)}║
║   Port:        ${config.port.toString().padEnd(43)}║
║   API Prefix:  ${config.apiPrefix.padEnd(43)}║
║                                                           ║
║   Health:      http://localhost:${config.port}/health${' '.repeat(20)}║
║   API Docs:    ${apiPrefix.padEnd(43)}║
║                                                           ║
║   Status:      ✅ Ready                                   ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
  `);

  logger.info('Security features enabled:');
  logger.info('  ✓ Helmet (security headers)');
  logger.info('  ✓ CORS protection');
  logger.info('  ✓ Rate limiting');
  logger.info('  ✓ Input sanitization');
  logger.info('  ✓ JWT authentication');
  logger.info('  ✓ Request logging');

  logger.info(`Server ready to accept connections on port ${config.port}`);
});

// Graceful shutdown
const shutdown = (signal: string) => {
  logger.info(`${signal} received. Starting graceful shutdown...`);

  server.close(() => {
    logger.info('HTTP server closed');

    // Cleanup MCP client
    if (mcpClient) {
      try {
        mcpClient.shutdown();
        logger.info('MCP client shut down');
      } catch (error) {
        logger.error('Error shutting down MCP client:', error);
      }
    }

    logger.info('Graceful shutdown completed');
    process.exit(0);
  });

  // Force shutdown after 10 seconds
  setTimeout(() => {
    logger.error('Forced shutdown after timeout');
    process.exit(1);
  }, 10000);
};

process.on('SIGTERM', () => shutdown('SIGTERM'));
process.on('SIGINT', () => shutdown('SIGINT'));

// Handle unhandled rejections
process.on('unhandledRejection', (reason: any, promise: Promise<any>) => {
  logger.error('Unhandled Rejection at:', promise, 'reason:', reason);
});

// Handle uncaught exceptions
process.on('uncaughtException', (error: Error) => {
  logger.error('Uncaught Exception:', error);
  shutdown('UNCAUGHT_EXCEPTION');
});

export default app;
