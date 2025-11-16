/**
 * LLM Service - Intégration avec Google Gemini pour traduction et analyse
 */

import { GoogleGenerativeAI } from '@google/generative-ai';
import config from '../config';
import logger from '../utils/logger';
import { ChatMessage } from '../types';

export class LLMService {
  private genAI: GoogleGenerativeAI | null = null;
  private model: any = null;

  constructor() {
    if (config.llm.provider === 'gemini' && config.llm.geminiApiKey) {
      this.genAI = new GoogleGenerativeAI(config.llm.geminiApiKey);
      this.model = this.genAI.getGenerativeModel({ model: 'gemini-pro' });
    } else {
      logger.warn('LLM service not configured. Set GEMINI_API_KEY or MISTRAL_API_KEY.');
    }
  }

  /**
   * Prompts système pour différents contextes
   */
  private getSystemPrompt(type: 'admin' | 'client'): string {
    if (type === 'admin') {
      return `Tu es J.A.R.V.I.S., l'assistant IA expert en cybersécurité pour les administrateurs de la plateforme Sentinelle-MCP.

**Ton rôle :**
- Traduire les commandes en langage naturel en actions Wazuh spécifiques
- Analyser les alertes de sécurité et fournir des recommandations techniques
- Aider à la remédiation des incidents de sécurité
- Fournir des informations détaillées sur les menaces et vulnérabilités

**Ton style :**
- Précis, technique et concis
- Utilise des termes de cybersécurité appropriés
- Fournis des actions concrètes et des commandes précises
- Toujours professionnel et orienté solution

**Contexte :**
- Tu as accès à un cluster Wazuh pour la surveillance de sécurité
- Tu peux lancer des Active Response, bannir des IPs, analyser des logs
- Tu dois prioriser la sécurité et la disponibilité des systèmes

Réponds en français, de manière directe et actionnable.`;
    } else {
      return `Tu es le Concierge, l'assistant IA bienveillant de la plateforme Sentinelle-MCP pour les clients non-techniques.

**Ton rôle :**
- Expliquer l'état de sécurité en termes simples et rassurants
- Traduire les alertes techniques en langage compréhensible
- Répondre aux questions sur la protection et la conformité
- Ne JAMAIS montrer d'informations techniques (IPs, logs bruts, IDs de règles)

**Ton style :**
- Pédagogue, rassurant et accessible
- Utilise des métaphores simples (alarme, garde du corps, bouclier)
- Évite tout jargon technique
- Toujours positif et orienté tranquillité d'esprit

**Règles strictes :**
- JAMAIS d'adresses IP, de noms d'hôtes ou d'identifiants techniques
- JAMAIS de logs bruts ou de code
- Toujours traduire en "ce que cela signifie pour mon entreprise"
- Mode lecture seule uniquement - tu ne peux pas modifier la configuration

Réponds en français, comme si tu parlais à un gérant de PME qui n'y connaît rien en informatique.`;
    }
  }

  /**
   * Traduire une alerte Wazuh en français simple pour les clients
   */
  async translateAlertForClient(alert: string): Promise<string> {
    if (!this.model) {
      return alert; // Fallback si LLM non configuré
    }

    try {
      const prompt = `Traduis cette alerte de sécurité Wazuh en français simple pour un client non-technique.
Explique ce qui s'est passé et si c'est grave, sans utiliser de jargon technique.
Garde ta réponse à 1-2 phrases maximum.

Alerte : ${alert}

Traduction simple :`;

      const result = await this.model.generateContent(prompt);
      const response = await result.response;
      const text = response.text().trim();

      logger.debug(`Translated alert: ${alert} -> ${text}`);
      return text;
    } catch (error) {
      logger.error('Error translating alert:', error);
      return alert; // Fallback
    }
  }

  /**
   * Traduire plusieurs alertes en batch
   */
  async translateAlertsForClient(alerts: string[]): Promise<string[]> {
    return Promise.all(alerts.map(alert => this.translateAlertForClient(alert)));
  }

