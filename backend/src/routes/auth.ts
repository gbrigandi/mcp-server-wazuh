/**
 * Authentication routes
 */

import { Router, Request, Response } from 'express';
import jwt from 'jsonwebtoken';
import bcrypt from 'bcrypt';
import { body, validationResult } from 'express-validator';
import config from '../config';
import logger from '../utils/logger';
import { authRateLimiter } from '../middleware/security';
import { User, JWTPayload } from '../types';

const router = Router();

// In-memory user store (replace with database in production)
// TODO: Implement database for user management
const users: Map<string, User> = new Map();

// Initialize admin user
const initializeAdminUser = async () => {
  const adminId = 'admin-1';
  const hashedPassword = await bcrypt.hash(config.admin.password, config.bcryptRounds);

  users.set(config.admin.username, {
    id: adminId,
    username: config.admin.username,
    email: config.admin.email,
    role: 'admin',
    createdAt: new Date(),
    // Store hashed password separately (not in User type)
    ...(({ password: hashedPassword } as any)),
  });

  logger.info('Admin user initialized');
};

initializeAdminUser();

/**
 * POST /api/v1/auth/login
 * Login endpoint
 */
router.post(
  '/login',
  authRateLimiter,
  [
    body('username').isString().trim().notEmpty(),
    body('password').isString().notEmpty(),
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({ errors: errors.array() });
        return;
      }

      const { username, password } = req.body;

      const user = users.get(username) as any;
      if (!user) {
        logger.warn(`Login attempt with unknown username: ${username}`);
        res.status(401).json({ error: 'Invalid credentials' });
        return;
      }

      const isValidPassword = await bcrypt.compare(password, user.password);
      if (!isValidPassword) {
        logger.warn(`Failed login attempt for user: ${username}`);
        res.status(401).json({ error: 'Invalid credentials' });
        return;
      }

      const payload: JWTPayload = {
        userId: user.id,
        username: user.username,
        role: user.role,
        groupId: user.groupId,
      };

      const token = jwt.sign(payload, config.jwt.secret, {
        expiresIn: config.jwt.expiresIn,
      });

      // Update last login
      user.lastLogin = new Date();

      logger.info(`User logged in: ${username} (${user.role})`);

      res.json({
        token,
        user: {
          id: user.id,
          username: user.username,
          email: user.email,
          role: user.role,
          groupId: user.groupId,
        },
      });
    } catch (error) {
      logger.error('Login error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }
);

/**
 * POST /api/v1/auth/register-client
 * Register a new client (admin only - would be protected by auth middleware)
 */
router.post(
  '/register-client',
  [
    body('username').isString().trim().notEmpty(),
    body('password').isString().isLength({ min: 8 }),
    body('email').isEmail(),
    body('groupId').isString().trim().notEmpty(),
  ],
  async (req: Request, res: Response) => {
    try {
      const errors = validationResult(req);
      if (!errors.isEmpty()) {
        res.status(400).json({ errors: errors.array() });
        return;
      }

      const { username, password, email, groupId } = req.body;

      if (users.has(username)) {
        res.status(409).json({ error: 'Username already exists' });
        return;
      }

      const hashedPassword = await bcrypt.hash(password, config.bcryptRounds);
      const userId = `client-${Date.now()}`;

      const newUser: User & { password: string } = {
        id: userId,
        username,
        email,
        role: 'client',
        groupId,
        createdAt: new Date(),
        password: hashedPassword,
      };

      users.set(username, newUser as any);

      logger.info(`New client registered: ${username} (group: ${groupId})`);

      res.status(201).json({
        message: 'Client registered successfully',
        user: {
          id: userId,
          username,
          email,
          role: 'client',
          groupId,
        },
      });
    } catch (error) {
      logger.error('Registration error:', error);
      res.status(500).json({ error: 'Internal server error' });
    }
  }
);

export default router;
