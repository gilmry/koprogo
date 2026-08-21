<script lang="ts">
  // Story B3 (Phase B FE) — composant atomique réutilisable.
  //
  // Réutilisé par :
  //   - B3 Mandate (cette story) — table `MandateList` colonne "Expire".
  //   - B4 RoleDelegation        — colonne "Expire" + ligne "Délégation".
  //   - B6 SyndicResponse SLA    — pattern frère `SlaBadge` héritera de ce
  //     squelette en remplaçant "Expire" par "SLA".
  //
  // INV-FE9 (daltoniens) : on combine TROIS canaux d'information :
  //   1. couleur (vert / orange / rouge / gris)
  //   2. texte ("Expire dans 12 mois", "Expiré", etc.)
  //   3. icône SVG (warning / clock / cross) — `aria-hidden="true"` pour
  //      éviter doublon lecteur d'écran (le label est déjà sur le span).
  //
  // A11y : `aria-label` complet sur le `<span>` parent (lecteurs d'écran)
  // + texte visible pour vue claire ; pas de tooltip-only.
  //
  // Auto-refresh : `$effect` réveille `now` toutes les 60s pour qu'un
  // mandate qui passe d'urgent à expired soit visible sans reload.
  // Le timer est nettoyé via la return-fn de `$effect` (best practice
  // Svelte 5 runes — équivalent `onDestroy`).
  //
  // data-testid : composable via prop `idSuffix` — un parent peut écrire
  // `data-testid="mandate-expiration-badge-{id}"` en passant `idSuffix={id}`.
  // Sans idSuffix → `expiration-badge` (cf. stories.md §B3 data-testid table).

  import { expirationStatus } from "../../utils/dateBadge";

  let {
    validUntil,
    idSuffix = undefined,
    /** Permet d'injecter un `now` fixe en test (déterministe). */
    nowOverride = undefined,
  }: {
    validUntil: string | Date;
    idSuffix?: string | undefined;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // Reactive "now" — re-tick toutes les 60s pour basculer la palette quand
  // un mandate franchit un seuil (J-30 → soon, J-7 → urgent, J0 → expired).
  // -------------------------------------------------------------------------

  // ⚠ on lit `nowOverride` via `$effect` ci-dessous (jamais directement dans
  // l'initialiseur `$state`) pour éviter le warning Svelte 5
  // `state_referenced_locally` — même pattern que SlaBadge.svelte.
  let now = $state<Date>(new Date());

  $effect(() => {
    // En mode test (nowOverride fourni), on ne lance PAS de timer — on veut
    // une snapshot déterministe.
    if (nowOverride !== undefined) {
      now = nowOverride;
      return;
    }
    const intv = setInterval(() => {
      now = new Date();
    }, 60_000);
    return () => clearInterval(intv);
  });

  // -------------------------------------------------------------------------
  // Dérivations — pures, via le helper testé `dateBadge.ts`.
  // -------------------------------------------------------------------------

  let status = $derived(expirationStatus(validUntil, now));

  let classes = $derived(
    {
      fresh: "bg-green-100 text-green-800 border-green-300",
      soon: "bg-orange-100 text-orange-800 border-orange-300",
      urgent: "bg-red-100 text-red-800 border-red-300",
      expired: "bg-gray-200 text-gray-700 border-gray-300",
    }[status.level],
  );

  let testId = $derived(
    idSuffix !== undefined ? `expiration-badge-${idSuffix}` : "expiration-badge",
  );
</script>

<span
  data-testid={testId}
  data-level={status.level}
  data-days-remaining={status.daysRemaining}
  class={`inline-flex items-center gap-1 rounded border px-2 py-1 text-xs font-medium ${classes}`}
  role="status"
  aria-label={status.label}
>
  <!-- Icône : warning pour urgent/expired, clock pour soon, check pour fresh. -->
  {#if status.level === "urgent"}
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
  {:else if status.level === "expired"}
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
  {:else if status.level === "soon"}
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
  {:else}
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
  {/if}
  <span>{status.label}</span>
</span>
