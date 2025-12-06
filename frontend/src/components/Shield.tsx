/**
 * Shield Component - Affiche le statut de protection
 */

'use client';

import { useEffect, useState } from 'react';
import { motion } from 'framer-motion';
import { Shield as ShieldIcon, CheckCircle2, AlertTriangle, XCircle } from 'lucide-react';

interface ShieldProps {
  status: 'protected' | 'warning' | 'critical';
  protectedSystems?: number;
  className?: string;
}

export default function Shield({ status, protectedSystems, className = '' }: ShieldProps) {
  const [isAnimating, setIsAnimating] = useState(true);

  useEffect(() => {
    const timer = setTimeout(() => setIsAnimating(false), 3000);
    return () => clearTimeout(timer);
  }, []);

  const statusConfig = {
    protected: {
      color: 'text-status-protected',
      glowColor: 'rgba(16, 185, 129, 0.5)',
      Icon: CheckCircle2,
      message: 'Protégé',
    },
    warning: {
      color: 'text-status-warning',
      glowColor: 'rgba(245, 158, 11, 0.5)',
      Icon: AlertTriangle,
      message: 'Attention',
    },
    critical: {
      color: 'text-status-critical',
      glowColor: 'rgba(239, 68, 68, 0.5)',
      Icon: XCircle,
      message: 'Critique',
    },
  };

  const config = statusConfig[status];
  const Icon = config.Icon;

  return (
    <div className={`relative flex flex-col items-center justify-center ${className}`}>
      {/* Background Glow */}
      <motion.div
        className="absolute inset-0 rounded-full blur-3xl"
        style={{
          background: `radial-gradient(circle, ${config.glowColor} 0%, transparent 70%)`,
        }}
        animate={
          isAnimating
            ? {
                scale: [1, 1.2, 1],
                opacity: [0.5, 0.8, 0.5],
              }
            : {}
        }
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: 'easeInOut',
        }}
      />

      {/* Shield Icon */}
      <motion.div
        className="relative z-10"
        initial={{ scale: 0, rotate: -180 }}
        animate={{ scale: 1, rotate: 0 }}
        transition={{
          type: 'spring',
          stiffness: 260,
          damping: 20,
        }}
      >
        <div className="relative">
          <ShieldIcon className={`w-32 h-32 ${config.color}`} strokeWidth={1.5} />
          <Icon
            className={`absolute top-1/2 left-1/2 transform -translate-x-1/2 -translate-y-1/2 w-16 h-16 ${config.color}`}
          />
        </div>
      </motion.div>

      {/* Status Text */}
      <motion.div
        className="mt-4 text-center"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.3 }}
      >
        <h2 className={`text-2xl font-bold ${config.color}`}>{config.message}</h2>
        {protectedSystems !== undefined && (
          <p className="text-gray-400 mt-2">
            {protectedSystems} système{protectedSystems > 1 ? 's' : ''} protégé{protectedSystems > 1 ? 's' : ''}
          </p>
        )}
      </motion.div>

      {/* Rotating Ring */}
      <motion.div
        className={`absolute inset-0 rounded-full border-2 ${config.color.replace('text-', 'border-')}`}
        style={{
          borderStyle: 'dashed',
        }}
        animate={{
          rotate: 360,
        }}
        transition={{
          duration: 20,
          repeat: Infinity,
          ease: 'linear',
        }}
      />
    </div>
  );
}
