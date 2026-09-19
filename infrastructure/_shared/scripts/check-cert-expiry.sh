#!/bin/bash
################################################################################
# Décide si une alerte de renouvellement TLS doit partir (issue #453, @negative).
#
# "Un renouvellement automatique dont l'échec est silencieux est une panne à
# date fixe" : ce script alerte donc dès qu'un renouvellement échoue, sans
# attendre que le certificat soit proche de l'expiration, ET alerte aussi
# quand la marge restante passe sous le seuil (défaut 60j, cf. DoD #453 —
# lego tourne 2x/mois sur des certs 90j, la marge doit rester >= 60j en
# permanence pour absorber deux échecs consécutifs avant l'expiration réelle).
#
# Usage: check-cert-expiry.sh <cert.pem> <ok|failed> [seuil_jours=60]
#
# Exit codes:
#   0 = OK, pas d'alerte
#   1 = ALERTE (à relayer par l'appelant, ex. `gh issue create`)
#   2 = erreur d'usage
#
# Ne lit jamais la clé privée : uniquement les métadonnées publiques du
# certificat (openssl x509 -enddate). Rien de secret n'est jamais imprimé.
################################################################################
set -uo pipefail

CERT_PATH="${1:-}"
RENEWAL_STATUS="${2:-}"
THRESHOLD_DAYS="${3:-60}"

if [ -z "${CERT_PATH}" ] || [ -z "${RENEWAL_STATUS}" ]; then
    echo "Usage: $0 <cert.pem> <ok|failed> [seuil_jours=60]" >&2
    exit 2
fi

if [ "${RENEWAL_STATUS}" = "failed" ]; then
    echo "ALERT reason=\"renewal failed\" cert=${CERT_PATH}"
    exit 1
fi

if [ "${RENEWAL_STATUS}" != "ok" ]; then
    echo "Usage: statut de renouvellement invalide '${RENEWAL_STATUS}' (attendu: ok|failed)" >&2
    exit 2
fi

if [ ! -r "${CERT_PATH}" ]; then
    echo "ALERT reason=\"certificate file not found or unreadable\" cert=${CERT_PATH}"
    exit 1
fi

ENDDATE_LINE="$(openssl x509 -enddate -noout -in "${CERT_PATH}" 2>/dev/null)"
if [ -z "${ENDDATE_LINE}" ]; then
    echo "ALERT reason=\"cannot parse certificate\" cert=${CERT_PATH}"
    exit 1
fi

NOT_AFTER="${ENDDATE_LINE#notAfter=}"
NOT_AFTER_EPOCH="$(date -d "${NOT_AFTER}" +%s 2>/dev/null)"
if [ -z "${NOT_AFTER_EPOCH}" ]; then
    # macOS/BSD date n'accepte pas -d de la même façon que GNU date.
    NOT_AFTER_EPOCH="$(date -j -f "%b %d %T %Y %Z" "${NOT_AFTER}" +%s 2>/dev/null)"
fi
if [ -z "${NOT_AFTER_EPOCH}" ]; then
    echo "ALERT reason=\"cannot compute expiry date\" cert=${CERT_PATH}"
    exit 1
fi

NOW_EPOCH="$(date +%s)"
# Division arrondie au jour supérieur : une marge de "59j 23h" doit compter
# comme 60j restants, pas 59 (sinon le seuil se déclenche une journée trop tôt).
DAYS_REMAINING=$(( (NOT_AFTER_EPOCH - NOW_EPOCH + 86399) / 86400 ))

if [ "${DAYS_REMAINING}" -lt "${THRESHOLD_DAYS}" ]; then
    echo "ALERT reason=\"expires in ${DAYS_REMAINING}d (< ${THRESHOLD_DAYS}d threshold)\" cert=${CERT_PATH}"
    exit 1
fi

echo "OK cert=${CERT_PATH} expires_in=${DAYS_REMAINING}d threshold=${THRESHOLD_DAYS}d"
exit 0
