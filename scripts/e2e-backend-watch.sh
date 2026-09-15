#!/usr/bin/env bash
# scripts/e2e-backend-watch.sh
#
# Story #880 — le backend de la pile de recette tourne sous cargo-watch
# (backend/Dockerfile.dev:71). Une recompilation en cours de campagne coupe
# le service ~90s ; sans témoin, les échecs qu'elle cause sont indiscernables
# d'une régression (83 des 95 échecs du 2026-09-13, cf. issue #880).
#
# Ce script surveille une seule chose : l'empreinte du processus backend
# (PID + heure de démarrage du process, rien de plus — cf. @security), et
# journalise chaque redémarrage observé pendant qu'il tourne.
#
# Sous-commandes :
#   fingerprint
#       Imprime l'empreinte courante ("PID:EPOCH") sur stdout et sort 0,
#       ou n'imprime rien et sort 1 si le backend n'est pas joignable ou
#       si le processus n'est pas trouvé.
#
#   watch --out FILE [--interval N] [--max-samples N]
#       Échantillonne l'empreinte toutes les N secondes (défaut 3) et
#       ajoute une ligne JSON à FILE pour chaque transition observée :
#         - {"event":"monitoring_unavailable","at":...}
#           si la toute première empreinte échoue de façon soutenue —
#           @edge, aucune vérification n'est prétendue.
#         - {"event":"backend_restart","detected_at":...,"down_since":...,
#            "recovered_at":...,"previous":"PID:EPOCH"|null,
#            "current":"PID:EPOCH"|null}
#           à chaque redémarrage détecté (recovered_at=null si le backend
#           ne s'est pas relevé avant la fin de la surveillance).
#       Sans --max-samples, boucle jusqu'à réception de SIGTERM/SIGINT.
#       N'écrit RIEN quand rien ne bouge (@happy : aucun bruit ajouté).
#
# Variables :
#   KOPROGO_E2E_BACKEND_EXEC   commande utilisée pour interroger le backend
#                              (défaut : "docker compose exec -T backend sh -c").
#                              Les tests la pointent vers un fixture pour
#                              simuler stabilité / redémarrage / indisponibilité
#                              sans docker réel.
#
# Exit codes de `watch` : toujours 0 (le témoin ne fait jamais échouer la
# campagne lui-même ; c'est scripts/e2e-guarded.sh qui décide du verdict).

set -uo pipefail

BACKEND_EXEC="${KOPROGO_E2E_BACKEND_EXEC:-docker compose exec -T backend sh -c}"

# Cherche un process dont le cmdline contient "koprogo-api" et imprime
# "PID:EPOCH_DEMARRAGE" (ctime du répertoire /proc/<pid>, approximation sûre
# de l'heure de démarrage — aucune dépendance à `pgrep`/`awk`, absents de
# l'image de dev). Un identifiant de processus et un horodatage : rien de
# plus ne quitte le conteneur (@security).
readonly PROBE='for p in /proc/[0-9]*; do
  pid=${p#/proc/}
  if grep -qa "koprogo-api" "$p/cmdline" 2>/dev/null; then
    st=$(stat -c %Y "$p" 2>/dev/null) || continue
    printf "%s:%s" "$pid" "$st"
    exit 0
  fi
done
exit 1'

probe_fingerprint() {
  # shellcheck disable=SC2206 — BACKEND_EXEC est un préfixe de commande
  # volontairement splitté sur les espaces ("docker compose exec -T backend
  # sh -c") ; les fixtures de test n'en contiennent pas.
  local -a exec_cmd=(${BACKEND_EXEC})
  local out rc
  out="$("${exec_cmd[@]}" "${PROBE}" 2>/dev/null)"
  rc=$?
  if [[ ${rc} -ne 0 ]]; then
    return 1
  fi
  # Jamais fait confiance à une sortie non conforme : une empreinte hostile
  # ou un bavardage inattendu du fixture/backend est rejeté, pas recopié
  # (@security) — une empreinte invalide vaut "indisponible".
  if [[ ! "${out}" =~ ^[0-9]+:[0-9]+$ ]]; then
    return 1
  fi
  printf "%s" "${out}"
  return 0
}

now_iso() {
  date -u +%Y-%m-%dT%H:%M:%SZ
}

cmd_fingerprint() {
  local fp
  if fp="$(probe_fingerprint)"; then
    printf "%s\n" "${fp}"
    return 0
  fi
  return 1
}

cmd_watch() {
  local out="" interval=3 max_samples=0
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --out)
        out="$2"
        shift 2
        ;;
      --interval)
        interval="$2"
        shift 2
        ;;
      --max-samples)
        max_samples="$2"
        shift 2
        ;;
      *)
        echo "watch: unknown argument: $1" >&2
        return 2
        ;;
    esac
  done
  if [[ -z "${out}" ]]; then
    echo "watch: --out FILE requis" >&2
    return 2
  fi
  : > "${out}"

  local stop=0
  trap 'stop=1' TERM INT

  # Ligne de base : jusqu'à 3 tentatives rapprochées pour ne pas confondre
  # un aléa isolé avec une réelle indisponibilité (@edge : pas de démon
  # Docker, banc distant).
  local baseline="" attempt
  for attempt in 1 2 3; do
    if baseline="$(probe_fingerprint)"; then
      break
    fi
    baseline=""
    sleep 0.2 2>/dev/null || sleep 1
  done

  if [[ -z "${baseline}" ]]; then
    printf '{"event":"monitoring_unavailable","at":"%s"}\n' "$(now_iso)" >> "${out}"
    return 0
  fi

  local last_fp="${baseline}" down_since="" sample sample_count=0
  while [[ ${stop} -eq 0 ]]; do
    if [[ ${max_samples} -gt 0 && ${sample_count} -ge ${max_samples} ]]; then
      break
    fi
    sample_count=$((sample_count + 1))

    if sample="$(probe_fingerprint)"; then
      if [[ -n "${down_since}" ]]; then
        printf '{"event":"backend_restart","detected_at":"%s","down_since":"%s","recovered_at":"%s","previous":"%s","current":"%s"}\n' \
          "$(now_iso)" "${down_since}" "$(now_iso)" "${last_fp}" "${sample}" >> "${out}"
        last_fp="${sample}"
        down_since=""
      elif [[ "${sample}" != "${last_fp}" ]]; then
        printf '{"event":"backend_restart","detected_at":"%s","down_since":null,"recovered_at":"%s","previous":"%s","current":"%s"}\n' \
          "$(now_iso)" "$(now_iso)" "${last_fp}" "${sample}" >> "${out}"
        last_fp="${sample}"
      fi
    else
      if [[ -z "${down_since}" ]]; then
        down_since="$(now_iso)"
      fi
    fi

    if [[ ${max_samples} -gt 0 && ${sample_count} -ge ${max_samples} ]]; then
      break
    fi
    sleep "${interval}" 2>/dev/null &
    wait $! 2>/dev/null
  done

  if [[ -n "${down_since}" ]]; then
    printf '{"event":"backend_restart","detected_at":"%s","down_since":"%s","recovered_at":null,"previous":"%s","current":null}\n' \
      "${down_since}" "${down_since}" "${last_fp}" >> "${out}"
  fi
  return 0
}

main() {
  local sub="${1:-}"
  shift || true
  case "${sub}" in
    fingerprint)
      cmd_fingerprint "$@"
      ;;
    watch)
      cmd_watch "$@"
      ;;
    *)
      echo "usage: $0 {fingerprint|watch --out FILE [--interval N] [--max-samples N]}" >&2
      return 2
      ;;
  esac
}

if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
  main "$@"
  exit $?
fi
