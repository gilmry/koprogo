<script lang="ts">
  // Story B8 (Phase B FE) — Wrapper Svelte pour la page /syndic/contractor-evaluations.
  //
  // Orchestre :
  //   - Fetch initial : listSpecs() (filter Approved côté FE pour passer au form),
  //     liste users mandatables / contractors via `/users`, current user id.
  //   - Bouton "Nouvelle évaluation" → modal <ContractorEvaluationForm>.
  //   - Pas de liste d'évaluations à ce niveau (cf. ContractorReputation pour
  //     consulter par contractor — la liste cross-contractor n'est PAS dans
  //     le périmètre B8).
  //
  // Pattern wrapper cohérent B3/B7 (MandatesPage / TechnicalSpecsPage).

  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { authStore } from "../../../stores/auth";
  import { api } from "../../api";
  import { listSpecs } from "../../api/technical_specs";
  import {
    createEvaluation,
    type ContractorEvaluationDto,
    type CreateContractorEvaluationRequest,
  } from "../../api/contractor_evaluations";
  import ContractorEvaluationForm from "./ContractorEvaluationForm.svelte";

  type UserLike = {
    id: string;
    email: string;
    first_name?: string;
    last_name?: string;
    role: string;
  };

  type SpecLike = {
    id: string;
    title: string;
    version: string;
    status: string;
  };

  type TicketLike = {
    id: string;
    title: string;
  };

  let loading = $state<boolean>(true);
  let showForm = $state<boolean>(false);
  let currentUserId = $state<string | null>(null);
  let contractors = $state<Array<{ id: string; label: string }>>([]);
  let specs = $state<SpecLike[]>([]);
  let tickets = $state<TicketLike[]>([]);
  /** Évaluations créées dans la session courante — affichées pour feedback
   *  optimiste. Pas de liste persistée à ce niveau (pattern cross-contractor
   *  hors périmètre B8). */
  let recentEvaluations = $state<ContractorEvaluationDto[]>([]);

  async function loadInitial(): Promise<void> {
    loading = true;
    try {
      // currentUserId via authStore (cohérent B3 MandateIssueForm pattern).
      try {
        const state = get(authStore);
        currentUserId = state.user?.id ?? null;
      } catch {
        currentUserId = null;
      }

      const [specsResp, usersResp] = await Promise.all([
        listSpecs().catch(() => [] as SpecLike[]),
        api
          .get<{ data: UserLike[] }>("/users")
          .catch(() => ({ data: [] as UserLike[] })),
      ]);

      specs = specsResp.map((s) => ({
        id: s.id,
        title: s.title,
        version: s.version,
        status: s.status,
      }));

      // Contractors = users dont le rôle est "contractor" (ou voisins métier
      // selon backend — on est défensif si la liste varie).
      const usrs = usersResp.data ?? [];
      const CONTRACTOR_ROLES = new Set(["contractor"]);
      contractors = usrs
        .filter((u) => CONTRACTOR_ROLES.has(u.role))
        .map((u) => ({
          id: u.id,
          label:
            `${u.first_name ?? ""} ${u.last_name ?? ""}`.trim() || u.email,
        }));

      // Tickets : pour l'instant on charge "my tickets" du syndic (liste
      // courte ciblée). Le filtre par building/spec se fera en B8+.
      try {
        const tk = await api.get<TicketLike[]>("/tickets/my");
        tickets = (tk ?? []).map((t) => ({
          id: t.id,
          title: t.title ?? `Ticket ${t.id.slice(0, 8)}`,
        }));
      } catch {
        tickets = [];
      }
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });

  async function handleSubmit(
    req: CreateContractorEvaluationRequest,
  ): Promise<ContractorEvaluationDto> {
    const created = await createEvaluation(req);
    recentEvaluations = [created, ...recentEvaluations];
    showForm = false;
    return created;
  }
</script>

<div class="flex flex-col gap-6">
  <header class="flex items-center justify-between">
    <h1
      class="text-2xl font-semibold text-gray-900"
      data-testid="contractor-evaluations-page-title"
    >
      Évaluations contractor
    </h1>
    <button
      type="button"
      data-testid="contractor-eval-new-button"
      class="min-h-[44px] rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700 disabled:opacity-50"
      onclick={() => (showForm = true)}
      disabled={loading || contractors.length === 0}
    >
      Nouvelle évaluation
    </button>
  </header>

  {#if loading}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      Chargement…
    </p>
  {:else if recentEvaluations.length === 0}
    <p
      data-testid="contractor-eval-list-empty"
      class="text-sm text-gray-500"
    >
      Aucune évaluation enregistrée pour le moment dans cette session.
      Consultez la page « Réputation contractor » pour l'historique complet.
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid="contractor-eval-list"
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Contractor
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Globale
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Date
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Commentaire
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each recentEvaluations as ev (ev.id)}
            <tr data-testid={`contractor-eval-row-${ev.id}`}>
              <td class="px-3 py-2 text-gray-900">
                <a
                  href={`/contractor-reputation?contractorId=${encodeURIComponent(ev.contractor_user_id)}`}
                  class="text-blue-600 hover:underline"
                  data-testid={`contractor-eval-reputation-link-${ev.id}`}
                >
                  Voir réputation
                </a>
              </td>
              <td class="px-3 py-2 text-gray-900 font-mono font-semibold">
                {ev.scores.overall}/5
              </td>
              <td class="px-3 py-2 text-gray-600">
                {new Date(ev.created_at).toISOString().slice(0, 10)}
              </td>
              <td class="px-3 py-2 text-gray-700 max-w-xs truncate">
                {ev.comment}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if showForm}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/30 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="contractor-eval-form-title"
    >
      <div
        class="bg-white rounded shadow-lg max-w-3xl w-full max-h-[90vh] overflow-y-auto"
      >
        <ContractorEvaluationForm
          {currentUserId}
          {contractors}
          {specs}
          {tickets}
          onSubmit={handleSubmit}
          onCancel={() => (showForm = false)}
        />
      </div>
    </div>
  {/if}
</div>
