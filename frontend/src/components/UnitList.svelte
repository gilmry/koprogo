<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import type { Unit, PageResponse, Building } from '../lib/types';
  import Pagination from './Pagination.svelte';
  import UnitOwners from './UnitOwners.svelte';
  import UnitCreateModal from './UnitCreateModal.svelte';
  import UnitEditModal from './UnitEditModal.svelte';
  import Button from './ui/Button.svelte';

  export let buildingId: string | null = null;

  $: isSuperAdmin = $authStore.user?.role === 'superadmin';

  let units: Unit[] = [];
  let loading = true;
  let error = '';
  let building: Building | null = null;

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  // Expanded units (to show owners)
  let expandedUnits: Set<string> = new Set();

  // Modal state
  let showCreateModal = false;
  let showEditModal = false;
  let selectedUnit: Unit | null = null;
  let showDeleteConfirm = false;
  let unitToDelete: Unit | null = null;

  onMount(async () => {
    if (buildingId) {
      await loadBuilding();
    }
    await loadUnits();
  });

  async function loadBuilding() {
    if (!buildingId) return;
    try {
      building = await api.get<Building>(`/buildings/${buildingId}`);
    } catch (e) {
      console.error('Error loading building:', e);
    }
  }

  async function loadUnits() {
    try {
      loading = true;

      if (buildingId) {
        // Endpoint without pagination for building-specific units
        const response = await api.get<Unit[]>(`/buildings/${buildingId}/units`);
        units = response;
        totalItems = response.length;
        totalPages = 1;
        currentPage = 1;
      } else {
        // Paginated endpoint for all units
        const endpoint = `/units?page=${currentPage}&per_page=${perPage}`;
        const response = await api.get<PageResponse<Unit>>(endpoint);
        units = response.data;
        totalItems = response.pagination.total_items;
        totalPages = response.pagination.total_pages;
        currentPage = response.pagination.current_page;
        perPage = response.pagination.per_page;
      }

      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : $_('units.loadError');
      console.error('Error loading units:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadUnits();
  }

  function toggleUnitExpanded(unitId: string) {
    if (expandedUnits.has(unitId)) {
      expandedUnits.delete(unitId);
    } else {
      expandedUnits.add(unitId);
    }
    expandedUnits = expandedUnits; // Trigger reactivity
  }

  function getUnitTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'Apartment': $_('units.types.apartment'),
      'Parking': $_('units.types.parking'),
      'Cellar': $_('units.types.cellar')
    };
    return labels[type] || type;
  }

  function getUnitTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      'Apartment': '🏠',
      'Parking': '🚗',
      'Cellar': '📦'
    };
    return icons[type] || '📋';
  }

  function handleEditUnit(unit: Unit) {
    selectedUnit = unit;
    showEditModal = true;
  }

  function handleDeleteClick(unit: Unit) {
    unitToDelete = unit;
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    if (!unitToDelete) return;

    try {
      await api.delete(`/units/${unitToDelete.id}`);
      showDeleteConfirm = false;
      unitToDelete = null;
      await loadUnits();
    } catch (e) {
      error = e instanceof Error ? e.message : $_('units.deleteError');
      console.error('Error deleting unit:', e);
      showDeleteConfirm = false;
    }
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    unitToDelete = null;
  }

  // Calcul du total des tantièmes de tous les lots
  $: totalQuotas = units.reduce((sum, unit) => sum + (unit.quota || 0), 0);
  $: expectedTotal = building?.total_tantiemes || 1000;
  $: quotasMismatch = Math.abs(totalQuotas - expectedTotal) > 0.5;
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} lot{totalItems !== 1 ? 's' : ''}
    </p>
    {#if buildingId && isSuperAdmin}
      <Button variant="primary" on:click={() => showCreateModal = true}>
        + {$_('units.addUnit')}
      </Button>
    {/if}
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">{$_('common.loading')}</p>
  {:else if units.length === 0}
    <p class="text-center text-gray-600 py-8">
      {$_('units.noUnits')}
    </p>
  {:else}
    <div class="grid gap-4">
      {#each units as unit (unit.id)}
        <div class="bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition">
          <div class="p-4">
            <div class="flex justify-between items-start">
              <div class="flex items-start gap-3 flex-1">
                <span class="text-3xl">{getUnitTypeIcon(unit.unit_type)}</span>
                <div class="flex-1">
                  <h3 class="text-lg font-semibold text-gray-900">
                    {$_('units.lot')} {unit.unit_number}
                  </h3>
                  <p class="text-gray-600 text-sm mt-1">
                    {getUnitTypeLabel(unit.unit_type)} - {$_('units.floor')} {unit.floor}
                  </p>
                  <div class="flex gap-4 mt-2 text-sm text-gray-500">
                    <span>📐 {unit.surface_area} m²</span>
                    <span>🔢 {Math.round(unit.quota)}/{building?.total_tantiemes || 1000}èmes</span>
                  </div>
                </div>
              </div>
              <div class="flex gap-2 ml-4">
                {#if buildingId && isSuperAdmin}
                  <button
                    on:click={() => handleEditUnit(unit)}
                    class="px-3 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
                    title={$_('units.editUnit')}
                  >
                    ✏️
                  </button>
                  <button
                    on:click={() => handleDeleteClick(unit)}
                    class="px-3 py-2 text-sm font-medium text-white bg-red-600 rounded-lg hover:bg-red-700 transition"
                    title={$_('units.deleteUnit')}
                  >
                    🗑️
                  </button>
                {/if}
                <button
                  on:click={() => toggleUnitExpanded(unit.id)}
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
                  title={expandedUnits.has(unit.id) ? $_('units.hideOwners') : $_('units.showOwners')}
                >
                  {expandedUnits.has(unit.id) ? '▼' : '▶'} {$_('units.owners')}
                </button>
              </div>
            </div>
          </div>

          <!-- Expanded section showing owners -->
          {#if expandedUnits.has(unit.id)}
            <div class="border-t border-gray-200 bg-gray-50 p-4">
              <UnitOwners unitId={unit.id} />
            </div>
          {/if}
        </div>
      {/each}
    </div>

    <!-- Récapitulatif des tantièmes -->
    {#if building && units.length > 0}
      <div class="mt-4 p-3 bg-gray-50 border border-gray-200 rounded-lg">
        <div class="flex justify-between items-center">
          <span class="font-semibold text-gray-700">{$_('units.totalQuotas')}</span>
          <div class="text-right">
            <span class="text-xl font-bold" class:text-green-600={!quotasMismatch} class:text-red-600={quotasMismatch}>
              {Math.round(totalQuotas)}/{expectedTotal}èmes
            </span>
          </div>
        </div>
        {#if quotasMismatch}
          <p class="text-xs text-red-600 mt-1">
            {$_('units.quotasMismatch', { values: { current: Math.round(totalQuotas), expected: expectedTotal, diff: Math.round(totalQuotas - expectedTotal) } })}
          </p>
        {:else}
          <p class="text-xs text-green-600 mt-1">
            {$_('units.quotasCorrect')}
          </p>
        {/if}
      </div>
    {/if}

    {#if totalPages > 1}
      <Pagination
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        perPage={perPage}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}

  {#if buildingId && building}
    <UnitCreateModal
      bind:open={showCreateModal}
      buildingId={buildingId}
      organizationId={building.organization_id}
      totalTantiemes={building.total_tantiemes}
      on:created={loadUnits}
      on:close={() => showCreateModal = false}
    />

    <UnitEditModal
      bind:open={showEditModal}
      unit={selectedUnit}
      totalTantiemes={building.total_tantiemes}
      on:updated={loadUnits}
      on:close={() => {
        showEditModal = false;
        selectedUnit = null;
      }}
    />
  {/if}

  <!-- Delete Confirmation Modal -->
  {#if showDeleteConfirm && unitToDelete}
    <div class="fixed inset-0 z-50 overflow-y-auto">
      <div class="flex min-h-screen items-center justify-center p-4">
        <!-- Backdrop -->
        <div
          class="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
          on:click={cancelDelete}
        ></div>

        <!-- Modal -->
        <div class="relative bg-white rounded-lg shadow-xl max-w-md w-full p-6 z-10">
          <div class="mb-4">
            <h3 class="text-xl font-bold text-gray-900 mb-2">{$_('common.confirmDelete')}</h3>
            <p class="text-gray-600">
              {$_('units.confirmDeleteMessage')} <strong>{unitToDelete.unit_number}</strong> ?
            </p>
            <p class="text-sm text-red-600 mt-2">
              {$_('units.deleteWarning')}
            </p>
          </div>

          <div class="flex gap-2">
            <Button variant="danger" on:click={confirmDelete}>
              {$_('common.delete')}
            </Button>
            <Button variant="outline" on:click={cancelDelete}>
              {$_('common.cancel')}
            </Button>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
