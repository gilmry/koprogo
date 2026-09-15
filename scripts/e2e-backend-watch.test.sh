#!/usr/bin/env bash
# scripts/e2e-backend-watch.test.sh
#
# Tests 4-catégories du témoin d'interruption scripts/e2e-backend-watch.sh
# (story #880). RED-first : ce fichier doit échouer contre un script absent
# ou naïf avant d'écrire la version qui le satisfait.
#
# Chaque test pointe KOPROGO_E2E_BACKEND_EXEC vers un fixture jetable qui
# rejoue une séquence d'empreintes prédéterminée (fichier `answers`, une
# réponse par appel) — aucun docker réel requis pour valider la logique de
# détection.
#
# Usage: ./scripts/e2e-backend-watch.test.sh

set -uo pipefail

WATCH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/e2e-backend-watch.sh"
FAILURES=0
TESTS_RUN=0

fixture_dir() {
  mktemp -d "${TMPDIR:-/tmp}/e2e-backend-watch-test.XXXXXX"
}

# Écrit un fixture "sh -c"-compatible qui rejoue $1 (une réponse par ligne,
# la dernière ligne se répète indéfiniment une fois la liste épuisée) sans
# jamais interpréter l'argument qu'on lui passe (le sondage réel).
write_fixture() {
  local dir="$1"
  shift
  printf '%s\n' "$@" > "${dir}/answers"
  cat > "${dir}/fake-exec.sh" <<'FIXTURE'
#!/usr/bin/env bash
dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
count=0
[[ -f "${dir}/state" ]] && count="$(cat "${dir}/state")"
count=$((count + 1))
echo "${count}" > "${dir}/state"
line="$(sed -n "${count}p" "${dir}/answers")"
if [[ -z "${line}" ]]; then
  line="$(tail -n1 "${dir}/answers")"
fi
if [[ "${line}" == "DOWN" ]]; then
  exit 1
fi
printf "%s" "${line}"
exit 0
FIXTURE
  chmod +x "${dir}/fake-exec.sh"
}

assert_exit_code() {
  local label="$1" expected="$2" actual="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ "${expected}" != "${actual}" ]]; then
    echo "FAIL — ${label}: attendu exit ${expected}, obtenu ${actual}"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

assert_line_count() {
  local label="$1" expected="$2" file="$3"
  local actual
  actual="$(wc -l < "${file}" | tr -d ' ')"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ "${expected}" != "${actual}" ]]; then
    echo "FAIL — ${label}: attendu ${expected} ligne(s), obtenu ${actual}"
    echo "       contenu: $(cat "${file}")"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

assert_file_contains() {
  local label="$1" needle="$2" file="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if grep -qF -- "${needle}" "${file}"; then
    echo "ok   — ${label}"
  else
    echo "FAIL — ${label}: « ${needle} » absent de $(cat "${file}")"
    FAILURES=$((FAILURES + 1))
  fi
}

assert_file_not_contains() {
  local label="$1" needle="$2" file="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if grep -qF -- "${needle}" "${file}"; then
    echo "FAIL — ${label}: « ${needle} » ne devait PAS apparaître dans $(cat "${file}")"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

# --- @happy --------------------------------------------------------------
# Le backend ne bouge pas : le témoin n'ajoute aucun bruit (fichier vide).
test_happy_stable_backend_no_noise() {
  local dir events
  dir="$(fixture_dir)"
  write_fixture "${dir}" "111:1000" "111:1000" "111:1000" "111:1000"
  events="${dir}/events.jsonl"
  KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    bash "${WATCH}" watch --out "${events}" --interval 0 --max-samples 3
  assert_exit_code "@happy watch sort 0 quand rien ne bouge" 0 "$?"
  assert_line_count "@happy aucun événement écrit (silence)" 0 "${events}"
  rm -rf "${dir}"
}

# --- @negative -------------------------------------------------------------
# Le backend redémarre en cours de campagne : le témoin NOMME la coupure,
# son heure de constat, et l'empreinte avant/après.
test_negative_restart_is_named_with_timestamp() {
  local dir events
  dir="$(fixture_dir)"
  # ligne1=baseline, ligne2=stable, ligne3=coupure (down), ligne4=relève
  write_fixture "${dir}" "111:1000" "111:1000" "DOWN" "222:2000"
  events="${dir}/events.jsonl"
  KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    bash "${WATCH}" watch --out "${events}" --interval 0 --max-samples 3
  assert_line_count "@negative exactement un redémarrage journalisé" 1 "${events}"
  assert_file_contains "@negative nomme l'événement" '"event":"backend_restart"' "${events}"
  assert_file_contains "@negative garde l'empreinte AVANT" '"previous":"111:1000"' "${events}"
  assert_file_contains "@negative garde l'empreinte APRÈS" '"current":"222:2000"' "${events}"
  assert_file_contains "@negative porte un horodatage de constat" '"detected_at":"' "${events}"
  assert_file_contains "@negative garde l'heure de la coupure" '"down_since":"' "${events}"
  rm -rf "${dir}"
}

# --- @edge -----------------------------------------------------------------
# Empreinte indisponible dès le départ (pas de démon Docker, banc distant) :
# le témoin le signale et ne prétend pas avoir vérifié — pas de faux "0".
test_edge_monitoring_unavailable_is_declared() {
  local dir events
  dir="$(fixture_dir)"
  write_fixture "${dir}" "DOWN"
  events="${dir}/events.jsonl"
  KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    bash "${WATCH}" watch --out "${events}" --interval 0 --max-samples 5
  assert_exit_code "@edge watch reste exit 0 (ne fait pas échouer la campagne)" 0 "$?"
  assert_line_count "@edge une seule déclaration d'indisponibilité" 1 "${events}"
  assert_file_contains "@edge nomme l'indisponibilité, pas un zéro silencieux" \
    '"event":"monitoring_unavailable"' "${events}"
  rm -rf "${dir}"
}

# --- @security ---------------------------------------------------------------
# Une sortie hostile (tentative de fuite d'un chemin d'hôte, ou de format
# inattendu) n'est jamais recopiée telle quelle dans l'artefact — seule une
# empreinte "PID:EPOCH" strictement conforme est acceptée.
test_security_malformed_fingerprint_never_echoed() {
  local dir events marker
  dir="$(fixture_dir)"
  marker="/home/runner/.ssh/id_rsa_hostile_marker"
  write_fixture "${dir}" "111:1000 ${marker}"
  events="${dir}/events.jsonl"
  KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    bash "${WATCH}" watch --out "${events}" --interval 0 --max-samples 2
  assert_file_not_contains "@security le chemin d'hôte hostile n'atteint pas l'artefact" \
    "${marker}" "${events}"
  # Sortie non conforme au format PID:EPOCH == indisponible, jamais un
  # redémarrage "réussi" bâti sur une empreinte invalide.
  assert_file_not_contains "@security aucun redémarrage bâti sur une empreinte invalide" \
    '"event":"backend_restart"' "${events}"
  rm -rf "${dir}"
}

test_happy_stable_backend_no_noise
test_negative_restart_is_named_with_timestamp
test_edge_monitoring_unavailable_is_declared
test_security_malformed_fingerprint_never_echoed

echo ""
echo "------------------------------------------------------------"
echo "${TESTS_RUN} assertions, ${FAILURES} échec(s)"

if [[ "${FAILURES}" -gt 0 ]]; then
  exit 1
fi
exit 0
