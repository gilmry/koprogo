<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import type { Building, PageResponse } from '../lib/types';
  import BuildingForm from './admin/BuildingForm.svelte';
  import ConfirmDialog from './ui/ConfirmDialog.svelte';
  import Button from './ui/Button.svelte';
  import Pagination from './Pagination.svelte';
  import { withLoadingState, withErrorHandling } from '../lib/utils/error.utils';

  $: isSuperAdmin = $authStore.user?.role === 'superadmin';

  let buildings: Building[] = [];
  let loading = true;
  let error = '';
  let showFormModal = false;
  let showConfirmDialog = false;
  let selectedBuilding: Building | null = null;
  let formMode: 'create' | 'edit' = 'create';
  let actionLoading = false;
  let searchTerm = '';

  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    await withLoadingState({
      action: () => api.get<PageResponse<Building>>(
        `/buildings?page=${currentPage}&per_page=${perPage}`
      ),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      errorMessage: $_('buildings.errorLoading'),
      onSuccess: (response) => {
        buildings = response.data;
        totalItems = response.pagination.total_items;
        totalPages = response.pagination.total_pages;
        currentPage = response.pagination.current_page;
        perPage = response.pagination.per_page;
      },
    });
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

    await withErrorHandling({
      action: () => api.delete(`/buildings/${selectedBuilding!.id}`),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('buildings.deletedSuccess'),
      errorMessage: $_('common.error'),
      onSuccess: async () => {
        showConfirmDialog = false;
        selectedBuilding = null;
        await loadBuildings();
      },
    });
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
      <h1 class="text-3xl font-bold text-gray-900">{$_('buildings.title')}</h1>
      <p class="mt-1 text-sm text-gray-600">
        {$_('buildings.subtitle')}
      </p>
    </div>
    {#if isSuperAdmin}
      <Button variant="primary" on:click={handleCreate} data-testid="create-building-button">
        ➕ {$_('buildings.new')}
      </Button>
    {/if}
  </div>

  <!-- Search -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="relative">
      <label for="building-search" class="sr-only">{$_('buildings.searchLabel')}</label>
      <input
        id="building-search"
        type="text"
        bind:value={searchTerm}
        placeholder={$_('buildings.searchPlaceholder')}
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
        <p class="mt-2 text-gray-600">{$_('common.loading')}</p>
      </div>
    {:else if filteredBuildings.length === 0}
      <div class="p-12 text-center text-gray-500">
        {searchTerm ? $_('buildings.noResults') : $_('buildings.noBuildings')}
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
                    🏢 {building.total_units} {$_('buildings.units')}
                    {#if building.construction_year}
                      · 🏗️ {$_('buildings.builtIn')} {building.construction_year}
                    {/if}
                  </p>
                </div>
              </div>
              <div class="flex items-center space-x-2 ml-4">
                {#if isSuperAdmin}
                  <button
                    on:click={() => handleEdit(building)}
                    class="text-primary-600 hover:text-primary-900"
                    title={$_('common.edit')}
                    disabled={actionLoading}
                    data-testid="edit-building-button"
                  >
                    ✏️
                  </button>
                  <button
                    on:click={() => handleDeleteClick(building)}
                    class="text-red-600 hover:text-red-900"
                    title={$_('common.delete')}
                    disabled={actionLoading}
                    data-testid="delete-building-button"
                  >
                    🗑️
                  </button>
                {/if}
                <a
                  href={`/building-detail?id=${building.id}`}
                  class="text-primary-600 hover:text-primary-900 text-sm font-medium"
                >
                  {$_('buildings.details')} →
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
          {filteredBuildings.length === 1 ? $_('buildings.buildingSingular') : $_('buildings.buildingPlural')}
          {searchTerm ? ` (${$_('common.filtered')})` : ''}
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
  title={$_('buildings.confirmDeleteTitle')}
  message={`${$_('buildings.confirmDeleteMessage', { values: { name: selectedBuilding?.name || '' } })}`}
  confirmText={$_('common.delete')}
  cancelText={$_('common.cancel')}
  variant="danger"
  loading={actionLoading}
  on:confirm={handleDeleteConfirm}
  on:cancel={() => {
    showConfirmDialog = false;
    selectedBuilding = null;
  }}
/>
