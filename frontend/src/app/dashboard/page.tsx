/**
 * Client Dashboard - Interface utilisateur "Zéro Jargon"
 */

'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Shield as ShieldIcon, Activity, Server, LogOut, MessageCircle } from 'lucide-react';
import Shield from '@/components/Shield';
import MetricCard from '@/components/MetricCard';
import AlertFeed from '@/components/AlertFeed';
import ChatInterface from '@/components/ChatInterface';
import { api } from '@/lib/api';
import { getUser, clearAuth, isClient } from '@/lib/auth';
import { PortalSummary } from '@/types';

export default function DashboardPage() {
  const router = useRouter();
  const [summary, setSummary] = useState<PortalSummary | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [showChat, setShowChat] = useState(false);
  const user = getUser();

  useEffect(() => {
    if (!user || !isClient()) {
      router.push('/login');
      return;
    }

    loadSummary();
    const interval = setInterval(loadSummary, 30000); // Refresh every 30s
    return () => clearInterval(interval);
  }, [user, router]);

  const loadSummary = async () => {
    if (!user?.groupId) return;

    try {
      const data = await api.getPortalSummary(user.groupId);
      setSummary(data);
    } catch (error) {
      console.error('Failed to load summary:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleLogout = () => {
    clearAuth();
    router.push('/login');
  };

  const handleSendMessage = async (message: string): Promise<string> => {
    if (!user?.groupId) return 'Erreur: Groupe non identifié';
    const response = await api.sendClientMessage(user.groupId, message);
    return response.message;
  };

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <div className="w-16 h-16 border-4 border-cyan-400 border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-gray-400">Chargement de votre tableau de bord...</p>
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
          <h1 className="text-3xl font-bold text-white mb-2">
            Bonjour, {user?.username} 👋
          </h1>
          <p className="text-gray-400">Voici l'état de votre protection</p>
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
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Left Column - Shield and Metrics */}
          <div className="lg:col-span-2 space-y-6">
            {/* Shield Status */}
            <div className="glass rounded-lg p-8">
              <Shield
                status={summary?.shieldStatus || 'protected'}
                protectedSystems={summary?.agentStatuses.active}
                className="h-64"
              />
            </div>

            {/* Metrics */}
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <MetricCard
                title="Systèmes Protégés"
                value={summary?.agentStatuses.active || 0}
                subtitle={`sur ${summary?.agentStatuses.total || 0} total`}
                icon={Server}
              />
              <MetricCard
                title="Score de Conformité"
                value={`${summary?.securityScore.sca || 0}%`}
                subtitle="Politiques de sécurité"
                icon={ShieldIcon}
                trend="up"
                trendValue="+5%"
              />
              <MetricCard
                title="État des Vulnérabilités"
                value={`${summary?.securityScore.vulnerabilities || 0}%`}
                subtitle="Systèmes à jour"
                icon={Activity}
              />
            </div>

            {/* Alert Feed */}
            <AlertFeed alerts={summary?.recentAlerts || []} />
          </div>

          {/* Right Column - Chat */}
          <div className="lg:col-span-1">
            <ChatInterface
              title="💬 Concierge"
              placeholder="Posez vos questions sur votre sécurité..."
              onSendMessage={handleSendMessage}
              variant="client"
              className="h-full"
            />
          </div>
        </div>
      </div>

      {/* Floating Chat Button (mobile) */}
      <button
        onClick={() => setShowChat(!showChat)}
        className="lg:hidden fixed bottom-6 right-6 w-14 h-14 bg-cyan-400 hover:bg-cyan-500
                 rounded-full shadow-cyan-glow flex items-center justify-center
                 transition-all z-50"
      >
        <MessageCircle className="w-6 h-6 text-dark-bg" />
      </button>
    </div>
  );
}
