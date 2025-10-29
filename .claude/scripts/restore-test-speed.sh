#!/bin/bash
# Script pour retirer les pauses ajoutées par slow-down-tests.sh
# Usage: bash .claude/scripts/restore-test-speed.sh [test_file]

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

TEST_FILE=${1:-""}

echo -e "${BLUE}⚡ Restauration de la vitesse normale des tests${NC}"
echo ""

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTS_DIR="$PROJECT_ROOT/frontend/tests/e2e"

# Fonction pour retirer les pauses
remove_pauses_from_file() {
    local file="$1"
    local temp_file="${file}.tmp"

    echo -e "${BLUE}📝 Nettoyage de $(basename "$file")...${NC}"

    # Supprimer toutes les lignes page.waitForTimeout()
    grep -v "await page\.waitForTimeout(" "$file" > "$temp_file" || true

    mv "$temp_file" "$file"

    echo -e "${GREEN}✅ Pauses retirées${NC}"
}

if [ -n "$TEST_FILE" ]; then
    if [ ! -f "$TEST_FILE" ]; then
        echo -e "${RED}❌ Fichier non trouvé: $TEST_FILE${NC}"
        exit 1
    fi

    remove_pauses_from_file "$TEST_FILE"
else
    # Traiter tous les fichiers
    count=0
    for test_file in "$TESTS_DIR"/*.spec.ts; do
        if [ -f "$test_file" ]; then
            remove_pauses_from_file "$test_file"
            ((count++))
        fi
    done

    echo ""
    echo -e "${GREEN}✅ $count fichiers nettoyés${NC}"
fi

echo ""
echo -e "${GREEN}✅ Vitesse normale restaurée!${NC}"
echo ""
