/**
 * Admin routes - J.A.R.V.I.S. interface
 */

import { Router, Request, Response } from 'express';
import { body, validationResult } from 'express-validator';
import { authenticateToken, requireAdmin } from '../middleware/auth';
import { getMCPClient } from '../services/mcpClient';
import { getLLMService } from '../services/llmService';
import logger from '../utils/logger';
import { AdminChatRequest } from '../types';

const router = Router();

// All admin routes require authentication and admin role
router.use(authenticateToken);
router.use(requireAdmin);

/**
 * POST /api/v1/admin/chat
 * J.A.R.V.I.S. chat endpoint - Natural language commands
 */
router.post(
  '/chat',
  [
    body('message').isString().trim().notEmpty().isLength({ max: 2000 }),
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({ errors: errors.array() });
        return;
      }

      const { message, context = {} }: AdminChatRequest = req.body;

      logger.info(`Admin chat request from ${req.user?.username}: "${message}"`);

      const llmService = getLLMService();

      // Get AI response
      const aiResponse = await llmService.chatAdmin(message);

      // Extract potential actions
      const actions = await llmService.extractActionsFromAdminMessage(message);

      res.json({
        message: aiResponse,
        actions,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      logger.error('Admin chat error:', error);
      res.status(500).json({ error: 'Failed to process chat request' });
    }
  }
);

/**
 * GET /api/v1/admin/alerts
 * Get recent alerts (with admin-level details)
 */
router.get('/alerts', async (req: Request, res: Response) => {
  try {
    const limit = parseInt(req.query.limit as string) || 50;

    const mcpClient = getMCPClient();
    const response = await mcpClient.getAlertSummary(limit);

    if (response.isError) {
      res.status(500).json({ error: 'Failed to fetch alerts' });
      return;
    }

    res.json({
      alerts: response.content,
      count: response.content.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error fetching admin alerts:', error);
    res.status(500).json({ error: 'Failed to fetch alerts' });
  }
});

/**
 * GET /api/v1/admin/agents
 * Get all agents with detailed status
 */
router.get('/agents', async (req: Request, res: Response) => {
  try {
    const status = (req.query.status as string) || 'active';
    const limit = parseInt(req.query.limit as string) || 100;

    const mcpClient = getMCPClient();
    const response = await mcpClient.getAgents(status, limit);

    if (response.isError) {
      res.status(500).json({ error: 'Failed to fetch agents' });
      return;
    }

    res.json({
      agents: response.content,
      count: response.content.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error fetching agents:', error);
    res.status(500).json({ error: 'Failed to fetch agents' });
  }
});

/**
 * GET /api/v1/admin/cluster/health
 * Get Wazuh cluster health
 */
router.get('/cluster/health', async (req: Request, res: Response) => {
  try {
    const mcpClient = getMCPClient();
    const response = await mcpClient.getClusterHealth();

    if (response.isError) {
      res.status(500).json({ error: 'Failed to fetch cluster health' });
      return;
    }

    res.json({
      health: response.content,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error fetching cluster health:', error);
    res.status(500).json({ error: 'Failed to fetch cluster health' });
  }
});

/**
 * GET /api/v1/admin/agent/:agentId/vulnerabilities
 * Get vulnerabilities for a specific agent
 */
router.get('/agent/:agentId/vulnerabilities', async (req: Request, res: Response) => {
  try {
    const { agentId } = req.params;
    const limit = parseInt(req.query.limit as string) || 100;

    const mcpClient = getMCPClient();
    const response = await mcpClient.getVulnerabilitySummary(agentId, limit);

    if (response.isError) {
      res.status(500).json({ error: 'Failed to fetch vulnerabilities' });
      return;
    }

    res.json({
      agentId,
      vulnerabilities: response.content,
      count: response.content.length,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error fetching vulnerabilities:', error);
    res.status(500).json({ error: 'Failed to fetch vulnerabilities' });
  }
});

/**
 * POST /api/v1/admin/mcp/tool
 * Execute any MCP tool directly (advanced)
 */
router.post(
  '/mcp/tool',
  [
    body('name').isString().trim().notEmpty(),
    body('arguments').isObject(),
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({ errors: errors.array() });
        return;
      }

      const { name, arguments: args } = req.body;

      logger.info(`Admin executing MCP tool: ${name} by ${req.user?.username}`);

      const mcpClient = getMCPClient();
      const response = await mcpClient.callTool({ name, arguments: args });

      res.json({
        result: response,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      logger.error('Error executing MCP tool:', error);
      res.status(500).json({ error: 'Failed to execute MCP tool' });
    }
  }
);

export default router;
