<script lang="ts">
  // Story B3 (Phase B FE) — Form d'émission d'un mandat (syndic / superadmin).
  //
  // Parent BE story 3.4 (Mandate aggregate + invariants INV-13/14/15/16) +
  // wireframe stories.md §B3 (table data-testid + AC 4-cat).
  //
  // Invariants exposés en UI (gating client AVANT POST, redondant avec backend) :
  //   INV-14 : `valid_until - now <= 5 ans` (anti-abuse). Hors fenêtre → submit
  //            disabled + helper rouge inline (cf. AC @edge).
  //   INV-15 : `subject_user_id !== issuer`. Sélectionner soi-même → submit
  //            disabled + helper "Vous ne pouvez pas vous mandater vous-même"
  //            (cf. AC @security — backend 422 sinon, mais on coupe court).
  //   INV-16 : reason 10..=500 chars. Counter passe rouge hors fenêtre + submit
  //            disabled (cf. AC @negative).
  //
  // a11y (WCAG 2.1 AA — memory `a11y-wcag-aa-baseline`) :
  //   - Chaque `<label>` lie son contrôle via `for`/`id`.
  //   - Erreurs inline avec `aria-describedby` + `aria-invalid`.
  //   - Counter reason : `aria-live="polite"` pour annoncer le seuil dépassé.
  //   - Radio scope_kind : `<fieldset><legend>` (group implicite).
  //   - Submit disabled communique état via `aria-disabled` (en plus de `disabled`).
  //
  // data-testid (cf. stories.md §B3) :
  //   mandate-subject-select        / mandate-subject-option-{userId}
  //   mandate-kind-select           / mandate-kind-option-{kind}
  //   mandate-scope-type-radio-{building|acp}
  //   mandate-scope-id-select       / mandate-scope-id-option-{id}
  //   mandate-reason-textarea       / mandate-reason-counter
  //   mandate-valid-until-input
  //   mandate-issue-submit          / mandate-cancel
  //   mandate-error-{field}         (un par champ — affiché si error[field])
  //
  // i18n : labels via `$_` (fallback FR statique). Voir dateBadge.ts pour
  // motivation du fallback (Story B12+ portera la i18n NL/EN/DE).

  import { _ } from "../../i18n";
  import { toast } from "../../../stores/toast";
  import { authStore } from "../../../stores/auth";
  import { get } from "svelte/store";
  import {
    issueMandate,
    type MandateResponse,
    type IssueMandateRequest,
  } from "../../api/mandates";

  // -------------------------------------------------------------------------
  // Props — onSuccess remonte la nouvelle ligne au parent (MandateList).
  // -------------------------------------------------------------------------

  let {
    /** Liste des users sélectionnables comme `subject_user_id`. Fournie par le
     *  parent pour découpler le fetch (la liste vit dans le store ou via API
     *  parent — pas notre responsabilité ici). */
    subjects = [],
    /** Liste des scopes (buildings + acps) sélectionnables. Le parent fournit
     *  un tableau hétérogène typé `kind` + `id` + `label`. */
    scopes = [],
    /** Callback succès — remonté avec la `MandateResponse` du backend. */
    onSuccess = undefined,
    /** Callback annulation (close modal côté parent). */
    onCancel = undefined,
    /** Injection clock pour tests déterministes (équivalent ExpirationBadge). */
    nowOverride = undefined,
  }: {
    subjects?: Array<{ id: string; label: string }>;
    scopes?: Array<{ id: string; kind: "building" | "acp"; label: string }>;
    onSuccess?: (m: MandateResponse) => void;
    onCancel?: () => void;
    nowOverride?: Date | undefined;
  } = $props();

  // -------------------------------------------------------------------------
  // State du formulaire
  // -------------------------------------------------------------------------

  // Les kinds sont fixés par le backend (MandateKind enum — Story 3.4).
  // Ordre métier : juridique → technique.
  const KINDS = ["lawyer", "notary", "amo", "architect", "bet", "warden"] as const;
  type Kind = (typeof KINDS)[number];

  let subjectUserId = $state<string>("");
  let kind = $state<Kind>("notary");
  let scopeKind = $state<"building" | "acp">("building");
  let scopeId = $state<string>("");
  let reason = $state<string>("");
  let validUntil = $state<string>(""); // YYYY-MM-DD (input type=date)

  let submitting = $state<boolean>(false);

  // -------------------------------------------------------------------------
  // Dérivations & validation (INV-14 / INV-15 / INV-16)
  // -------------------------------------------------------------------------

  /** Now injectable — pour tests déterministes (vs `new Date()` flaky). */
  let now = $derived(nowOverride ?? new Date());

  /** Issuer (= user courant). Lecture défensive : authStore peut être en
   *  cours de réhydratation au mount (#550 silent-refresh). */
  let issuerId = $derived.by(() => {
    try {
      const state = get(authStore);
      return state.user?.id ?? null;
    } catch {
      return null;
    }
  });

  /** Counter reason : codepoints Unicode (gotcha #2 stories.md §B3). */
  let reasonCharCount = $derived([...reason].length);

  let reasonOk = $derived(reasonCharCount >= 10 && reasonCharCount <= 500);

  /** Filtre les options de scope_id selon le radio sélectionné. */
  let scopeOptions = $derived(scopes.filter((s) => s.kind === scopeKind));

  /** Date max autorisée (INV-14 : today + 5 ans = 5*365 = 1825 jours). */
  let maxValidUntil = $derived.by(() => {
    const max = new Date(now.getTime());
    max.setUTCDate(max.getUTCDate() + 5 * 365);
    return max.toISOString().slice(0, 10);
  });

  /** Date min autorisée : demain (valid_until strictement futur). */
  let minValidUntil = $derived.by(() => {
    const min = new Date(now.getTime());
    min.setUTCDate(min.getUTCDate() + 1);
    return min.toISOString().slice(0, 10);
  });

  /** valid_until parsé en Date (fin de jour UTC) pour comparaisons. */
  let validUntilDate = $derived<Date | null>(
    validUntil ? new Date(`${validUntil}T23:59:59Z`) : null,
  );

  /** Hors fenêtre INV-14 (>5 ans depuis maintenant) ?
   *
   * Calculé en granularité JOUR (pas en ms) pour matcher la sémantique
   * user-facing "5 ans = 5*365 jours civils". On compare le delta de jours
   * civils UTC entre `now` (truncated à 00:00) et `validUntilDate`. */
  let validUntilOutOfRange = $derived.by(() => {
    if (!validUntilDate) return false;
    const nowDay = Date.UTC(
      now.getUTCFullYear(),
      now.getUTCMonth(),
      now.getUTCDate(),
    );
    const targetDay = Date.UTC(
      validUntilDate.getUTCFullYear(),
      validUntilDate.getUTCMonth(),
      validUntilDate.getUTCDate(),
    );
    const deltaDays = Math.round((targetDay - nowDay) / (24 * 60 * 60 * 1000));
    return deltaDays > 5 * 365;
  });

  /** valid_until dans le passé ? (strictement <= now en granularité jour). */
  let validUntilInPast = $derived.by(() => {
    if (!validUntilDate) return false;
    return validUntilDate.getTime() <= now.getTime();
  });

  /** INV-15 : subject != issuer. */
  let selfMandate = $derived(
    subjectUserId !== "" && issuerId !== null && subjectUserId === issuerId,
  );

  /** Map d'erreurs inline (clé = data-testid suffix). */
  let errors = $derived.by<Record<string, string>>(() => {
    const e: Record<string, string> = {};
    if (subjectUserId === "") e.subject = "Sélectionnez un mandataire.";
    if (selfMandate)
      e.subject = "Vous ne pouvez pas vous mandater vous-même (INV-15).";
    if (scopeId === "") e.scopeId = "Sélectionnez un scope (immeuble ou ACP).";
    if (!reasonOk)
      e.reason =
        reasonCharCount < 10
          ? `Motif trop court (${reasonCharCount}/10 minimum).`
          : `Motif trop long (${reasonCharCount}/500 maximum).`;
    if (validUntil === "") e.validUntil = "Date d'expiration obligatoire.";
    else if (validUntilInPast)
      e.validUntil = "La date d'expiration doit être strictement future.";
    else if (validUntilOutOfRange)
      e.validUntil = "Durée maximale 5 ans (INV-14 anti-abus).";
    return e;
  });

  let formValid = $derived(Object.keys(errors).length === 0);

  // -------------------------------------------------------------------------
  // Handlers
  // -------------------------------------------------------------------------

  async function handleSubmit(ev: SubmitEvent): Promise<void> {
    ev.preventDefault();
    if (!formValid || submitting) return;
    submitting = true;
    try {
      // Backend attend ISO 8601 (TIMESTAMPTZ). On envoie l'instant fin de
      // journée UTC pour valid_until (23:59:59Z), cohérent avec la
      // sémantique "expire en fin de journée du J indiqué".
      const req: IssueMandateRequest = {
        subject_user_id: subjectUserId,
        kind,
        scope_kind: scopeKind,
        scope_id: scopeId,
        reason: reason.trim(),
        valid_until: `${validUntil}T23:59:59Z`,
      };
      const created = await issueMandate(req);
      toast.success($_("mandate.issue.success") || "Mandat émis.");
      onSuccess?.(created);
      // Reset partiel — on garde le scope radio (UX : le syndic émet souvent
      // plusieurs mandats sur le même immeuble en série).
      subjectUserId = "";
      reason = "";
      validUntil = "";
    } catch {
      // Le wrapper `api.ts` a déjà toasté l'erreur 4xx/5xx — pas besoin de
      // double feedback ici.
    } finally {
      submitting = false;
    }
  }
