#!/bin/bash

# ==============================================
# SENTINELLE-MCP - Quick Start Script
# ==============================================

set -e  # Exit on error

echo "🛡️  SENTINELLE-MCP - Installation Rapide"
echo "========================================"
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if .env exists
if [ ! -f ".env" ]; then
    echo -e "${YELLOW}⚠️  Fichier .env non trouvé${NC}"
    echo "Création depuis .env.example..."

    if [ -f ".env.example" ]; then
        cp .env.example .env
        echo -e "${GREEN}✓ Fichier .env créé${NC}"
        echo ""
        echo -e "${YELLOW}⚠️  IMPORTANT: Éditez le fichier .env avec vos configurations :${NC}"
        echo "   - JWT_SECRET (générez une clé aléatoire longue)"
        echo "   - ADMIN_PASSWORD (mot de passe admin sécurisé)"
        echo "   - WAZUH_API_HOST (votre serveur Wazuh)"
        echo "   - WAZUH_API_USERNAME et WAZUH_API_PASSWORD"
        echo "   - WAZUH_INDEXER_HOST"
        echo "   - GEMINI_API_KEY (clé API Google Gemini)"
        echo ""
        echo "Appuyez sur Entrée une fois que vous avez édité .env..."
        read
    else
        echo -e "${RED}✗ Fichier .env.example non trouvé !${NC}"
        exit 1
    fi
fi

# Check if MCP server binary exists
echo "Vérification du serveur MCP..."
if [ ! -f "target/release/mcp-server-wazuh" ]; then
    echo -e "${YELLOW}⚠️  Binaire MCP non trouvé. Compilation...${NC}"
    cargo build --release
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ MCP Server compilé avec succès${NC}"
    else
        echo -e "${RED}✗ Erreur lors de la compilation du MCP Server${NC}"
        exit 1
    fi
else
    echo -e "${GREEN}✓ MCP Server trouvé${NC}"
fi

# Check Docker
echo "Vérification de Docker..."
if ! command -v docker &> /dev/null; then
    echo -e "${RED}✗ Docker n'est pas installé !${NC}"
    echo "Installez Docker : https://docs.docker.com/get-docker/"
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo -e "${RED}✗ Docker Compose n'est pas installé !${NC}"
    echo "Installez Docker Compose : https://docs.docker.com/compose/install/"
    exit 1
fi

echo -e "${GREEN}✓ Docker installé${NC}"

# Stop existing containers
echo ""
echo "Arrêt des conteneurs existants..."
docker-compose down 2>/dev/null || true

# Build and start
echo ""
echo -e "${CYAN}🚀 Construction et démarrage des services...${NC}"
docker-compose up --build -d

# Wait for services
echo ""
echo "Attente du démarrage des services..."
sleep 5

# Check health
echo ""
echo "Vérification de l'état des services..."

# Backend health check
for i in {1..10}; do
    if curl -s http://localhost:3001/health > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Backend API : OK${NC}"
        break
    else
        if [ $i -eq 10 ]; then
            echo -e "${RED}✗ Backend API : ERREUR (timeout)${NC}"
            echo "Vérifiez les logs : docker-compose logs backend"
        else
            echo "   Attente du backend... ($i/10)"
            sleep 2
        fi
    fi
done

# Frontend health check
for i in {1..10}; do
    if curl -s http://localhost:3000 > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Frontend : OK${NC}"
        break
    else
        if [ $i -eq 10 ]; then
            echo -e "${RED}✗ Frontend : ERREUR (timeout)${NC}"
            echo "Vérifiez les logs : docker-compose logs frontend"
        else
            echo "   Attente du frontend... ($i/10)"
            sleep 2
        fi
    fi
done

# Success
echo ""
echo -e "${GREEN}╔═══════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║                                                   ║${NC}"
echo -e "${GREEN}║   🎉  SENTINELLE-MCP EST PRÊT !                  ║${NC}"
echo -e "${GREEN}║                                                   ║${NC}"
echo -e "${GREEN}╚═══════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Accès à l'application :${NC}"
echo "   🌐 Frontend    : http://localhost:3000"
echo "   🔌 Backend API : http://localhost:3001"
echo "   ❤️  Health     : http://localhost:3001/health"
echo ""
echo -e "${CYAN}Identifiants par défaut :${NC}"
echo "   Username : admin"
echo "   Password : (voir ADMIN_PASSWORD dans .env)"
echo ""
echo -e "${CYAN}Commandes utiles :${NC}"
echo "   Logs         : docker-compose logs -f"
echo "   Arrêt        : docker-compose down"
echo "   Redémarrage  : docker-compose restart"
echo ""
echo -e "${YELLOW}📖 Documentation complète : DEPLOYMENT.md${NC}"
echo ""
