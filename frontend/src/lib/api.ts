/**
 * API client utilities
 */

import axios, { AxiosInstance, AxiosError } from 'axios';
import { AuthResponse, PortalSummary, ChatResponse } from '@/types';

const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';
const API_PREFIX = process.env.NEXT_PUBLIC_API_PREFIX || '/api/v1';

class ApiClient {
  private client: AxiosInstance;

  constructor() {
    this.client = axios.create({
      baseURL: `${API_URL}${API_PREFIX}`,
      headers: {
        'Content-Type': 'application/json',
      },
    });

    // Add auth token to requests
    this.client.interceptors.request.use((config) => {
      if (typeof window !== 'undefined') {
        const token = localStorage.getItem('sentinelle_token');
        if (token) {
          config.headers.Authorization = `Bearer ${token}`;
        }
      }
      return config;
    });

    // Handle 401 responses
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        if (error.response?.status === 401 && typeof window !== 'undefined') {
          localStorage.removeItem('sentinelle_token');
          localStorage.removeItem('sentinelle_user');
          window.location.href = '/login';
        }
        return Promise.reject(error);
      }
    );
  }

  // Authentication
  async login(username: string, password: string): Promise<AuthResponse> {
    const response = await this.client.post<AuthResponse>('/auth/login', {
      username,
      password,
    });
    return response.data;
  }

  // Portal
  async getPortalSummary(groupId: string): Promise<PortalSummary> {
    const response = await this.client.get<{ summary: PortalSummary }>(
      `/portal/summary/${groupId}`
    );
    return response.data.summary;
  }

  async getPortalStatus(groupId: string): Promise<any> {
    const response = await this.client.get(`/portal/status/${groupId}`);
    return response.data;
  }

  // Client Chat
  async sendClientMessage(groupId: string, message: string): Promise<ChatResponse> {
    const response = await this.client.post<ChatResponse>('/client/chat', {
      groupId,
      message,
    });
    return response.data;
  }

  // Admin Chat
  async sendAdminMessage(message: string, context?: any): Promise<ChatResponse> {
    const response = await this.client.post<ChatResponse>('/admin/chat', {
      message,
      context,
    });
    return response.data;
  }

  // Admin - Get Alerts
  async getAlerts(limit = 50): Promise<any> {
    const response = await this.client.get('/admin/alerts', {
      params: { limit },
    });
    return response.data;
  }

  // Admin - Get Agents
  async getAgents(status = 'active', limit = 100): Promise<any> {
    const response = await this.client.get('/admin/agents', {
      params: { status, limit },
    });
    return response.data;
  }

  // Admin - Get Cluster Health
  async getClusterHealth(): Promise<any> {
    const response = await this.client.get('/admin/cluster/health');
    return response.data;
  }
}

export const api = new ApiClient();
export default api;
