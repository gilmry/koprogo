<script lang="ts">
  // Story B8 (Phase B FE) — ContractorEvaluationForm.
  //
  // Form d'évaluation d'un contractor après prestation — parent BE story 3.9
  // (commit `c53a7e1`). Append-only (INV-24 BE) : pas d'edit / delete des
  // évaluations passées (cf. ContractorReputation aussi).
  //
  // GATED par TechnicalSpec préalable signée (INV-21 BE) : le select des
  // TechnicalSpec ne montre QUE les spec en status="Approved". Si l'utilisateur
  // sélectionne quand même une spec Draft/PendingSignatures/Superseded (cas
  // technique impossible via UI sauf concurrent edit), le backend renverra 422
  // → toast auto via api.ts.
  //
  // INV exposés en UI (gating client AVANT POST) :
  //   INV-22 BE (anti self-eval) : si `contractor_user_id === currentUser.id`
  //     → submit DISABLED + message UI "Un contractor ne peut pas s'évaluer
  //     lui-même". AC @security stories.md §B8.
  //   - Tous les scores requis (1..5) — gérés par le ScoreInput atomique.
  //   - Comment 10..2000 chars après trim — counter en rouge sous le seuil,
  //     submit disabled. AC @negative stories.md §B8.
  //
  // INV-FE9 (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`) :
  //   - Chaque `<label>` lie son contrôle via for/id.
  //   - Erreurs inline aria-describedby + aria-invalid.
  //   - Counter comment aria-live="polite".
  //   - ScoreInput utilise fieldset+legend (cf. ScoreInput.svelte).
  //   - Submit disabled communique état via aria-disabled.
  //
  // data-testid (cf. stories.md §B8) :
  //   contractor-eval-contractor-select         + options
  //   contractor-eval-spec-select               (filtré sur Approved)
  //   contractor-eval-tickets-link              (multi-select tickets)
  //   contractor-eval-scores-{quality|timeliness|communication|cost|overall}
  //   contractor-eval-comment-textarea          + counter
  //   contractor-eval-submit
  //   contractor-eval-self-eval-warning         (banner @security)
  //   contractor-eval-error-{field}
  //
  // Pattern DI : on injecte `currentUserId` via prop pour découpler des stores
  // (cohérent test-helpers). En prod, le parent (page Astro) le récupère du
  // `authStore`.

  import { toast } from "../../../stores/toast";
  import {
    EVAL_MIN_COMMENT_LENGTH,
    EVAL_MAX_COMMENT_LENGTH,
    SCORE_DIMENSIONS,
    SCORE_DIMENSION_LABELS_FR,
    isValidScores,
    type CreateContractorEvaluationRequest,
    type ContractorEvaluationDto,
    type EvaluationScoresDto,
    type ScoreDimension,
  } from "../../api/contractor_evaluations";
  import ScoreInput from "../shared/ScoreInput.svelte";

  // ---------------------------------------------------------------------------
  // Types props
  // ---------------------------------------------------------------------------

  type ContractorOption = { id: string; label: string };
  type SpecOption = {
    id: string;
    title: string;
    version: string;
    status: string; // libellé status — backend libre
  };
  type TicketOption = { id: string; title: string };

  let {
    /** User id courant (côté FE). Sert au gating anti self-evaluation INV-22. */
    currentUserId,
    /** Liste des contractors sélectionnables (parent fournit via API users
     *  scopée à l'org). */
    contractors = [],
    /** Liste des TechnicalSpec disponibles. Le composant filtre côté FE pour
     *  ne montrer QUE les Approved (cf. Gotcha stories.md §B8). */
    specs = [],
    /** Liste des tickets sélectionnables (du building lié à la spec, ou
     *  globalement scope user). Multi-select optionnel — 0..N. */
    tickets = [],
    /** Callback submit — le parent gère le POST via `createEvaluation` et
     *  remonte la DTO créée. Pattern DI cohérent B7 / B6. */
    onSubmit,
    /** Callback annulation (close modal côté parent). */
    onCancel = undefined as undefined | (() => void),
  }: {
    currentUserId: string | null;
    contractors?: ContractorOption[];
    specs?: SpecOption[];
    tickets?: TicketOption[];
    onSubmit: (
      req: CreateContractorEvaluationRequest,
    ) => Promise<ContractorEvaluationDto>;
    onCancel?: () => void;
  } = $props();

  // ---------------------------------------------------------------------------
  // State du formulaire
  // ---------------------------------------------------------------------------

  let contractorUserId = $state<string>("");
  let technicalSpecId = $state<string>("");
  let linkedTicketIds = $state<string[]>([]);
  let comment = $state<string>("");

  // Scores 1..5 ou null (pas encore choisi). Le ScoreInput atomique borne
  // nativement via radios — impossible de saisir 0 ou 6.
  let scoreQuality = $state<number | null>(null);
  let scoreTimeliness = $state<number | null>(null);
  let scoreCommunication = $state<number | null>(null);
  let scoreCostCompliance = $state<number | null>(null);
  let scoreOverall = $state<number | null>(null);

  let submitting = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Dérivations & validation
  // ---------------------------------------------------------------------------

  /** Filtre côté FE : seules les Approved sont sélectionnables (INV-21 BE).
   *  Gotcha stories.md §B8 — backend pourrait fournir un query param dédié,
   *  on fait le filter ici de manière défensive. */
  let approvedSpecs = $derived(
    specs.filter((s) => s.status === "Approved"),
  );

  /** Tous les scores fournis (pas null) → on peut construire EvaluationScoresDto. */
  let scoresProvided = $derived(
    scoreQuality !== null &&
      scoreTimeliness !== null &&
      scoreCommunication !== null &&
      scoreCostCompliance !== null &&
      scoreOverall !== null,
  );

  /** EvaluationScoresDto (ou null si pas complet) — utilisé pour validation
   *  ET pour le payload submit. */
  let scoresDto = $derived<EvaluationScoresDto | null>(
    scoresProvided
      ? {
          quality: scoreQuality as number,
          timeliness: scoreTimeliness as number,
          communication: scoreCommunication as number,
          cost_compliance: scoreCostCompliance as number,
          overall: scoreOverall as number,
        }
      : null,
  );

  /** Counter comment : codepoints Unicode (gotcha #2 B3). */
  let commentCharCount = $derived([...comment.trim()].length);

  let commentOk = $derived(
    commentCharCount >= EVAL_MIN_COMMENT_LENGTH &&
      commentCharCount <= EVAL_MAX_COMMENT_LENGTH,
  );

  /** INV-22 BE : evaluator (current user) != contractor. Si l'utilisateur
   *  sélectionne lui-même comme contractor → bloqué côté FE.
   *  AC @security stories.md §B8. */
  let isSelfEvaluation = $derived(
    contractorUserId !== "" &&
      currentUserId !== null &&
      contractorUserId === currentUserId,
  );

  /** Map d'erreurs inline (clé = data-testid suffix). */
  let errors = $derived.by<Record<string, string>>(() => {
    const e: Record<string, string> = {};
    if (contractorUserId === "") e.contractor = "Sélectionnez un contractor.";
    if (isSelfEvaluation)
      e.contractor =
        "Un contractor ne peut pas s'évaluer lui-même (INV-22).";
    if (technicalSpecId === "")
      e.spec =
        approvedSpecs.length === 0
          ? "Aucune fiche technique Approuvée — créez-en une au préalable."
          : "Sélectionnez une fiche technique Approuvée.";
    if (!scoresProvided)
      e.scores = "Renseignez les 5 critères de notation (1 à 5).";
    else if (scoresDto && !isValidScores(scoresDto))
      e.scores = "Scores invalides (attendu : entiers 1 à 5).";
    if (!commentOk)
      e.comment =
        commentCharCount < EVAL_MIN_COMMENT_LENGTH
          ? `Commentaire trop court (${commentCharCount}/${EVAL_MIN_COMMENT_LENGTH} minimum).`
          : `Commentaire trop long (${commentCharCount}/${EVAL_MAX_COMMENT_LENGTH} maximum).`;
    return e;
  });

  let formValid = $derived(Object.keys(errors).length === 0);

  // ---------------------------------------------------------------------------
  // Helpers : mapping ScoreDimension → state setter
  // ---------------------------------------------------------------------------

  function setScore(dim: ScoreDimension, next: number): void {
    switch (dim) {
      case "quality":
        scoreQuality = next;
        break;
      case "timeliness":
        scoreTimeliness = next;
        break;
      case "communication":
        scoreCommunication = next;
        break;
      case "cost_compliance":
        scoreCostCompliance = next;
        break;
      case "overall":
        scoreOverall = next;
        break;
    }
  }

  function getScore(dim: ScoreDimension): number | null {
    switch (dim) {
      case "quality":
        return scoreQuality;
      case "timeliness":
        return scoreTimeliness;
      case "communication":
        return scoreCommunication;
      case "cost_compliance":
        return scoreCostCompliance;
      case "overall":
        return scoreOverall;
    }
  }

  /** Suffixe data-testid pour le `scores-{dim}` — stories.md §B8 utilise
   *  "cost" pour cost_compliance. */
  function scoreTestSuffix(dim: ScoreDimension): string {
    return dim === "cost_compliance" ? "cost" : dim;
  }

  function toggleTicket(id: string, checked: boolean): void {
    if (checked) {
      if (!linkedTicketIds.includes(id)) {
        linkedTicketIds = [...linkedTicketIds, id];
      }
    } else {
      linkedTicketIds = linkedTicketIds.filter((t) => t !== id);
    }
  }

  // ---------------------------------------------------------------------------
  // Submit
  // ---------------------------------------------------------------------------

  async function handleSubmit(ev: SubmitEvent): Promise<void> {
    ev.preventDefault();
    if (!formValid || submitting || !scoresDto) return;
    submitting = true;
    try {
      const req: CreateContractorEvaluationRequest = {
        contractor_user_id: contractorUserId,
        technical_spec_id: technicalSpecId,
        linked_ticket_ids: [...linkedTicketIds],
        scores: scoresDto,
        comment: comment.trim(),
      };
      await onSubmit(req);
      toast.success("Évaluation enregistrée.");
    } catch {
      // toast déjà émis par api.ts pour 4xx/5xx
    } finally {
      submitting = false;
    }
  }