  /**
   * Chat admin (J.A.R.V.I.S.) - Interpréter commandes et fournir expertise
   */
  async chatAdmin(message: string, history: ChatMessage[] = []): Promise<string> {
    if (!this.model) {
      return 'Service IA non configuré. Veuillez configurer GEMINI_API_KEY.';
    }

    try {
      const systemPrompt = this.getSystemPrompt('admin');

      // Construire l'historique de conversation
      const conversationHistory = history
        .map(msg => `${msg.role === 'user' ? 'Admin' : 'J.A.R.V.I.S.'}: ${msg.content}`)
        .join('\n');

      const fullPrompt = `${systemPrompt}

${conversationHistory ? `Historique de conversation:\n${conversationHistory}\n\n` : ''}Admin: ${message}

J.A.R.V.I.S.:`;

      const result = await this.model.generateContent(fullPrompt);
      const response = await result.response;
      const text = response.text().trim();

      logger.info(`Admin chat: "${message}" -> "${text.substring(0, 100)}..."`);
      return text;
    } catch (error) {
      logger.error('Error in admin chat:', error);
      throw new Error('Erreur lors de la communication avec le service IA');
    }
  }

  /**
   * Chat client (Concierge) - Réponses rassurantes et pédagogiques
   */
  async chatClient(message: string, history: ChatMessage[] = []): Promise<string> {
    if (!this.model) {
      return 'Désolé, le service de chat n\'est pas disponible actuellement.';
    }

    try {
      const systemPrompt = this.getSystemPrompt('client');

      const conversationHistory = history
        .map(msg => `${msg.role === 'user' ? 'Client' : 'Concierge'}: ${msg.content}`)
        .join('\n');

      const fullPrompt = `${systemPrompt}

${conversationHistory ? `Historique de conversation:\n${conversationHistory}\n\n` : ''}Client: ${message}

Concierge:`;

      const result = await this.model.generateContent(fullPrompt);
      const response = await result.response;
      const text = response.text().trim();

      logger.info(`Client chat: "${message}" -> "${text.substring(0, 100)}..."`);
      return text;
    } catch (error) {
      logger.error('Error in client chat:', error);
      throw new Error('Désolé, je rencontre un problème technique. Veuillez réessayer.');
    }
  }

  /**
   * Analyser et extraire des actions depuis un message admin
   */
  async extractActionsFromAdminMessage(message: string): Promise<string[]> {
    if (!this.model) {
      return [];
    }

    try {
      const prompt = `Analyse ce message d'un administrateur de sécurité et extrais les actions à effectuer.
Retourne uniquement une liste d'actions concrètes, une par ligne.
Si aucune action n'est demandée, retourne "AUCUNE".

Message : ${message}

Actions :`;

      const result = await this.model.generateContent(prompt);
      const response = await result.response;
      const text = response.text().trim();

      if (text === 'AUCUNE' || text === '') {
        return [];
      }

      return text.split('\n').filter(line => line.trim() !== '');
    } catch (error) {
      logger.error('Error extracting actions:', error);
      return [];
    }
  }

  /**
   * Générer un résumé de sécurité pour le dashboard client
   */
  async generateSecuritySummary(data: {
    agentCount: number;
    activeAgents: number;
    recentAlerts: number;
    criticalAlerts: number;
  }): Promise<string> {
    if (!this.model) {
      return `Vous avez ${data.activeAgents} systèmes protégés sur ${data.agentCount}. ${data.recentAlerts} événements détectés récemment.`;
    }

    try {
      const prompt = `Génère un résumé de sécurité court et rassurant pour un client non-technique, basé sur ces données :
- ${data.agentCount} systèmes au total
- ${data.activeAgents} systèmes actifs
- ${data.recentAlerts} alertes récentes
- ${data.criticalAlerts} alertes critiques

Résumé en 2-3 phrases maximum, style rassurant et pédagogue :`;

      const result = await this.model.generateContent(prompt);
      const response = await result.response;
      return response.text().trim();
    } catch (error) {
      logger.error('Error generating security summary:', error);
      return `Vous avez ${data.activeAgents} systèmes protégés sur ${data.agentCount}. ${data.recentAlerts} événements détectés récemment.`;
    }
  }
}

// Singleton instance
let llmServiceInstance: LLMService | null = null;

export function getLLMService(): LLMService {
  if (!llmServiceInstance) {
    llmServiceInstance = new LLMService();
  }
  return llmServiceInstance;
}

export default LLMService;
