<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Owner, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';
  import OwnerEditModal from './OwnerEditModal.svelte';
  import OwnerCreateModal from './OwnerCreateModal.svelte';
  import OwnerUnits from './OwnerUnits.svelte';

  let owners: Owner[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  // Modal state
  let isEditModalOpen = false;
  let isCreateModalOpen = false;
  let selectedOwner: Owner | null = null;

  // Expanded owners (to show units)
  let expandedOwners: Set<string> = new Set();

  function toggleOwnerExpanded(ownerId: string) {
    if (expandedOwners.has(ownerId)) {
      expandedOwners.delete(ownerId);
    } else {
      expandedOwners.add(ownerId);
    }
    expandedOwners = expandedOwners; // Trigger reactivity
  }

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
      totalItems = response.pagination.total_items;
      totalPages = response.pagination.total_pages;
      currentPage = response.pagination.current_page;
      perPage = response.pagination.per_page;
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

  function openEditModal(owner: Owner) {
    selectedOwner = owner;
    isEditModalOpen = true;
  }

  function openCreateModal() {
    isCreateModalOpen = true;
  }

  function closeEditModal() {
    isEditModalOpen = false;
    selectedOwner = null;
  }

  function closeCreateModal() {
    isCreateModalOpen = false;
  }

  async function handleOwnerSaved() {
    await loadOwners();
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} copropriétaire{totalItems !== 1 ? 's' : ''}
    </p>
    <button
      on:click={openCreateModal}
      class="px-4 py-2 text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition font-medium"
    >
      + Ajouter un copropriétaire
    </button>
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
        <div class="bg-white border border-gray-200 rounded-lg overflow-hidden hover:shadow-md transition">
          <div class="p-4">
            <div class="flex justify-between items-start">
              <div class="flex-1">
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
              <div class="flex gap-2 ml-4">
                <button
                  on:click={() => toggleOwnerExpanded(owner.id)}
                  class="px-3 py-2 text-sm font-medium text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition"
                  title={expandedOwners.has(owner.id) ? 'Masquer les lots' : 'Voir les lots'}
                >
                  {expandedOwners.has(owner.id) ? '▼' : '▶'} Lots
                </button>
                <button
                  on:click={() => openEditModal(owner)}
                  class="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
                >
                  Modifier
                </button>
              </div>
            </div>
          </div>

          <!-- Expanded section showing units -->
          {#if expandedOwners.has(owner.id)}
            <div class="border-t border-gray-200 bg-gray-50 p-4">
              <OwnerUnits ownerId={owner.id} />
            </div>
          {/if}
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

<!-- Owner Edit Modal -->
<OwnerEditModal
  owner={selectedOwner}
  isOpen={isEditModalOpen}
  on:close={closeEditModal}
  on:save={handleOwnerSaved}
/>

<!-- Owner Create Modal -->
<OwnerCreateModal
  isOpen={isCreateModalOpen}
  on:close={closeCreateModal}
  on:save={handleOwnerSaved}
/>
