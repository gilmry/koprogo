<script lang="ts">
  // Story B5 (Phase B FE) — composant atomique SeveritySelector.
  //
  // Réutilisé par :
  //   - TicketCreate.svelte — section conditionnelle si kind=Complaint.
  //
  // Contrat métier (cf. api.d.ts TicketSeverity) : 4 niveaux fermés
  //   low | normal | high | critical
  // Toute valeur hors set est impossible côté UI (set fermé garanti) —
  // backend renvoie 422 si bypass DevTools.
  //
  // a11y (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`) :
  //   - <fieldset><legend> pour grouper les radios (lecteur d'écran
  //     annonce "Severity, group of 4").
  //   - aria-required="true" pour signaler le champ obligatoire.
  //   - Couleur ≠ unique indicateur — texte + radio visuel (INV-FE9
  //     daltoniens, cf. ExpirationBadge.svelte commentaire).
  //
  // data-testid (cf. stories.md §B5 + mission) :
  //   ticket-severity-radio-low / -normal / -high / -critical

  import type { components } from "../../../types/api";

  // Set fermé re-exporté pour le composant + parent.
  export const SEVERITY_VALUES = [
    "low",
    "normal",
    "high",
    "critical",
  ] as const;
  type TicketSeverity = components["schemas"]["TicketSeverity"];

  let {
    value = $bindable<TicketSeverity | "">(""),
    /** Nom du group radio — permet plusieurs SeveritySelector sur la même
     *  page sans collision (cf. pattern MandateIssueForm scope). */
    name = "ticket-severity",
    /** Si true, marque le champ requis (aria-required + visuel). */
    required = false,
  }: {
    value?: TicketSeverity | "";
    name?: string;
    required?: boolean;
  } = $props();

  // Labels FR — i18n NL/EN/DE viendra avec Story B12+ (cf. dateBadge.ts).
  const LABELS: Record<TicketSeverity, string> = {
    low: "Basse",
    normal: "Normale",
    high: "Haute",
    critical: "Critique",
  };
</script>

<!-- aria-required n'est pas supporté sur le role implicite "group" du
     <fieldset> ; on l'expose via data-required pour la lecture par les
     tests, et le marquage visuel + asterisque assure la sémantique
     pour l'utilisateur. Le `required` HTML est porté par chaque <input>. -->
<fieldset
  class="ticket-severity-selector flex flex-col gap-2 rounded border border-gray-200 bg-white p-3"
  data-required={required}
>
  <legend class="text-sm font-medium text-gray-700">
    Gravité {#if required}<span aria-hidden="true" class="text-red-600">*</span
      >{/if}
  </legend>
  <div class="flex flex-col gap-1 sm:flex-row sm:gap-4">
    {#each SEVERITY_VALUES as sev (sev)}
      <label class="inline-flex items-center gap-2 text-sm">
        <input
          type="radio"
          {name}
          value={sev}
          bind:group={value}
          data-testid={`ticket-severity-radio-${sev}`}
          {required}
          class="h-4 w-4 cursor-pointer text-blue-600 focus-visible:outline-2 focus-visible:outline-offset-2"
        />
        <span>{LABELS[sev]}</span>
      </label>
    {/each}
  </div>
</fieldset>
