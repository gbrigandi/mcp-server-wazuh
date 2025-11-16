/**
 * Client routes - Concierge interface (read-only, friendly)
 */

import { Router, Request, Response } from 'express';
import { body, validationResult } from 'express-validator';
import { authenticateToken, requireClient, requireOwnGroup } from '../middleware/auth';
import { getLLMService } from '../services/llmService';
import logger from '../utils/logger';
import { ClientChatRequest } from '../types';

const router = Router();

// All client routes require authentication and client role
router.use(authenticateToken);
router.use(requireClient);

/**
 * POST /api/v1/client/chat
 * Concierge chat endpoint - Reassuring, pedagogical responses
 */
router.post(
  '/chat',
  requireOwnGroup,
  [
    body('message').isString().trim().notEmpty().isLength({ max: 1000 }),
    body('groupId').isString().trim().notEmpty(),
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({ errors: errors.array() });
        return;
      }

      const { message, groupId }: ClientChatRequest = req.body;

      logger.info(`Client chat request from ${req.user?.username} (group ${groupId}): "${message}"`);

      const llmService = getLLMService();

      // Get friendly, reassuring response
      const aiResponse = await llmService.chatClient(message);

      res.json({
        message: aiResponse,
        timestamp: new Date().toISOString(),
      });
    } catch (error) {
      logger.error('Client chat error:', error);
      res.status(500).json({
        error: 'Désolé, je rencontre un problème technique. Veuillez réessayer dans quelques instants.',
      });
    }
  }
);

export default router;
