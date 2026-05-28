<script lang="ts">
  // Story 2.x follow-up — Admin UI minimal pour CRUD ACP.
  // Backend: GET/POST /acps (Story 1.1 acp_handlers.rs).
  //
  // Pourquoi un admin UI minimal et pas un dashboard riche :
  // l'ACP est une racine de domaine récente (post-#602) ; cette page
  // débloque l'usage admin (créer ACPs pour les organizations) sans
  // attendre une page riche avec filtres/edit/archive. Ces gestes
  // arrivent en Story 4.x (governance).

  import { onMount } from "svelte";
  import {
    listAcps,
    createAcp,
    type AcpResponseDto,
    type CreateAcpDto,
  } from "../../lib/api/acps";

  let acps = $state<AcpResponseDto[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showCreate = $state(false);
  let creating = $state(false);

  // Formulaire création
  let form = $state<CreateAcpDto>({
    organization_id: null,
    name: "",
    address_street: "",
    address_postal_code: "",
    address_city: "",
    bce_number: null,
  });

  async function refresh(): Promise<void> {
    loading = true;
    error = null;
    try {
      acps = await listAcps();
    } catch (e) {
      error = e instanceof Error ? e.message : "Erreur de chargement";
    } finally {
      loading = false;
    }
  }

  onMount(refresh);

  async function submitCreate(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    creating = true;
    try {
      await createAcp({
        ...form,
        organization_id: form.organization_id || null,
        bce_number: form.bce_number || null,
      });
      // Reset + refresh
      form = {
        organization_id: null,
        name: "",
        address_street: "",
        address_postal_code: "",
        address_city: "",
        bce_number: null,
      };
      showCreate = false;
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Erreur de création";
    } finally {
      creating = false;
    }
  }
</script>

<div class="space-y-6" data-testid="admin-acps-page">
  <div class="flex items-center justify-between">
    <div>
      <h1 class="text-2xl font-bold text-gray-900">Associations de Copropriétaires (ACP)</h1>
      <p class="text-sm text-gray-600 mt-1">
        Racine de l'arbre métier : 1 ACP regroupe N immeubles. Cabinet syndic
        optionnel (auto-géré si vide).
      </p>
    </div>
    <button
      type="button"
      onclick={() => (showCreate = !showCreate)}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
      data-testid="acp-create-toggle"
    >
      {showCreate ? "Annuler" : "+ Nouvelle ACP"}
    </button>
  </div>

  {#if showCreate}
    <form
      onsubmit={submitCreate}
      class="bg-white border border-gray-200 rounded-lg p-6 space-y-4"
      data-testid="acp-create-form"
    >
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <label class="block">
          <span class="text-sm font-medium text-gray-700">Nom *</span>
          <input
            type="text"
            bind:value={form.name}
            required
            minlength="2"
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-name"
          />
        </label>
        <label class="block">
          <span class="text-sm font-medium text-gray-700">
            Organization ID (cabinet syndic, optionnel)
          </span>
          <input
            type="text"
            bind:value={form.organization_id}
            placeholder="UUID — vide = ACP auto-gérée"
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-org-id"
          />
        </label>
        <label class="block">
          <span class="text-sm font-medium text-gray-700">Rue *</span>
          <input
            type="text"
            bind:value={form.address_street}
            required
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-street"
          />
        </label>
        <label class="block">
          <span class="text-sm font-medium text-gray-700">Code postal *</span>
          <input
            type="text"
            bind:value={form.address_postal_code}
            required
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-postal"
          />
        </label>
        <label class="block">
          <span class="text-sm font-medium text-gray-700">Ville *</span>
          <input
            type="text"
            bind:value={form.address_city}
            required
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-city"
          />
        </label>
        <label class="block">
          <span class="text-sm font-medium text-gray-700">Numéro BCE</span>
          <input
            type="text"
            bind:value={form.bce_number}
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-bce"
          />
        </label>
      </div>
      <div class="flex justify-end gap-2">
        <button
          type="button"
          onclick={() => (showCreate = false)}
          class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700"
        >
          Annuler
        </button>
        <button
          type="submit"
          disabled={creating}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg disabled:opacity-50"
          data-testid="acp-form-submit"
        >
          {creating ? "Création..." : "Créer"}
        </button>
      </div>
    </form>
  {/if}

  {#if loading}
    <p class="text-gray-500" data-testid="acps-loading">Chargement...</p>
  {:else if error}
    <p class="text-red-600" data-testid="acps-error">{error}</p>
  {:else if acps.length === 0}
    <p class="text-gray-500 italic" data-testid="acps-empty">
      Aucune ACP. Créez-en une via le bouton ci-dessus.
    </p>
  {:else}
    <table class="min-w-full divide-y divide-gray-200" data-testid="acps-table">
      <thead class="bg-gray-50">
        <tr>
          <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Nom</th>
          <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Slug</th>
          <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Cabinet</th>
          <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">Adresse</th>
          <th class="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">BCE</th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each acps as acp (acp.id)}
          <tr data-testid="acp-row-{acp.id}">
            <td class="px-4 py-2 text-sm font-medium text-gray-900">{acp.name}</td>
            <td class="px-4 py-2 text-sm text-gray-500">{acp.slug}</td>
            <td class="px-4 py-2 text-sm text-gray-500">
              {acp.organization_id ?? "Auto-gérée"}
            </td>
            <td class="px-4 py-2 text-sm text-gray-500">
              {acp.address_street}, {acp.address_postal_code} {acp.address_city}
            </td>
            <td class="px-4 py-2 text-sm text-gray-500">{acp.bce_number ?? "—"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
