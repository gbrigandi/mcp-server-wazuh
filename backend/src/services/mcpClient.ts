/**
 * MCP Client Service - Communique avec le serveur MCP Wazuh (Rust)
 */

import { spawn, ChildProcess } from 'child_process';
import config from '../config';
import logger from '../utils/logger';
import { MCPToolCall, MCPResponse } from '../types';

export class MCPClient {
  private process: ChildProcess | null = null;
  private requestId = 0;
  private pendingRequests = new Map<number, {
    resolve: (value: MCPResponse) => void;
    reject: (error: Error) => void;
  }>();

  constructor() {
    this.initialize();
  }

  private initialize(): void {
    try {
      logger.info(`Initializing MCP server from: ${config.wazuh.mcpServerPath}`);

      // Spawn the Rust MCP server process
      this.process = spawn(config.wazuh.mcpServerPath, [], {
        env: {
          ...process.env,
          WAZUH_API_HOST: config.wazuh.apiHost,
          WAZUH_API_PORT: config.wazuh.apiPort.toString(),
          WAZUH_API_USERNAME: config.wazuh.apiUsername,
          WAZUH_API_PASSWORD: config.wazuh.apiPassword,
          WAZUH_INDEXER_HOST: config.wazuh.indexerHost,
          WAZUH_INDEXER_PORT: config.wazuh.indexerPort.toString(),
          WAZUH_INDEXER_USERNAME: config.wazuh.indexerUsername,
          WAZUH_INDEXER_PASSWORD: config.wazuh.indexerPassword,
          WAZUH_VERIFY_SSL: config.wazuh.verifySSL.toString(),
          WAZUH_TEST_PROTOCOL: config.wazuh.testProtocol,
          RUST_LOG: 'info',
        },
        stdio: ['pipe', 'pipe', 'pipe'],
      });

      let buffer = '';

      // Handle stdout (MCP responses)
      this.process.stdout?.on('data', (data: Buffer) => {
        buffer += data.toString();
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';

        for (const line of lines) {
          if (line.trim()) {
            try {
              const response = JSON.parse(line);
              this.handleResponse(response);
            } catch (err) {
              logger.error('Failed to parse MCP response:', err);
            }
          }
        }
      });

      // Handle stderr (logs)
      this.process.stderr?.on('data', (data: Buffer) => {
        const msg = data.toString().trim();
        if (msg) {
          logger.debug(`MCP Server: ${msg}`);
        }
      });

      // Handle process errors
      this.process.on('error', (error: Error) => {
        logger.error('MCP Server process error:', error);
        this.reconnect();
      });

      // Handle process exit
      this.process.on('exit', (code: number | null, signal: string | null) => {
        logger.warn(`MCP Server process exited with code ${code}, signal ${signal}`);
        this.process = null;
        this.reconnect();
      });

      // Initialize the MCP protocol
      this.sendInitialize();
    } catch (error) {
      logger.error('Failed to initialize MCP client:', error);
      throw error;
    }
  }

  private sendInitialize(): void {
    const initMessage = {
      jsonrpc: '2.0',
      id: this.requestId++,
      method: 'initialize',
      params: {
        protocolVersion: '2024-11-05',
        capabilities: {
          sampling: {},
          roots: { listChanged: true },
        },
        clientInfo: {
          name: 'sentinelle-mcp-backend',
          version: '1.0.0',
        },
      },
    };

    this.sendMessage(initMessage);
  }

  private sendMessage(message: unknown): void {
    if (!this.process || !this.process.stdin) {
      throw new Error('MCP server process not available');
    }

    const json = JSON.stringify(message) + '\n';
    this.process.stdin.write(json);
  }

  private handleResponse(response: any): void {
    if (response.id !== undefined && this.pendingRequests.has(response.id)) {
      const { resolve, reject } = this.pendingRequests.get(response.id)!;
      this.pendingRequests.delete(response.id);

      if (response.error) {
        reject(new Error(response.error.message || 'MCP request failed'));
      } else {
        resolve(response.result);
      }
    }
  }

  private reconnect(): void {
    logger.info('Attempting to reconnect to MCP server in 5 seconds...');
    setTimeout(() => {
      try {
        this.initialize();
      } catch (error) {
        logger.error('Failed to reconnect to MCP server:', error);
      }
    }, 5000);
  }

  /**
   * Call a Wazuh MCP tool
   */
  async callTool(toolCall: MCPToolCall): Promise<MCPResponse> {
    return new Promise((resolve, reject) => {
      const id = this.requestId++;

      this.pendingRequests.set(id, { resolve, reject });

      const message = {
        jsonrpc: '2.0',
        id,
        method: 'tools/call',
        params: {
          name: toolCall.name,
          arguments: toolCall.arguments,
        },
      };

      try {
        this.sendMessage(message);

        // Timeout after 30 seconds
        setTimeout(() => {
          if (this.pendingRequests.has(id)) {
            this.pendingRequests.delete(id);
            reject(new Error('MCP request timeout'));
          }
        }, 30000);
      } catch (error) {
        this.pendingRequests.delete(id);
        reject(error);
      }
    });
  }

  /**
   * Get alert summary from Wazuh
   */
  async getAlertSummary(limit = 10): Promise<MCPResponse> {
    return this.callTool({
      name: 'get_wazuh_alert_summary',
      arguments: { limit },
    });
  }

  /**
   * Get agents list
   */
  async getAgents(status = 'active', limit = 100): Promise<MCPResponse> {
    return this.callTool({
      name: 'get_wazuh_agents',
      arguments: { status, limit },
    });
  }

  /**
   * Get vulnerability summary for an agent
   */
  async getVulnerabilitySummary(agentId: string, limit = 100): Promise<MCPResponse> {
    return this.callTool({
      name: 'get_wazuh_vulnerability_summary',
      arguments: { agent_id: agentId, limit },
    });
  }

  /**
   * Get cluster health
   */
  async getClusterHealth(): Promise<MCPResponse> {
    return this.callTool({
      name: 'get_wazuh_cluster_health',
      arguments: {},
    });
  }

  /**
   * Cleanup on shutdown
   */
  shutdown(): void {
    if (this.process) {
      logger.info('Shutting down MCP server process...');
      this.process.kill();
      this.process = null;
    }
  }
}

// Singleton instance
let mcpClientInstance: MCPClient | null = null;

export function getMCPClient(): MCPClient {
  if (!mcpClientInstance) {
    mcpClientInstance = new MCPClient();
  }
  return mcpClientInstance;
}

export default MCPClient;