</script>

<form
  class="mandate-form flex flex-col gap-4 p-4 bg-white rounded shadow-sm"
  onsubmit={handleSubmit}
  aria-labelledby="mandate-issue-title"
  novalidate
>
  <h2 id="mandate-issue-title" class="text-lg font-semibold text-gray-900">
    {$_("mandate.issue.title") || "Émettre un mandat"}
  </h2>

  <!-- Subject (mandataire) -->
  <div class="flex flex-col gap-1">
    <label for="mandate-subject" class="text-sm font-medium text-gray-700">
      {$_("mandate.field.subject") || "Mandataire"}
    </label>
    <select
      id="mandate-subject"
      data-testid="mandate-subject-select"
      bind:value={subjectUserId}
      aria-invalid={errors.subject ? "true" : "false"}
      aria-describedby={errors.subject ? "mandate-error-subject" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    >
      <option value="" disabled>— {$_("common.choose") || "Choisir…"} —</option>
      {#each subjects as s (s.id)}
        <option
          value={s.id}
          data-testid={`mandate-subject-option-${s.id}`}
          disabled={s.id === issuerId}
        >
          {s.label}{s.id === issuerId ? " (vous-même)" : ""}
        </option>
      {/each}
    </select>
    {#if errors.subject}
      <p
        id="mandate-error-subject"
        data-testid="mandate-error-subject"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.subject}
      </p>
    {/if}
  </div>

  <!-- Kind -->
  <div class="flex flex-col gap-1">
    <label for="mandate-kind" class="text-sm font-medium text-gray-700">
      {$_("mandate.field.kind") || "Type de mandat"}
    </label>
    <select
      id="mandate-kind"
      data-testid="mandate-kind-select"
      bind:value={kind}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    >
      {#each KINDS as k (k)}
        <option value={k} data-testid={`mandate-kind-option-${k}`}>
          {k}
        </option>
      {/each}
    </select>
  </div>

  <!-- Scope type (radio Building / ACP) -->
  <fieldset class="flex flex-col gap-1">
    <legend class="text-sm font-medium text-gray-700">
      {$_("mandate.field.scopeType") || "Périmètre"}
    </legend>
    <div class="flex gap-4">
      <label class="inline-flex items-center gap-2 text-sm">
        <input
          type="radio"
          name="mandate-scope-type"
          value="building"
          data-testid="mandate-scope-type-radio-building"
          bind:group={scopeKind}
        />
        <span>{$_("mandate.scope.building") || "Immeuble"}</span>
      </label>
      <label class="inline-flex items-center gap-2 text-sm">
        <input
          type="radio"
          name="mandate-scope-type"
          value="acp"
          data-testid="mandate-scope-type-radio-acp"
          bind:group={scopeKind}
        />
        <span>{$_("mandate.scope.acp") || "ACP"}</span>
      </label>
    </div>
  </fieldset>

  <!-- Scope ID (filtré sur le radio) -->
  <div class="flex flex-col gap-1">
    <label for="mandate-scope-id" class="text-sm font-medium text-gray-700">
      {scopeKind === "building"
        ? $_("mandate.field.building") || "Immeuble cible"
        : $_("mandate.field.acp") || "ACP cible"}
    </label>
    <select
      id="mandate-scope-id"
      data-testid="mandate-scope-id-select"
      bind:value={scopeId}
      aria-invalid={errors.scopeId ? "true" : "false"}
      aria-describedby={errors.scopeId ? "mandate-error-scopeId" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    >
      <option value="" disabled>— {$_("common.choose") || "Choisir…"} —</option>
      {#each scopeOptions as s (s.id)}
        <option value={s.id} data-testid={`mandate-scope-id-option-${s.id}`}>
          {s.label}
        </option>
      {/each}
    </select>
    {#if errors.scopeId}
      <p
        id="mandate-error-scopeId"
        data-testid="mandate-error-scopeId"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.scopeId}
      </p>
    {/if}
  </div>

  <!-- Reason (textarea + counter unicode-safe) -->
  <div class="flex flex-col gap-1">
    <label for="mandate-reason" class="text-sm font-medium text-gray-700">
      {$_("mandate.field.reason") || "Motif"}
    </label>
    <textarea
      id="mandate-reason"
      data-testid="mandate-reason-textarea"
      bind:value={reason}
      rows="4"
      maxlength={500}
      aria-invalid={errors.reason ? "true" : "false"}
      aria-describedby="mandate-reason-counter mandate-error-reason"
      class="border border-gray-300 rounded px-3 py-2 text-sm font-mono"
      required
    ></textarea>
    <p
      id="mandate-reason-counter"
      data-testid="mandate-reason-counter"
      class={`text-xs ${reasonOk ? "text-gray-500" : "text-red-600"}`}
      aria-live="polite"
    >
      {reasonCharCount} / 500 (min. 10)
    </p>
    {#if errors.reason}
      <p
        id="mandate-error-reason"
        data-testid="mandate-error-reason"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.reason}
      </p>
    {/if}
  </div>

  <!-- valid_until -->
  <div class="flex flex-col gap-1">
    <label for="mandate-valid-until" class="text-sm font-medium text-gray-700">
      {$_("mandate.field.validUntil") || "Valide jusqu'au"}
    </label>
    <input
      id="mandate-valid-until"
      data-testid="mandate-valid-until-input"
      type="date"
      bind:value={validUntil}
      min={minValidUntil}
      max={maxValidUntil}
      aria-invalid={errors.validUntil ? "true" : "false"}
      aria-describedby={errors.validUntil
        ? "mandate-error-validUntil"
        : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    />
    {#if errors.validUntil}
      <p
        id="mandate-error-validUntil"
        data-testid="mandate-error-validUntil"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.validUntil}
      </p>
    {/if}
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-2 mt-2">
    <button
      type="button"
      data-testid="mandate-cancel"
      class="px-4 py-2 text-sm border border-gray-300 rounded text-gray-700 hover:bg-gray-50"
      onclick={() => onCancel?.()}
      disabled={submitting}
    >
      {$_("common.cancel") || "Annuler"}
    </button>
    <button
      type="submit"
      data-testid="mandate-issue-submit"
      class="px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      disabled={!formValid || submitting}
      aria-disabled={!formValid || submitting}
    >
      {submitting
        ? $_("common.submitting") || "Émission…"
        : $_("mandate.action.issue") || "Émettre le mandat"}
    </button>
  </div>
</form>
