<script lang="ts">
  // Story B7 (Phase B FE) — TechnicalSpecDetail.
  //
  // Vue détail d'une TechnicalSpec : title, version, status, deliverables,
  // attachments, signatures + actions :
  //   - "Soumettre pour signatures" (Draft uniquement)
  //   - "Nouvelle version (bump)" (Approved / Superseded — Draft aussi possible)
  //   - "Signer" (PendingSignatures + user a rôle dans required_signatures)
  //
  // INV-FE9 (a11y) :
  //   - Modal warning bump major avec `aria-modal="true"` + focus trap basique
  //     (auto-focus du bouton "Confirmer" à l'ouverture).
  //   - Status badge avec aria-label explicite (texte + couleur — daltoniens).
  //
  // SECURITY (cf. AC @security stories.md §B7) :
  //   Le bouton "Signer" n'apparaît QUE si :
  //     1. spec.status === "PendingSignatures"
  //     2. currentUserRole ∈ required_signatures
  //     3. Si rôle mandataire (amo/lawyer/architect) → mandate actif fourni
  //   Sinon le bouton est ABSENT du DOM (pas juste disabled — défense en
  //   profondeur).
  //
  // data-testid (cf. stories.md §B7) :
  //   tech-spec-detail-title
  //   tech-spec-detail-version
  //   tech-spec-detail-status-badge
  //   tech-spec-deliverable-list-{index}
  //   tech-spec-attachment-{index}
  //   tech-spec-submit-for-sign      (button — Draft only)
  //   tech-spec-bump-button          (button — déclenche modal new version)
  //   tech-spec-signatures-list
  //   tech-spec-signature-row-{userId}-{role}
  //   tech-spec-bump-modal           (modal de confirmation bump MAJOR)
  //   tech-spec-bump-confirm
  //   tech-spec-bump-cancel

  import { toast } from "../../../stores/toast";
  import {
    MANDATARY_ROLES,
    type SignatoryRole,
    type SignTechnicalSpecRequest,
    type TechnicalSpecDto,
    type TechnicalSpecSignatureDto,
  } from "../../api/technical_specs";
  import TechnicalSpecSignatureForm from "./TechnicalSpecSignatureForm.svelte";

  // ---------------------------------------------------------------------------
  // Props
  // ---------------------------------------------------------------------------

  let {
    spec,
    /** Signatures déjà posées (read-only — append-only par invariant). */
    signatures = [] as TechnicalSpecSignatureDto[],
    /** Rôle effectif du user courant pour cette spec — null si non éligible. */
    currentUserRole = null as SignatoryRole | null,
    /** Mandate actif du user pour le rôle (UUID + validUntil) — null si pas
     *  applicable ou si user direct. */
    activeMandate = null as { id: string; validUntil: string } | null,
    /** Callback "Soumettre pour signatures" (Draft → PendingSignatures). */
    onSubmitForSign,
    /** Callback "Bump" — le parent ouvre TechnicalSpecCreate en mode="bump". */
    onBump,
    /** Callback signature — le parent dispatch vers signSpec(). */
    onSign,
    /** Map user_id → label humain (résolution des signatures listées). */
    userLabels = {} as Record<string, string>,
  }: {
    spec: TechnicalSpecDto;
    signatures?: TechnicalSpecSignatureDto[];
    currentUserRole?: SignatoryRole | null;
    activeMandate?: { id: string; validUntil: string } | null;
    onSubmitForSign: (id: string) => Promise<void>;
    onBump: (spec: TechnicalSpecDto) => void;
    onSign: (
      id: string,
      req: SignTechnicalSpecRequest,
    ) => Promise<TechnicalSpecSignatureDto>;
    userLabels?: Record<string, string>;
  } = $props();

  // ---------------------------------------------------------------------------
  // Dérivations — gating signature + status + actions
  // ---------------------------------------------------------------------------

  let isDraft = $derived(spec.status === "Draft");
  let isPendingSignatures = $derived(spec.status === "PendingSignatures");
  let isApproved = $derived(spec.status === "Approved");
  let isSuperseded = $derived(spec.status === "Superseded");

  /** Le user a-t-il un rôle parmi les required_signatures de la spec ? */
  let userHasRequiredRole = $derived(
    currentUserRole !== null &&
      spec.required_signatures.includes(currentUserRole),
  );

  /** Le user est-il en rôle mandataire qui requiert un mandate actif ? */
  let userIsMandataryRole = $derived(
    currentUserRole !== null &&
      MANDATARY_ROLES.includes(currentUserRole),
  );

  /** Pré-requis mandate satisfait ? (si rôle mandataire → mandate actif requis) */
  let mandatePrereqOk = $derived(
    !userIsMandataryRole || activeMandate !== null,
  );

  /** Le user a-t-il DÉJÀ signé sous son rôle ? (INV unique (user, role)). */
  let userAlreadySigned = $derived.by<boolean>(() => {
    if (currentUserRole === null) return false;
    return signatures.some(
      (s) => s.role === currentUserRole && s.signatory_user_id !== "",
    );
  });

  /** Le bouton "Signer" est-il RENDU dans le DOM ?
   *
   *  Défense en profondeur (cf. AC @security) — on ne render PAS le bouton si
   *  les pré-conditions ne sont pas réunies :
   *    - spec en PendingSignatures
   *    - user a un rôle dans required_signatures
   *    - mandate pré-requis OK
   *    - user n'a pas déjà signé sous ce rôle
   */
  let canSign = $derived(
    isPendingSignatures &&
      userHasRequiredRole &&
      mandatePrereqOk &&
      !userAlreadySigned,
  );

  // ---------------------------------------------------------------------------
  // Bump modal — warning si MAJOR
  // ---------------------------------------------------------------------------

  let bumpModalOpen = $state<boolean>(false);

  function openBumpModal(): void {
    bumpModalOpen = true;
  }

  function closeBumpModal(): void {
    bumpModalOpen = false;
  }

  async function confirmBump(): Promise<void> {
    bumpModalOpen = false;
    onBump(spec);
  }

  // ---------------------------------------------------------------------------
  // Submit for sign action
  // ---------------------------------------------------------------------------

  let submittingForSign = $state<boolean>(false);

  async function handleSubmitForSign(): Promise<void> {
    if (submittingForSign) return;
    submittingForSign = true;
    try {
      await onSubmitForSign(spec.id);
      toast.success("Fiche soumise pour signatures.");
    } catch {
      // toast déjà émis par api.ts
    } finally {
      submittingForSign = false;
    }
  }

  // ---------------------------------------------------------------------------
  // Helpers display
  // ---------------------------------------------------------------------------

  function statusBadgeClasses(s: string): string {
    return (
      {
        Draft: "bg-gray-100 text-gray-700 border-gray-300",
        PendingSignatures: "bg-orange-100 text-orange-800 border-orange-300",
        Approved: "bg-green-100 text-green-800 border-green-300",
        Superseded: "bg-gray-200 text-gray-500 border-gray-300",
      }[s] ?? "bg-gray-100 text-gray-700 border-gray-300"
    );
  }

  function statusLabel(s: string): string {
    return (
      {
        Draft: "Brouillon",
        PendingSignatures: "En attente de signatures",
        Approved: "Approuvée",
        Superseded: "Remplacée",
      }[s] ?? s
    );
  }

  function userLabel(userId: string): string {
    return userLabels[userId] ?? userId.slice(0, 8);
  }

  function formatSignedAt(iso: string): string {
    try {
      return new Intl.DateTimeFormat("fr-BE", {
        day: "numeric",
        month: "long",
        year: "numeric",
      }).format(new Date(iso));
    } catch {
      return iso;
    }
  }
