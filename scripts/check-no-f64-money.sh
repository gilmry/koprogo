#!/usr/bin/env bash
#
# Gate ADR-0008 — aucun `f64` sur un montant, une quote-part ou une valeur
# alimentant un seuil légal (Issue #661).
#
# L'ADR-0008 §A pose la règle : « Any monetary amount, quote-part, or value
# feeding a legal threshold MUST be Decimal / NUMERIC end-to-end », avec une
# liste de carve-outs **fermée**. Cette liste ne vaut que si quelque chose la
# fait respecter : sans ce gate, chaque nouveau champ monétaire en `f64` passe
# en revue sans être vu (c'est exactement ainsi que le quorum d'AG est resté en
# `f64` pendant des mois — #661).
#
# Principe : on ne scanne pas tous les `f64` (l'IoT, les scores et les
# pourcentages d'affichage en ont légitimement), mais uniquement ceux dont le
# nom de symbole appartient au lexique monétaire / quotité ci-dessous.
#
# Usage : scripts/check-no-f64-money.sh [chemin]   (défaut : backend/src)
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/backend/src}"

# Lexique : un symbole qui contient un de ces fragments ET est typé f64.
MONEY_LEXICON='amount|montant|price|prix|cost|budget|quota|quotite|tantieme|millieme|penalty|owed|balance|debit|credit|solde|provision|contribution|arrears|variance'

# Carve-outs ADR-0008 §A + amendement 2026-07-31, et dette tracée.
# Format : <chemin relatif à backend/src>:<motif de symbole>  # justification
ALLOWLIST=(
  # --- ADR-0008 §A : liste fermée ---
  "domain/entities/etat_date.rs:unit_area"                    # surface m², mesure physique
  "domain/entities/unit.rs:area_m2"                           # surface m², mesure physique
  "domain/entities/challenge.rs:"                             # gamification non-PCMN
  "domain/entities/resolution.rs:"                            # % d'affichage depuis comptes entiers
  # --- ADR-0009 : IoT / énergie ---
  "domain/entities/iot_reading.rs:"
  "domain/entities/energy_campaign.rs:"
  "domain/entities/energy_bill_upload.rs:"
  "application/dto/iot_dto.rs:"
  "application/dto/energy_campaign_dto.rs:"
  "application/dto/energy_bill_upload_dto.rs:"
  "application/ports/iot_repository.rs:"
  "application/ports/linky_api_client.rs:"
  "application/ports/mqtt_energy_port.rs:"
  "application/ports/grid_participation_port.rs:"
  "application/ports/energy_campaign_repository.rs:"
  "application/use_cases/iot_use_cases.rs:"
  "application/use_cases/energy_campaign_use_cases.rs:"
  "application/use_cases/energy_bill_upload_use_cases.rs:"
  "application/use_cases/boinc_use_cases.rs:"
  "infrastructure/database/repositories/iot_repository_impl.rs:"
  "infrastructure/database/repositories/energy_campaign_repository_impl.rs:"
  "infrastructure/external/linky_api_client_impl.rs:"
  # --- Données de démonstration, jamais un calcul opposable ---
  "infrastructure/database/seed.rs:"
  # --- DETTE CONNUE ET TRACÉE (à résorber, pas un carve-out accordé) ---
  # (payment_reminder a été RÉSORBÉ — converti en Decimal, migration
  #  20260826000000. Ses lignes ont été retirées de cette liste : c'est le
  #  sens de marche attendu, une entrée de dette se supprime, elle ne se
  #  transforme pas en carve-out.)
  # work_report / technical_inspection : coûts de travaux et d'inspections en
  # euros, en f64. Même nature de défaut que payment_reminder — gelé, à traiter.
  "domain/entities/work_report.rs:cost"
  "domain/entities/technical_inspection.rs:cost"
  "application/dto/work_report_dto.rs:cost"
  "application/dto/technical_inspection_dto.rs:cost"
  "application/dto/filters.rs:_cost"
  # stats_dto : agrégat de dépenses en attente, affiché sur un tableau de bord.
  "application/dto/stats_dto.rs:pending_expenses_amount"
  # mcp_sse_handlers : formatage d'affichage cents -> "12.34" pour un outil MCP,
  # ne réalimente aucun calcul.
  "infrastructure/web/handlers/mcp_sse_handlers.rs:amount_eur"
)

is_allowed() {
  local file="$1" line="$2"
  for entry in "${ALLOWLIST[@]}"; do
    local allowed_file="${entry%%:*}"
    local allowed_sym="${entry#*:}"
    if [[ "$file" == *"$allowed_file" ]]; then
      [[ -z "$allowed_sym" ]] && return 0
      [[ "$line" == *"$allowed_sym"* ]] && return 0
    fi
  done
  return 1
}

violations=0
while IFS= read -r hit; do
  file="${hit%%:*}"
  rest="${hit#*:}"
  lineno="${rest%%:*}"
  code="${rest#*:}"

  # Ignorer commentaires et doc-comments.
  trimmed="$(echo "$code" | sed 's/^[[:space:]]*//')"
  [[ "$trimmed" == //* ]] && continue

  if ! is_allowed "$file" "$code"; then
    echo "❌ $file:$lineno"
    echo "   $trimmed"
    violations=$((violations + 1))
  fi
done < <(grep -rnE "\bf64\b" "$ROOT" --include='*.rs' | grep -iE "($MONEY_LEXICON)" || true)

if [[ "$violations" -gt 0 ]]; then
  cat >&2 <<MSG

──────────────────────────────────────────────────────────────────
$violations violation(s) de l'ADR-0008 §A.

Un montant, une quote-part ou une valeur alimentant un seuil légal
doit être \`Decimal\` de bout en bout — jamais \`f64\`, et jamais via
un aller-retour Decimal→f64→Decimal.

Deux issues possibles :
  1. convertir le champ en \`Decimal\` (cas normal) ;
  2. si — et seulement si — la valeur n'est ni un montant ni une
     quotité (score, mesure physique, % d'affichage jamais comparé
     à un seuil), faire signer un amendement ADR-0008 puis
     l'ajouter à l'ALLOWLIST de ce script.

Ajouter une ligne à l'ALLOWLIST sans amendement signé revient à
désactiver le gate.
──────────────────────────────────────────────────────────────────
MSG
  exit 1
fi

echo "✅ ADR-0008 : aucun f64 monétaire hors liste fermée."
