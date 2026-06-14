<script lang="ts">
  // Story B7 (Phase B FE) — TechnicalSpecCreate.
  //
  // Form initial de création d'une TechnicalSpec (status Draft) — parent BE
  // story 3.8 (commit `d820c39`).
  //
  // Aussi utilisé en mode "bump" (nouvelle version) : le parent passe
  // `mode="bump"` + `previousVersion`. Le payload diffère (bump → endpoint
  // POST /technical-specs/{id}/bump avec BumpTechnicalSpecRequest, sinon
  // CreateTechnicalSpecRequest). La sélection est faite côté parent via
  // `onSubmit` (callback DI).
  //
  // INV exposés en UI (gating client AVANT POST, redondant avec backend) :
  //   - SemVer strict (cf. AC @edge) : "1.0.0" OK, "v1.0.0" rejeté,
  //     "1.0.0-rc1" rejeté, "1.0" rejeté. Validation via `isValidSemver`.
  //   - description ≥ 50 chars (cf. AC @negative) : counter rouge sous le
  //     seuil + submit disabled.
  //   - deliverables ≥ 1 (cf. AC @negative) : aucun → submit disabled +
  //     helper inline.
  //   - En mode bump : nouvelle version doit être strictement > précédente
  //     (compareSemver > 0). Sinon submit disabled.
  //
  // INV-FE9 (a11y WCAG 2.1 AA) :
  //   - Chaque label lié au contrôle (for/id).
  //   - Erreurs inline aria-describedby + aria-invalid.
  //   - Counter description aria-live="polite".
  //   - Modal warning bump major (géré par le parent — TechnicalSpecDetail).
  //
  // data-testid (cf. stories.md §B7) :
  //   tech-spec-title-input
  //   tech-spec-description-textarea  + counter
  //   tech-spec-version-major-input / -minor-input / -patch-input
  //   tech-spec-deliverable-row-{index}
  //   tech-spec-deliverable-input-{index}
  //   tech-spec-deliverable-add
  //   tech-spec-deliverable-remove-{index}
  //   tech-spec-required-sig-select   (multi-select)
  //   tech-spec-attach-upload         (placeholder — pattern EvidenceUpload B5)
  //   tech-spec-create-submit
  //   tech-spec-create-error-{field}

  import { toast } from "../../../stores/toast";
  import {
    SIGNATORY_ROLES,
    TECH_SPEC_MIN_DESCRIPTION_LENGTH,
    TECH_SPEC_MAX_DESCRIPTION_LENGTH,
    isValidSemver,
    parseSemver,
    compareSemver,
    type CreateTechnicalSpecRequest,
    type BumpTechnicalSpecRequest,
    type SignatoryRole,
    type TechnicalSpecDto,
  } from "../../api/technical_specs";

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  let {
    /** ACP cible — obligatoire en mode "create" (le DTO l'exige).
     *  Pas demandé à l'UI : le parent l'injecte (contexte syndic mono-ACP). */
    acpId,
    /** Building cible optionnel (nullable). */
    buildingId = null,
    /** Mode du form : "create" (initial, default) ou "bump" (nouvelle version).
     *  En mode bump, le titre/description sont préremplis depuis previousVersion. */
    mode = "create" as "create" | "bump",
    /** En mode bump : la spec source dont on bump la version. */
    previousVersion = undefined as TechnicalSpecDto | undefined,
    /** Callback submit — le parent gère le dispatch vers createSpec ou
     *  bumpVersion selon le mode. Le résultat (TechnicalSpecDto) est remonté
     *  au parent (toast + navigation côté parent). */
    onSubmit,
    /** Callback annulation (close modal côté parent). */
    onCancel = undefined as undefined | (() => void),
  }: {
    acpId: string;
    buildingId?: string | null;
    mode?: "create" | "bump";
    previousVersion?: TechnicalSpecDto;
    onSubmit: (
      req: CreateTechnicalSpecRequest | BumpTechnicalSpecRequest,
    ) => Promise<TechnicalSpecDto>;
    onCancel?: () => void;
  } = $props();

  // ---------------------------------------------------------------------------
  // State du formulaire — initialisé depuis previousVersion en mode bump
  // ---------------------------------------------------------------------------
  //
  // Note Svelte 5 : on capture la valeur initiale de `previousVersion` (prop)
  // via une lambda d'init pour éviter le warning `state_referenced_locally`.
  // C'est intentionnel : la prop ne change pas après mount (un parent qui
  // remonte un autre `previousVersion` doit remonter le composant).

  const initTitle = (): string => previousVersion?.title ?? "";
  const initDescription = (): string => previousVersion?.description ?? "";
  const initDeliverables = (): string[] =>
    previousVersion?.deliverables && previousVersion.deliverables.length > 0
      ? [...previousVersion.deliverables]
      : [""];
  const initRequiredSignatures = (): string[] =>
    previousVersion?.required_signatures
      ? [...previousVersion.required_signatures]
      : ["syndic"];
  const initAttachments = (): string[] =>
    previousVersion?.attachments ? [...previousVersion.attachments] : [];

  let title = $state<string>(initTitle());
  let description = $state<string>(initDescription());
  let deliverables = $state<string[]>(initDeliverables());
  let requiredSignatures = $state<string[]>(initRequiredSignatures());
  let attachments = $state<string[]>(initAttachments());

  // Version inputs séparés (major/minor/patch) — UX plus claire que un seul
  // input string. En mode bump : prérempli avec MAJOR+1.0.0 par défaut.
  function defaultVersionParts(): { major: number; minor: number; patch: number } {
    if (mode === "bump" && previousVersion) {
      const p = parseSemver(previousVersion.version);
      if (p) {
        // Default = bump minor (préserve signatures). L'utilisateur choisit.
        return { major: p.major, minor: p.minor + 1, patch: 0 };
      }
    }
    return { major: 1, minor: 0, patch: 0 };
  }

  const initialParts = defaultVersionParts();
  let major = $state<number>(initialParts.major);
  let minor = $state<number>(initialParts.minor);
  let patch = $state<number>(initialParts.patch);

  let submitting = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // Dérivations & validation
  // ---------------------------------------------------------------------------

  /** String semver dérivée des inputs séparés. */
  let versionString = $derived(`${major}.${minor}.${patch}`);

  let versionValid = $derived(isValidSemver(versionString));

  /** En mode bump : la nouvelle version DOIT être strictement > précédente. */
  let bumpVersionStrictlyGreater = $derived.by<boolean>(() => {
    if (mode !== "bump" || !previousVersion) return true;
    const prev = parseSemver(previousVersion.version);
    const next = parseSemver(versionString);
    if (!prev || !next) return false;
    return compareSemver(next, prev) > 0;
  });

  /** Counter description — codepoints unicode (gotcha #2 B3). */
  let descriptionCharCount = $derived([...description].length);
  let descriptionOk = $derived(
    descriptionCharCount >= TECH_SPEC_MIN_DESCRIPTION_LENGTH &&
      descriptionCharCount <= TECH_SPEC_MAX_DESCRIPTION_LENGTH,
  );

  /** Deliverables non vides après trim. */
  let deliverablesNonEmpty = $derived(
    deliverables.filter((d) => d.trim().length > 0),
  );

  let titleOk = $derived(title.trim().length > 0);

  /** Map d'erreurs inline. */
  let errors = $derived.by<Record<string, string>>(() => {
    const e: Record<string, string> = {};
    if (!titleOk) e.title = "Titre obligatoire.";
    if (!descriptionOk) {
      e.description =
        descriptionCharCount < TECH_SPEC_MIN_DESCRIPTION_LENGTH
          ? `Description trop courte (${descriptionCharCount}/${TECH_SPEC_MIN_DESCRIPTION_LENGTH} minimum).`
          : `Description trop longue (${descriptionCharCount}/${TECH_SPEC_MAX_DESCRIPTION_LENGTH} maximum).`;
    }
    if (!versionValid)
      e.version =
        "Format SemVer strict obligatoire (ex: 1.0.0) — pas de prefix 'v', pas de pre-release.";
    else if (mode === "bump" && !bumpVersionStrictlyGreater)
      e.version = `La nouvelle version doit être strictement supérieure à ${previousVersion?.version ?? ""}.`;
    if (deliverablesNonEmpty.length === 0)
      e.deliverables = "Au moins 1 deliverable requis.";
    if (requiredSignatures.length === 0)
      e.requiredSignatures = "Au moins 1 rôle signataire requis.";
    return e;
  });

  let formValid = $derived(Object.keys(errors).length === 0);

  // ---------------------------------------------------------------------------
  // Actions
  // ---------------------------------------------------------------------------

  function addDeliverable(): void {
    deliverables = [...deliverables, ""];
  }

  function removeDeliverable(index: number): void {
    if (deliverables.length <= 1) {
      // Garde au moins une ligne — sinon vidé total.
      deliverables = [""];
      return;
    }
    deliverables = deliverables.filter((_, i) => i !== index);
  }

  function updateDeliverable(index: number, value: string): void {
    deliverables = deliverables.map((d, i) => (i === index ? value : d));
  }

  function toggleRequiredSignature(role: SignatoryRole, checked: boolean): void {
    if (checked) {
      if (!requiredSignatures.includes(role)) {
        requiredSignatures = [...requiredSignatures, role];
      }
    } else {
      requiredSignatures = requiredSignatures.filter((r) => r !== role);
    }
  }

  async function handleSubmit(ev: SubmitEvent): Promise<void> {
    ev.preventDefault();
    if (!formValid || submitting) return;
    submitting = true;
    try {
      const cleanedDeliverables = deliverablesNonEmpty.map((d) => d.trim());
      let req: CreateTechnicalSpecRequest | BumpTechnicalSpecRequest;
      if (mode === "bump") {
        req = {
          version: versionString,
          title: title.trim(),
          description: description.trim(),
          deliverables: cleanedDeliverables,
          required_signatures: [...requiredSignatures],
          attachments: [...attachments],
        };
      } else {
        req = {
          acp_id: acpId,
          building_id: buildingId,
          title: title.trim(),
          description: description.trim(),
          version: versionString,
          deliverables: cleanedDeliverables,
          required_signatures: [...requiredSignatures],
          attachments: [...attachments],
        };
      }
      const created = await onSubmit(req);
      toast.success(
        mode === "bump"
          ? `Nouvelle version ${created.version} créée.`
          : `Fiche technique ${created.title} créée (Draft).`,
      );
    } catch {
      // toast déjà émis par api.ts pour 4xx/5xx
    } finally {
      submitting = false;
    }
  }
