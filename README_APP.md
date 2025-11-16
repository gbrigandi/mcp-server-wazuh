# 🛡️ Sentinelle-MCP - Plateforme de Sécurité Managée

> **"Votre Garde du Corps Numérique"**

Une plateforme MDR (Managed Detection & Response) moderne, pilotée par l'IA, conçue pour les TPE/PME. Interface "Zéro Jargon" pour les clients, puissance totale pour les administrateurs.

---

## 🎯 Vision du Projet

Sentinelle-MCP transforme la complexité de Wazuh SIEM en une expérience simple et rassurante pour les clients non-techniques, tout en offrant aux administrateurs une interface surpuissante alimentée par l'IA.

### Problème Résolu

- ❌ Les SIEM traditionnels sont des "usines à gaz" illisibles
- ❌ Les TPE/PME n'ont pas de moyen de prouver leur conformité RGPD
- ❌ Les alertes techniques sont incompréhensibles pour les non-experts

### Notre Solution

- ✅ **Interface Client "Iron Man"** : Dashboard visuel sans jargon technique
- ✅ **Chatbot Concierge IA** : Réponses pédagogiques et rassurantes
- ✅ **J.A.R.V.I.S. Admin** : Commandes en langage naturel pour les pros
- ✅ **Traduction IA** : Toutes les alertes traduites en français simple

---

## 🏗️ Architecture Technique

```
┌─────────────────────────────────────────────────────────────┐
│                    SENTINELLE-MCP                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌──────────────┐           ┌──────────────────┐          │
│  │   Frontend   │◄─────────►│   Backend API    │          │
│  │   (Next.js)  │   REST    │  (Node.js/TS)    │          │
│  │              │           │                  │          │
│  │ • Dashboard  │           │ • Auth (JWT)     │          │
│  │ • Chat UI    │           │ • LLM Service    │          │
│  │ • Admin      │           │ • MCP Client     │          │
│  └──────────────┘           └────────┬─────────┘          │
│                                      │                     │
│                              ┌───────▼────────┐            │
│                              │   MCP Server   │            │
│                              │    (Rust)      │            │
│                              │                │            │
│                              │ • Wazuh Bridge │            │
│                              │ • Tools (20+)  │            │
│                              └───────┬────────┘            │
│                                      │                     │
└──────────────────────────────────────┼─────────────────────┘
                                       │
                          ┌────────────▼──────────────┐
                          │   Wazuh Cluster (HA)     │
                          │                          │
                          │  • Manager API :55000    │
                          │  • Indexer :9200         │
                          │  • Agents (clients)      │
                          └──────────────────────────┘
```

---

## ✨ Fonctionnalités Principales

### 🎨 Interface Client (Dashboard)

- **Bouclier Visuel** : Statut de protection en un coup d'œil
  - 🟢 Protégé : Tout va bien
  - 🟡 Attention : Actions recommandées
  - 🔴 Critique : Intervention nécessaire

- **Métriques Simplifiées**
  - Nombre de systèmes protégés
  - Score de conformité (%)
  - État des vulnérabilités

- **Journal de Bord**
  - Alertes traduites en français simple
  - Pas d'IPs, pas de logs bruts
  - "Ce que ça signifie pour mon entreprise"

- **Chatbot Concierge**
  - Répond aux questions en langage naturel
  - Ton rassurant et pédagogue
  - Powered by Gemini AI

### 🤖 Interface Admin (J.A.R.V.I.S.)

- **Commandes en Langage Naturel**
  ```
  "Bannis l'IP 192.168.1.100"
  "Montre-moi les vulnérabilités critiques"
  "Quel est l'état du cluster Wazuh ?"
  ```

- **Accès Complet aux Données**
  - Vue détaillée des alertes
  - Gestion des agents
  - Santé du cluster
  - Exécution d'outils MCP avancés

- **Chat Expert IA**
  - Analyse technique approfondie
  - Recommandations de remédiation
  - Documentation contextuelle

---

## 🚀 Stack Technologique

### Backend
- **Node.js 20** + **TypeScript 5**
- **Express.js** : API REST
- **JWT** : Authentication sécurisée
- **Google Gemini** : LLM pour IA conversationnelle
- **Winston** : Logging
- **Zod** : Validation

