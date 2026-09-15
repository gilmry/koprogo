#!/usr/bin/env bash
# check-docs-root-orphans.test.sh
#
# Tests 4-catégories du garde-fou scripts/check-docs-root-orphans.sh
# (story #854). RED-first : ce fichier doit échouer contre un script
# absent ou naïf avant d'écrire la version qui le satisfait.
#
# Usage: ./scripts/check-docs-root-orphans.test.sh

set -uo pipefail

GUARD="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/check-docs-root-orphans.sh"
FAILURES=0
TESTS_RUN=0

fixture_dir() {
  mktemp -d "${TMPDIR:-/tmp}/docs-root-orphans-test.XXXXXX"
}

assert_exit_code() {
  local label="$1"
  local expected="$2"
  local actual="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ "${expected}" != "${actual}" ]]; then
    echo "FAIL — ${label}: attendu exit ${expected}, obtenu ${actual}"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

assert_output_contains() {
  local label="$1"
  local needle="$2"
  local haystack="$3"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ "${haystack}" != *"${needle}"* ]]; then
    echo "FAIL — ${label}: sortie ne contient pas « ${needle} »"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — ${label}"
  fi
}

# --- @happy ------------------------------------------------------------
# Racine ne contenant que des fichiers de la liste blanche : la garde passe.
test_happy_whitelist_only() {
  local dir output rc
  dir="$(fixture_dir)"
  : > "${dir}/WBS_v0_1_0.md"
  : > "${dir}/HUMAN_REVIEW_PLAN_v0.1.0.md"
  output="$(bash "${GUARD}" --root "${dir}" 2>&1)"; rc=$?
  assert_exit_code "@happy racine 100% liste blanche → exit 0" 0 "${rc}"
  assert_output_contains "@happy message OK" "OK" "${output}"
  rm -rf "${dir}"
}

# --- @negative -----------------------------------------------------------
# Un nouveau markdown déposé à la racine, hors liste blanche : la garde
# doit le signaler nommément et échouer.
test_negative_new_orphan_is_flagged() {
  local dir output rc
  dir="$(fixture_dir)"
  : > "${dir}/WBS_v0_1_0.md"
  : > "${dir}/NOTE_DE_SERVICE_IMPROVISEE.md"
  output="$(bash "${GUARD}" --root "${dir}" 2>&1)"; rc=$?
  assert_exit_code "@negative orphelin non listé → exit 1" 1 "${rc}"
  assert_output_contains "@negative nomme le fichier fautif" "NOTE_DE_SERVICE_IMPROVISEE.md" "${output}"
  rm -rf "${dir}"
}

# --- @edge -----------------------------------------------------------
# Bornes : racine vide (rien à signaler), fichier whitelisté seul,
# sous-dossier déjà trié ignoré (maxdepth 1), et un nom de fichier
# whitelisté qui réapparaît dans un sous-dossier ne doit pas fausser le
# résultat racine.
test_edge_empty_root() {
  local dir output rc
  dir="$(fixture_dir)"
  output="$(bash "${GUARD}" --root "${dir}" 2>&1)"; rc=$?
  assert_exit_code "@edge racine vide → exit 0" 0 "${rc}"
  rm -rf "${dir}"
}

test_edge_subdirectory_not_scanned() {
  local dir output rc
  dir="$(fixture_dir)"
  : > "${dir}/WBS_v0_1_0.md"
  mkdir -p "${dir}/frontend"
  : > "${dir}/frontend/UN_GUIDE_DEJA_RANGE.md"
  output="$(bash "${GUARD}" --root "${dir}" 2>&1)"; rc=$?
  assert_exit_code "@edge sous-dossier déjà trié ignoré (maxdepth 1) → exit 0" 0 "${rc}"
  rm -rf "${dir}"
}

test_edge_missing_root_is_usage_error() {
  local output rc
  output="$(bash "${GUARD}" --root "/nonexistent/$$/docs" 2>&1)"; rc=$?
  assert_exit_code "@edge --root introuvable → exit 2 (erreur d'usage, pas un faux OK)" 2 "${rc}"
  rm -rf "${dir:-}" 2>/dev/null || true
}

# --- @security -----------------------------------------------------------
# Un nom de fichier hostile (métacaractères shell) ne doit ni casser le
# script ni être exécuté — seulement lu comme un nom de fichier littéral.
test_security_hostile_filename_no_command_injection() {
  local dir output rc marker
  dir="$(fixture_dir)"
  marker="${dir}/PWNED_MARKER"
  : > "${dir}/WBS_v0_1_0.md"
  # Un nom de fichier qui ressemblerait à une substitution de commande si
  # le script interpolait sans discipline de quoting.
  touch "${dir}/\$(touch ${marker}).md" 2>/dev/null || true
  output="$(bash "${GUARD}" --root "${dir}" 2>&1)"; rc=$?
  assert_exit_code "@security nom de fichier hostile → toujours détecté comme orphelin (exit 1)" 1 "${rc}"
  TESTS_RUN=$((TESTS_RUN + 1))
  if [[ -e "${marker}" ]]; then
    echo "FAIL — @security le nom de fichier hostile a été exécuté (injection de commande)"
    FAILURES=$((FAILURES + 1))
  else
    echo "ok   — @security aucune exécution de commande via le nom de fichier"
  fi
  rm -rf "${dir}"
}

test_happy_whitelist_only
test_negative_new_orphan_is_flagged
test_edge_empty_root
test_edge_subdirectory_not_scanned
test_edge_missing_root_is_usage_error
test_security_hostile_filename_no_command_injection

echo ""
echo "------------------------------------------------------------"
echo "${TESTS_RUN} assertions, ${FAILURES} échec(s)"

if [[ "${FAILURES}" -gt 0 ]]; then
  exit 1
fi
exit 0
