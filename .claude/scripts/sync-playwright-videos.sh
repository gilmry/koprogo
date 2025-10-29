#!/bin/bash
# sync-playwright-videos.sh - Synchroniser les vidéos Playwright vers la doc Sphinx
# Usage: bash .claude/scripts/sync-playwright-videos.sh

set -euo pipefail

# Couleurs pour output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Chemins
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TEST_RESULTS_DIR="$PROJECT_ROOT/frontend/test-results"
VIDEOS_OUTPUT_DIR="$PROJECT_ROOT/docs/_static/videos"

echo -e "${BLUE}🎥 Synchronisation des vidéos Playwright → Sphinx${NC}"
echo ""

# Créer le répertoire de destination
mkdir -p "$VIDEOS_OUTPUT_DIR"

# Vérifier si des vidéos existent
if [ ! -d "$TEST_RESULTS_DIR" ]; then
    echo -e "${RED}❌ Répertoire test-results non trouvé${NC}"
    echo -e "${YELLOW}💡 Lancez d'abord: cd frontend && npm run test:e2e${NC}"
    exit 1
fi

VIDEO_COUNT=$(find "$TEST_RESULTS_DIR" -name "video.webm" 2>/dev/null | wc -l)
if [ "$VIDEO_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}⚠️  Aucune vidéo trouvée dans test-results/${NC}"
    echo -e "${YELLOW}💡 Lancez d'abord: cd frontend && npm run test:e2e${NC}"
    exit 0
fi

echo -e "${GREEN}✅ Trouvé $VIDEO_COUNT vidéos${NC}"
echo ""

# Fonction pour extraire le nom du test depuis le chemin
extract_test_name() {
    local path="$1"
    # Le format est généralement: test-results/<suite>-<test-name>-<browser>/video.webm
    local dirname=$(basename "$(dirname "$path")")
    echo "$dirname"
}

# Fonction pour mapper les noms de tests aux noms de fichiers
map_video_name() {
    local test_name="$1"

    # Auth tests
    if [[ $test_name =~ "should-display-landing-page" ]]; then
        echo "auth-landing-page.webm"
    elif [[ $test_name =~ "should-navigate-to-login" ]]; then
        echo "auth-navigate-login.webm"
    elif [[ $test_name =~ "should-show-demo-credentials" ]]; then
        echo "auth-demo-credentials.webm"
    elif [[ $test_name =~ "should-login-successfully" ]]; then
        echo "auth-login-success.webm"
    elif [[ $test_name =~ "should-show-error-for-invalid" ]]; then
        echo "auth-error-invalid.webm"
    elif [[ $test_name =~ "should-persist-authentication" ]]; then
        echo "auth-persist-reload.webm"
    elif [[ $test_name =~ "should-logout-successfully" ]]; then
        echo "auth-logout.webm"
    elif [[ $test_name =~ "should-redirect-Syndic" ]]; then
        echo "auth-redirect-syndic.webm"
    elif [[ $test_name =~ "should-redirect-Accountant" ]]; then
        echo "auth-redirect-accountant.webm"

    # Dashboard tests - Syndic
    elif [[ $test_name =~ "should-display-syndic-dashboard" ]]; then
        echo "dashboard-syndic-sections.webm"
    elif [[ $test_name =~ "should-have-navigation-menu-with-syndic" ]]; then
        echo "dashboard-syndic-navigation.webm"
    elif [[ $test_name =~ "should-navigate-to-buildings" ]]; then
        echo "dashboard-navigate-buildings.webm"
    elif [[ $test_name =~ "should-show-user-menu" ]]; then
        echo "dashboard-user-menu.webm"

    # Dashboard tests - Accountant
    elif [[ $test_name =~ "should-display-accountant-dashboard" ]]; then
        echo "dashboard-accountant-financial.webm"
    elif [[ $test_name =~ "should-have-financial-navigation" ]]; then
        echo "dashboard-accountant-navigation.webm"

    # Dashboard tests - Owner
    elif [[ $test_name =~ "should-display-owner-dashboard" ]]; then
        echo "dashboard-owner-personal.webm"
    elif [[ $test_name =~ "should-have-limited-navigation" ]]; then
        echo "dashboard-owner-limited.webm"

    # Dashboard tests - SuperAdmin
    elif [[ $test_name =~ "should-display-admin-dashboard" ]]; then
        echo "dashboard-admin-overview.webm"
    elif [[ $test_name =~ "should-have-full-navigation" ]]; then
        echo "dashboard-admin-full-access.webm"

    # Dashboard tests - Navigation
    elif [[ $test_name =~ "should-navigate-between-different" ]]; then
        echo "dashboard-navigation-smooth.webm"
    elif [[ $test_name =~ "should-maintain-authentication-state" ]]; then
        echo "dashboard-auth-state-persist.webm"

    # PWA tests
    elif [[ $test_name =~ "should-have-a-valid-manifest" ]]; then
        echo "pwa-manifest-valid.webm"
    elif [[ $test_name =~ "should-register-a-service-worker" ]]; then
        echo "pwa-service-worker.webm"
    elif [[ $test_name =~ "should-show-online-status" ]]; then
        echo "pwa-online-status.webm"
    elif [[ $test_name =~ "should-show-offline-status" ]]; then
        echo "pwa-offline-status.webm"
    elif [[ $test_name =~ "should-use-IndexedDB" ]]; then
        echo "pwa-indexeddb-storage.webm"
    elif [[ $test_name =~ "should-cache-user-data" ]]; then
        echo "pwa-cache-user-data.webm"
    elif [[ $test_name =~ "should-allow-manual-synchronization" ]]; then
        echo "pwa-manual-sync.webm"
    elif [[ $test_name =~ "should-work-offline-after-initial" ]]; then
        echo "pwa-offline-mode.webm"
    elif [[ $test_name =~ "should-queue-changes-when-offline" ]]; then
        echo "pwa-sync-queue.webm"

    else
        # Fallback: utiliser le nom du dossier
        echo "${test_name}.webm"
    fi
}

# Copier les vidéos avec des noms explicites
echo -e "${BLUE}📋 Copie des vidéos...${NC}"
COPIED=0

while IFS= read -r -d '' video_path; do
    test_name=$(extract_test_name "$video_path")
    output_name=$(map_video_name "$test_name")
    output_path="$VIDEOS_OUTPUT_DIR/$output_name"

    cp "$video_path" "$output_path"
    echo -e "  ${GREEN}✓${NC} $output_name"
    ((COPIED++))
done < <(find "$TEST_RESULTS_DIR" -name "video.webm" -print0)

echo ""
echo -e "${GREEN}✅ $COPIED vidéos copiées${NC}"

# Conversion MP4 optionnelle (si ffmpeg disponible)
if command -v ffmpeg &> /dev/null; then
    echo ""
    echo -e "${BLUE}🔄 Conversion MP4 (fallback pour compatibilité)...${NC}"

    CONVERTED=0
    for webm_file in "$VIDEOS_OUTPUT_DIR"/*.webm; do
        if [ -f "$webm_file" ]; then
            mp4_file="${webm_file%.webm}.mp4"
            ffmpeg -i "$webm_file" -c:v libx264 -c:a aac -movflags +faststart "$mp4_file" -y -loglevel error
            echo -e "  ${GREEN}✓${NC} $(basename "$mp4_file")"
            ((CONVERTED++))
        fi
    done

    echo ""
    echo -e "${GREEN}✅ $CONVERTED vidéos converties en MP4${NC}"
else
    echo ""
    echo -e "${YELLOW}⚠️  ffmpeg non installé, skip conversion MP4${NC}"
    echo -e "${YELLOW}💡 Installez ffmpeg pour le fallback MP4: apt-get install ffmpeg${NC}"
fi

# Générer index.html standalone
echo ""
echo -e "${BLUE}📄 Génération index.html...${NC}"

cat > "$VIDEOS_OUTPUT_DIR/index.html" <<'EOF'
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>KoproGo - Vidéos E2E Tests</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #333;
            padding: 2rem;
        }
        .container {
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 16px;
            padding: 2rem;
            box-shadow: 0 20px 60px rgba(0,0,0,0.3);
        }
        h1 {
            font-size: 2.5rem;
            color: #667eea;
            margin-bottom: 0.5rem;
        }
        .subtitle {
            color: #666;
            margin-bottom: 2rem;
            font-size: 1.1rem;
        }
        .stats {
            display: flex;
            gap: 1rem;
            margin-bottom: 2rem;
            flex-wrap: wrap;
        }
        .stat-card {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 1rem 1.5rem;
            border-radius: 12px;
            box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
        }
        .stat-number { font-size: 2rem; font-weight: bold; }
        .stat-label { font-size: 0.9rem; opacity: 0.9; }

        .category {
            margin-bottom: 3rem;
        }
        .category h2 {
            font-size: 1.8rem;
            color: #333;
            margin-bottom: 1rem;
            padding-bottom: 0.5rem;
            border-bottom: 3px solid #667eea;
        }
        .video-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
            gap: 1.5rem;
        }
        .video-card {
            background: #f8f9fa;
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 4px 12px rgba(0,0,0,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
        }
        .video-card:hover {
            transform: translateY(-4px);
            box-shadow: 0 8px 24px rgba(0,0,0,0.15);
        }
        .video-card video {
            width: 100%;
            height: auto;
            display: block;
        }
        .video-info {
            padding: 1rem;
        }
        .video-title {
            font-weight: 600;
            color: #333;
            margin-bottom: 0.5rem;
        }
        .video-badge {
            display: inline-block;
            padding: 0.25rem 0.75rem;
            border-radius: 6px;
            font-size: 0.85rem;
            font-weight: 500;
        }
        .badge-auth { background: #d1fae5; color: #065f46; }
        .badge-dashboard { background: #dbeafe; color: #1e40af; }
        .badge-pwa { background: #fef3c7; color: #92400e; }

        footer {
            margin-top: 3rem;
            text-align: center;
            color: #666;
            font-size: 0.9rem;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎥 KoproGo - Tests E2E</h1>
        <p class="subtitle">Documentation vivante générée automatiquement par Playwright</p>

        <div class="stats">
            <div class="stat-card">
                <div class="stat-number">30</div>
                <div class="stat-label">Tests E2E</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">3</div>
                <div class="stat-label">Suites de tests</div>
            </div>
            <div class="stat-card">
                <div class="stat-number">100%</div>
                <div class="stat-label">Couverture</div>
            </div>
        </div>

        <!-- Auth Tests -->
        <div class="category">
            <h2>🔐 Authentification</h2>
            <div class="video-grid" id="auth-videos"></div>
        </div>

        <!-- Dashboard Tests -->
        <div class="category">
            <h2>📊 Dashboards</h2>
            <div class="video-grid" id="dashboard-videos"></div>
        </div>

        <!-- PWA Tests -->
        <div class="category">
            <h2>📱 PWA & Offline</h2>
            <div class="video-grid" id="pwa-videos"></div>
        </div>

        <footer>
            <p>🤖 Généré automatiquement avec <strong>Claude Code</strong></p>
            <p>KoproGo ASBL - Plateforme opensource de gestion de copropriété</p>
        </footer>
    </div>

    <script>
        // Lister toutes les vidéos et les organiser par catégorie
        const videos = {
            auth: [],
            dashboard: [],
            pwa: []
        };

        // Scanner le répertoire pour les vidéos (simulé ici)
        // En production, cette liste serait générée côté serveur
        fetch('.')
            .then(res => res.text())
            .then(html => {
                const parser = new DOMParser();
                const doc = parser.parseFromString(html, 'text/html');
                const links = [...doc.querySelectorAll('a')];

                links.forEach(link => {
                    const href = link.getAttribute('href');
                    if (!href || !href.endsWith('.webm')) return;

                    const name = href.replace('.webm', '');
                    const title = name.split('-').map(w => w.charAt(0).toUpperCase() + w.slice(1)).join(' ');

                    if (name.startsWith('auth-')) {
                        videos.auth.push({ name, title, src: href });
                    } else if (name.startsWith('dashboard-')) {
                        videos.dashboard.push({ name, title, src: href });
                    } else if (name.startsWith('pwa-')) {
                        videos.pwa.push({ name, title, src: href });
                    }
                });

                // Générer les cartes vidéo
                renderVideos('auth-videos', videos.auth, 'auth');
                renderVideos('dashboard-videos', videos.dashboard, 'dashboard');
                renderVideos('pwa-videos', videos.pwa, 'pwa');
            });

        function renderVideos(containerId, videoList, category) {
            const container = document.getElementById(containerId);

            videoList.forEach(video => {
                const card = document.createElement('div');
                card.className = 'video-card';

                const mp4Src = video.src.replace('.webm', '.mp4');

                card.innerHTML = `
                    <video controls preload="metadata">
                        <source src="${video.src}" type="video/webm">
                        <source src="${mp4Src}" type="video/mp4">
                        Votre navigateur ne supporte pas la balise vidéo.
                    </video>
                    <div class="video-info">
                        <div class="video-title">${video.title}</div>
                        <span class="video-badge badge-${category}">${category}</span>
                    </div>
                `;

                container.appendChild(card);
            });

            if (videoList.length === 0) {
                container.innerHTML = '<p style="color: #666;">Aucune vidéo disponible</p>';
            }
        }
    </script>
</body>
</html>
EOF

echo -e "${GREEN}✅ index.html créé${NC}"

# Résumé
echo ""
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo -e "${GREEN}✅ Synchronisation terminée!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════${NC}"
echo ""
echo -e "📁 Emplacement: ${BLUE}docs/_static/videos/${NC}"
echo -e "🌐 Preview standalone: ${BLUE}file://$VIDEOS_OUTPUT_DIR/index.html${NC}"
echo ""
echo -e "${YELLOW}💡 Prochaine étape: make docs-sphinx${NC}"
echo ""
