# Guide de Déploiement - Sentinelle-MCP

## Vue d'ensemble

Sentinelle-MCP est une plateforme complète de sécurité managée comprenant :

- **Backend API** (Node.js/TypeScript) : Serveur REST avec intégration MCP et LLM
- **Frontend** (Next.js/React) : Interface utilisateur Dark Mode/Cyan
- **MCP Server** (Rust) : Bridge vers Wazuh SIEM
- **Wazuh** (Externe) : Cluster SIEM pour la surveillance de sécurité

## Prérequis

### Infrastructure

1. **Serveur Wazuh** (déjà existant)
   - Wazuh Manager API (port 55000)
   - Wazuh Indexer (port 9200)
   - Credentials d'accès

2. **Serveur Application**
   - Docker et Docker Compose installés
   - Minimum 2 CPU, 4GB RAM
   - 20GB de stockage

3. **Services Externes**
   - Compte Google Cloud (pour Gemini API)
   - Nom de domaine (optionnel, pour production)

## Installation Rapide (Développement)

### 1. Cloner et Préparer

```bash
git clone https://github.com/votre-org/sentinelle-mcp.git
cd sentinelle-mcp
```

### 2. Compiler le MCP Server (Rust)

```bash
# Le serveur MCP Rust doit être compilé avant le déploiement
cargo build --release

# Le binaire sera dans target/release/mcp-server-wazuh
```

### 3. Configuration

```bash
# Copier le fichier d'exemple
cp .env.example .env

# Éditer avec vos valeurs
nano .env
```

**Variables critiques à configurer :**

```env
# Sécurité (OBLIGATOIRE)
JWT_SECRET=<générer-une-clé-aléatoire-longue>
ADMIN_PASSWORD=<mot-de-passe-admin-sécurisé>

# Wazuh
WAZUH_API_HOST=votre-wazuh-server.com
WAZUH_API_USERNAME=votre-user
WAZUH_API_PASSWORD=votre-password
WAZUH_INDEXER_HOST=votre-indexer.com
WAZUH_INDEXER_USERNAME=admin
WAZUH_INDEXER_PASSWORD=admin-password

# LLM
GEMINI_API_KEY=votre-clé-api-gemini
```

### 4. Démarrage avec Docker Compose

```bash
# Construire et démarrer tous les services
docker-compose up -d

# Vérifier les logs
docker-compose logs -f

# Vérifier l'état
docker-compose ps
```

### 5. Accès à l'Application

- **Frontend** : http://localhost:3000
- **Backend API** : http://localhost:3001
- **Health Check** : http://localhost:3001/health

**Identifiants par défaut** :
- Username : `admin` (configuré dans .env)
- Password : `<ADMIN_PASSWORD de .env>`

## Installation Manuelle (Sans Docker)

### Backend

```bash
cd backend

# Installer les dépendances
npm install

# Copier et configurer .env
cp .env.example .env
nano .env

# Compiler TypeScript
npm run build

# Démarrer en développement
npm run dev

# Ou en production
npm start
```

### Frontend

```bash
cd frontend

# Installer les dépendances
npm install

# Copier et configurer .env
cp .env.example .env
nano .env

# Démarrer en développement
npm run dev

# Ou compiler pour production
npm run build
npm start
```

## Configuration Wazuh

Le serveur MCP nécessite l'accès à :

1. **Wazuh Manager API** (port 55000)
   - Endpoint : `https://<wazuh-host>:55000`
   - Authentification : Basic Auth
   - Permissions : Lecture des agents, alertes, règles, vulnérabilités

2. **Wazuh Indexer** (port 9200)
   - Endpoint : `https://<indexer-host>:9200`
   - Authentification : Basic Auth
   - Permissions : Lecture des index wazuh-alerts-*

### Créer un Utilisateur API Wazuh

```bash
# Sur le serveur Wazuh Manager
/var/ossec/bin/wazuh-authd -u <username> -p <password>

# Vérifier les permissions
curl -u <username>:<password> -k -X GET "https://localhost:55000/"
```

## Configuration LLM (Gemini)

### 1. Obtenir une Clé API Google Gemini

1. Aller sur : https://makersuite.google.com/app/apikey
2. Créer une nouvelle clé API
3. Copier la clé dans `.env` : `GEMINI_API_KEY=...`

### Alternative : Mistral AI

```env
LLM_PROVIDER=mistral
MISTRAL_API_KEY=votre-clé-mistral
```

## Sécurité en Production

### 1. Variables d'Environnement Sensibles

**IMPORTANT** : Ne JAMAIS commiter les fichiers `.env` !

```bash
# Générer un JWT secret sécurisé
openssl rand -base64 64

# Utiliser un gestionnaire de secrets (ex: Docker Secrets, Vault)
```

### 2. HTTPS/TLS

