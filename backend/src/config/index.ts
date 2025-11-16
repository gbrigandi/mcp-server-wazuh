/**
 * Configuration centrale pour l'application Sentinelle-MCP
 */

import dotenv from 'dotenv';
import path from 'path';

dotenv.config();

export const config = {
  env: process.env.NODE_ENV || 'development',
  port: parseInt(process.env.PORT || '3001', 10),
  apiPrefix: process.env.API_PREFIX || '/api/v1',

  // Security
  jwt: {
    secret: process.env.JWT_SECRET || 'change-this-secret-key',
    expiresIn: process.env.JWT_EXPIRES_IN || '24h',
  },
  bcryptRounds: parseInt(process.env.BCRYPT_ROUNDS || '12', 10),

  // CORS
  cors: {
    origin: process.env.CORS_ORIGIN || 'http://localhost:3000',
    credentials: true,
  },

  // Rate Limiting
  rateLimit: {
    windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000', 10),
    maxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100', 10),
  },

  // Wazuh (via MCP)
  wazuh: {
    mcpServerPath: process.env.MCP_SERVER_PATH || path.join(__dirname, '../../../target/release/mcp-server-wazuh'),
    apiHost: process.env.WAZUH_API_HOST || 'localhost',
    apiPort: parseInt(process.env.WAZUH_API_PORT || '55000', 10),
    apiUsername: process.env.WAZUH_API_USERNAME || 'wazuh',
    apiPassword: process.env.WAZUH_API_PASSWORD || 'wazuh',
    indexerHost: process.env.WAZUH_INDEXER_HOST || 'localhost',
    indexerPort: parseInt(process.env.WAZUH_INDEXER_PORT || '9200', 10),
    indexerUsername: process.env.WAZUH_INDEXER_USERNAME || 'admin',
    indexerPassword: process.env.WAZUH_INDEXER_PASSWORD || 'admin',
    verifySSL: process.env.WAZUH_VERIFY_SSL === 'true',
    testProtocol: process.env.WAZUH_TEST_PROTOCOL || 'https',
  },

  // AI/LLM
  llm: {
    provider: process.env.LLM_PROVIDER || 'gemini',
    geminiApiKey: process.env.GEMINI_API_KEY,
    mistralApiKey: process.env.MISTRAL_API_KEY,
  },

  // Logging
  logging: {
    level: process.env.LOG_LEVEL || 'info',
    filePath: process.env.LOG_FILE_PATH || './logs/sentinelle.log',
  },

  // Admin
  admin: {
    username: process.env.ADMIN_USERNAME || 'admin',
    password: process.env.ADMIN_PASSWORD || 'change-this-password',
    email: process.env.ADMIN_EMAIL || 'admin@sentinelle.pro',
  },
};

export default config;
