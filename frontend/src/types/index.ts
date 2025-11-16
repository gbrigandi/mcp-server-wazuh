/**
 * Type definitions for Sentinelle-MCP Frontend
 */

export interface User {
  id: string;
  username: string;
  email: string;
  role: 'admin' | 'client';
  groupId?: string;
}

export interface AuthResponse {
  token: string;
  user: User;
}

export interface PortalSummary {
  groupId: string;
  agentStatuses: {
    total: number;
    active: number;
    disconnected: number;
  };
  securityScore: {
    sca: number;
    vulnerabilities: number;
  };
  recentAlerts: Alert[];
  shieldStatus: 'protected' | 'warning' | 'critical';
}

export interface Alert {
  id: string;
  timestamp: string;
  severity: 'normal' | 'critical';
  translatedMessage: string;
}

export interface ChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
}

export interface ChatResponse {
  message: string;
  actions?: string[];
  timestamp: string;
}