### Frontend
- **Next.js 14** (App Router)
- **React 18**
- **TypeScript 5**
- **Tailwind CSS** : Design system Dark Mode/Cyan
- **Framer Motion** : Animations fluides
- **Lucide React** : Icônes modernes

### MCP Server (Bridge Wazuh)
- **Rust** (stable)
- **rmcp** : Framework MCP
- **wazuh-client** : Client Wazuh natif

### Infrastructure
- **Docker** + **Docker Compose**
- **Nginx** : Reverse proxy (production)
- **Let's Encrypt** : SSL/TLS

---

## 📦 Structure du Projet

```
sentinelle-mcp/
├── backend/                    # API Node.js/TypeScript
│   ├── src/
│   │   ├── config/            # Configuration centralisée
│   │   ├── middleware/        # Auth, sécurité, validation
│   │   ├── routes/            # Routes API (auth, admin, client, portal)
│   │   ├── services/          # MCP client, LLM service
│   │   ├── types/             # Types TypeScript
│   │   └── server.ts          # Point d'entrée
│   ├── package.json
│   └── Dockerfile
│
├── frontend/                   # Application Next.js
│   ├── src/
│   │   ├── app/               # Pages Next.js 14 (App Router)
│   │   │   ├── login/
│   │   │   ├── dashboard/     # Dashboard client
│   │   │   └── admin/         # Interface J.A.R.V.I.S.
│   │   ├── components/        # Composants React
│   │   │   ├── Shield.tsx     # Bouclier visuel
│   │   │   ├── MetricCard.tsx
│   │   │   ├── AlertFeed.tsx
│   │   │   └── ChatInterface.tsx
│   │   ├── lib/               # Utilitaires (API client, auth)
│   │   └── styles/            # CSS global (Tailwind)
│   ├── package.json
│   └── Dockerfile
│
├── src/                        # MCP Server Rust (fork original)
│   ├── main.rs
│   ├── tools/                 # Outils Wazuh (alerts, agents, etc.)
│   └── lib.rs
│
├── docker-compose.yml          # Orchestration complète
├── .env.example                # Template de configuration
├── DEPLOYMENT.md               # Guide de déploiement détaillé
├── prp-prd                     # Document de vision/specs
└── README_APP.md               # Ce fichier
```

---

## 🎨 Design System

### Palette de Couleurs

```css
/* Dark Mode Base */
--bg-dark:          #0a0e1a  /* Fond principal */
--surface-dark:     #111827  /* Cartes/panneaux */
--surface-light:    #1f2937  /* Inputs, etc. */
--border:           #374151  /* Bordures */

/* Cyan Électrique - Signature Sentinelle */
--cyan-primary:     #22d3ee  /* Actions principales */
--cyan-glow:        rgba(34, 211, 238, 0.3)

/* Status Colors */
--status-protected: #10b981  /* Vert */
--status-warning:   #f59e0b  /* Orange */
--status-critical:  #ef4444  /* Rouge */
```

### Composants Clés

- **Glass Morphism** : Effet verre translucide avec backdrop-filter
- **Glow Effects** : Ombres portées cyan pour les CTA
- **Animations** : Transitions fluides avec Framer Motion
- **Grid Cyberpunk** : Grille de fond subtile (50px)

---

## 🔐 Sécurité

### Mesures Implémentées

1. **Authentication**
   - JWT avec expiration (24h par défaut)
   - Bcrypt pour les mots de passe (12 rounds)
   - Role-based access control (Admin/Client)

2. **Protection API**
   - Helmet.js : Headers de sécurité HTTP
   - CORS configuré strictement
   - Rate limiting (100 req/15min)
   - Input sanitization (suppression null bytes, XSS)

3. **Validation**
   - Express-validator pour les entrées
   - Zod pour les schémas TypeScript
   - Type safety complet

4. **Secrets Management**
   - Variables d'environnement (.env)
   - Jamais de secrets en dur dans le code
   - .gitignore pour tous les fichiers sensibles

### Bonnes Pratiques Production

```bash
# Générer un JWT secret fort
openssl rand -base64 64

# Utiliser HTTPS avec certificat Let's Encrypt
certbot --nginx -d sentinelle.votre-domaine.com

# Firewall strict
ufw allow 22/tcp   # SSH seulement
ufw allow 443/tcp  # HTTPS public
ufw deny 3000/tcp  # Frontend pas exposé directement
ufw deny 3001/tcp  # Backend pas exposé directement
```