Pour la production, utilisez un reverse proxy (Nginx/Traefik) avec Let's Encrypt :

```nginx
# Exemple Nginx
server {
    listen 443 ssl http2;
    server_name sentinelle.votre-domaine.com;

    ssl_certificate /etc/letsencrypt/live/sentinelle.votre-domaine.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/sentinelle.votre-domaine.com/privkey.pem;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    location /api {
        proxy_pass http://localhost:3001;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### 3. Firewall

```bash
# Autoriser seulement les ports nécessaires
ufw allow 22/tcp    # SSH
ufw allow 443/tcp   # HTTPS
ufw allow 3000/tcp  # Frontend (si pas de reverse proxy)
ufw allow 3001/tcp  # Backend API (si pas de reverse proxy)
ufw enable
```

### 4. Mise à Jour des Dépendances

```bash
# Backend
cd backend && npm audit fix

# Frontend
cd frontend && npm audit fix

# MCP Server Rust
cargo update
cargo audit
```

## Gestion des Utilisateurs

### Créer un Client via API

```bash
curl -X POST http://localhost:3001/api/v1/auth/register-client \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <admin-token>" \
  -d '{
    "username": "client-pme",
    "password": "SecurePassword123!",
    "email": "contact@pme-client.fr",
    "groupId": "group-pme-001"
  }'
```

### Authentification JWT

Les tokens JWT expirent après 24h par défaut. Configurer via `JWT_EXPIRES_IN` dans `.env`.

## Monitoring et Logs

### Logs Docker

```bash
# Tous les services
docker-compose logs -f

# Backend seulement
docker-compose logs -f backend

# Frontend seulement
docker-compose logs -f frontend
```

### Logs Applicatifs

- **Backend** : `backend/logs/sentinelle.log`
- **Frontend** : Logs dans stdout du container

### Health Checks

```bash
# Backend
curl http://localhost:3001/health

# Cluster Wazuh (via Backend API)
curl -H "Authorization: Bearer <token>" \
  http://localhost:3001/api/v1/admin/cluster/health
```

## Sauvegarde et Restauration

### Données à Sauvegarder

1. **Configuration** : fichiers `.env`
2. **Base de données utilisateurs** : À implémenter (actuellement en mémoire)
3. **Logs** : `backend/logs/`

### Restauration

```bash
# Restaurer la configuration
cp backup/.env .env

# Redémarrer les services
docker-compose down
docker-compose up -d
```

## Dépannage

### Le backend ne démarre pas

```bash
# Vérifier les logs
docker-compose logs backend

# Vérifier que le MCP server existe
ls -lh target/release/mcp-server-wazuh

# Recompiler si nécessaire
cargo build --release
```

### Erreur de connexion à Wazuh

```bash
# Tester manuellement la connexion
curl -u <username>:<password> -k \
  https://<wazuh-host>:55000/

# Vérifier les variables d'environnement
docker-compose exec backend env | grep WAZUH
```

### Le LLM ne répond pas

```bash
# Vérifier la clé API Gemini
curl -H "x-goog-api-key: $GEMINI_API_KEY" \
  https://generativelanguage.googleapis.com/v1/models

# Vérifier les quotas Google Cloud
```

### Erreur "Token expired"

Les tokens JWT expirent. L'utilisateur doit se reconnecter. Pour prolonger :

```env
JWT_EXPIRES_IN=7d  # 7 jours au lieu de 24h
```

## Architecture de Déploiement Production

```
┌─────────────────────────────────────────────────┐
│  Internet                                       │
└────────────────┬────────────────────────────────┘
                 │
         ┌───────▼──────┐
         │  CloudFlare  │ (CDN + DDoS Protection)
         │  ou WAF      │
         └───────┬──────┘
                 │
         ┌───────▼──────┐
         │  Nginx/      │ (Reverse Proxy + SSL)
         │  Traefik     │
         └───┬──────┬───┘
             │      │
    ┌────────▼──┐ ┌▼────────────┐
    │ Frontend  │ │  Backend    │
    │ (Next.js) │ │  (Node.js)  │
    │ Port 3000 │ │  Port 3001  │
    └───────────┘ └──────┬──────┘
                         │
                  ┌──────▼──────┐
                  │  MCP Server │
                  │  (Rust)     │
                  └──────┬──────┘
                         │
              ┌──────────▼──────────┐
              │  Wazuh Cluster      │
              │  (Externe)          │
              │  - Manager:55000    │
              │  - Indexer:9200     │
              └─────────────────────┘
```

## Support et Ressources

- **Documentation Wazuh** : https://documentation.wazuh.com
- **MCP Protocol** : https://modelcontextprotocol.io
- **Google Gemini API** : https://ai.google.dev/docs

## Licence

MIT License - Voir LICENSE file
