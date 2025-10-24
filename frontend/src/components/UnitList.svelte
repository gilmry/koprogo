<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Unit, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  let units: Unit[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadUnits();
  });

  async function loadUnits() {
    try {
      loading = true;
      const response = await api.get<PageResponse<Unit>>(
        `/units?page=${currentPage}&per_page=${perPage}`
      );

      units = response.data;
      totalItems = response.total;
      totalPages = response.total_pages;
      currentPage = response.page;
      perPage = response.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des lots';
      console.error('Error loading units:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadUnits();
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} lot{totalItems !== 1 ? 's' : ''}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if units.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucun lot enregistré.
    </p>
  {:else}
    <div class="grid gap-4">
      {#each units as unit}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div>
              <h3 class="text-lg font-semibold text-gray-900">
                Lot {unit.unit_number}
              </h3>
              <p class="text-gray-600 text-sm mt-1">
                🏢 Étage {unit.floor} · {unit.unit_type}
              </p>
              <p class="text-gray-500 text-sm">
                📐 {unit.surface_area}m² · Quote-part: {unit.ownership_share}%
              </p>
              {#if unit.owner_id}
                <p class="text-green-600 text-sm font-medium mt-1">
                  ✓ Propriétaire assigné
                </p>
              {:else}
                <p class="text-gray-400 text-sm mt-1">
                  ○ Non assigné
                </p>
              {/if}
            </div>
            <button class="text-primary-600 hover:text-primary-700 text-sm font-medium">
              Détails →
            </button>
          </div>
        </div>
      {/each}
    </div>

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
</div>
