<script lang="ts">
  // Track H Story H1 — Banner conformité immeuble (FR-H1 / INV-H1).
  //
  // Rendu UNIQUEMENT quand `status.is_conformant === false`. Affiche un
  // message narratif structuré : « lots manquants » + « somme quotas en deçà
  // de X / basis (acte de base) » + « contactez l'admin ».
  //
  // Décisions :
  //  - A11y : `role="alert"` + `aria-live="polite"` — annoncé au lecteur
  //    d'écran sans interrompre. Texte explicite (mémoire `a11y-wcag-aa-baseline`).
  //  - i18n : `$_` interpolation `{values: {...}}` — quota_basis interpolé
  //    pour supporter 1000 / 10000 / autre (bug fix Story H1).
  //  - Decimal-as-string : `quota_delta` jamais parseFloat (mémoire
  //    `no-f64-in-money` + ADR-0007). Affichage FR-BE via `formatDecimalFRBE`.
  //  - data-testid stables (mémoire `data-testid-systematic`).

  import { _ } from "../../i18n";
  import { formatDecimalFRBE } from "../../utils/conformity";
  import type { ConformityStatus } from "../../types/conformity";

  let {
    status,
    buildingId,
    buildingName,
  }: {
    status: ConformityStatus;
    buildingId: string;
    buildingName: string;
  } = $props();

  // Convention BE Track H Story H1 : `quota_delta = quota_basis - quota_sum`.
  // Positif = manque (cas drift typique). Négatif = surplus (cas excès).
  let quotaDeltaIsZero = $derived(
    status.quota_delta === "0" ||
      status.quota_delta === "0.0" ||
      status.quota_delta === "+0" ||
      status.quota_delta === "-0",
  );

  // Absolute delta in display form (sans signe) pour le message « X / basis ».
  let quotaDeltaDisplay = $derived(
    formatDecimalFRBE(
      status.quota_delta.startsWith("-")
        ? status.quota_delta.slice(1)
        : status.quota_delta,
    ),
  );

  let unitsDeltaLabel = $derived(
    status.units_delta > 0
      ? $_("conformity.units_missing", { values: { n: status.units_delta } })
      : status.units_delta < 0
        ? $_("conformity.units_extra", {
            values: { n: Math.abs(status.units_delta) },
          })
        : "",
  );

  // Identifiant unique pour le label aria — sécurise l'a11y si plusieurs
  // banners sur la page (peu probable mais robuste).
  let bannerLabelId = $derived(`conformity-banner-title-${buildingId}`);
</script>

{#if !status.is_conformant}
  <div
    role="alert"
    aria-live="polite"
    aria-labelledby={bannerLabelId}
    class="bg-red-50 border-l-4 border-red-500 text-red-800 p-4 my-4 flex items-start gap-3"
    data-testid="conformity-banner"
    data-building-id={buildingId}
  >
    <span aria-hidden="true" class="text-xl">⚠️</span>
    <div class="flex-1">
      <strong id={bannerLabelId} data-testid="conformity-banner-title">
        {$_("conformity.banner_title", { values: { name: buildingName } })}
      </strong>
      <ul class="mt-1 list-disc list-inside text-sm">
        {#if status.units_delta !== 0}
          <li data-testid="conformity-units-delta">{unitsDeltaLabel}</li>
        {/if}
        {#if !quotaDeltaIsZero}
          <li
            data-testid="conformity-quota-delta"
            data-basis={status.quota_basis}
          >
            {$_("conformity.quota_off", {
              values: {
                delta: quotaDeltaDisplay,
                basis: status.quota_basis,
              },
            })}
            <span data-testid="conformity-quota-basis" class="sr-only"
              >/ {status.quota_basis}</span
            >
          </li>
        {/if}
      </ul>
      <p class="mt-1 text-sm" data-testid="conformity-contact-admin">
        {$_("conformity.contact_admin")}
      </p>
    </div>
  </div>
{/if}
