#!/bin/bash
################################################################################
# Tests pour check-cert-expiry.sh (issue #453, story C9.1)
#
# RED-first : ce fichier existe avant check-cert-expiry.sh et doit échouer
# tant que le script n'existe pas. Couvre les 4 classes imposées par la story :
# @happy @negative @edge @security.
#
# Usage: ./check-cert-expiry.test.sh
################################################################################
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SUT="${SCRIPT_DIR}/check-cert-expiry.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

FAILURES=0

assert_exit() {
    local desc="$1" expected="$2" actual="$3"
    if [ "$expected" != "$actual" ]; then
        echo "FAIL: ${desc} (attendu exit=${expected}, obtenu exit=${actual})"
        FAILURES=$((FAILURES + 1))
    else
        echo "PASS: ${desc}"
    fi
}

assert_contains() {
    local desc="$1" haystack="$2" needle="$3"
    if [[ "${haystack}" != *"${needle}"* ]]; then
        echo "FAIL: ${desc} (attendu que la sortie contienne : ${needle})"
        FAILURES=$((FAILURES + 1))
    else
        echo "PASS: ${desc}"
    fi
}

assert_not_contains() {
    local desc="$1" haystack="$2" needle="$3"
    if [[ "${haystack}" == *"${needle}"* ]]; then
        echo "FAIL: ${desc} (la sortie NE DOIT PAS contenir : ${needle})"
        FAILURES=$((FAILURES + 1))
    else
        echo "PASS: ${desc}"
    fi
}

make_cert() {
    # make_cert <path_sortie> <jours_validite>
    openssl req -x509 -newkey rsa:2048 -nodes -keyout "${TMP}/key.pem" \
        -out "$1" -days "$2" -subj "/CN=test.dev.koprogo.com" 2>/dev/null
}

# --- @happy : cert valide avec large marge (90j), renouvellement ok -> pas d'alerte ---
make_cert "${TMP}/happy.pem" 90
out=$("${SUT}" "${TMP}/happy.pem" ok 60); code=$?
assert_exit "@happy cert 90j / seuil 60j -> pas d'alerte" 0 "${code}"
assert_contains "@happy message explicite OK" "${out}" "OK"

# --- @negative : renouvellement signalé en échec, même avec un cert encore valide longtemps.
#     C'est le cœur de la story : "un renouvellement automatique dont l'échec est
#     silencieux est une panne à date fixe" -> l'alerte part immédiatement, pas
#     seulement quand le cert est proche de l'expiration. ---
out=$("${SUT}" "${TMP}/happy.pem" failed 60); code=$?
assert_exit "@negative renouvellement failed -> alerte immédiate malgré marge" 1 "${code}"
assert_contains "@negative raison = échec de renouvellement" "${out}" "renewal failed"

# --- @negative : fichier cert absent (ex. premier run jamais réussi) -> alerte, pas de succès silencieux ---
out=$("${SUT}" "${TMP}/absent.pem" ok 60); code=$?
assert_exit "@negative cert absent -> alerte" 1 "${code}"

# --- @edge : exactement au seuil (60j restants) -> pas d'alerte (limite côté sain) ---
make_cert "${TMP}/edge60.pem" 60
out=$("${SUT}" "${TMP}/edge60.pem" ok 60); code=$?
assert_exit "@edge exactement 60j restants -> pas d'alerte" 0 "${code}"

# --- @edge : juste sous le seuil (59j restants) -> alerte ---
make_cert "${TMP}/edge59.pem" 59
out=$("${SUT}" "${TMP}/edge59.pem" ok 60); code=$?
assert_exit "@edge 59j restants -> alerte" 1 "${code}"

# --- @edge : cert illisible / corrompu -> alerte, jamais un faux OK ---
echo "ceci n'est pas un certificat" > "${TMP}/corrupt.pem"
out=$("${SUT}" "${TMP}/corrupt.pem" ok 60); code=$?
assert_exit "@edge cert corrompu -> alerte" 1 "${code}"

# --- @edge : usage incorrect (arguments manquants) -> code distinct de ALERT(1) et OK(0) ---
"${SUT}" >"${TMP}/usage_out" 2>&1
code=$?
assert_exit "@edge usage sans arguments -> code usage=2" 2 "${code}"

# --- @security : jamais de clé privée dans la sortie, même pointé par erreur sur une clé privée ---
out=$("${SUT}" "${TMP}/key.pem" ok 60 2>&1); code=$?
assert_not_contains "@security jamais de PRIVATE KEY dans la sortie" "${out}" "PRIVATE KEY"
assert_exit "@security clé privée passée comme cert -> alerte (pas un cert valide)" 1 "${code}"

echo
if [ "${FAILURES}" -eq 0 ]; then
    echo "Tous les tests passent (0 échec)."
    exit 0
else
    echo "${FAILURES} test(s) en échec."
    exit 1
fi
