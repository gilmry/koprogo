#!/bin/bash
#
# Script pour activer/désactiver le rate limiting de l'API
# Usage: ./toggle-rate-limiting.sh [on|off|status]
#

set -euo pipefail

ENV_FILE="${ENV_FILE:-../backend/.env.vps}"
COMPOSE_FILE="${COMPOSE_FILE:-../docker-compose.vps.yml}"

# Couleurs
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Fonction pour afficher le status actuel
show_status() {
    echo -e "${YELLOW}Checking rate limiting status...${NC}"

    if [ -f "$ENV_FILE" ]; then
        if grep -q "^ENABLE_RATE_LIMITING=false" "$ENV_FILE"; then
            echo -e "${RED}⚠️  Rate limiting is DISABLED${NC}"
            echo "   (Not safe for production!)"
            return 1
        elif grep -q "^ENABLE_RATE_LIMITING=true" "$ENV_FILE"; then
            echo -e "${GREEN}✅ Rate limiting is ENABLED${NC}"
            return 0
        else
            echo -e "${GREEN}✅ Rate limiting is ENABLED (default)${NC}"
            return 0
        fi
    else
        echo -e "${YELLOW}⚠️  Env file not found: $ENV_FILE${NC}"
        echo -e "${GREEN}✅ Rate limiting is ENABLED (default)${NC}"
        return 0
    fi
}

# Fonction pour désactiver le rate limiting
disable_rate_limiting() {
    echo -e "${YELLOW}Disabling rate limiting...${NC}"

    if [ ! -f "$ENV_FILE" ]; then
        echo -e "${RED}Error: Env file not found: $ENV_FILE${NC}"
        exit 1
    fi

    # Supprimer les anciennes entrées
    sed -i '/^ENABLE_RATE_LIMITING=/d' "$ENV_FILE"

    # Ajouter la nouvelle configuration
    echo "ENABLE_RATE_LIMITING=false" >> "$ENV_FILE"

    echo -e "${GREEN}✅ Rate limiting disabled${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  Don't forget to restart the backend:${NC}"
    echo "   docker compose -f $COMPOSE_FILE restart backend"
    echo ""
    echo -e "${RED}⚠️  IMPORTANT: Re-enable after load testing!${NC}"
    echo "   ./toggle-rate-limiting.sh on"
}

# Fonction pour activer le rate limiting
enable_rate_limiting() {
    echo -e "${YELLOW}Enabling rate limiting...${NC}"

    if [ ! -f "$ENV_FILE" ]; then
        echo -e "${RED}Error: Env file not found: $ENV_FILE${NC}"
        exit 1
    fi

    # Supprimer l'entrée ENABLE_RATE_LIMITING (utilise la valeur par défaut = true)
    sed -i '/^ENABLE_RATE_LIMITING=/d' "$ENV_FILE"

    echo -e "${GREEN}✅ Rate limiting enabled (using default)${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  Don't forget to restart the backend:${NC}"
    echo "   docker compose -f $COMPOSE_FILE restart backend"
}

# Main
case "${1:-status}" in
    off|disable)
        disable_rate_limiting
        ;;
    on|enable)
        enable_rate_limiting
        ;;
    status|check)
        show_status
        ;;
    *)
        echo "Usage: $0 [on|off|status]"
        echo ""
        echo "Commands:"
        echo "  on, enable   - Enable rate limiting (production mode)"
        echo "  off, disable - Disable rate limiting (load testing mode)"
        echo "  status       - Show current status (default)"
        echo ""
        echo "Environment variables:"
        echo "  ENV_FILE     - Path to .env file (default: ../backend/.env.vps)"
        echo "  COMPOSE_FILE - Path to docker-compose file (default: ../docker-compose.vps.yml)"
        echo ""
        echo "Examples:"
        echo "  $0 off                                    # Disable for load testing"
        echo "  $0 on                                     # Re-enable for production"
        echo "  ENV_FILE=../backend/.env $0 status        # Check local dev status"
        exit 1
        ;;
esac
