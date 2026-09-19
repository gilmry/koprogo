#!/usr/bin/env bash
# scripts/e2e-guarded.sh
#
# Story #880 — encadre UNE campagne e2e (la commande passée après `--`) d'une
# surveillance de l'empreinte du backend (scripts/e2e-backend-watch.sh). Si le
# backend redémarre pendant que la commande tourne, le rapport le dit — en
# toutes lettres, avec l'heure — et le code de sortie distingue « des tests
# ont échoué » de « non mesuré » (cf. issue #880, 83 échecs du 2026-09-13
# imputés à tort à une régression alors que le backend avait recompilé
# deux fois sous cargo-watch pendant la campagne).
#
# Usage :
#   scripts/e2e-guarded.sh -- <commande e2e réelle...>
#
# Exit codes :
#   <code de la commande encadrée>  — @happy : aucun redémarrage détecté,
#                                     monitoring disponible. Verdict inchangé,
#                                     aucun bruit ajouté.
#   <code de la commande encadrée>  — @edge : monitoring INDISPONIBLE dès le
#                                     départ (pas de démon Docker, banc
#                                     distant). Le rapport le signale ; on
#                                     n'invente pas un rouge qu'on n'a pas
#                                     vérifié.
#   75 (EX_TEMPFAIL, sysexits.h)    — @negative : redémarrage détecté pendant
#                                     la campagne. NON MESURÉE, quel qu'ait
#                                     été le code de la commande encadrée —
#                                     aucun échec postérieur à la coupure
#                                     n'est un verdict.
#
# Artefacts (dans $KOPROGO_E2E_ARTIFACT_DIR, défaut frontend/test-results) :
#   backend-restarts.jsonl   — un événement par ligne (scripts/e2e-backend-watch.sh)
#   campaign-verdict.json    — résumé humainement lisible du verdict
#
# Variables :
#   KOPROGO_E2E_ARTIFACT_DIR    dossier de sortie (défaut: frontend/test-results)
#   KOPROGO_E2E_WATCH_INTERVAL  secondes entre deux sondes (défaut: 3)
#   KOPROGO_E2E_BACKEND_EXEC    transmise telle quelle à e2e-backend-watch.sh

set -uo pipefail

readonly EX_TEMPFAIL=75

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WATCH_SCRIPT="${HERE}/e2e-backend-watch.sh"

ARTIFACT_DIR="${KOPROGO_E2E_ARTIFACT_DIR:-frontend/test-results}"
INTERVAL="${KOPROGO_E2E_WATCH_INTERVAL:-3}"

if [[ "${1:-}" != "--" ]]; then
  echo "usage: $0 -- <commande e2e...>" >&2
  exit 2
fi
shift

if [[ $# -eq 0 ]]; then
  echo "usage: $0 -- <commande e2e...> (commande manquante)" >&2
  exit 2
fi

mkdir -p "${ARTIFACT_DIR}"
EVENTS_FILE="${ARTIFACT_DIR}/backend-restarts.jsonl"
VERDICT_FILE="${ARTIFACT_DIR}/campaign-verdict.json"

bash "${WATCH_SCRIPT}" watch --out "${EVENTS_FILE}" --interval "${INTERVAL}" &
WATCH_PID=$!

"$@"
CMD_RC=$?

kill -TERM "${WATCH_PID}" 2>/dev/null || true
wait "${WATCH_PID}" 2>/dev/null || true

monitoring="ok"
if grep -qF '"event":"monitoring_unavailable"' "${EVENTS_FILE}" 2>/dev/null; then
  monitoring="unavailable"
fi

restart_lines=()
if [[ "${monitoring}" == "ok" ]] && [[ -s "${EVENTS_FILE}" ]]; then
  while IFS= read -r line; do
    [[ -n "${line}" ]] && restart_lines+=("${line}")
  done < <(grep -F '"event":"backend_restart"' "${EVENTS_FILE}" 2>/dev/null || true)
fi

restart_count=${#restart_lines[@]}

now_iso() { date -u +%Y-%m-%dT%H:%M:%SZ; }

{
  echo "{"
  echo "  \"generated_at\": \"$(now_iso)\","
  echo "  \"wrapped_command_exit_code\": ${CMD_RC},"
  if [[ "${monitoring}" == "unavailable" ]]; then
    echo "  \"monitoring\": \"unavailable\","
    echo "  \"backend_restarts\": null,"
    echo "  \"measured\": false"
  elif [[ ${restart_count} -gt 0 ]]; then
    echo "  \"monitoring\": \"ok\","
    echo "  \"backend_restarts\": ["
    for i in "${!restart_lines[@]}"; do
      sep=","
      [[ $i -eq $((restart_count - 1)) ]] && sep=""
      echo "    ${restart_lines[$i]}${sep}"
    done
    echo "  ],"
    echo "  \"measured\": false"
  else
    echo "  \"monitoring\": \"ok\","
    echo "  \"backend_restarts\": [],"
    echo "  \"measured\": true"
  fi
  echo "}"
} > "${VERDICT_FILE}"

echo ""
echo "------------------------------------------------------------"
if [[ "${monitoring}" == "unavailable" ]]; then
  echo "⚠️  e2e-guarded : empreinte backend indisponible — non vérifié (voir ${VERDICT_FILE})."
  echo "   Code de sortie de la commande encadrée transmis tel quel : ${CMD_RC}."
  exit "${CMD_RC}"
elif [[ ${restart_count} -gt 0 ]]; then
  echo "🔴 e2e-guarded : ${restart_count} redémarrage(s) backend détecté(s) pendant la campagne."
  echo "   Campagne NON MESURÉE — aucun échec postérieur à une coupure n'est un verdict."
  echo "   Détail : ${EVENTS_FILE}"
  exit "${EX_TEMPFAIL}"
else
  echo "✅ e2e-guarded : backend stable pendant toute la campagne (aucun redémarrage)."
  exit "${CMD_RC}"
fi
