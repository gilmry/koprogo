<script lang="ts">
  // Story B7 (Phase B FE) — Wrapper Svelte pour la liste TechnicalSpec.
  //
  // Page : /syndic/technical-specs
  //
  // Orchestre :
  //   - Fetch initial via `listSpecs()` au mount.
  //   - Bouton "Nouvelle fiche" → modal `<TechnicalSpecCreate>`.
  //   - Table simple des specs (lien → /syndic/technical-spec?id=X).
  //   - Refresh optimiste après création.

  import { onMount } from "svelte";
  import {
    listSpecs,
    createSpec,
    type CreateTechnicalSpecRequest,
    type BumpTechnicalSpecRequest,
    type TechnicalSpecDto,
  } from "../../api/technical_specs";
  import { listAcps } from "../../api/acps";
  import TechnicalSpecCreate from "./TechnicalSpecCreate.svelte";

  let specs = $state<TechnicalSpecDto[]>([]);
  let loading = $state<boolean>(true);
  let showForm = $state<boolean>(false);
  let acps = $state<Array<{ id: string; name: string }>>([]);
  let selectedAcpId = $state<string>("");

  async function loadInitial(): Promise<void> {
    loading = true;
    try {
      const [s, a] = await Promise.all([
        listSpecs().catch(() => [] as TechnicalSpecDto[]),
        listAcps().catch(() => []),
      ]);
      specs = s;
      acps = (a ?? []).map(
        (x: { id: string; name: string }) => ({ id: x.id, name: x.name }),
      );
      if (acps.length > 0 && selectedAcpId === "") {
        selectedAcpId = acps[0].id;
      }
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadInitial();
  });

  async function handleSubmit(
    req: CreateTechnicalSpecRequest | BumpTechnicalSpecRequest,
  ): Promise<TechnicalSpecDto> {
    // En mode "create" depuis cette page, on appelle createSpec (le bump
    // se fait depuis TechnicalSpecDetail). Le payload sait s'il a acp_id.
    if ("acp_id" in req) {
      const created = await createSpec(req as CreateTechnicalSpecRequest);
      specs = [created, ...specs];
      showForm = false;
      return created;
    }
    throw new Error("Bump non géré depuis la liste — utiliser la vue détail.");
  }

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
</script>

<div class="flex flex-col gap-6">
  <header class="flex items-center justify-between">
    <h1 class="text-2xl font-semibold text-gray-900">Fiches techniques</h1>
    <button
      type="button"
      data-testid="tech-spec-new-button"
      class="min-h-[44px] rounded-md bg-blue-600 px-4 py-2 text-sm font-semibold text-white hover:bg-blue-700"
      onclick={() => (showForm = true)}
      disabled={loading || acps.length === 0}
    >
      Nouvelle fiche technique
    </button>
  </header>

  {#if loading}
    <p class="text-sm text-gray-500" role="status" aria-live="polite">
      Chargement…
    </p>
  {:else if specs.length === 0}
    <p data-testid="tech-spec-list-empty" class="text-sm text-gray-500">
      Aucune fiche technique pour le moment.
    </p>
  {:else}
    <div class="overflow-x-auto">
      <table
        data-testid="tech-spec-list"
        class="min-w-full divide-y divide-gray-200 text-sm"
      >
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Titre
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Version
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Status
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-left font-medium text-gray-700"
            >
              Créée le
            </th>
            <th
              scope="col"
              class="px-3 py-2 text-right font-medium text-gray-700"
            >
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-100">
          {#each specs as s (s.id)}
            <tr data-testid={`tech-spec-list-row-${s.id}`}>
              <td class="px-3 py-2 text-gray-900">{s.title}</td>
              <td class="px-3 py-2 text-gray-700 font-mono">v{s.version}</td>
              <td class="px-3 py-2">
                <span
                  class={`inline-flex items-center rounded border px-2 py-1 text-xs font-medium ${statusBadgeClasses(s.status)}`}
                >
                  {s.status}
                </span>
              </td>
              <td class="px-3 py-2 text-gray-600">
                {new Date(s.created_at).toISOString().slice(0, 10)}
              </td>
              <td class="px-3 py-2 text-right">
                <a
                  href={`/syndic/technical-spec?id=${encodeURIComponent(s.id)}`}
                  data-testid={`tech-spec-detail-link-${s.id}`}
                  class="text-xs text-blue-600 hover:underline"
                >
                  Détail
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

  {#if showForm && selectedAcpId}
    <div
      class="fixed inset-0 z-40 flex items-center justify-center bg-black/30 p-4"
      role="dialog"
      aria-modal="true"
      aria-labelledby="tech-spec-create-modal-title"
    >
      <div
        class="bg-white rounded shadow-lg max-w-3xl w-full max-h-[90vh] overflow-y-auto"
      >
        <div class="px-4 py-2 border-b">
          <label class="text-xs text-gray-600">
            ACP cible
            <select
              data-testid="tech-spec-acp-select"
              bind:value={selectedAcpId}
              class="ml-2 border border-gray-300 rounded px-2 py-1 text-sm"
            >
              {#each acps as a (a.id)}
                <option value={a.id}>{a.name}</option>
              {/each}
            </select>
          </label>
        </div>
        <TechnicalSpecCreate
          acpId={selectedAcpId}
          onSubmit={handleSubmit}
          onCancel={() => (showForm = false)}
        />
      </div>
    </div>
  {/if}
</div>
