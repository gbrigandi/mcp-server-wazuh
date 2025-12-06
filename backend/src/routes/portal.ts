/**
 * Portal routes - Client dashboard data
 */

import { Router, Request, Response } from 'express';
import { authenticateToken, requireOwnGroup } from '../middleware/auth';
import { getMCPClient } from '../services/mcpClient';
import { getLLMService } from '../services/llmService';
import logger from '../utils/logger';
import { PortalSummary } from '../types';

const router = Router();

// All portal routes require authentication
router.use(authenticateToken);

/**
 * GET /api/v1/portal/summary/:groupId
 * Get aggregated security summary for a group (simplified for clients)
 */
router.get('/summary/:groupId', requireOwnGroup, async (req: Request, res: Response) => {
  try {
    const { groupId } = req.params;

    logger.info(`Portal summary request for group ${groupId} by ${req.user?.username}`);

    const mcpClient = getMCPClient();
    const llmService = getLLMService();

    // Fetch data in parallel
    const [agentsResponse, alertsResponse] = await Promise.all([
      mcpClient.getAgents('active', 100),
      mcpClient.getAlertSummary(5),
    ]);

    // Parse agent data
    let totalAgents = 0;
    let activeAgents = 0;
    let disconnectedAgents = 0;

    if (!agentsResponse.isError && agentsResponse.content.length > 0) {
      const agentText = agentsResponse.content[0].text || '';
      const agentLines = agentText.split('\n');

      // Count agents by parsing the text output
      totalAgents = agentLines.filter(line => line.includes('Agent ID:')).length;
      activeAgents = agentLines.filter(line => line.includes('Status: active')).length;
      disconnectedAgents = agentLines.filter(line => line.includes('Status: disconnected')).length;
    }

    // Parse and translate alerts
    const recentAlerts: PortalSummary['recentAlerts'] = [];

    if (!alertsResponse.isError && alertsResponse.content.length > 0) {
      const alertTexts = alertsResponse.content
        .map(c => c.text)
        .filter(t => t && t.trim() !== '');

      // Translate each alert
      const translatedAlerts = await llmService.translateAlertsForClient(alertTexts);

      for (let i = 0; i < Math.min(alertTexts.length, 5); i++) {
        const alertText = alertTexts[i];
        const translated = translatedAlerts[i];

        // Parse basic alert info (simplified)
        const idMatch = alertText.match(/Alert ID: ([^\n]+)/);
        const timeMatch = alertText.match(/Time: ([^\n]+)/);
        const levelMatch = alertText.match(/Level: (\d+)/);

        recentAlerts.push({
          id: idMatch ? idMatch[1] : `alert-${i}`,
          timestamp: timeMatch ? timeMatch[1] : new Date().toISOString(),
          severity: levelMatch && parseInt(levelMatch[1]) >= 10 ? 'critical' : 'normal',
          translatedMessage: translated,
        });
      }
    }

    // Determine shield status
    let shieldStatus: PortalSummary['shieldStatus'] = 'protected';
    const criticalAlerts = recentAlerts.filter(a => a.severity === 'critical').length;

    if (criticalAlerts > 0 || disconnectedAgents > totalAgents / 2) {
      shieldStatus = 'critical';
    } else if (disconnectedAgents > 0) {
      shieldStatus = 'warning';
    }

    // Mock security scores (in real implementation, fetch from SCA and vulnerability tools)
    const securityScore = {
      sca: 85, // Mock value
      vulnerabilities: 90, // Mock value
    };

    const summary: PortalSummary = {
      groupId,
      agentStatuses: {
        total: totalAgents,
        active: activeAgents,
        disconnected: disconnectedAgents,
      },
      securityScore,
      recentAlerts,
      shieldStatus,
    };

    res.json({
      summary,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error generating portal summary:', error);
    res.status(500).json({ error: 'Failed to generate portal summary' });
  }
});

/**
 * GET /api/v1/portal/status/:groupId
 * Get simplified status for shield display
 */
router.get('/status/:groupId', requireOwnGroup, async (req: Request, res: Response) => {
  try {
    const { groupId } = req.params;

    const mcpClient = getMCPClient();

    // Get basic agent count
    const agentsResponse = await mcpClient.getAgents('active', 10);

    let agentCount = 0;
    if (!agentsResponse.isError && agentsResponse.content.length > 0) {
      const agentText = agentsResponse.content[0].text || '';
      agentCount = agentText.split('\n').filter(line => line.includes('Agent ID:')).length;
    }

    const status = agentCount > 0 ? 'protected' : 'warning';

    res.json({
      groupId,
      status,
      protectedSystems: agentCount,
      message: agentCount > 0
        ? `${agentCount} système${agentCount > 1 ? 's' : ''} protégé${agentCount > 1 ? 's' : ''}`
        : 'Aucun système protégé actuellement',
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Error getting portal status:', error);
    res.status(500).json({ error: 'Failed to get status' });
  }
});

export default router;
