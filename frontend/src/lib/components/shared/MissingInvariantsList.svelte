<script lang="ts">
  // Track H Story H3 — Liste des invariants Art. 3.87 §3-5 CC manquants pour
  // clôturer une AG. Rendu en dessous du bouton « Clôturer » sur la fiche
  // meeting. Permet au syndic de voir les conditions à remplir.
  //
  // Décisions :
  //  - A11y : `<ul role="list">` + chaque `<li>` est lisible au screen reader.
  //    Le bouton « Clôturer » associé (côté MeetingDetail) doit pointer vers
  //    cet `<ul>` via `aria-describedby` pour expliciter la cause du disable.
  //  - i18n : `$_` interpolation des params (open_resolutions, quotas).
  //    Clés `meeting.missing.{ConvocationsNotSent | VotesNotClosed | ...}`.
  //  - Decimal-as-string : les quotas QuorumNotReached restent string
  //    (mémoire `no-f64-in-money` + ADR-0007).
  //  - data-testid stables : `missing-invariants-list` + `missing-invariant-<lowercase-type>`
  //    (mémoire `data-testid-systematic`).
  //  - Robustesse : type inconnu → fallback safe (label générique, log
  //    debug). Ne crash JAMAIS.

  import { _ } from "../../i18n";
  import type { MissingInvariant } from "../../types/meeting";

  let { invariants }: { invariants: MissingInvariant[] } = $props();

  /**
   * Construit la clé i18n + les valeurs interpolées pour un invariant donné.
   * Retourne `null` pour les types inconnus (fallback géré côté template).
   */
  function labelFor(inv: MissingInvariant): {
    key: string;
    values: Record<string, string | number>;
  } | null {
    switch (inv.type) {
      case "ConvocationsNotSent":
        return { key: "meeting.missing.ConvocationsNotSent", values: {} };
      case "VotesNotClosed":
        return {
          key: "meeting.missing.VotesNotClosed",
          values: { open_resolutions: inv.open_resolutions },
        };
      case "AttendanceNotRecorded":
        return { key: "meeting.missing.AttendanceNotRecorded", values: {} };
      case "QuorumNotReached":
        return {
          key: "meeting.missing.QuorumNotReached",
          values: {
            attended_quotas: inv.attended_quotas,
            total_quotas: inv.total_quotas,
          },
        };
      case "MinutesDraftMissing":
        return { key: "meeting.missing.MinutesDraftMissing", values: {} };
      default:
        // Type inconnu (drift FE vs BE schema, ou nouveau variant non
        // déployé côté FE). Pas de crash : on log + on rend un label safe.
        // Le `default` est inatteignable selon TS strict mais protège
        // l'exécution runtime.
        return null;
    }
  }

  /**
   * Renvoie un suffixe stable pour le data-testid (lowercase du type).
   * Pour les types inconnus, "unknown" est utilisé.
   */
  function testIdSuffix(inv: MissingInvariant): string {
    return inv.type.toLowerCase();
  }
</script>

<ul
  class="space-y-2 list-none my-2"
  data-testid="missing-invariants-list"
  aria-label={$_("meeting.complete.missing_aria_label")}
>
  {#each invariants as inv (inv.type + JSON.stringify(inv))}
    {@const label = labelFor(inv)}
    <li
      class="flex items-start gap-2 text-sm text-red-800"
      data-testid="missing-invariant-{testIdSuffix(inv)}"
    >
      <span aria-hidden="true" class="text-red-600 font-bold">✗</span>
      <span>
        {#if label}
          {$_(label.key, { values: label.values })}
        {:else}
          <!-- Fallback safe pour type inconnu : on affiche le type brut sans
               jamais lever ni laisser un champ vide. -->
          {inv.type}
        {/if}
      </span>
    </li>
  {/each}
</ul>
