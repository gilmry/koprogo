<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { Building, PageResponse } from '../lib/types';
  import BuildingForm from './admin/BuildingForm.svelte';
  import ConfirmDialog from './ui/ConfirmDialog.svelte';
  import Button from './ui/Button.svelte';
  import Pagination from './Pagination.svelte';

  let buildings: Building[] = [];
  let loading = true;
  let error = '';
  let showFormModal = false;
  let showConfirmDialog = false;
  let selectedBuilding: Building | null = null;
  let formMode: 'create' | 'edit' = 'create';
  let actionLoading = false;
  let searchTerm = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    try {
      loading = true;
      error = '';
      const response = await api.get<PageResponse<Building>>(
        `/buildings?page=${currentPage}&per_page=${perPage}`
      );

      buildings = response.data;
      totalItems = response.pagination.total_items;
      totalPages = response.pagination.total_pages;
      currentPage = response.pagination.current_page;
      perPage = response.pagination.per_page;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des immeubles';
      console.error('Error loading buildings:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadBuildings();
  }

  const handleCreate = () => {
    selectedBuilding = null;
    formMode = 'create';
    showFormModal = true;
  };

  const handleEdit = (building: Building) => {
    selectedBuilding = building;
    formMode = 'edit';
    showFormModal = true;
  };

  const handleDeleteClick = (building: Building) => {
    selectedBuilding = building;
    showConfirmDialog = true;
  };

  const handleDeleteConfirm = async () => {
    if (!selectedBuilding) return;

    actionLoading = true;
    try {
      await api.delete(`/buildings/${selectedBuilding.id}`);
      toast.show('Immeuble supprimé avec succès', 'success');
      showConfirmDialog = false;
      selectedBuilding = null;
      await loadBuildings();
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : 'Une erreur est survenue';
      toast.show(errorMessage, 'error');
    } finally {
      actionLoading = false;
    }
  };

  const handleFormSuccess = async () => {
    await loadBuildings();
  };

  $: filteredBuildings = buildings.filter((building) => {
    if (!searchTerm) return true;
    const search = searchTerm.toLowerCase();
    return (
      building.name.toLowerCase().includes(search) ||
      building.address.toLowerCase().includes(search) ||
      building.city.toLowerCase().includes(search) ||
      building.postal_code.toLowerCase().includes(search)
    );
  });
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex justify-between items-center">
    <div>
      <h1 class="text-3xl font-bold text-gray-900">Immeubles</h1>
      <p class="mt-1 text-sm text-gray-600">
        Gérer les immeubles de votre copropriété
      </p>
    </div>
    <Button variant="primary" on:click={handleCreate} data-testid="create-building-button">
      ➕ Nouvel immeuble
    </Button>
  </div>

  <!-- Search -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="relative">
      <input
        type="text"
        bind:value={searchTerm}
        placeholder="Rechercher par nom, adresse, ville..."
        data-testid="building-search-input"
        class="w-full px-4 py-2 pl-10 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
      />
      <span class="absolute left-3 top-2.5 text-gray-400">🔍</span>
    </div>
  </div>

  <!-- Error Message -->
  {#if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
      ⚠️ {error}
    </div>
  {/if}

  <!-- Buildings Grid -->
  <div class="bg-white rounded-lg shadow overflow-hidden">
    {#if loading}
      <div class="p-12 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
        <p class="mt-2 text-gray-600">Chargement...</p>
      </div>
    {:else if filteredBuildings.length === 0}
      <div class="p-12 text-center text-gray-500">
        {searchTerm ? 'Aucun immeuble trouvé pour cette recherche.' : 'Aucun immeuble enregistré. Créez-en un pour commencer !'}
      </div>
    {:else}
      <div class="divide-y divide-gray-200" data-testid="buildings-list">
        {#each filteredBuildings as building (building.id)}
          <div
            class="p-6 hover:bg-gray-50 transition"
            data-testid="building-card"
            data-building-id={building.id}
            data-building-name={building.name}
          >
            <div class="flex justify-between items-start">
              <div class="flex-1">
                <h3 class="text-lg font-semibold text-gray-900" data-testid="building-name">
                  {building.name}
                </h3>
                <div class="mt-2 space-y-1">
                  <p class="text-sm text-gray-600" data-testid="building-address">
                    📍 {building.address}, {building.postal_code} {building.city}
                  </p>
                  <p class="text-sm text-gray-500">
                    🏢 {building.total_units} lots
                    {#if building.construction_year}
                      · 🏗️ Construit en {building.construction_year}
                    {/if}
                  </p>
                </div>
              </div>
              <div class="flex items-center space-x-2 ml-4">
                <button
                  on:click={() => handleEdit(building)}
                  class="text-primary-600 hover:text-primary-900"
                  title="Modifier"
                  disabled={actionLoading}
                  data-testid="edit-building-button"
                >
                  ✏️
                </button>
                <button
                  on:click={() => handleDeleteClick(building)}
                  class="text-red-600 hover:text-red-900"
                  title="Supprimer"
                  disabled={actionLoading}
                  data-testid="delete-building-button"
                >
                  🗑️
                </button>
                <a
                  href={`/building-detail?id=${building.id}`}
                  class="text-primary-600 hover:text-primary-900 text-sm font-medium"
                >
                  Détails →
                </a>
              </div>
            </div>
          </div>
        {/each}
      </div>

      <!-- Footer -->
      <div class="bg-gray-50 px-6 py-3 border-t border-gray-200">
        <p class="text-sm text-gray-700">
          <span class="font-medium">{filteredBuildings.length}</span>
          {filteredBuildings.length === 1 ? 'immeuble' : 'immeubles'}
          {searchTerm ? ' (filtrés)' : ''}
        </p>
      </div>
    {/if}
  </div>

  <!-- Pagination -->
  {#if !loading && totalPages > 1 && !searchTerm}
    <Pagination
      currentPage={currentPage}
      totalPages={totalPages}
      totalItems={totalItems}
      perPage={perPage}
      onPageChange={handlePageChange}
    />
  {/if}
</div>

<!-- Building Form Modal -->
<BuildingForm
  bind:isOpen={showFormModal}
  building={selectedBuilding}
  mode={formMode}
  on:success={handleFormSuccess}
  on:close={() => {
    showFormModal = false;
    selectedBuilding = null;
  }}
/>

<!-- Delete Confirmation Dialog -->
<ConfirmDialog
  bind:isOpen={showConfirmDialog}
  title="Confirmer la suppression"
  message="Êtes-vous sûr de vouloir supprimer l'immeuble '{selectedBuilding?.name}' ? Cette action est irréversible et supprimera également tous les lots, dépenses et données associées."
  confirmText="Supprimer"
  cancelText="Annuler"
  variant="danger"
  loading={actionLoading}
  on:confirm={handleDeleteConfirm}
  on:cancel={() => {
    showConfirmDialog = false;
    selectedBuilding = null;
  }}
/>
