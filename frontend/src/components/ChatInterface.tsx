/**
 * ChatInterface Component - Interface de chat pour Concierge et J.A.R.V.I.S.
 */

'use client';

import { useState, useRef, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { Send, Bot, User as UserIcon, Loader2 } from 'lucide-react';
import { ChatMessage } from '@/types';

interface ChatInterfaceProps {
  title: string;
  placeholder?: string;
  onSendMessage: (message: string) => Promise<string>;
  variant?: 'client' | 'admin';
  className?: string;
}

export default function ChatInterface({
  title,
  placeholder = 'Posez votre question...',
  onSendMessage,
  variant = 'client',
  className = '',
}: ChatInterfaceProps) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [input, setInput] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSend = async () => {
    if (!input.trim() || isLoading) return;

    const userMessage: ChatMessage = {
      role: 'user',
      content: input.trim(),
      timestamp: new Date(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setInput('');
    setIsLoading(true);

    try {
      const response = await onSendMessage(userMessage.content);

      const assistantMessage: ChatMessage = {
        role: 'assistant',
        content: response,
        timestamp: new Date(),
      };

      setMessages((prev) => [...prev, assistantMessage]);
    } catch (error) {
      const errorMessage: ChatMessage = {
        role: 'assistant',
        content: 'Désolé, une erreur est survenue. Veuillez réessayer.',
        timestamp: new Date(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const iconColor = variant === 'admin' ? 'text-cyan-400' : 'text-status-protected';
  const bgGradient = variant === 'admin'
    ? 'from-cyan-400/20 to-cyan-600/20'
    : 'from-status-protected/20 to-green-600/20';

  return (
    <div className={`glass rounded-lg flex flex-col ${className}`}>
      {/* Header */}
      <div className={`p-4 border-b border-dark-border bg-gradient-to-r ${bgGradient}`}>
        <h2 className="text-xl font-bold text-white flex items-center gap-2">
          <Bot className={`w-6 h-6 ${iconColor}`} />
          {title}
        </h2>
      </div>

      {/* Messages */}
      <div className="flex-1 p-4 overflow-y-auto space-y-4 min-h-[400px] max-h-[600px]">
        <AnimatePresence>
          {messages.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-500">
              <div className="text-center">
                <Bot className="w-16 h-16 mx-auto mb-4 opacity-30" />
                <p>Commencez une conversation...</p>
              </div>
            </div>
          ) : (
            messages.map((message, index) => (
              <motion.div
                key={index}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0 }}
                transition={{ duration: 0.3 }}
                className={`flex gap-3 ${
                  message.role === 'user' ? 'justify-end' : 'justify-start'
                }`}
              >
                {message.role === 'assistant' && (
                  <div className={`p-2 rounded-full ${iconColor.replace('text-', 'bg-')}/10`}>
                    <Bot className={`w-6 h-6 ${iconColor}`} />
                  </div>
                )}

                <div
                  className={`max-w-[75%] p-4 rounded-lg ${
                    message.role === 'user'
                      ? 'bg-cyan-400/20 text-white'
                      : 'bg-dark-surfaceLight text-gray-200'
                  }`}
                >
                  <p className="whitespace-pre-wrap leading-relaxed">{message.content}</p>
                  <p className="text-xs text-gray-500 mt-2">
                    {message.timestamp.toLocaleTimeString('fr-FR', {
                      hour: '2-digit',
                      minute: '2-digit',
                    })}
                  </p>
                </div>

                {message.role === 'user' && (
                  <div className="p-2 rounded-full bg-cyan-400/10">
                    <UserIcon className="w-6 h-6 text-cyan-400" />
                  </div>
                )}
              </motion.div>
            ))
          )}
        </AnimatePresence>

        {isLoading && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex gap-3 justify-start"
          >
            <div className={`p-2 rounded-full ${iconColor.replace('text-', 'bg-')}/10`}>
              <Bot className={`w-6 h-6 ${iconColor}`} />
            </div>
            <div className="bg-dark-surfaceLight p-4 rounded-lg">
              <Loader2 className="w-5 h-5 animate-spin text-cyan-400" />
            </div>
          </motion.div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div className="p-4 border-t border-dark-border">
        <div className="flex gap-2">
          <input
            type="text"
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyPress={handleKeyPress}
            placeholder={placeholder}
            disabled={isLoading}
            className="flex-1 px-4 py-3 bg-dark-surfaceLight border border-dark-border rounded-lg
                     text-white placeholder-gray-500 focus:outline-none focus:border-cyan-400
                     transition-colors disabled:opacity-50"
          />
          <button
            onClick={handleSend}
            disabled={!input.trim() || isLoading}
            className="px-6 py-3 bg-cyan-400 hover:bg-cyan-500 text-dark-bg rounded-lg
                     font-medium transition-all disabled:opacity-50 disabled:cursor-not-allowed
                     hover:shadow-cyan-glow flex items-center gap-2"
          >
            {isLoading ? (
              <Loader2 className="w-5 h-5 animate-spin" />
            ) : (
              <Send className="w-5 h-5" />
            )}
          </button>
        </div>
      </div>
    </div>
  );
}
