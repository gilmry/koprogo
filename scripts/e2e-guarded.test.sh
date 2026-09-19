#!/usr/bin/env bash
# scripts/e2e-guarded.test.sh
#
# Tests 4-catégories de l'orchestrateur scripts/e2e-guarded.sh (story #880).
# RED-first : ce fichier doit échouer contre un script absent ou naïf avant
# d'écrire la version qui le satisfait.
#
# Ces tests emploient un vrai minuteur (la commande encadrée dort quelques
# secondes pendant que le fixture backend change de réponse) : c'est de
# l'orchestration de processus, pas une logique pure. La marge est large
# pour rester fiable sous un hôte chargé.
#
# Usage: ./scripts/e2e-guarded.test.sh

set -uo pipefail

GUARDED="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/e2e-guarded.sh"
FAILURES=0
TESTS_RUN=0

fixture_dir() {
  mktemp -d "${TMPDIR:-/tmp}/e2e-guarded-test.XXXXXX"
}

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

assert_file_contains() {
  local label="$1" needle="$2" file="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if grep -qF -- "${needle}" "${file}"; then
    echo "ok   — ${label}"
  else
    echo "FAIL — ${label}: « ${needle} » absent de $(cat "${file}" 2>/dev/null)"
    FAILURES=$((FAILURES + 1))
  fi
}

assert_file_not_contains() {
  local label="$1" needle="$2" file="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if grep -qF -- "${needle}" "${file}" 2>/dev/null; then
    echo "FAIL — ${label}: « ${needle} » ne devait PAS apparaître dans $(cat "${file}")"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

# --- @happy ----------------------------------------------------------------
# Backend stable pendant toute la campagne : verdict inchangé, mesuré.
test_happy_stable_backend_passthrough() {
  local dir artifacts out rc
  dir="$(fixture_dir)"
  artifacts="${dir}/artifacts"
  write_fixture "${dir}" "111:1000" "111:1000" "111:1000" "111:1000" "111:1000" "111:1000"
  out="$(KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    KOPROGO_E2E_ARTIFACT_DIR="${artifacts}" \
    KOPROGO_E2E_WATCH_INTERVAL=0.3 \
    bash "${GUARDED}" -- bash -c 'exit 0' 2>&1)"
  rc=$?
  assert_exit_code "@happy code de la commande transmis tel quel" 0 "${rc}"
  assert_file_contains "@happy verdict measured=true" '"measured": true' "${artifacts}/campaign-verdict.json"
  assert_file_contains "@happy aucun redémarrage journalisé" '"backend_restarts": []' "${artifacts}/campaign-verdict.json"
  rm -rf "${dir}"
}

# --- @negative ---------------------------------------------------------------
# Backend redémarre pendant la campagne : verdict NON MESURÉ, code de sortie
# distinct (EX_TEMPFAIL=75), la coupure et son heure sont nommées.
test_negative_restart_invalidates_campaign() {
  local dir artifacts out rc
  dir="$(fixture_dir)"
  artifacts="${dir}/artifacts"
  # Nombreuses réponses stables puis un changement, pour couvrir toutes les
  # sondes qui auront le temps de tourner pendant les 2s de la commande.
  write_fixture "${dir}" \
    "111:1000" "111:1000" "111:1000" "DOWN" "DOWN" \
    "222:2000" "222:2000" "222:2000" "222:2000" "222:2000" "222:2000" "222:2000"
  out="$(KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    KOPROGO_E2E_ARTIFACT_DIR="${artifacts}" \
    KOPROGO_E2E_WATCH_INTERVAL=0.25 \
    bash "${GUARDED}" -- bash -c 'sleep 2; exit 0' 2>&1)"
  rc=$?
  assert_exit_code "@negative code de sortie EX_TEMPFAIL (75), pas 0 ni 1" 75 "${rc}"
  assert_file_contains "@negative verdict measured=false" '"measured": false' "${artifacts}/campaign-verdict.json"
  assert_file_contains "@negative nomme le redémarrage dans l'artefact" '"event":"backend_restart"' "${artifacts}/campaign-verdict.json"
  assert_file_contains "@negative horodate la coupure dans l'artefact" '"detected_at":"' "${artifacts}/campaign-verdict.json"
  echo "${out}" | grep -qF "NON MESURÉE" && { echo "ok   — @negative message humain nomme la campagne non mesurée"; TESTS_RUN=$((TESTS_RUN+1)); } \
    || { echo "FAIL — @negative aucun message humain « NON MESURÉE » dans la sortie"; FAILURES=$((FAILURES+1)); TESTS_RUN=$((TESTS_RUN+1)); }
  rm -rf "${dir}"
}

# --- @edge -------------------------------------------------------------------
# Empreinte indisponible dès le départ : le résultat de la commande encadrée
# est transmis tel quel (on ne prétend pas avoir vérifié), mais l'artefact
# déclare explicitly l'absence de mesure (measured=false, jamais un faux 0).
test_edge_monitoring_unavailable_declared_not_verified() {
  local dir artifacts out rc
  dir="$(fixture_dir)"
  artifacts="${dir}/artifacts"
  write_fixture "${dir}" "DOWN"
  out="$(KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    KOPROGO_E2E_ARTIFACT_DIR="${artifacts}" \
    KOPROGO_E2E_WATCH_INTERVAL=0.2 \
    bash "${GUARDED}" -- bash -c 'exit 0' 2>&1)"
  rc=$?
  assert_exit_code "@edge code de la commande transmis tel quel malgré l'incertitude" 0 "${rc}"
  assert_file_contains "@edge monitoring déclaré indisponible" '"monitoring": "unavailable"' "${artifacts}/campaign-verdict.json"
  assert_file_contains "@edge backend_restarts vaut null, pas 0" '"backend_restarts": null' "${artifacts}/campaign-verdict.json"
  assert_file_contains "@edge measured=false (absence de mesure explicite)" '"measured": false' "${artifacts}/campaign-verdict.json"
  rm -rf "${dir}"
}

# --- @security -----------------------------------------------------------------
# Le fixture (chemin sur l'hôte) n'apparaît dans aucun artefact produit.
test_security_no_host_path_leak_in_artifacts() {
  local dir artifacts out rc
  dir="$(fixture_dir)"
  artifacts="${dir}/artifacts"
  write_fixture "${dir}" "111:1000" "111:1000"
  out="$(KOPROGO_E2E_BACKEND_EXEC="${dir}/fake-exec.sh" \
    KOPROGO_E2E_ARTIFACT_DIR="${artifacts}" \
    KOPROGO_E2E_WATCH_INTERVAL=0.3 \
    bash "${GUARDED}" -- bash -c 'exit 0' 2>&1)"
  rc=$?
  assert_file_not_contains "@security le chemin du fixture (hôte) n'apparaît pas dans le verdict" \
    "${dir}/fake-exec.sh" "${artifacts}/campaign-verdict.json"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ "${out}" == *"${dir}/fake-exec.sh"* ]]; then
    echo "FAIL — @security le chemin du fixture (hôte) fuite dans la sortie humaine"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — @security aucune fuite de chemin d'hôte dans la sortie humaine"
  fi
  rm -rf "${dir}"
}

test_happy_stable_backend_passthrough
test_negative_restart_invalidates_campaign
test_edge_monitoring_unavailable_declared_not_verified
test_security_no_host_path_leak_in_artifacts

echo ""
echo "------------------------------------------------------------"
echo "${TESTS_RUN} assertions, ${FAILURES} échec(s)"

if [[ "${FAILURES}" -gt 0 ]]; then
  exit 1
fi
exit 0