---

## 🚦 Démarrage Rapide

### Prérequis

- Docker & Docker Compose
- Serveur Wazuh accessible
- Clé API Google Gemini
- Node.js 20+ (pour dev sans Docker)
- Rust stable (pour compiler le MCP server)

### Installation (3 minutes)

```bash
# 1. Compiler le MCP Server Rust
cargo build --release

# 2. Configurer l'environnement
cp .env.example .env
nano .env  # Éditer avec vos valeurs Wazuh et Gemini

# 3. Démarrer avec Docker Compose
docker-compose up -d

# 4. Vérifier
curl http://localhost:3001/health
# {"status":"healthy","timestamp":"..."}
```

### Accès

- **Application** : http://localhost:3000
- **Login** : `admin` / `<ADMIN_PASSWORD de .env>`

---

## 📊 Endpoints API Principaux

### Authentication

```bash
POST /api/v1/auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password"
}

Response:
{
  "token": "eyJhbGc...",
  "user": { "id": "...", "username": "admin", "role": "admin" }
}
```

### Portal (Client)

```bash
GET /api/v1/portal/summary/:groupId
Authorization: Bearer <token>

Response:
{
  "summary": {
    "groupId": "...",
    "agentStatuses": { "total": 10, "active": 9, "disconnected": 1 },
    "securityScore": { "sca": 85, "vulnerabilities": 90 },
    "recentAlerts": [...],
    "shieldStatus": "protected"
  }
}
```

### Admin Chat (J.A.R.V.I.S.)

```bash
POST /api/v1/admin/chat
Authorization: Bearer <admin-token>
Content-Type: application/json

{
  "message": "Bannis l'IP 192.168.1.100"
}

Response:
{
  "message": "Pour bannir l'IP 192.168.1.100, je vais...",
  "actions": ["ban_ip", "create_firewall_rule"],
  "timestamp": "..."
}
```

---

## 🧪 Tests et Qualité

### Linting

```bash
# Backend
cd backend
npm run lint

# Frontend
cd frontend
npm run lint
```

### Type Checking

```bash
# Backend
cd backend
npm run type-check

# Frontend
cd frontend
npm run type-check
```

### Build Verification

```bash
# Backend
cd backend
npm run build

# Frontend
cd frontend
npm run build
```

---

## 📈 Roadmap

### Phase 1 : MVP (✅ Complété)
- [x] Architecture backend complète
- [x] Interface client "Zéro Jargon"
- [x] Interface admin J.A.R.V.I.S.
- [x] Intégration LLM (Gemini)
- [x] Design Dark Mode/Cyan
- [x] Docker Compose

### Phase 2 : Production (En cours)
- [ ] Base de données (PostgreSQL pour les users)
- [ ] Système de notifications (email/SMS)
- [ ] Dashboard analytics avancé
- [ ] Export PDF de rapports
- [ ] Multi-tenant complet
- [ ] CI/CD GitHub Actions

### Phase 3 : Scale (Futur)
- [ ] Kubernetes deployment
- [ ] Cache Redis
- [ ] Queue système (Bull/RabbitMQ)
- [ ] Monitoring (Prometheus/Grafana)
- [ ] API webhooks pour intégrations

---

## 👥 Contribution

Ce projet suit les meilleures pratiques de développement :

1. **Branches** : `main` (production), `develop` (staging), `feature/*`
2. **Commits** : Messages clairs et descriptifs
3. **Code Review** : PR obligatoires avant merge
4. **Documentation** : Code commenté + docs à jour

---

## 📝 Licence

MIT License - Voir fichier `LICENSE`

---

## 🙏 Remerciements

- **Wazuh** : Pour le SIEM open-source incroyable
- **Google Gemini** : Pour l'IA conversationnelle
- **Model Context Protocol** : Pour le standard MCP
- **Communauté Rust & Node.js** : Pour les outils exceptionnels

---

## 📞 Support

- **Documentation complète** : Voir `DEPLOYMENT.md`
- **Issues** : GitHub Issues
- **Email** : admin@sentinelle.pro

---

**🛡️ Sentinelle-MCP - Votre Garde du Corps Numérique est prêt à protéger vos clients !**
