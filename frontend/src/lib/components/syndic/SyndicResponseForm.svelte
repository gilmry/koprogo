<script lang="ts">
  // Story B6 (Phase B FE) — SyndicResponseForm (append-only).
  //
  // Parent BE story 3.7 — SyndicResponse (commit `c505c2b`).
  //
  // INV-FE8 (append-only) : ce form n'édite JAMAIS une response existante —
  // chaque submit crée une nouvelle entrée (POST). Pas de bouton "Edit" ni
  // ailleurs (cf. AC @security stories.md §B6).
  //
  // INV-FE9 (a11y) :
  //   - textarea + label associé via `for`/`id`.
  //   - counter chars avec aria-live="polite" pour annoncer min/max.
  //   - submit min-h-[44px] pour cible tactile WCAG 2.5.5.
  //
  // data-testid (cf. stories.md §B6 + mission) :
  //   syndic-response-body-textarea
  //   syndic-response-body-counter
  //   syndic-response-action-proposed-select
  //   syndic-response-submit
  //   syndic-response-form-error

  import { toast } from "../../../stores/toast";
  import {
    respondToTicket,
    SYNDIC_RESPONSE_ACTIONS,
    SYNDIC_RESPONSE_MIN_BODY_LENGTH,
    SYNDIC_RESPONSE_MAX_BODY_LENGTH,
    type CreateSyndicResponseRequest,
    type SyndicResponseAction,
    type SyndicResponseDto,
  } from "../../api/syndic_responses";

  // ---------------------------------------------------------------------------
  // Props (Svelte 5 runes) — DI sur l'API pour faciliter Vitest.
  // ---------------------------------------------------------------------------

  let {
    /** UUID du ticket cible — vient du parent (TicketDetail / page Astro). */
    ticketId,
    /** Callback après création réussie — le parent rafraîchit la conversation. */
    onCreated = undefined as undefined | ((r: SyndicResponseDto) => void),
    /** Injection de la fonction d'émission — facilite mock Vitest. */
    onRespond = respondToTicket as (
      id: string,
      req: CreateSyndicResponseRequest,
    ) => Promise<SyndicResponseDto>,
  }: {
    ticketId: string;
    onCreated?: (r: SyndicResponseDto) => void;
    onRespond?: (
      id: string,
      req: CreateSyndicResponseRequest,
    ) => Promise<SyndicResponseDto>;
  } = $props();

  // ---------------------------------------------------------------------------
  // State local
  // ---------------------------------------------------------------------------

  let body = $state<string>("");
  /** Optionnel — null = on n'envoie pas le champ. */
  let actionProposed = $state<SyndicResponseAction | "">("");
  let errorMessage = $state<string>("");
  let submitting = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  /** Longueur du body après trim — c'est ce que le backend valide. */
  let trimmedLength = $derived(body.trim().length);

  let tooShort = $derived(
    trimmedLength > 0 && trimmedLength < SYNDIC_RESPONSE_MIN_BODY_LENGTH,
  );
  let tooLong = $derived(trimmedLength > SYNDIC_RESPONSE_MAX_BODY_LENGTH);

  /** Submit disabled tant que body invalide ou submission en cours. */
  let submitDisabled = $derived(
    submitting ||
      trimmedLength < SYNDIC_RESPONSE_MIN_BODY_LENGTH ||
      trimmedLength > SYNDIC_RESPONSE_MAX_BODY_LENGTH,
  );

  let counterClasses = $derived(
    tooShort || tooLong
      ? "text-red-600 font-semibold"
      : "text-gray-500",
  );

  let counterLabel = $derived(
    tooShort
      ? `${trimmedLength} / ${SYNDIC_RESPONSE_MIN_BODY_LENGTH} minimum`
      : tooLong
        ? `${trimmedLength} / ${SYNDIC_RESPONSE_MAX_BODY_LENGTH} maximum dépassé`
        : `${trimmedLength} / ${SYNDIC_RESPONSE_MAX_BODY_LENGTH}`,
  );

  // ---------------------------------------------------------------------------
  // Helpers d'affichage
  // ---------------------------------------------------------------------------

  function actionLabel(a: SyndicResponseAction): string {
    const labels: Record<SyndicResponseAction, string> = {
      schedule_inspection: "Planifier une inspection",
      request_quote: "Demander un devis",
      closed_no_action: "Clôturer sans action",
      escalated_board: "Escalader au conseil",
      other: "Autre",
    };
    return labels[a];
  }

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  async function submit(): Promise<void> {
    if (submitDisabled) return;
    submitting = true;
    errorMessage = "";
    try {
      const req: CreateSyndicResponseRequest = {
        body: body.trim(),
        // Le backend accepte `null` ou champ absent — on envoie null pour être
        // explicite (cf. schema `action_proposed?: string | null`).
        action_proposed: actionProposed === "" ? null : actionProposed,
      };
      const created = await onRespond(ticketId, req);
      toast.success("Réponse postée.");
      onCreated?.(created);
      // Reset (append-only — pas de "draft" persistant).
      body = "";
      actionProposed = "";
    } catch (err) {
      // toast déjà émis par le wrapper api.ts pour 401/403/429/5xx ; on
      // affiche en plus un message inline pour 4xx-validation (422).
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      submitting = false;
    }
  }