</script>

<article
  class="tech-spec-detail flex flex-col gap-4 p-4 bg-white rounded shadow-sm"
  aria-labelledby="tech-spec-detail-title-h"
>
  <!-- Header : title + version + status badge -->
  <header class="flex flex-wrap items-center gap-3">
    <h2
      id="tech-spec-detail-title-h"
      data-testid="tech-spec-detail-title"
      class="text-lg font-semibold text-gray-900"
    >
      {spec.title}
    </h2>
    <span
      data-testid="tech-spec-detail-version"
      class="font-mono text-sm text-gray-600"
    >
      v{spec.version}
    </span>
    <span
      data-testid="tech-spec-detail-status-badge"
      data-status={spec.status}
      class={`inline-flex items-center rounded border px-2 py-1 text-xs font-medium ${statusBadgeClasses(spec.status)}`}
      role="status"
      aria-label={`Status: ${statusLabel(spec.status)}`}
    >
      {statusLabel(spec.status)}
    </span>
  </header>

  <!-- Description -->
  <section class="text-sm text-gray-800 whitespace-pre-wrap">
    {spec.description}
  </section>

  <!-- Deliverables -->
  <section class="flex flex-col gap-1">
    <h3 class="text-sm font-medium text-gray-700">Livrables</h3>
    <ol class="list-decimal list-inside text-sm text-gray-800">
      {#each spec.deliverables as deliverable, idx (idx)}
        <li data-testid={`tech-spec-deliverable-list-${idx}`}>
          {deliverable}
        </li>
      {/each}
    </ol>
  </section>

  <!-- Attachments (URLs cliquables) -->
  {#if spec.attachments.length > 0}
    <section class="flex flex-col gap-1">
      <h3 class="text-sm font-medium text-gray-700">Pièces jointes</h3>
      <ul class="text-sm">
        {#each spec.attachments as att, idx (idx)}
          <li>
            <a
              data-testid={`tech-spec-attachment-${idx}`}
              href={att}
              target="_blank"
              rel="noopener noreferrer"
              class="text-blue-600 hover:underline break-all"
            >
              {att}
            </a>
          </li>
        {/each}
      </ul>
    </section>
  {/if}

  <!-- Signatures requises + reçues -->
  <section class="flex flex-col gap-2">
    <h3 class="text-sm font-medium text-gray-700">
      Signatures (reçues {signatures.length} / {spec.required_signatures.length}
      requises)
    </h3>
    <ul
      data-testid="tech-spec-signatures-list"
      class="text-sm flex flex-col gap-1"
      aria-label="Signatures de la fiche technique"
    >
      {#each signatures as sig (sig.id)}
        <li
          data-testid={`tech-spec-signature-row-${sig.signatory_user_id}-${sig.role}`}
          class="flex items-center gap-2 text-gray-800"
        >
          <span aria-hidden="true">✓</span>
          <span class="font-medium">{userLabel(sig.signatory_user_id)}</span>
          <span class="text-xs text-gray-600">({sig.role})</span>
          <span class="ml-auto text-xs text-gray-500">
            {formatSignedAt(sig.signed_at)}
          </span>
          {#if sig.mandate_id}
            <span class="text-xs text-gray-500"
              >via mandat #{sig.mandate_id.slice(0, 8)}</span
            >
          {/if}
        </li>
      {/each}
    </ul>
    <!-- Required signatures pas encore reçues -->
    {#each spec.required_signatures as req (req)}
      {#if !signatures.some((s) => s.role === req)}
        <p
          class="text-xs text-gray-500"
          data-testid={`tech-spec-signature-missing-${req}`}
        >
          ⏳ En attente : {req}
        </p>
      {/if}
    {/each}
  </section>

  <!-- Actions -->
  <footer class="flex flex-wrap items-center gap-2 mt-2">
    {#if isDraft}
      <button
        type="button"
        data-testid="tech-spec-submit-for-sign"
        onclick={() => void handleSubmitForSign()}
        disabled={submittingForSign}
        class="min-h-[44px] rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white shadow-sm hover:bg-blue-700 focus-visible:outline-2 focus-visible:outline-offset-2 disabled:cursor-not-allowed disabled:bg-gray-300"
      >
        {submittingForSign ? "Envoi…" : "Soumettre pour signatures"}
      </button>
    {/if}

    {#if isApproved || isSuperseded || isDraft}
      <button
        type="button"
        data-testid="tech-spec-bump-button"
        onclick={openBumpModal}
        class="min-h-[44px] rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-semibold text-gray-700 hover:bg-gray-50 focus-visible:outline-2 focus-visible:outline-offset-2"
      >
        Nouvelle version (bump)
      </button>
    {/if}
  </footer>

  <!-- Signature form (rendu UNIQUEMENT si canSign — cf. AC @security) -->
  {#if canSign}
    <TechnicalSpecSignatureForm
      specId={spec.id}
      role={currentUserRole!}
      {activeMandate}
      {onSign}
    />
  {/if}
</article>

<!-- Modal de confirmation bump (warning si major bump) -->
{#if bumpModalOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/30 p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="tech-spec-bump-modal-title"
    data-testid="tech-spec-bump-modal"
  >
    <div class="bg-white rounded shadow-lg p-6 max-w-md w-full">
      <h3
        id="tech-spec-bump-modal-title"
        class="text-base font-semibold mb-2"
      >
        Créer une nouvelle version ?
      </h3>
      <p class="text-sm text-gray-700 mb-2">
        Vous allez créer une nouvelle version à partir de
        <strong class="font-mono">v{spec.version}</strong>.
      </p>
      <p class="text-sm text-orange-700 mb-4">
        ⚠ Si vous augmentez le numéro <strong>MAJOR</strong>, toutes les
        signatures précédentes seront invalidées et les mandataires devront
        re-signer. Les bumps <strong>minor/patch</strong> préservent les
        signatures.
      </p>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          data-testid="tech-spec-bump-cancel"
          class="px-3 py-1 text-sm border border-gray-300 rounded text-gray-700"
          onclick={closeBumpModal}
        >
          Annuler
        </button>
        <!-- svelte-ignore a11y_autofocus -->
        <button
          type="button"
          data-testid="tech-spec-bump-confirm"
          class="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700"
          onclick={() => void confirmBump()}
          autofocus
        >
          Continuer
        </button>
      </div>
    </div>
  </div>
{/if}
