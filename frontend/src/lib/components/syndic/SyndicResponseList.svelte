<script lang="ts">
  // Story B6 (Phase B FE) — SyndicResponseList (read-only conversation).
  //
  // Parent BE story 3.7 — SyndicResponse (commit `c505c2b`).
  //
  // INV-FE8 (append-only) : aucune action de mutation sur les lignes — pas
  // de bouton "Edit" ni "Delete" (cf. AC @security stories.md §B6).
  //
  // Pattern : on accepte une prop `initialResponses` (build Astro SSG /
  // test) ET on peut
  // fetcher au mount si non fournie. Le parent peut rafraîchir via
  // `onCreated` du `SyndicResponseForm`.
  //
  // data-testid (cf. stories.md §B6) :
  //   syndic-response-list
  //   syndic-response-row-{id}
  //   syndic-response-row-author-{id}
  //   syndic-response-row-action-{id}
  //   syndic-response-row-body-{id}
  //   syndic-response-row-timestamp-{id}

  import { _ } from "../../i18n";
  import {
    listResponsesForTicket,
    type SyndicResponseDto,
  } from "../../api/syndic_responses";

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  let {
    ticketId,
    /** Liste initiale (build Astro SSG / injection test). Si non fournie → fetch au mount. */
    initialResponses = undefined,
    /** Map syndic_user_id → label humain (fallback UUID slice). */
    authorLabels = {},
  }: {
    ticketId: string;
    initialResponses?: SyndicResponseDto[];
    authorLabels?: Record<string, string>;
  } = $props();

  // ---------------------------------------------------------------------------
  // State local
  // ---------------------------------------------------------------------------

  let responses = $state<SyndicResponseDto[]>([]);
  let loading = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Fetch initial si pas de liste injectée
  // ---------------------------------------------------------------------------

  async function fetchResponses(): Promise<void> {
    loading = true;
    try {
      responses = await listResponsesForTicket(ticketId);
    } catch {
      // toast déjà émis par api.ts pour 4xx/5xx
    } finally {
      loading = false;
    }
  }

  // Sync prop → state au mount + à chaque changement de prop. Si la prop
  // est `undefined`, on fetche au lieu de remplacer.
  $effect(() => {
    if (initialResponses !== undefined) {
      responses = initialResponses;
    } else {
      void fetchResponses();
    }
  });

  // ---------------------------------------------------------------------------
  // Helpers d'affichage
  // ---------------------------------------------------------------------------

  function authorLabel(r: SyndicResponseDto): string {
    return authorLabels[r.syndic_user_id] ?? r.syndic_user_id.slice(0, 8);
  }

  function actionLabel(action: string | null | undefined): string {
    if (!action) return "";
    const labels: Record<string, string> = {
      schedule_inspection: "Planifier inspection",
      request_quote: "Demander devis",
      closed_no_action: "Clôturé sans action",
      escalated_board: "Escaladé au conseil",
      other: "Autre",
    };
    return labels[action] ?? action;
  }

  function formatTimestamp(iso: string): string {
    try {
      const d = new Date(iso);
      const dateFmt = new Intl.DateTimeFormat("fr-BE", {
        day: "numeric",
        month: "long",
        year: "numeric",
      }).format(d);
      const timeFmt = new Intl.DateTimeFormat("fr-BE", {
        hour: "2-digit",
        minute: "2-digit",
      }).format(d);
      return `${dateFmt} à ${timeFmt}`;
    } catch {
      return iso;
    }
  }
</script>

<section
  class="syndic-response-list-section flex flex-col gap-3"
  aria-labelledby="syndic-response-list-title"
>
  <h3
    id="syndic-response-list-title"
    class="text-base font-semibold text-gray-900"
  >
    {$_("ticket.syndic_responses.title") || "Réponses du syndic"}
  </h3>

  {#if loading && responses.length === 0}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      {$_("common.loading") || "Chargement…"}
    </p>
  {:else if responses.length === 0}
    <p class="text-sm text-gray-500" data-testid="syndic-response-list-empty">
      {$_("ticket.syndic_responses.empty") || "Aucune réponse pour l'instant."}
    </p>
  {:else}
    <ol
      data-testid="syndic-response-list"
      class="space-y-3"
      aria-label="Conversation chronologique"
    >
      {#each responses as r (r.id)}
        <li
          data-testid={`syndic-response-row-${r.id}`}
          class="rounded-md border border-gray-200 bg-gray-50 p-3 text-sm"
        >
          <div class="mb-1 flex items-center justify-between gap-2 text-xs text-gray-600">
            <span
              data-testid={`syndic-response-row-author-${r.id}`}
              class="font-medium text-gray-900"
            >
              {authorLabel(r)}
            </span>
            <time
              data-testid={`syndic-response-row-timestamp-${r.id}`}
              datetime={r.created_at}
              class="text-gray-500"
            >
              {formatTimestamp(r.created_at)}
            </time>
          </div>
          {#if r.action_proposed}
            <p
              data-testid={`syndic-response-row-action-${r.id}`}
              class="mb-1 inline-block rounded bg-blue-100 px-2 py-0.5 text-xs font-medium text-blue-800"
            >
              {actionLabel(r.action_proposed)}
            </p>
          {/if}
          <p
            data-testid={`syndic-response-row-body-${r.id}`}
            class="whitespace-pre-wrap text-gray-800"
          >
            {r.body}
          </p>
        </li>
      {/each}
    </ol>
  {/if}
</section>
