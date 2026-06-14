<script lang="ts">
  // Story B5 (Phase B FE) — TicketCreate (form Owner-facing, full version).
  //
  // Parent BE story 3.6 — Ticket complaint (`2142019`).
  //
  // Contrat métier :
  //   - Form Owner-facing pour créer Request (par défaut) OU Complaint.
  //   - Si kind=Complaint → sections additionnelles visibles :
  //     * SeveritySelector (low/normal/high/critical, requis)
  //     * incident_date (date passée requise)
  //     * EvidenceUpload (0-10 fichiers, optionnel → badge "Preuves
  //       manquantes" si vide @edge)
  //     * WitnessSelector (0-10 témoins, optionnel)
  //   - Si kind=Request → comportement strict identique au form initial
  //     (rétro-compat : tests existants verts, cf. mission).
  //
  // a11y (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`) :
  //   - Chaque champ a un <label> lié via for/id.
  //   - Erreurs inline avec aria-describedby + aria-invalid.
  //   - Counter description aria-live="polite" pour annoncer min/max.
  //   - Submit min-h-[44px] cible tactile WCAG 2.5.5.
  //
  // data-testid (cf. stories.md §B5 + mission) :
  //   ticket-create-kind-select / -title-input / -description-textarea /
  //   ticket-create-description-counter / -category-select / -priority-select
  //   ticket-create-incident-date-input
  //   ticket-create-submit
  //   ticket-create-evidence-warning (badge "Preuves manquantes" si Complaint
  //     sans evidence ni witness)
  //   ticket-create-error
  //
  // Pattern d'injection : `onCreate` injectable pour Vitest (pas vi.mock global).

  import { ticketsApi, TicketPriority, TicketCategory, TicketKind } from "../../api/tickets";
  import type {
    CreateTicketDto,
    Ticket,
    TicketSeverity,
  } from "../../api/tickets";
  import SeveritySelector from "./SeveritySelector.svelte";
  import EvidenceUpload from "./EvidenceUpload.svelte";
  import WitnessSelector from "./WitnessSelector.svelte";
  import type { WitnessCandidate } from "./WitnessSelector.svelte";

  // ---------------------------------------------------------------------------
  // Constantes métier (cf. backend Ticket entity)
  // ---------------------------------------------------------------------------

  /** Title : 5..200 chars (cf. BE Ticket aggregate). */
  export const TICKET_TITLE_MIN = 5;
  export const TICKET_TITLE_MAX = 200;
  /** Description : 20..5000 chars (Story B5 — 20 min pour Complaint dossier). */
  export const TICKET_DESCRIPTION_MIN = 20;
  export const TICKET_DESCRIPTION_MAX = 5000;

  let {
    /** UUID du building cible. */
    buildingId,
    /** UUID du lot — optionnel. */
    unitId = undefined as string | undefined,
    /** Owners du building (pour autocomplete WitnessSelector). */
    witnessCandidates = [] as WitnessCandidate[],
    /** UserId du current user — bloque self-witness. */
    currentUserId = "" as string,
    /** Callback succès — parent peut rediriger. */
    onCreated = undefined as ((t: Ticket) => void) | undefined,
    /** Callback annulation — parent peut close modal / nav back. */
    onCancel = undefined as (() => void) | undefined,
    /** Injection ticketsApi.create pour faciliter Vitest (pas vi.mock global). */
    onCreate = ticketsApi.create.bind(ticketsApi) as (
      data: CreateTicketDto,
    ) => Promise<Ticket>,
  }: {
    buildingId: string;
    unitId?: string;
    witnessCandidates?: WitnessCandidate[];
    currentUserId?: string;
    onCreated?: (t: Ticket) => void;
    onCancel?: () => void;
    onCreate?: (data: CreateTicketDto) => Promise<Ticket>;
  } = $props();

  // ---------------------------------------------------------------------------
  // State formulaire
  // ---------------------------------------------------------------------------

  let kind = $state<typeof TicketKind.Request | typeof TicketKind.Complaint>(
    TicketKind.Request,
  );
  let title = $state<string>("");
  let description = $state<string>("");
  let priority = $state<TicketPriority>(TicketPriority.Medium);
  let category = $state<TicketCategory>(TicketCategory.Other);

  // Sections conditionnelles (kind=Complaint)
  let severity = $state<TicketSeverity | "">("");
  let incidentDate = $state<string>(""); // YYYY-MM-DD (input type=date)
  let evidenceAttachments = $state<string[]>([]);
  let witnesses = $state<string[]>([]);

  let submitting = $state<boolean>(false);
  let errorMessage = $state<string>("");

  // ---------------------------------------------------------------------------
  // Derivations
  // ---------------------------------------------------------------------------

  let isComplaint = $derived(kind === TicketKind.Complaint);

  let descLength = $derived(description.trim().length);
  let titleLength = $derived(title.trim().length);

  let descTooShort = $derived(
    descLength > 0 && descLength < TICKET_DESCRIPTION_MIN,
  );
  let descTooLong = $derived(descLength > TICKET_DESCRIPTION_MAX);

  let descCounterClasses = $derived(
    descTooShort || descTooLong
      ? "text-red-600 font-semibold"
      : "text-gray-500",
  );

  let descCounterLabel = $derived(
    descTooShort
      ? `${descLength} / ${TICKET_DESCRIPTION_MIN} minimum`
      : descTooLong
        ? `${descLength} / ${TICKET_DESCRIPTION_MAX} maximum dépassé`
        : `${descLength} / ${TICKET_DESCRIPTION_MAX}`,
  );

  /** incident_date dans le futur ? (granularité jour) */
  let incidentDateInFuture = $derived.by<boolean>(() => {
    if (!incidentDate) return false;
    const target = new Date(`${incidentDate}T00:00:00Z`);
    const today = new Date();
    // Comparaison jour-civil UTC.
    const todayUtcDay = Date.UTC(
      today.getUTCFullYear(),
      today.getUTCMonth(),
      today.getUTCDate(),
    );
    return target.getTime() > todayUtcDay;
  });

  /** Affiche le badge "Preuves manquantes" — Complaint text-only. */
  let evidenceWarning = $derived(
    isComplaint &&
      evidenceAttachments.length === 0 &&
      witnesses.length === 0,
  );

  let titleValid = $derived(
    titleLength >= TICKET_TITLE_MIN && titleLength <= TICKET_TITLE_MAX,
  );
  let descValid = $derived(
    descLength >= TICKET_DESCRIPTION_MIN && descLength <= TICKET_DESCRIPTION_MAX,
  );
  let complaintFieldsValid = $derived(
    !isComplaint ||
      (severity !== "" && incidentDate !== "" && !incidentDateInFuture),
  );

  let submitDisabled = $derived(
    submitting || !titleValid || !descValid || !complaintFieldsValid,
  );

  /** Date max input HTML (=aujourd'hui, pas le futur). */
  let maxIncidentDate = $derived.by<string>(() => {
    return new Date().toISOString().slice(0, 10);
  });

  // ---------------------------------------------------------------------------
  // Submit
  // ---------------------------------------------------------------------------

  async function handleSubmit(ev: SubmitEvent): Promise<void> {
    ev.preventDefault();
    if (submitDisabled) return;
    submitting = true;
    errorMessage = "";
    try {
      const dto: CreateTicketDto = {
        building_id: buildingId,
        title: title.trim(),
        description: description.trim(),
        priority,
        category,
      };
      if (unitId && unitId.trim() !== "") dto.unit_id = unitId;
      if (isComplaint) {
        dto.kind = TicketKind.Complaint;
        // severity garanti non-vide par submitDisabled — narrow type.
        if (severity !== "") dto.severity = severity;
        // incident_date en début de jour UTC.
        if (incidentDate)
          dto.incident_date = `${incidentDate}T00:00:00Z`;
        if (evidenceAttachments.length > 0)
          dto.evidence_attachments = evidenceAttachments;
        if (witnesses.length > 0) dto.witnesses = witnesses;
      } else {
        dto.kind = TicketKind.Request;
      }
      const created = await onCreate(dto);
      onCreated?.(created);
      // Reset complet (Owner peut créer plusieurs tickets d'affilée).
      title = "";
      description = "";
      severity = "";
      incidentDate = "";
      evidenceAttachments = [];
      witnesses = [];
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;
    } finally {
      submitting = false;
    }
  }
