#!/bin/bash
# Script pour ralentir les tests E2E en ajoutant des pauses
# Usage: bash .claude/scripts/slow-down-tests.sh [delay_ms] [test_file]

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

DELAY=${1:-1000}  # Délai par défaut: 1000ms (1 seconde)
TEST_FILE=${2:-""}

echo -e "${BLUE}🐌 Ralentissement des tests E2E${NC}"
echo -e "${YELLOW}Délai entre chaque action: ${DELAY}ms${NC}"
echo ""

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TESTS_DIR="$PROJECT_ROOT/frontend/tests/e2e"

# Fonction pour ajouter des pauses dans un fichier
add_pauses_to_file() {
    local file="$1"
    local temp_file="${file}.tmp"

    echo -e "${BLUE}📝 Modification de $(basename "$file")...${NC}"

    # D'abord, supprimer tous les waitForTimeout existants pour éviter les doublons
    grep -v "await page\.waitForTimeout(" "$file" > "${temp_file}.clean" || cp "$file" "${temp_file}.clean"

    # Ensuite, ajouter page.waitForTimeout() après chaque action
    # Actions à ralentir: goto, click, fill, press, getByRole().click/fill
    sed -E "
        # Après page.goto()
        s/(await page\.goto\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après page.click()
        s/(await page\.click\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après page.fill()
        s/(await page\.fill\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après page.press()
        s/(await page\.press\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après getByRole().click(), getByText().click(), etc.
        s/(await page\.get[^(]+\([^)]+\)\.click\(\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après getByRole().fill()
        s/(await page\.get[^(]+\([^)]+\)\.fill\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g

        # Après getByRole().press()
        s/(await page\.get[^(]+\([^)]+\)\.press\([^)]+\);)/\1\n  await page.waitForTimeout(${DELAY});/g
    " "${temp_file}.clean" > "$temp_file"

    # Remplacer le fichier original
    mv "$temp_file" "$file"
    rm -f "${temp_file}.clean"

    echo -e "${GREEN}✅ Pauses ajoutées${NC}"
}

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

# Si un fichier spécifique est donné
if [ -n "$TEST_FILE" ]; then
    if [ ! -f "$TEST_FILE" ]; then
        echo -e "${RED}❌ Fichier non trouvé: $TEST_FILE${NC}"
        exit 1
    fi

    add_pauses_to_file "$TEST_FILE"
else
    # Traiter tous les fichiers .spec.ts
    echo -e "${YELLOW}Traitement de tous les tests dans $TESTS_DIR${NC}"
    echo ""

    count=0
    for test_file in "$TESTS_DIR"/*.spec.ts; do
        if [ -f "$test_file" ]; then
            add_pauses_to_file "$test_file"
            ((count++))
        fi
    done

    echo ""
    echo -e "${GREEN}✅ $count fichiers modifiés${NC}"
fi

echo ""
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Ralentissement terminé!${NC}"
echo -e "${GREEN}════════════════════════════════════════${NC}"
echo ""
echo -e "💡 Pour retirer les pauses:"
echo -e "   ${BLUE}make test-e2e-restore-speed${NC}"
echo ""
echo -e "💡 Pour lancer les tests ralentis:"
echo -e "   ${BLUE}cd frontend && npm run test:e2e${NC}"
echo ""
