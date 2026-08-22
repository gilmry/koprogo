<script lang="ts">
  // Story 1.4 — Badge conformité immeuble (FR11, INV-1).
  //
  // Affiche `is_conformant` (count(units)==total_units && SUM(quotas)==1000)
  // + détail count + somme quotas + delta lisible.
  //
  // Décisions techniques :
  // - `quotaSum` / `quotaDelta` sont des **strings** (Decimal-as-string côté
  //   backend, cf. ADR-0007 + mémoire `no-f64-in-money`). On ne fait JAMAIS
  //   `parseFloat()` / `Number()` dessus — affichage formaté FR-BE via une
  //   helper qui préserve la précision décimale en string.
  // - data-testid stable, i18n-safe, refactor-safe (cf. mémoire `data-testid-systematic`).
  // - Couleurs WCAG 2.1 AA (contraste validé — cf. mémoire `a11y-wcag-aa-baseline`).
  import { _ } from "../../lib/i18n";

  let {
    isConformant,
    unitsCount,
    totalUnits,
    quotaSum,
    quotaDelta,
  }: {
    isConformant: boolean;
    unitsCount: number;
    totalUnits: number;
    /** Decimal-as-string (ex: "1000", "999.5", "0"). NE PAS parseFloat ! */
    quotaSum: string;
    /** Decimal-as-string signed (ex: "0", "-1", "500"). */
    quotaDelta: string;
  } = $props();

  /**
   * Formatte un Decimal-string en notation FR-BE (séparateur virgule, sans
   * conversion float qui détruirait la précision Decimal).
   *
   * Stratégie : on remplace UNIQUEMENT le point décimal par une virgule ;
   * aucun parseFloat — la string reste fidèle bit-à-bit.
   */
  function formatDecimalFRBE(s: string): string {
    if (!s) return "—";
    // strip leading "+" si présent ; conserver "-" pour les négatifs
    const cleaned = s.startsWith("+") ? s.slice(1) : s;
    return cleaned.replace(".", ",");
  }

  /** Signe lisible pour le delta ("+500" / "0" / "-1"). */
  function formatDeltaSigned(s: string): string {
    if (!s) return "—";
    const cleaned = s.startsWith("+") ? s.slice(1) : s;
    if (cleaned === "0" || cleaned === "0.0") return "0";
    if (cleaned.startsWith("-")) return formatDecimalFRBE(cleaned);
    return "+" + formatDecimalFRBE(cleaned);
  }

  // Convention backend (building.rs) : quota_delta = total_tantiemes - quota_sum,
  // donc un DÉFICIT (quotas manquants) est POSITIF et un SURPLUS est NÉGATIF -
  // l'inverse de ce qu'on lirait naïvement. Un déficit (ex: immeuble sans
  // aucune unité créée) est le cas le plus grave -> rouge ; un surplus (ex:
  // léger dépassement d'arrondi au-dessus de 1000) -> orange.
  let badgeClass = $derived(
    isConformant
      ? "bg-green-100 text-green-800 border-green-300"
      : quotaDelta.startsWith("-")
        ? "bg-orange-100 text-orange-800 border-orange-300"
        : "bg-red-100 text-red-800 border-red-300",
  );

  let badgeIcon = $derived(isConformant ? "✅" : "⚠️");
  let badgeLabelKey = $derived(
    isConformant
      ? "buildings.conformity.conformant"
      : "buildings.conformity.nonConformant",
  );
</script>

<div
  class="inline-flex flex-col items-start gap-1 p-3 rounded-lg border {badgeClass}"
  data-testid="building-conformity-badge"
  role="status"
  aria-live="polite"
>
  <div class="flex items-center gap-2 font-semibold text-sm">
    <span aria-hidden="true">{badgeIcon}</span>
    <span>{$_(badgeLabelKey)}</span>
  </div>

  <dl class="grid grid-cols-[auto_1fr] gap-x-2 text-xs">
    <dt>{$_("buildings.conformity.unitsCount")}:</dt>
    <dd data-testid="building-units-count">
      <span>{unitsCount}</span>
      <span class="opacity-70"> / {totalUnits}</span>
    </dd>

    <dt>{$_("buildings.conformity.quotaSum")}:</dt>
    <dd data-testid="building-quota-sum">
      {formatDecimalFRBE(quotaSum)}
      <span class="opacity-70">{$_("buildings.conformity.expected1000")}</span>
    </dd>

    <dt>{$_("buildings.conformity.quotaDelta")}:</dt>
    <dd data-testid="building-quota-delta">
      {formatDeltaSigned(quotaDelta)}
    </dd>
  </dl>
</div>
