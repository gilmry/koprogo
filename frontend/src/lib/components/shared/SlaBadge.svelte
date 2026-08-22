<script lang="ts">
  // Story B6 (Phase B FE) — composant atomique réutilisable.
  //
  // Sœur sémantique de `ExpirationBadge.svelte` (B3/B4) : même squelette
  // (couleur + texte + icône, INV-FE9 daltoniens) mais sémantique SLA.
  //
  // Réutilisé par :
  //   - B6 SyndicResponse SLA — ticket-detail page (cette story).
  //   - Futur : compteurs SLA contrats prestataires (post-B8).
  //
  // INV-FE9 (daltoniens) : on combine TROIS canaux d'information :
  //   1. couleur (vert / orange / rouge)
  //   2. texte ("Sous SLA (3j)", "Échéance dans 6h ⚠", "Hors SLA")
  //   3. icône SVG (check / clock / warning / cross) — `aria-hidden="true"`.
  //
  // A11y : `aria-label` complet sur le `<span>` parent (lecteurs d'écran)
  // + texte visible pour vue claire ; pas de tooltip-only. data-testid
  // composable via `idSuffix` ; tooltip optionnel via prop `dueTooltip`.
  //
  // Auto-refresh : `$effect` réveille `now` toutes les 60s pour qu'un
  // ticket qui franchit l'échéance bascule visuellement sans reload
  // (même pattern qu'ExpirationBadge).

  import { slaStatus } from "../../utils/dateBadge";

  let {
    /** ISO 8601 du `sla_due_at` (backend ticket). */
    dueAt,
    /** ISO 8601 de la 1re réponse syndic (`null` si pas encore répondu). */
    respondedAt = null,
    /** ISO 8601 de la création du ticket — pour calculer le % temps restant. */
    createdAt = undefined,
    /** Suffixe data-testid (ex: ticket id pour `ticket-sla-badge-{id}`). */
    idSuffix = undefined,
    /** Tooltip optionnel — affiché en `title` natif pour debug syndic. */
    dueTooltip = undefined,
    /** Permet d'injecter un `now` fixe en test (déterministe). */
    nowOverride = undefined,
  }: {
    dueAt: string | Date;
    respondedAt?: string | Date | null;
    createdAt?: string | Date | undefined;
    idSuffix?: string | undefined;
    dueTooltip?: string | undefined;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // Reactive "now" — re-tick toutes les 60s pour basculer la palette quand
  // un ticket franchit un seuil SLA (50% → warning, 25% → urgent, due → breach).
  // -------------------------------------------------------------------------

  // ⚠ on lit `nowOverride` via une lambda d'initialisation pour éviter le
  // warning Svelte 5 `state_referenced_locally` (capture-initial-value).
  let tickedNow = $state<Date>(new Date());

  $effect(() => {
    if (nowOverride !== undefined) {
      tickedNow = nowOverride;
      return;
    }
    tickedNow = new Date();
    const intv = setInterval(() => {
      tickedNow = new Date();
    }, 60_000);
    return () => clearInterval(intv);
  });

  let now = $derived(nowOverride ?? tickedNow);

  // -------------------------------------------------------------------------
  // Dérivations — pures, via le helper testé `dateBadge.ts`.
  // -------------------------------------------------------------------------

  let status = $derived(slaStatus(dueAt, respondedAt, createdAt, now));

  let classes = $derived(
    {
      met: "bg-green-100 text-green-800 border-green-300",
      fresh: "bg-green-100 text-green-800 border-green-300",
      warning: "bg-orange-100 text-orange-800 border-orange-300",
      urgent: "bg-red-100 text-red-800 border-red-300",
      breached: "bg-red-100 text-red-800 border-red-300",
    }[status.level],
  );

  let testId = $derived(
    idSuffix !== undefined ? `ticket-sla-badge-${idSuffix}` : "ticket-sla-badge",
  );

  let tooltipTestId = $derived(
    idSuffix !== undefined
      ? `ticket-sla-due-tooltip-${idSuffix}`
      : "ticket-sla-due-tooltip",
  );
</script>

<span
  data-testid={testId}
  data-level={status.level}
  data-remaining-ratio={status.remainingRatio.toFixed(2)}
  data-response-delta-hours={status.responseDeltaHours ?? ""}
  class={`inline-flex items-center gap-1 rounded border px-2 py-1 text-xs font-medium ${classes}`}
  role="status"
  aria-label={status.label}
  title={dueTooltip ?? undefined}
>
  <!-- Icône : check (met/fresh), clock (warning), warning (urgent), cross (breached) -->
  {#if status.level === "met" || status.level === "fresh"}
    <svg
      aria-hidden="true"
      width="12"
      height="12"
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <path d="M2 6 L5 9 L10 3" />
    </svg>
  {:else if status.level === "warning"}
    <svg
      aria-hidden="true"
      width="12"
      height="12"
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <circle cx="6" cy="6" r="5" />
      <path d="M6 3 V6 L8 8" />
    </svg>
  {:else if status.level === "urgent"}
    <svg
      aria-hidden="true"
      width="12"
      height="12"
      viewBox="0 0 12 12"
      fill="currentColor"
    >
      <path
        d="M6 0 L11 11 L1 11 Z M6 4 V8 M6 9 V10"
        stroke="currentColor"
        stroke-width="1"
        fill="none"
      />
    </svg>
  {:else}
    <!-- breached → cross -->
    <svg
      aria-hidden="true"
      width="12"
      height="12"
      viewBox="0 0 12 12"
      fill="none"
      stroke="currentColor"
      stroke-width="1.5"
    >
      <line x1="2" y1="2" x2="10" y2="10" />
      <line x1="10" y1="2" x2="2" y2="10" />
    </svg>
  {/if}
  <span>{status.label}</span>
  {#if dueTooltip}
    <span data-testid={tooltipTestId} class="sr-only">{dueTooltip}</span>
  {/if}
</span>
