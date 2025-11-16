/**
 * MetricCard Component - Carte de métrique avec design cyberpunk
 */

'use client';

import { motion } from 'framer-motion';
import { LucideIcon } from 'lucide-react';

interface MetricCardProps {
  title: string;
  value: string | number;
  subtitle?: string;
  icon: LucideIcon;
  trend?: 'up' | 'down' | 'neutral';
  trendValue?: string;
  className?: string;
}

export default function MetricCard({
  title,
  value,
  subtitle,
  icon: Icon,
  trend,
  trendValue,
  className = '',
}: MetricCardProps) {
  const trendColors = {
    up: 'text-status-protected',
    down: 'text-status-critical',
    neutral: 'text-gray-400',
  };

  return (
    <motion.div
      className={`glass rounded-lg p-6 hover:border-cyan-400/30 transition-all duration-300 ${className}`}
      whileHover={{ scale: 1.02 }}
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm text-gray-400 mb-1">{title}</p>
          <div className="flex items-baseline gap-2">
            <h3 className="text-3xl font-bold text-white">{value}</h3>
            {trend && trendValue && (
              <span className={`text-sm ${trendColors[trend]}`}>
                {trend === 'up' ? '↑' : trend === 'down' ? '↓' : '→'} {trendValue}
              </span>
            )}
          </div>
          {subtitle && <p className="text-xs text-gray-500 mt-1">{subtitle}</p>}
        </div>

        <div className="p-3 bg-cyan-400/10 rounded-lg">
          <Icon className="w-6 h-6 text-cyan-400" />
        </div>
      </div>

      {/* Animated Border */}
      <div className="mt-4 h-1 bg-dark-surfaceLight rounded-full overflow-hidden">
        <motion.div
          className="h-full bg-gradient-to-r from-cyan-400 to-cyan-600"
          initial={{ width: '0%' }}
          animate={{ width: '100%' }}
          transition={{ duration: 1, delay: 0.2 }}
        />
      </div>
    </motion.div>
  );
}
