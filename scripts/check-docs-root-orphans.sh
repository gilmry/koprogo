#!/usr/bin/env bash
# check-docs-root-orphans.sh
#
# Story #854 / RFC 0003 lint gate.
#
# Refuse tout fichier .md à la racine de docs/ qui ne figure pas sur la
# liste blanche explicite ci-dessous. La racine de docs/ n'est pas un
# tiroir : un markdown qui y atterrit doit être un document de pilotage
# signé (WBS, RICE, Gantt, revue humaine v0.1.0), pas une note en attente
# de rangement. Tout le reste appartient à un sous-dossier thématique
# (docs/frontend/, docs/backend/, docs/deployment/, docs/legal/,
# docs/user-guides/, docs/governance/), à une ADR, à un RFC, à une issue,
# ou à docs/archive/.
#
# Sans cette garde, le tas racine se reconstitue de lui-même — c'est
# exactement ce que huit fichiers de mars 2026 ont démontré (issue #854).
#
# Exit codes:
#   0 — clean, aucun markdown hors liste blanche à la racine de docs/
#   1 — au moins un markdown non listé (imprimé avec son nom)
#   2 — erreur d'usage (répertoire --root introuvable)
#
# Usage:
#   ./scripts/check-docs-root-orphans.sh
#   ./scripts/check-docs-root-orphans.sh --root /chemin/vers/docs

set -euo pipefail

ROOT="${CLAUDE_PROJECT_DIR:-$(git rev-parse --show-toplevel 2>/dev/null || pwd)}/docs"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --root)
      ROOT="$2"
      shift 2
      ;;
    -h|--help)
      sed -n '2,26p' "$0" 2>/dev/null || true
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

if [[ ! -d "${ROOT}" ]]; then
  echo "check-docs-root-orphans: ${ROOT} does not exist." >&2
  exit 2
fi

# Liste blanche — documents de pilotage vivants, signés, dont la place est
# la racine (cf. RFC 0003 « Ce qui reste à la racine »). Toute addition à
# cette liste doit être justifiée en commentaire ci-dessus, jamais silencieuse.
WHITELIST=(
  "WBS_v0_1_0.md"
  "HUMAN_REVIEW_PLAN_v0.1.0.md"
  "HUMAN_REVIEW_REPORT_v0.1.0.md"
  "BACKLOG_STRUCTURE_v0_1_0.md"
  "GANTT_PASSES_v0_1_0.md"
  "RICE_PRODUIT_v0_1_0.md"
)

is_whitelisted() {
  local candidate="$1"
  local entry
  for entry in "${WHITELIST[@]}"; do
    if [[ "${candidate}" == "${entry}" ]]; then
      return 0
    fi
  done
  return 1
}

orphans=()
while IFS= read -r -d '' file; do
  name="$(basename -- "${file}")"
  if ! is_whitelisted "${name}"; then
    orphans+=("${name}")
  fi
# maxdepth 1 : seule la racine de docs/ est concernée, pas les sous-dossiers
# déjà triés (docs/frontend/, docs/adr/, etc.).
done < <(find "${ROOT}" -maxdepth 1 -type f -name '*.md' -print0 | sort -z)

if [[ ${#orphans[@]} -gt 0 ]]; then
  echo "============================================================"
  echo "check-docs-root-orphans: ${#orphans[@]} markdown non référencé(s) à la racine de docs/"
  echo "============================================================"
  for name in "${orphans[@]}"; do
    echo "  - docs/${name}"
  done
  echo ""
  echo "Chaque markdown de la racine de docs/ doit soit :"
  echo "  1. rejoindre un sous-dossier thématique + entrée toctree (guide vivant),"
  echo "  2. devenir une ADR (docs/adr/) si la décision est tranchée,"
  echo "  3. devenir un RFC ou une issue GitHub si la question est ouverte,"
  echo "  4. partir en docs/archive/ avec sa date de péremption en tête,"
  echo "  5. ou rejoindre la liste blanche ci-dessus, avec sa justification écrite."
  echo "Voir docs/governance/rfc/0003-ranger-la-documentation-eparpillee.rst."
  exit 1
fi

echo "check-docs-root-orphans: OK — ${ROOT} ne contient que la liste blanche."
exit 0
