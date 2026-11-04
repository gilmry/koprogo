#!/bin/bash
# Lanceur k6 partagé par les scripts de charge.
#
#   source "$(dirname "$0")/k6.sh"
#   k6_verifier            # k6 local, ou Docker pour l'image épinglée
#   k6_run <scénario> <utilisateurs virtuels> <durée>
#
# <scénario> est un fichier de load-tests/k6/ sans extension (mixed,
# authenticated-realistic…). La cible vient de BASE_URL. Le résumé finit par
# une ligne `SLUIS_RESUME {...}` (débit, p50, p99, taux d'échec).

# Même version que le moteur de charge de Sluis.
K6_IMAGE="grafana/k6:2.3.0@sha256:2dd4756ec6e14e62778916e1eb850108e4b499304c02df56760fa0bdb0fba0b8"
K6_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../k6" && pwd)"

k6_verifier() {
    if command -v k6 &> /dev/null; then
        K6_MODE=local
    elif command -v docker &> /dev/null; then
        K6_MODE=docker
    else
        echo "❌ Error: ni k6 ni Docker ne sont disponibles"
        echo "Installer k6 : https://grafana.com/docs/k6/latest/set-up/install-k6/"
        exit 1
    fi
}

k6_run() {
    local scenario="$1" vus="$2" duree="$3"
    local options=(run --no-usage-report --quiet --vus "$vus" --duration "$duree")
    if [ "${K6_MODE:-}" = "local" ]; then
        BASE_URL="$BASE_URL" k6 "${options[@]}" "$K6_DIR/$scenario.js"
    else
        # --network host : BASE_URL=http://localhost:8080 vise l'hôte.
        docker run --rm -i --network host \
            -v "$K6_DIR":/scripts:ro \
            -e BASE_URL="$BASE_URL" \
            -e KOPROGO_EMAIL="${KOPROGO_EMAIL:-}" \
            -e KOPROGO_PASSWORD="${KOPROGO_PASSWORD:-}" \
            "$K6_IMAGE" "${options[@]}" "/scripts/$scenario.js"
    fi
}
