/**
 * Authentication middleware
 */

import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import config from '../config';
import logger from '../utils/logger';
import { JWTPayload } from '../types';

// Extend Express Request type
declare global {
  namespace Express {
    interface Request {
      user?: JWTPayload;
    }
  }
}

/**
 * Verify JWT token and attach user to request
 */
export function authenticateToken(req: Request, res: Response, next: NextFunction): void {
  const authHeader = req.headers['authorization'];
  const token = authHeader && authHeader.split(' ')[1]; // Bearer TOKEN

  if (!token) {
    res.status(401).json({ error: 'Access token required' });
    return;
  }

  try {
    const payload = jwt.verify(token, config.jwt.secret) as JWTPayload;
    req.user = payload;
    next();
  } catch (error) {
    logger.warn('Invalid token:', error);
    res.status(403).json({ error: 'Invalid or expired token' });
  }
}

/**
 * Require admin role
 */
export function requireAdmin(req: Request, res: Response, next: NextFunction): void {
  if (!req.user || req.user.role !== 'admin') {
    logger.warn(`Unauthorized admin access attempt by user: ${req.user?.username}`);
    res.status(403).json({ error: 'Admin access required' });
    return;
  }
  next();
}

/**
 * Require client role
 */
export function requireClient(req: Request, res: Response, next: NextFunction): void {
  if (!req.user || req.user.role !== 'client') {
    res.status(403).json({ error: 'Client access required' });
    return;
  }
  next();
}

/**
 * Require client to access their own group only
 */
export function requireOwnGroup(req: Request, res: Response, next: NextFunction): void {
  const groupId = req.params.groupId || req.body.groupId;

  if (!req.user) {
    res.status(401).json({ error: 'Authentication required' });
    return;
  }

  // Admin can access any group
  if (req.user.role === 'admin') {
    next();
    return;
  }

  // Client can only access their own group
  if (req.user.role === 'client' && req.user.groupId !== groupId) {
    logger.warn(`User ${req.user.username} attempted to access group ${groupId} (owns ${req.user.groupId})`);
    res.status(403).json({ error: 'Access denied to this group' });
    return;
  }

  next();
}