</script>

<section
  class="syndic-response-form rounded-lg border border-gray-200 bg-white p-4 shadow-sm"
  aria-labelledby="syndic-response-form-title"
>
  <h3
    id="syndic-response-form-title"
    class="mb-3 text-base font-semibold text-gray-900"
  >
    Répondre au ticket
  </h3>

  <form
    class="space-y-3"
    onsubmit={(e: SubmitEvent) => {
      e.preventDefault();
      void submit();
    }}
  >
    <!-- Body textarea -->
    <div>
      <label
        for="syndic-response-body-textarea"
        class="block text-sm font-medium text-gray-700"
      >
        Message
      </label>
      <textarea
        id="syndic-response-body-textarea"
        data-testid="syndic-response-body-textarea"
        bind:value={body}
        rows="4"
        minlength={SYNDIC_RESPONSE_MIN_BODY_LENGTH}
        maxlength={SYNDIC_RESPONSE_MAX_BODY_LENGTH}
        placeholder="Détaillez votre réponse au copropriétaire…"
        class="mt-1 w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
        aria-describedby="syndic-response-body-counter"
        aria-invalid={tooShort || tooLong ? "true" : "false"}
      ></textarea>
      <p
        id="syndic-response-body-counter"
        data-testid="syndic-response-body-counter"
        class={`mt-1 text-xs ${counterClasses}`}
        aria-live="polite"
      >
        {counterLabel}
      </p>
    </div>

    <!-- Action proposée (optionnelle) -->
    <div>
      <label
        for="syndic-response-action-proposed-select"
        class="block text-sm font-medium text-gray-700"
      >
        Action proposée (optionnel)
      </label>
      <select
        id="syndic-response-action-proposed-select"
        data-testid="syndic-response-action-proposed-select"
        bind:value={actionProposed}
        class="mt-1 min-h-[44px] w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
      >
        <option value="">— Aucune action —</option>
        {#each SYNDIC_RESPONSE_ACTIONS as a (a)}
          <option value={a}>{actionLabel(a)}</option>
        {/each}
      </select>
    </div>

    <!-- Erreur backend (422 typiquement) -->
    {#if errorMessage}
      <p
        data-testid="syndic-response-form-error"
        class="rounded-md border border-red-300 bg-red-50 p-3 text-sm text-red-700"
        role="alert"
        aria-live="polite"
      >
        {errorMessage}
      </p>
    {/if}

    <!-- Submit -->
    <button
      data-testid="syndic-response-submit"
      type="submit"
      disabled={submitDisabled}
      class="min-h-[44px] rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:bg-gray-300"
    >
      {submitting ? "Envoi…" : "Poster la réponse"}
    </button>
  </form>
</section>
