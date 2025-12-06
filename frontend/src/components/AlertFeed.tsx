/**
 * AlertFeed Component - Journal de bord des alertes traduites
 */

'use client';

import { motion } from 'framer-motion';
import { AlertTriangle, AlertCircle, Info } from 'lucide-react';
import { format } from 'date-fns';
import { fr } from 'date-fns/locale';
import { Alert } from '@/types';

interface AlertFeedProps {
  alerts: Alert[];
  className?: string;
}

export default function AlertFeed({ alerts, className = '' }: AlertFeedProps) {
  const getSeverityConfig = (severity: string) => {
    switch (severity) {
      case 'critical':
        return {
          Icon: AlertTriangle,
          color: 'text-status-critical',
          bgColor: 'bg-status-critical/10',
          borderColor: 'border-status-critical/30',
        };
      case 'warning':
        return {
          Icon: AlertCircle,
          color: 'text-status-warning',
          bgColor: 'bg-status-warning/10',
          borderColor: 'border-status-warning/30',
        };
      default:
        return {
          Icon: Info,
          color: 'text-cyan-400',
          bgColor: 'bg-cyan-400/10',
          borderColor: 'border-cyan-400/30',
        };
    }
  };

  return (
    <div className={`glass rounded-lg p-6 ${className}`}>
      <h2 className="text-xl font-bold text-white mb-4 flex items-center gap-2">
        <span className="text-cyan-400">📜</span> Journal de Bord
      </h2>

      <div className="space-y-3 max-h-96 overflow-y-auto pr-2">
        {alerts.length === 0 ? (
          <div className="text-center py-8 text-gray-500">
            <Info className="w-12 h-12 mx-auto mb-2 opacity-50" />
            <p>Aucune alerte récente</p>
          </div>
        ) : (
          alerts.map((alert, index) => {
            const config = getSeverityConfig(alert.severity);
            const Icon = config.Icon;

            return (
              <motion.div
                key={alert.id}
                className={`p-4 rounded-lg border ${config.bgColor} ${config.borderColor}`}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.1 }}
              >
                <div className="flex items-start gap-3">
                  <Icon className={`w-5 h-5 mt-0.5 ${config.color} flex-shrink-0`} />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm text-gray-200 leading-relaxed">
                      {alert.translatedMessage}
                    </p>
                    <p className="text-xs text-gray-500 mt-2">
                      {format(new Date(alert.timestamp), 'PPp', { locale: fr })}
                    </p>
                  </div>
                </div>
              </motion.div>
            );
          })
        )}
      </div>

      {alerts.length > 0 && (
        <div className="mt-4 pt-4 border-t border-dark-border">
          <p className="text-xs text-gray-500 text-center">
            {alerts.length} événement{alerts.length > 1 ? 's' : ''} récent{alerts.length > 1 ? 's' : ''}
          </p>
        </div>
      )}
    </div>
  );
}
