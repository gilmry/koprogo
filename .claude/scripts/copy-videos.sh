#!/bin/bash
# Script simplifié pour copier les vidéos E2E
set -e

echo "📹 Copie des vidéos Playwright → docs/_static/videos/"

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
TEST_RESULTS="$PROJECT_ROOT/frontend/test-results"
VIDEOS_DIR="$PROJECT_ROOT/docs/_static/videos"

# Créer le répertoire si nécessaire
mkdir -p "$VIDEOS_DIR"

# Vérifier si des vidéos existent
if [ ! -d "$TEST_RESULTS" ]; then
    echo "❌ Répertoire test-results non trouvé"
    echo "💡 Lancez d'abord: cd frontend && npm run test:e2e"
    exit 1
fi

VIDEO_COUNT=$(find "$TEST_RESULTS" -name "video.webm" 2>/dev/null | wc -l)
if [ "$VIDEO_COUNT" -eq 0 ]; then
    echo "⚠️  Aucune vidéo trouvée"
    echo "💡 Lancez d'abord: cd frontend && npm run test:e2e"
    exit 0
fi

# Nettoyer les anciennes vidéos
rm -f "$VIDEOS_DIR"/*.webm

# Copier les vidéos avec des noms basés sur le répertoire de test
counter=0
while IFS= read -r video_path; do
    # Extraire le nom du répertoire parent (contient le nom du test)
    dir_name=$(basename "$(dirname "$video_path")")
    # Nettoyer le suffixe -chromium
    clean_name="${dir_name%-chromium}"

    # Copier
    cp "$video_path" "$VIDEOS_DIR/${clean_name}.webm"
    counter=$((counter+1))
done < <(find "$TEST_RESULTS" -name "video.webm")

echo "✅ $counter vidéos copiées dans docs/_static/videos/"
echo ""
echo "🔄 Régénération de la page RST..."
python3 "$PROJECT_ROOT/.claude/scripts/generate-video-rst.py"

echo ""
echo "✅ Terminé ! Les vidéos sont prêtes pour la documentation."
echo "💡 Prochaine étape: cd docs && make html"