</script>

<form
  class="contractor-eval-form flex flex-col gap-4 p-4 bg-white rounded shadow-sm"
  onsubmit={handleSubmit}
  aria-labelledby="contractor-eval-form-title"
  novalidate
>
  <h2
    id="contractor-eval-form-title"
    class="text-lg font-semibold text-gray-900"
  >
    Nouvelle évaluation contractor
  </h2>

  <!-- Banner self-evaluation (INV-22 — AC @security) -->
  {#if isSelfEvaluation}
    <div
      data-testid="contractor-eval-self-eval-warning"
      class="rounded border border-red-300 bg-red-50 px-3 py-2 text-sm text-red-800"
      role="alert"
    >
      Un contractor ne peut pas s'évaluer lui-même (INV-22). Sélectionnez un
      autre contractor pour continuer.
    </div>
  {/if}

  <!-- Contractor select -->
  <div class="flex flex-col gap-1">
    <label
      for="contractor-eval-contractor"
      class="text-sm font-medium text-gray-700"
    >
      Contractor évalué <span class="text-red-600" aria-hidden="true">*</span>
    </label>
    <select
      id="contractor-eval-contractor"
      data-testid="contractor-eval-contractor-select"
      bind:value={contractorUserId}
      aria-invalid={errors.contractor ? "true" : "false"}
      aria-describedby={errors.contractor
        ? "contractor-eval-error-contractor"
        : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    >
      <option value="">— Sélectionner —</option>
      {#each contractors as c (c.id)}
        <option
          value={c.id}
          data-testid={`contractor-eval-contractor-option-${c.id}`}
        >
          {c.label}
        </option>
      {/each}
    </select>
    {#if errors.contractor}
      <p
        id="contractor-eval-error-contractor"
        data-testid="contractor-eval-error-contractor"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.contractor}
      </p>
    {/if}
  </div>

  <!-- TechnicalSpec select (filtre Approved côté FE — INV-21) -->
  <div class="flex flex-col gap-1">
    <label
      for="contractor-eval-spec"
      class="text-sm font-medium text-gray-700"
    >
      Fiche technique <span class="text-red-600" aria-hidden="true">*</span>
    </label>
    <select
      id="contractor-eval-spec"
      data-testid="contractor-eval-spec-select"
      bind:value={technicalSpecId}
      aria-invalid={errors.spec ? "true" : "false"}
      aria-describedby={errors.spec
        ? "contractor-eval-error-spec"
        : "contractor-eval-spec-helper"}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
      disabled={approvedSpecs.length === 0}
    >
      <option value="">— Sélectionner —</option>
      {#each approvedSpecs as s (s.id)}
        <option
          value={s.id}
          data-testid={`contractor-eval-spec-option-${s.id}`}
        >
          {s.title} (v{s.version})
        </option>
      {/each}
    </select>
    <p
      id="contractor-eval-spec-helper"
      class="text-xs text-gray-500"
    >
      Seules les fiches techniques au statut « Approuvée » sont éligibles
      (INV-21 — signature préalable obligatoire).
    </p>
    {#if errors.spec}
      <p
        id="contractor-eval-error-spec"
        data-testid="contractor-eval-error-spec"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.spec}
      </p>
    {/if}
  </div>

  <!-- Tickets liés (multi-select optionnel — 0..N) -->
  <fieldset class="flex flex-col gap-1">
    <legend class="text-sm font-medium text-gray-700">
      Tickets motivant l'évaluation (optionnel)
    </legend>
    {#if tickets.length === 0}
      <p
        data-testid="contractor-eval-tickets-empty"
        class="text-xs text-gray-500"
      >
        Aucun ticket disponible pour ce scope.
      </p>
    {:else}
      <div
        data-testid="contractor-eval-tickets-link"
        class="flex flex-col gap-1 max-h-40 overflow-y-auto border border-gray-200 rounded px-2 py-1"
        role="group"
        aria-label="Tickets liés à l'évaluation"
      >
        {#each tickets as t (t.id)}
          <label class="inline-flex items-center gap-2 text-sm">
            <input
              type="checkbox"
              data-testid={`contractor-eval-ticket-option-${t.id}`}
              checked={linkedTicketIds.includes(t.id)}
              onchange={(e) =>
                toggleTicket(t.id, (e.target as HTMLInputElement).checked)}
            />
            <span class="truncate">{t.title}</span>
          </label>
        {/each}
      </div>
    {/if}
  </fieldset>

  <!-- 5 scores ScoreInput atomique -->
  <div class="flex flex-col gap-3">
    <span class="text-sm font-medium text-gray-700">
      Notation (1 à 5) <span class="text-red-600" aria-hidden="true">*</span>
    </span>
    {#each SCORE_DIMENSIONS as dim (dim)}
      <div
        class="flex flex-col gap-1"
        data-testid={`contractor-eval-scores-${scoreTestSuffix(dim)}`}
      >
        <ScoreInput
          name={`contractor-eval-score-${dim}`}
          label={SCORE_DIMENSION_LABELS_FR[dim]}
          value={getScore(dim)}
          onChange={(n) => setScore(dim, n)}
          required={true}
          testIdPrefix={`contractor-eval-scores-${scoreTestSuffix(dim)}`}
        />
      </div>
    {/each}
    {#if errors.scores}
      <p
        data-testid="contractor-eval-error-scores"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.scores}
      </p>
    {/if}
  </div>

  <!-- Comment -->
  <div class="flex flex-col gap-1">
    <label
      for="contractor-eval-comment"
      class="text-sm font-medium text-gray-700"
    >
      Commentaire <span class="text-red-600" aria-hidden="true">*</span>
    </label>
    <textarea
      id="contractor-eval-comment"
      data-testid="contractor-eval-comment-textarea"
      bind:value={comment}
      rows="4"
      maxlength={EVAL_MAX_COMMENT_LENGTH}
      aria-invalid={errors.comment ? "true" : "false"}
      aria-describedby="contractor-eval-comment-counter contractor-eval-error-comment"
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    ></textarea>
    <p
      id="contractor-eval-comment-counter"
      data-testid="contractor-eval-comment-counter"
      class={`text-xs ${commentOk ? "text-gray-500" : "text-red-600"}`}
      aria-live="polite"
    >
      {commentCharCount} / {EVAL_MAX_COMMENT_LENGTH} (min.
      {EVAL_MIN_COMMENT_LENGTH})
    </p>
    {#if errors.comment}
      <p
        id="contractor-eval-error-comment"
        data-testid="contractor-eval-error-comment"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.comment}
      </p>
    {/if}
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-2 mt-2">
    {#if onCancel}
      <button
        type="button"
        data-testid="contractor-eval-cancel"
        class="px-4 py-2 text-sm border border-gray-300 rounded text-gray-700 hover:bg-gray-50"
        onclick={() => onCancel?.()}
        disabled={submitting}
      >
        Annuler
      </button>
    {/if}
    <button
      type="submit"
      data-testid="contractor-eval-submit"
      class="min-h-[44px] px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      disabled={!formValid || submitting || isSelfEvaluation}
      aria-disabled={!formValid || submitting || isSelfEvaluation}
    >
      {submitting ? "Envoi…" : "Enregistrer l'évaluation"}
    </button>
  </div>
</form>
