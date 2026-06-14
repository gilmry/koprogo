<script lang="ts">
  // Story B7 (Phase B FE) — Wrapper Svelte pour la vue détail d'une
  // TechnicalSpec.
  //
  // Page : /syndic/technical-spec?id=<UUID>
  //
  // Orchestre :
  //   - Lit l'id depuis `window.location.search` (cf. Gotcha #1 stories.md §B7
  //     — Astro static, query param impératif).
  //   - Fetch la spec + signatures + (à venir) versions historiques.
  //   - Détermine currentUserRole côté FE (via authStore).
  //   - Bind actions onSubmitForSign / onBump / onSign vers l'API.
  //   - Pour bump : ouvre la modal de création en mode="bump".

  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { authStore } from "../../../stores/auth";
  import {
    getSpec,
    listSignatures,
    listSpecs,
    submitForSignatures,
    signSpec,
    bumpVersion,
    type BumpTechnicalSpecRequest,
    type CreateTechnicalSpecRequest,
    type SignTechnicalSpecRequest,
    type SignatoryRole,
    type TechnicalSpecDto,
    type TechnicalSpecSignatureDto,
  } from "../../api/technical_specs";
  import { listMandates, type MandateResponse } from "../../api/mandates";
  import TechnicalSpecDetail from "./TechnicalSpecDetail.svelte";
  import TechnicalSpecCreate from "./TechnicalSpecCreate.svelte";
  import TechnicalSpecVersionTimeline from "./TechnicalSpecVersionTimeline.svelte";

  let spec = $state<TechnicalSpecDto | null>(null);
  let signatures = $state<TechnicalSpecSignatureDto[]>([]);
  let historyVersions = $state<TechnicalSpecDto[]>([]);
  let loading = $state<boolean>(true);
  let notFound = $state<boolean>(false);
  let bumpMode = $state<boolean>(false);

  // ---------------------------------------------------------------------------
  // currentUserRole : déduit du authStore + matching avec required_signatures
  // ---------------------------------------------------------------------------

  let currentUserRole = $state<SignatoryRole | null>(null);
  let activeMandate = $state<{ id: string; validUntil: string } | null>(null);

  function readSpecIdFromUrl(): string | null {
    if (typeof window === "undefined") return null;
    const params = new URLSearchParams(window.location.search);
    return params.get("id");
  }

  async function loadInitial(): Promise<void> {
    loading = true;
    notFound = false;
    const id = readSpecIdFromUrl();
    if (!id) {
      notFound = true;
      loading = false;
      return;
    }
    try {
      // Charge spec + signatures en parallèle.
      const [s, sigs] = await Promise.all([
        getSpec(id),
        listSignatures(id).catch(
          () => [] as TechnicalSpecSignatureDto[],
        ),
      ]);
      spec = s;
      signatures = sigs;

      // Charge l'historique des versions (filtre client side sur title/acp).
      try {
        const all = await listSpecs();
        historyVersions = all.filter(
          (x) =>
            x.title === s.title &&
            x.acp_id === s.acp_id &&
            x.building_id === s.building_id,
        );
      } catch {
        historyVersions = [s];
      }

      // Détermine currentUserRole + activeMandate.
      const auth = get(authStore);
      const user = auth.user as
        | { id: string; role: string }
        | null
        | undefined;
      if (user) {
        // Mapping rôle user → SignatoryRole.
        // - "syndic" et "superadmin" → syndic
        // - mandataires assignés → cherchés dans listMandates() ci-dessous.
        if (user.role === "syndic" || user.role === "superadmin") {
          currentUserRole = "syndic";
        } else {
          // Sinon, on regarde les mandats du user pour trouver un rôle
          // signataire compatible avec required_signatures.
          try {
            const mandates = await listMandates();
            const now = new Date().getTime();
            const userMandates = (mandates as MandateResponse[]).filter(
              (m) =>
                m.subject_user_id === user.id &&
                !m.revoked_at &&
                new Date(m.valid_until).getTime() > now,
            );
            for (const req of s.required_signatures) {
              const m = userMandates.find((x) => x.kind === req);
              if (m) {
                currentUserRole = req as SignatoryRole;
                activeMandate = { id: m.id, validUntil: m.valid_until };
                break;
              }
            }
          } catch {
            currentUserRole = null;
            activeMandate = null;
          }
        }
      }
    } catch {
      notFound = true;
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });

  // ---------------------------------------------------------------------------
  // Action handlers
  // ---------------------------------------------------------------------------

  async function handleSubmitForSign(id: string): Promise<void> {
    const updated = await submitForSignatures(id);
    spec = updated;
  }

  function handleBumpRequest(_source: TechnicalSpecDto): void {
    bumpMode = true;
  }

  async function handleBumpSubmit(
    req: CreateTechnicalSpecRequest | BumpTechnicalSpecRequest,
  ): Promise<TechnicalSpecDto> {
    if (!spec) throw new Error("Spec source manquante.");
    const created = await bumpVersion(
      spec.id,
      req as BumpTechnicalSpecRequest,
    );
    // Optimistic update : on navigue vers la nouvelle version.
    if (typeof window !== "undefined") {
      window.location.href = `/syndic/technical-spec?id=${encodeURIComponent(created.id)}`;
    }
    return created;
  }

  async function handleSign(
    id: string,
    req: SignTechnicalSpecRequest,
  ): Promise<TechnicalSpecSignatureDto> {
    const created = await signSpec(id, req);
    signatures = [...signatures, created];
    // Refresh la spec — son status peut être passé en Approved.
    try {
      spec = await getSpec(id);
    } catch {
      // ignore — le status reste l'ancien si erreur.
    }
    return created;
  }
</script>

{#if loading}
  <p class="text-sm text-gray-500" role="status" aria-live="polite">
    Chargement…
  </p>
{:else if notFound}
  <div
    class="rounded-md border border-red-200 bg-red-50 p-4 text-sm text-red-700"
    data-testid="tech-spec-not-found"
    role="alert"
  >
    Fiche technique introuvable ou accès refusé.
  </div>
{:else if spec}
  <div class="flex flex-col gap-6 lg:flex-row">
    <div class="flex-1">
      <TechnicalSpecDetail
        {spec}
        {signatures}
        {currentUserRole}
        {activeMandate}
        onSubmitForSign={handleSubmitForSign}
        onBump={handleBumpRequest}
        onSign={handleSign}
      />
    </div>
    <aside class="w-full lg:w-80">
      <TechnicalSpecVersionTimeline
        versions={historyVersions}
        currentVersionId={spec.id}
      />
    </aside>
  </div>

  {#if bumpMode}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/30 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tech-spec-bump-modal-title"
    >
      <div
        class="bg-white rounded shadow-lg max-w-3xl w-full max-h-[90vh] overflow-y-auto"
      >
        <TechnicalSpecCreate
          acpId={spec.acp_id}
          buildingId={spec.building_id}
          mode="bump"
          previousVersion={spec}
          onSubmit={handleBumpSubmit}
          onCancel={() => (bumpMode = false)}
        />
      </div>
    </div>
  {/if}
{/if}
