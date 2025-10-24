<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Owner, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  let owners: Owner[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadOwners();
  });

  async function loadOwners() {
    try {
      loading = true;
      const response = await api.get<PageResponse<Owner>>(
        `/owners?page=${currentPage}&per_page=${perPage}`
      );

      owners = response.data;
      totalItems = response.total;
      totalPages = response.total_pages;
      currentPage = response.page;
      perPage = response.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des copropriétaires';
      console.error('Error loading owners:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadOwners();
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} copropriétaire{totalItems !== 1 ? 's' : ''}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if owners.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucun copropriétaire enregistré.
    </p>
  {:else}
    <div class="grid gap-4">
      {#each owners as owner}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div>
              <h3 class="text-lg font-semibold text-gray-900">
                {owner.first_name} {owner.last_name}
              </h3>
              <p class="text-gray-600 text-sm mt-1">
                📧 {owner.email}
              </p>
              {#if owner.phone}
                <p class="text-gray-500 text-sm">
                  📞 {owner.phone}
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
