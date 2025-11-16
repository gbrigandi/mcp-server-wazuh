/**
 * Admin Dashboard - Interface J.A.R.V.I.S.
 */

'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import {
  Terminal,
  Activity,
  Server,
  AlertTriangle,
  LogOut,
  Shield as ShieldIcon,
} from 'lucide-react';
import ChatInterface from '@/components/ChatInterface';
import MetricCard from '@/components/MetricCard';
import { api } from '@/lib/api';
import { getUser, clearAuth, isAdmin } from '@/lib/auth';

export default function AdminPage() {
  const router = useRouter();
  const [metrics, setMetrics] = useState({
    totalAgents: 0,
    activeAgents: 0,
    recentAlerts: 0,
    clusterHealth: 'Unknown',
  });
  const [isLoading, setIsLoading] = useState(true);
  const user = getUser();

  useEffect(() => {
    if (!user || !isAdmin()) {
      router.push('/login');
      return;
    }

    loadMetrics();
    const interval = setInterval(loadMetrics, 30000);
    return () => clearInterval(interval);
  }, [user, router]);

  const loadMetrics = async () => {
    try {
      const [agentsData, alertsData, healthData] = await Promise.all([
        api.getAgents('active', 100),
        api.getAlerts(50),
        api.getClusterHealth(),
      ]);

      // Parse agents count
      const agentText = agentsData.agents?.[0]?.text || '';
      const agentCount = agentText.split('\n').filter((line: string) =>
        line.includes('Agent ID:')
      ).length;

      // Parse alerts count
      const alertCount = alertsData.alerts?.length || 0;

      // Parse cluster health
      const healthText = healthData.health?.[0]?.text || 'Unknown';
      const health = healthText.includes('enabled')
        ? 'Healthy'
        : 'Unknown';

      setMetrics({
        totalAgents: agentCount,
        activeAgents: agentCount,
        recentAlerts: alertCount,
        clusterHealth: health,
      });
    } catch (error) {
      console.error('Failed to load metrics:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogout = () => {
    clearAuth();
    router.push('/login');
  };

  const handleSendMessage = async (message: string): Promise<string> => {
    const response = await api.sendAdminMessage(message);
    return response.message;
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-cyan-400 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-400">Initialisation de J.A.R.V.I.S...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen p-6">
      {/* Header */}
      <motion.div
        className="mb-8 flex items-center justify-between"
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
      >
        <div>
          <h1 className="text-3xl font-bold text-white mb-2 flex items-center gap-3">
            <Terminal className="w-8 h-8 text-cyan-400" />
            J.A.R.V.I.S. Admin Console
          </h1>
          <p className="text-gray-400">
            Connecté en tant que <span className="text-cyan-400">{user?.username}</span>
          </p>
        </div>

        <button
          onClick={handleLogout}
          className="px-4 py-2 bg-dark-surfaceLight hover:bg-dark-border text-gray-300 rounded-lg
                   transition-colors flex items-center gap-2"
        >
          <LogOut className="w-4 h-4" />
          Déconnexion
        </button>
      </motion.div>

      <div className="max-w-7xl mx-auto">
        {/* Metrics */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-6">
          <MetricCard
            title="Agents Actifs"
            value={metrics.activeAgents}
            subtitle={`sur ${metrics.totalAgents} total`}
            icon={Server}
          />
          <MetricCard
            title="Alertes Récentes"
            value={metrics.recentAlerts}
            subtitle="Dernières 24h"
            icon={AlertTriangle}
          />
          <MetricCard
            title="Cluster Wazuh"
            value={metrics.clusterHealth}
            subtitle="État du cluster"
            icon={Activity}
          />
          <MetricCard
            title="Statut Global"
            value="Opérationnel"
            subtitle="Tous systèmes OK"
            icon={ShieldIcon}
          />
        </div>

        {/* Main Content - J.A.R.V.I.S. Chat */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          <div className="lg:col-span-2">
            <ChatInterface
              title="🤖 J.A.R.V.I.S. - Assistant Expert"
              placeholder="Commande en langage naturel (ex: 'Bannis l'IP 192.168.1.100')..."
              onSendMessage={handleSendMessage}
              variant="admin"
              className="h-[700px]"
            />
          </div>

          {/* Side Panel - Quick Actions */}
          <div className="space-y-4">
            <div className="glass rounded-lg p-6">
              <h3 className="text-lg font-bold text-white mb-4">Actions Rapides</h3>
              <div className="space-y-2">
                <button className="w-full px-4 py-3 bg-dark-surfaceLight hover:bg-dark-border
                                 text-left rounded-lg transition-colors text-sm">
                  📊 Rapport Hebdomadaire
                </button>
                <button className="w-full px-4 py-3 bg-dark-surfaceLight hover:bg-dark-border
                                 text-left rounded-lg transition-colors text-sm">
                  🔍 Scanner Vulnérabilités
                </button>
                <button className="w-full px-4 py-3 bg-dark-surfaceLight hover:bg-dark-border
                                 text-left rounded-lg transition-colors text-sm">
                  🚫 Liste IPs Bloquées
                </button>
                <button className="w-full px-4 py-3 bg-dark-surfaceLight hover:bg-dark-border
                                 text-left rounded-lg transition-colors text-sm">
                  ⚙️ Configuration Cluster
                </button>
              </div>
            </div>

            <div className="glass rounded-lg p-6">
              <h3 className="text-lg font-bold text-white mb-4">Infos Système</h3>
              <div className="space-y-3 text-sm">
                <div>
                  <p className="text-gray-400">Version MCP</p>
                  <p className="text-white font-mono">v1.0.0</p>
                </div>
                <div>
                  <p className="text-gray-400">Wazuh API</p>
                  <p className="text-status-protected">✓ Connecté</p>
                </div>
                <div>
                  <p className="text-gray-400">LLM Service</p>
                  <p className="text-status-protected">✓ Actif</p>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