</script>

<form
  class="tech-spec-form flex flex-col gap-4 p-4 bg-white rounded shadow-sm"
  onsubmit={handleSubmit}
  aria-labelledby="tech-spec-create-title"
  novalidate
>
  <h2
    id="tech-spec-create-title"
    class="text-lg font-semibold text-gray-900"
  >
    {mode === "bump"
      ? `Nouvelle version (bump depuis ${previousVersion?.version ?? ""})`
      : "Nouvelle fiche technique"}
  </h2>

  <!-- Title -->
  <div class="flex flex-col gap-1">
    <label for="tech-spec-title" class="text-sm font-medium text-gray-700">
      Titre
    </label>
    <input
      id="tech-spec-title"
      data-testid="tech-spec-title-input"
      type="text"
      bind:value={title}
      aria-invalid={errors.title ? "true" : "false"}
      aria-describedby={errors.title ? "tech-spec-create-error-title" : undefined}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
      required
    />
    {#if errors.title}
      <p
        id="tech-spec-create-error-title"
        data-testid="tech-spec-create-error-title"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.title}
      </p>
    {/if}
  </div>

  <!-- Description -->
  <div class="flex flex-col gap-1">
    <label
      for="tech-spec-description"
      class="text-sm font-medium text-gray-700"
    >
      Description
    </label>
    <textarea
      id="tech-spec-description"
      data-testid="tech-spec-description-textarea"
      bind:value={description}
      rows="5"
      maxlength={TECH_SPEC_MAX_DESCRIPTION_LENGTH}
      aria-invalid={errors.description ? "true" : "false"}
      aria-describedby="tech-spec-description-counter tech-spec-create-error-description"
      class="border border-gray-300 rounded px-3 py-2 text-sm font-mono"
      required
    ></textarea>
    <p
      id="tech-spec-description-counter"
      data-testid="tech-spec-description-counter"
      class={`text-xs ${descriptionOk ? "text-gray-500" : "text-red-600"}`}
      aria-live="polite"
    >
      {descriptionCharCount} / {TECH_SPEC_MAX_DESCRIPTION_LENGTH} (min.
      {TECH_SPEC_MIN_DESCRIPTION_LENGTH})
    </p>
    {#if errors.description}
      <p
        id="tech-spec-create-error-description"
        data-testid="tech-spec-create-error-description"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.description}
      </p>
    {/if}
  </div>

  <!-- Version (3 inputs major/minor/patch) -->
  <fieldset class="flex flex-col gap-1">
    <legend class="text-sm font-medium text-gray-700">Version (SemVer)</legend>
    <div class="flex items-center gap-2">
      <label class="flex flex-col text-xs text-gray-600">
        <span>Major</span>
        <input
          data-testid="tech-spec-version-major-input"
          type="number"
          min="0"
          step="1"
          bind:value={major}
          class="w-20 border border-gray-300 rounded px-2 py-1 text-sm"
        />
      </label>
      <span class="text-gray-400 mt-4">.</span>
      <label class="flex flex-col text-xs text-gray-600">
        <span>Minor</span>
        <input
          data-testid="tech-spec-version-minor-input"
          type="number"
          min="0"
          step="1"
          bind:value={minor}
          class="w-20 border border-gray-300 rounded px-2 py-1 text-sm"
        />
      </label>
      <span class="text-gray-400 mt-4">.</span>
      <label class="flex flex-col text-xs text-gray-600">
        <span>Patch</span>
        <input
          data-testid="tech-spec-version-patch-input"
          type="number"
          min="0"
          step="1"
          bind:value={patch}
          class="w-20 border border-gray-300 rounded px-2 py-1 text-sm"
        />
      </label>
      <span
        class="ml-2 mt-4 text-xs text-gray-500"
        data-testid="tech-spec-version-preview"
      >
        → {versionString}
      </span>
    </div>
    {#if errors.version}
      <p
        data-testid="tech-spec-create-error-version"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.version}
      </p>
    {/if}
  </fieldset>

  <!-- Deliverables (array dynamique) -->
  <div class="flex flex-col gap-2">
    <span class="text-sm font-medium text-gray-700">Livrables (deliverables)</span>
    {#each deliverables as deliverable, idx (idx)}
      <div
        class="flex items-center gap-2"
        data-testid={`tech-spec-deliverable-row-${idx}`}
      >
        <input
          data-testid={`tech-spec-deliverable-input-${idx}`}
          type="text"
          value={deliverable}
          oninput={(e) =>
            updateDeliverable(idx, (e.target as HTMLInputElement).value)}
          placeholder={`Livrable ${idx + 1}`}
          class="flex-1 border border-gray-300 rounded px-2 py-1 text-sm"
        />
        <button
          type="button"
          data-testid={`tech-spec-deliverable-remove-${idx}`}
          onclick={() => removeDeliverable(idx)}
          class="min-h-[32px] px-2 text-xs text-red-600 hover:text-red-800"
          aria-label={`Supprimer livrable ${idx + 1}`}
        >
          ✕
        </button>
      </div>
    {/each}
    <button
      type="button"
      data-testid="tech-spec-deliverable-add"
      onclick={addDeliverable}
      class="self-start text-xs text-blue-600 hover:text-blue-800 underline"
    >
      + Ajouter un livrable
    </button>
    {#if errors.deliverables}
      <p
        data-testid="tech-spec-create-error-deliverables"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.deliverables}
      </p>
    {/if}
  </div>

  <!-- Required signatures (multi-checkbox cohérent a11y > multi-select) -->
  <fieldset
    class="flex flex-col gap-1"
    aria-describedby={errors.requiredSignatures
      ? "tech-spec-create-error-requiredSignatures"
      : undefined}
  >
    <legend class="text-sm font-medium text-gray-700">
      Signatures requises
    </legend>
    <div
      class="flex flex-wrap gap-3"
      data-testid="tech-spec-required-sig-select"
      role="group"
      aria-label="Rôles signataires requis"
    >
      {#each SIGNATORY_ROLES as role (role)}
        <label class="inline-flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            data-testid={`tech-spec-required-sig-option-${role}`}
            checked={requiredSignatures.includes(role)}
            onchange={(e) =>
              toggleRequiredSignature(
                role,
                (e.target as HTMLInputElement).checked,
              )}
          />
          <span>{role}</span>
        </label>
      {/each}
    </div>
    {#if errors.requiredSignatures}
      <p
        id="tech-spec-create-error-requiredSignatures"
        data-testid="tech-spec-create-error-requiredSignatures"
        class="text-xs text-red-600"
        role="alert"
      >
        {errors.requiredSignatures}
      </p>
    {/if}
  </fieldset>

  <!-- Attachments (placeholder upload — pattern EvidenceUpload B5 simplifié) -->
  <div class="flex flex-col gap-1">
    <label
      for="tech-spec-attach-upload"
      class="text-sm font-medium text-gray-700"
    >
      Pièces jointes (URLs S3/MinIO — séparées par virgule)
    </label>
    <input
      id="tech-spec-attach-upload"
      data-testid="tech-spec-attach-upload"
      type="text"
      placeholder="ex: https://s3.example.com/devis-1.pdf, https://..."
      value={attachments.join(", ")}
      oninput={(e) => {
        const raw = (e.target as HTMLInputElement).value;
        attachments = raw
          .split(",")
          .map((s) => s.trim())
          .filter((s) => s.length > 0);
      }}
      class="border border-gray-300 rounded px-3 py-2 text-sm"
    />
    <p class="text-xs text-gray-500">
      Upload réel à brancher post-B7 (pattern EvidenceUpload B5).
    </p>
  </div>

  <!-- Actions -->
  <div class="flex justify-end gap-2 mt-2">
    {#if onCancel}
      <button
        type="button"
        data-testid="tech-spec-create-cancel"
        class="px-4 py-2 text-sm border border-gray-300 rounded text-gray-700 hover:bg-gray-50"
        onclick={() => onCancel?.()}
        disabled={submitting}
      >
        Annuler
      </button>
    {/if}
    <button
      type="submit"
      data-testid="tech-spec-create-submit"
      class="min-h-[44px] px-4 py-2 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
      disabled={!formValid || submitting}
      aria-disabled={!formValid || submitting}
    >
      {submitting
        ? "Envoi…"
        : mode === "bump"
          ? "Créer la nouvelle version"
          : "Créer la fiche technique"}
    </button>
  </div>
</form>
