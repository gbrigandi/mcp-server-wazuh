/**
 * Login Page
 */

'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { motion } from 'framer-motion';
import { Shield, Loader2, AlertCircle } from 'lucide-react';
import { api } from '@/lib/api';
import { setAuth } from '@/lib/auth';

export default function LoginPage() {
  const router = useRouter();
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState('');

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault();
    setError('');
    setIsLoading(true);

    try {
      const response = await api.login(username, password);
      setAuth(response.token, response.user);

      // Redirect based on role
      if (response.user.role === 'admin') {
        router.push('/admin');
      } else {
        router.push('/dashboard');
      }
    } catch (err: any) {
      setError(err.response?.data?.error || 'Identifiants invalides');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center p-4">
      {/* Background effects */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <motion.div
          className="absolute top-1/4 -left-1/4 w-96 h-96 bg-cyan-400 rounded-full opacity-10 blur-3xl"
          animate={{
            scale: [1, 1.2, 1],
            opacity: [0.1, 0.15, 0.1],
          }}
          transition={{
            duration: 8,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
        />
        <motion.div
          className="absolute bottom-1/4 -right-1/4 w-96 h-96 bg-cyan-600 rounded-full opacity-10 blur-3xl"
          animate={{
            scale: [1.2, 1, 1.2],
            opacity: [0.15, 0.1, 0.15],
          }}
          transition={{
            duration: 8,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
        />
      </div>

      <motion.div
        className="w-full max-w-md glass rounded-2xl p-8 shadow-2xl relative z-10"
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.5 }}
      >
        {/* Logo */}
        <div className="text-center mb-8">
          <motion.div
            className="inline-block"
            initial={{ scale: 0, rotate: -180 }}
            animate={{ scale: 1, rotate: 0 }}
            transition={{ type: 'spring', duration: 0.8 }}
          >
            <Shield className="w-20 h-20 text-cyan-400 mx-auto mb-4" strokeWidth={1.5} />
          </motion.div>
          <h1 className="text-3xl font-bold text-white mb-2">Sentinelle-MCP</h1>
          <p className="text-gray-400">Votre Garde du Corps Numérique</p>
        </div>

        {/* Error Message */}
        {error && (
          <motion.div
            className="mb-6 p-4 bg-status-critical/10 border border-status-critical/30 rounded-lg flex items-start gap-3"
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
          >
            <AlertCircle className="w-5 h-5 text-status-critical flex-shrink-0 mt-0.5" />
            <p className="text-sm text-status-critical">{error}</p>
          </motion.div>
        )}

        {/* Login Form */}
        <form onSubmit={handleLogin} className="space-y-6">
          <div>
            <label htmlFor="username" className="block text-sm font-medium text-gray-300 mb-2">
              Nom d'utilisateur
            </label>
            <input
              id="username"
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              required
              className="w-full px-4 py-3 bg-dark-surfaceLight border border-dark-border rounded-lg
                       text-white placeholder-gray-500 focus:outline-none focus:border-cyan-400
                       transition-colors"
              placeholder="Entrez votre nom d'utilisateur"
              disabled={isLoading}
            />
          </div>

          <div>
            <label htmlFor="password" className="block text-sm font-medium text-gray-300 mb-2">
              Mot de passe
            </label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              required
              className="w-full px-4 py-3 bg-dark-surfaceLight border border-dark-border rounded-lg
                       text-white placeholder-gray-500 focus:outline-none focus:border-cyan-400
                       transition-colors"
              placeholder="Entrez votre mot de passe"
              disabled={isLoading}
            />
          </div>

          <motion.button
            type="submit"
            disabled={isLoading}
            className="w-full py-3 bg-cyan-400 hover:bg-cyan-500 text-dark-bg font-bold rounded-lg
                     transition-all disabled:opacity-50 disabled:cursor-not-allowed
                     hover:shadow-cyan-glow flex items-center justify-center gap-2"
            whileHover={{ scale: isLoading ? 1 : 1.02 }}
            whileTap={{ scale: isLoading ? 1 : 0.98 }}
          >
            {isLoading ? (
              <>
                <Loader2 className="w-5 h-5 animate-spin" />
                Connexion en cours...
              </>
            ) : (
              'Se connecter'
            )}
          </motion.button>
        </form>

        {/* Footer */}
        <div className="mt-8 text-center text-sm text-gray-500">
          <p>&copy; 2025 Sentinelle.pro - Tous droits réservés</p>
        </div>
      </motion.div>
    </div>
  );
}