</script>

<form
  class="ticket-create-form flex flex-col gap-4 rounded bg-white p-4 shadow-sm"
  onsubmit={handleSubmit}
  aria-labelledby="ticket-create-title-h"
  novalidate
>
  <h2
    id="ticket-create-title-h"
    class="text-lg font-semibold text-gray-900"
  >
    {isComplaint ? "Déposer une plainte" : "Créer un ticket"}
  </h2>

  <!-- Kind selector (Request | Complaint) -->
  <div class="flex flex-col gap-1">
    <label
      for="ticket-create-kind-select"
      class="text-sm font-medium text-gray-700"
    >
      Type
    </label>
    <select
      id="ticket-create-kind-select"
      data-testid="ticket-create-kind-select"
      bind:value={kind}
      class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
    >
      <option value={TicketKind.Request}>Demande (incident / intervention)</option>
      <option value={TicketKind.Complaint}>Plainte (nuisance / dossier)</option>
    </select>
  </div>

  <!-- Title -->
  <div class="flex flex-col gap-1">
    <label
      for="ticket-create-title-input"
      class="text-sm font-medium text-gray-700"
    >
      Titre
    </label>
    <input
      id="ticket-create-title-input"
      data-testid="ticket-create-title-input"
      type="text"
      bind:value={title}
      minlength={TICKET_TITLE_MIN}
      maxlength={TICKET_TITLE_MAX}
      placeholder="Ex. Fuite couloir étage 2"
      class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
      required
    />
  </div>

  <!-- Description -->
  <div class="flex flex-col gap-1">
    <label
      for="ticket-create-description-textarea"
      class="text-sm font-medium text-gray-700"
    >
      Description
    </label>
    <textarea
      id="ticket-create-description-textarea"
      data-testid="ticket-create-description-textarea"
      bind:value={description}
      rows="5"
      minlength={TICKET_DESCRIPTION_MIN}
      maxlength={TICKET_DESCRIPTION_MAX}
      placeholder="Décrivez la situation en détail…"
      aria-describedby="ticket-create-description-counter"
      aria-invalid={descTooShort || descTooLong ? "true" : "false"}
      class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
      required
    ></textarea>
    <p
      id="ticket-create-description-counter"
      data-testid="ticket-create-description-counter"
      class={`text-xs ${descCounterClasses}`}
      aria-live="polite"
    >
      {descCounterLabel}
    </p>
  </div>

  <!-- Category -->
  <div class="flex flex-col gap-1">
    <label
      for="ticket-create-category-select"
      class="text-sm font-medium text-gray-700"
    >
      Catégorie
    </label>
    <select
      id="ticket-create-category-select"
      data-testid="ticket-create-category-select"
      bind:value={category}
      class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
    >
      <option value={TicketCategory.Plumbing}>Plomberie</option>
      <option value={TicketCategory.Electrical}>Électricité</option>
      <option value={TicketCategory.Heating}>Chauffage</option>
      <option value={TicketCategory.CommonAreas}>Parties communes</option>
      <option value={TicketCategory.Elevator}>Ascenseur</option>
      <option value={TicketCategory.Security}>Sécurité</option>
      <option value={TicketCategory.Cleaning}>Nettoyage</option>
      <option value={TicketCategory.Landscaping}>Espaces verts</option>
      <option value={TicketCategory.Other}>Autre</option>
    </select>
  </div>

  <!-- Priority -->
  <div class="flex flex-col gap-1">
    <label
      for="ticket-create-priority-select"
      class="text-sm font-medium text-gray-700"
    >
      Priorité
    </label>
    <select
      id="ticket-create-priority-select"
      data-testid="ticket-create-priority-select"
      bind:value={priority}
      class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
    >
      <option value={TicketPriority.Low}>Basse</option>
      <option value={TicketPriority.Medium}>Moyenne</option>
      <option value={TicketPriority.High}>Haute</option>
      <option value={TicketPriority.Critical}>Critique</option>
    </select>
  </div>

  <!-- ===== Section conditionnelle : kind=Complaint ===== -->
  {#if isComplaint}
    <SeveritySelector bind:value={severity} required />

    <div class="flex flex-col gap-1">
      <label
        for="ticket-create-incident-date-input"
        class="text-sm font-medium text-gray-700"
      >
        Date de l'incident
      </label>
      <input
        id="ticket-create-incident-date-input"
        data-testid="ticket-create-incident-date-input"
        type="date"
        bind:value={incidentDate}
        max={maxIncidentDate}
        aria-invalid={incidentDateInFuture ? "true" : "false"}
        aria-describedby={incidentDateInFuture
          ? "ticket-create-incident-date-error"
          : undefined}
        class="rounded border border-gray-300 px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2"
        required
      />
      {#if incidentDateInFuture}
        <p
          id="ticket-create-incident-date-error"
          data-testid="ticket-create-incident-date-error"
          class="text-xs text-red-600"
          role="alert"
        >
          La date d'incident ne peut être dans le futur.
        </p>
      {/if}
    </div>

    <EvidenceUpload bind:value={evidenceAttachments} />

    <WitnessSelector
      bind:value={witnesses}
      candidates={witnessCandidates}
      {currentUserId}
    />

    {#if evidenceWarning}
      <p
        data-testid="ticket-create-evidence-warning"
        class="rounded border border-orange-300 bg-orange-50 p-3 text-xs text-orange-800"
        role="note"
      >
        Preuves manquantes — votre dossier est plus solide avec des photos,
        vidéos ou témoins. Vous pouvez tout de même soumettre la plainte.
      </p>
    {/if}
  {/if}
  <!-- ===== /Section conditionnelle ===== -->

  {#if errorMessage}
    <p
      data-testid="ticket-create-error"
      class="rounded border border-red-300 bg-red-50 p-3 text-sm text-red-700"
      role="alert"
      aria-live="polite"
    >
      {errorMessage}
    </p>
  {/if}

  <div class="mt-2 flex justify-end gap-2">
    {#if onCancel}
      <button
        type="button"
        data-testid="ticket-create-cancel"
        class="rounded border border-gray-300 px-4 py-2 text-sm text-gray-700 hover:bg-gray-50"
        onclick={() => onCancel?.()}
        disabled={submitting}
      >
        Annuler
      </button>
    {/if}
    <button
      type="submit"
      data-testid="ticket-create-submit"
      disabled={submitDisabled}
      aria-disabled={submitDisabled}
      class="min-h-[44px] rounded bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:bg-gray-300"
    >
      {submitting
        ? "Envoi…"
        : isComplaint
          ? "Déposer la plainte"
          : "Créer le ticket"}
    </button>
  </div>
</form>
