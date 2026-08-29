<script lang="ts">
  // Admin UI de gestion des ACP — création, modification, suppression.
  // Backend : `acp_handlers.rs` (POST / GET / PUT / DELETE /acps).
  //
  // Cette page a longtemps été orpheline : livrée par 7d9aab08 (« ACPs
  // invisibles »), elle n'était liée depuis nulle part et n'était atteignable
  // qu'en tapant l'URL. Le point d'entrée vit désormais dans
  // `Navigation.svelte::getAdminItems()` et sur le tableau de bord admin.

  import { onMount } from "svelte";
  import {
    listAcps,
    createAcp,
    updateAcp,
    archiveAcp,
    type AcpResponseDto,
    type CreateAcpDto,
    type UpdateAcpDto,
  } from "../../lib/api/acps";
  import { api } from "../../lib/api";

  interface OrganizationOption {
    id: string;
    name: string;
  }

  let acps = $state<AcpResponseDto[]>([]);
  let organizations = $state<OrganizationOption[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showCreate = $state(false);
  let creating = $state(false);

  // Édition : `null` = aucune ligne en cours de modification.
  let editingId = $state<string | null>(null);
  let editForm = $state<UpdateAcpDto | null>(null);
  let saving = $state(false);

  // Suppression : id en attente de confirmation, puis id en cours.
  let confirmingArchiveId = $state<string | null>(null);
  let archivingId = $state<string | null>(null);

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

  async function loadOrganizations(): Promise<void> {
    try {
      const response = await api.get<{ data: OrganizationOption[] }>(
        "/organizations?per_page=1000",
      );
      organizations = response.data;
    } catch (e) {
      console.error("Error loading organizations:", e);
    }
  }

  // `undefined` accepté en plus de `null` : dans le type généré depuis la spec,
  // `organization_id` est optionnel (`Option<String>` côté Rust). Le corps
  // traitait déjà les deux cas via le test de véracité.
  function organizationLabel(
    organizationId: string | null | undefined,
  ): string {
    if (!organizationId) return "Auto-gérée";
    const org = organizations.find((o) => o.id === organizationId);
    return org ? org.name : organizationId;
  }

  onMount(() => {
    refresh();
    loadOrganizations();
  });

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

  /// Ouvre le formulaire d'édition, pré-rempli avec les valeurs courantes.
  ///
  /// Le pré-remplissage n'est pas cosmétique : `PUT /acps/{id}` est
  /// **full-state**, `name` et les trois champs d'adresse y sont obligatoires.
  /// Un formulaire vide écraserait donc ce qu'il n'affiche pas.
  function startEdit(acp: AcpResponseDto): void {
    error = null;
    confirmingArchiveId = null;
    editingId = acp.id;
    editForm = {
      name: acp.name,
      address_street: acp.address_street,
      address_postal_code: acp.address_postal_code,
      address_city: acp.address_city,
      bce_number: acp.bce_number ?? null,
      organization_id: acp.organization_id ?? null,
      total_tantiemes: acp.total_tantiemes,
    };
  }

  function cancelEdit(): void {
    editingId = null;
    editForm = null;
  }

  async function submitEdit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    if (!editingId || !editForm) return;
    saving = true;
    error = null;
    try {
      // `organization_id` est envoyé explicitement : le select montre toujours
      // un état, donc choisir « Aucun » vaut bien détachement (`null`), et non
      // « ne pas toucher ».
      await updateAcp(editingId, {
        ...editForm,
        bce_number: editForm.bce_number || null,
        organization_id: editForm.organization_id || null,
      });
      cancelEdit();
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Erreur de modification";
    } finally {
      saving = false;
    }
  }

  /// Supprime définitivement une ACP.
  ///
  /// Le backend renvoie **409** si l'ACP porte encore des immeubles. On affiche
  /// son message tel quel : il précise combien, ce qui est l'information utile
  /// pour agir.
  async function confirmArchive(id: string): Promise<void> {
    archivingId = id;
    error = null;
    try {
      await archiveAcp(id);
      confirmingArchiveId = null;
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : "Erreur de suppression";
    } finally {
      archivingId = null;
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
            Cabinet syndic (organisation, optionnel)
          </span>
          <select
            bind:value={form.organization_id}
            class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
            data-testid="acp-form-org-id"
          >
            <option value={null}>Aucun — ACP auto-gérée</option>
            {#each organizations as org (org.id)}
              <option value={org.id}>{org.name}</option>
            {/each}
          </select>
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
          <th class="px-4 py-2 text-right text-xs font-medium text-gray-500 uppercase">Actions</th>
        </tr>
      </thead>
      <tbody class="bg-white divide-y divide-gray-200">
        {#each acps as acp (acp.id)}
          <tr data-testid="acp-row-{acp.id}">
            <td class="px-4 py-2 text-sm font-medium text-gray-900">{acp.name}</td>
            <td class="px-4 py-2 text-sm text-gray-500">{acp.slug}</td>
            <td class="px-4 py-2 text-sm text-gray-500">
              {organizationLabel(acp.organization_id)}
            </td>
            <td class="px-4 py-2 text-sm text-gray-500">
              {acp.address_street}, {acp.address_postal_code} {acp.address_city}
            </td>
            <td class="px-4 py-2 text-sm text-gray-500">{acp.bce_number ?? "—"}</td>
            <td class="px-4 py-2 text-sm text-right whitespace-nowrap">
              <button
                type="button"
                onclick={() => startEdit(acp)}
                class="text-primary-600 hover:text-primary-800 mr-3"
                data-testid="acp-edit-{acp.id}"
              >
                Modifier
              </button>
              <button
                type="button"
                onclick={() => (confirmingArchiveId = acp.id)}
                class="text-red-600 hover:text-red-800"
                data-testid="acp-archive-{acp.id}"
              >
                Supprimer
              </button>
            </td>
          </tr>

          {#if confirmingArchiveId === acp.id}
            <tr class="bg-red-50" data-testid="acp-archive-confirm-{acp.id}">
              <td colspan="6" class="px-4 py-3 text-sm">
                <!-- Le backend nomme ce geste « archive » mais il exécute un
                     DELETE : le libellé doit dire ce qui se passe vraiment. -->
                <span class="text-red-800">
                  Supprimer définitivement « {acp.name} » ? Cette action est
                  irréversible. Une ACP portant encore des immeubles sera
                  refusée.
                </span>
                <span class="ml-3 inline-flex gap-2">
                  <button
                    type="button"
                    onclick={() => confirmArchive(acp.id)}
                    disabled={archivingId === acp.id}
                    class="px-3 py-1 bg-red-600 text-white rounded disabled:opacity-50"
                    data-testid="acp-archive-confirm"
                  >
                    {archivingId === acp.id ? "Suppression..." : "Confirmer"}
                  </button>
                  <button
                    type="button"
                    onclick={() => (confirmingArchiveId = null)}
                    class="px-3 py-1 border border-gray-300 rounded text-gray-700"
                    data-testid="acp-archive-cancel"
                  >
                    Annuler
                  </button>
                </span>
              </td>
            </tr>
          {/if}

          {#if editingId === acp.id && editForm}
            <tr class="bg-gray-50">
              <td colspan="6" class="px-4 py-4">
                <form
                  onsubmit={submitEdit}
                  class="space-y-4"
                  data-testid="acp-edit-form"
                >
                  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">Nom *</span>
                      <input
                        type="text"
                        bind:value={editForm.name}
                        required
                        minlength="2"
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-name"
                      />
                    </label>
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">
                        Cabinet syndic (vide = ACP auto-gérée)
                      </span>
                      <select
                        bind:value={editForm.organization_id}
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-org-id"
                      >
                        <option value={null}>Aucun — ACP auto-gérée</option>
                        {#each organizations as org (org.id)}
                          <option value={org.id}>{org.name}</option>
                        {/each}
                      </select>
                    </label>
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">Rue *</span>
                      <input
                        type="text"
                        bind:value={editForm.address_street}
                        required
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-street"
                      />
                    </label>
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">Code postal *</span>
                      <input
                        type="text"
                        bind:value={editForm.address_postal_code}
                        required
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-postal"
                      />
                    </label>
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">Ville *</span>
                      <input
                        type="text"
                        bind:value={editForm.address_city}
                        required
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-city"
                      />
                    </label>
                    <label class="block">
                      <span class="text-sm font-medium text-gray-700">Numéro BCE</span>
                      <input
                        type="text"
                        bind:value={editForm.bce_number}
                        class="mt-1 w-full px-3 py-2 border border-gray-300 rounded-lg"
                        data-testid="acp-edit-bce"
                      />
                    </label>
                  </div>
                  <div class="flex justify-end gap-2">
                    <button
                      type="button"
                      onclick={cancelEdit}
                      class="px-4 py-2 border border-gray-300 rounded-lg text-gray-700"
                      data-testid="acp-edit-cancel"
                    >
                      Annuler
                    </button>
                    <button
                      type="submit"
                      disabled={saving}
                      class="px-4 py-2 bg-primary-600 text-white rounded-lg disabled:opacity-50"
                      data-testid="acp-edit-submit"
                    >
                      {saving ? "Enregistrement..." : "Enregistrer"}
                    </button>
                  </div>
                </form>
              </td>
            </tr>
          {/if}
        {/each}
      </tbody>
    </table>
  {/if}
</div>
