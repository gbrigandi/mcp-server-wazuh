/**
 * Type definitions for Sentinelle-MCP Backend
 */

export interface User {
  id: string;
  username: string;
  email: string;
  role: 'admin' | 'client';
  groupId?: string;
  createdAt: Date;
  lastLogin?: Date;
}

export interface JWTPayload {
  userId: string;
  username: string;
  role: 'admin' | 'client';
  groupId?: string;
}

export interface WazuhAlert {
  id: string;
  timestamp: string;
  agent: string;
  level: number;
  description: string;
  translatedDescription?: string;
}

export interface AgentStatus {
  id: string;
  name: string;
  ip: string;
  status: 'active' | 'disconnected' | 'pending' | 'never_connected';
  os: string;
  lastKeepAlive?: string;
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
  recentAlerts: Array<{
    id: string;
    timestamp: string;
    severity: string;
    translatedMessage: string;
  }>;
  shieldStatus: 'protected' | 'warning' | 'critical';
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: Date;
}

export interface AdminChatRequest {
  message: string;
  context?: Record<string, unknown>;
}

export interface ClientChatRequest {
  message: string;
  groupId: string;
}

export interface ChatResponse {
  message: string;
  actions?: string[];
  data?: Record<string, unknown>;
}

export interface MCPToolCall {
  name: string;
  arguments: Record<string, unknown>;
}

export interface MCPResponse {
  content: Array<{
    type: string;
    text: string;
  }>;
  isError: boolean;
}
