<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import type { UnitOwner, Owner } from '../lib/types';
  import UnitOwnerEditModal from './UnitOwnerEditModal.svelte';
  import UnitOwnerAddModal from './UnitOwnerAddModal.svelte';
  import Button from './ui/Button.svelte';

  export let unitId: string;
  export let showHistory = false;

  $: canModifyOwnership = $authStore.user?.role === 'superadmin' || $authStore.user?.role === 'syndic';

  let unitOwners: (UnitOwner & { owner?: Owner })[] = [];
  let loading = true;
  let error = '';

  // Modal state
  let showEditModal = false;
  let showAddModal = false;
  let selectedUnitOwner: (UnitOwner & { owner?: Owner }) | null = null;
  let showDeleteConfirm = false;
  let unitOwnerToDelete: (UnitOwner & { owner?: Owner }) | null = null;

  onMount(async () => {
    await loadUnitOwners();
  });

  async function loadUnitOwners() {
    try {
      loading = true;
      const endpoint = showHistory
        ? `/units/${unitId}/owners/history`
        : `/units/${unitId}/owners`;

      const response = await api.get<UnitOwner[]>(endpoint);

      // Fetch owner details for each unit_owner
      unitOwners = await Promise.all(
        response.map(async (uo) => {
          try {
            const owner = await api.get<Owner>(`/owners/${uo.owner_id}`);
            return { ...uo, owner };
          } catch (e) {
            console.error(`Failed to load owner ${uo.owner_id}:`, e);
            return uo;
          }
        })
      );

      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des copropriétaires';
      console.error('Error loading unit owners:', e);
    } finally {
      loading = false;
    }
  }

  function formatPercentage(percentage: number): string {
    return `${(percentage * 100).toFixed(2)}%`;
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', {
      day: 'numeric',
      month: 'short',
      year: 'numeric'
    });
  }

  $: activeOwners = unitOwners.filter(uo => uo.is_active);
  $: inactiveOwners = unitOwners.filter(uo => !uo.is_active);
  $: totalPercentage = activeOwners.reduce((sum, uo) => sum + uo.ownership_percentage, 0);

  function handleEditUnitOwner(unitOwner: UnitOwner & { owner?: Owner }) {
    selectedUnitOwner = unitOwner;
    showEditModal = true;
  }

  function handleDeleteClick(unitOwner: UnitOwner & { owner?: Owner }) {
    unitOwnerToDelete = unitOwner;
    showDeleteConfirm = true;
  }

  async function confirmDelete() {
    if (!unitOwnerToDelete) return;

    try {
      await api.delete(`/units/${unitId}/owners/${unitOwnerToDelete.owner_id}`);
      showDeleteConfirm = false;
      unitOwnerToDelete = null;
      await loadUnitOwners();
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de la suppression de la relation';
      console.error('Error deleting unit owner:', e);
      showDeleteConfirm = false;
    }
  }

  function cancelDelete() {
    showDeleteConfirm = false;
    unitOwnerToDelete = null;
  }
</script>

<div class="space-y-4">
  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-4">Chargement...</p>
  {:else}
    <!-- Active Owners -->
    {#if activeOwners.length > 0}
      <div class="space-y-2">
        <div class="flex justify-between items-center">
          <h4 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
            Copropriétaires actuels
          </h4>
          {#if canModifyOwnership}
            <button
              on:click={() => showAddModal = true}
              class="px-3 py-1 text-xs font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
            >
              + Ajouter
            </button>
          {/if}
        </div>

        {#each activeOwners as unitOwner (unitOwner.id)}
          <div class="bg-white border border-gray-200 rounded-lg p-3">
            <div class="flex justify-between items-start">
              <div class="flex-1">
                {#if unitOwner.owner}
                  <div class="flex items-center gap-2">
                    <h5 class="font-semibold text-gray-900">
                      {unitOwner.owner.first_name} {unitOwner.owner.last_name}
                    </h5>
                    {#if unitOwner.is_primary_contact}
                      <span class="px-2 py-0.5 text-xs font-medium bg-primary-100 text-primary-800 rounded-full">
                        Contact principal
                      </span>
                    {/if}
                  </div>
                  <p class="text-sm text-gray-600 mt-1">
                    📧 {unitOwner.owner.email}
                  </p>
                  {#if unitOwner.owner.phone}
                    <p class="text-sm text-gray-500">
                      📞 {unitOwner.owner.phone}
                    </p>
                  {/if}
                {:else}
                  <p class="text-gray-500 italic">Chargement des détails...</p>
                {/if}
                <p class="text-xs text-gray-500 mt-1">
                  Depuis le {formatDate(unitOwner.start_date)}
                </p>
              </div>

              <div class="ml-2 flex items-center gap-1 sm:gap-2">
                <div class="text-right">
                  <p class="text-lg sm:text-xl font-bold text-primary-600">
                    {formatPercentage(unitOwner.ownership_percentage)}
                  </p>
                  <p class="text-xs text-gray-500 hidden sm:block">Quote-part</p>
                </div>
                {#if canModifyOwnership}
                  <button
                    on:click={() => handleEditUnitOwner(unitOwner)}
                    class="px-2 py-1.5 text-sm font-medium text-white bg-primary-600 rounded hover:bg-primary-700 transition"
                    title="Modifier la quote-part"
                  >
                    ✏️
                  </button>
                  <button
                    on:click={() => handleDeleteClick(unitOwner)}
                    class="px-2 py-1.5 text-sm font-medium text-white bg-red-600 rounded hover:bg-red-700 transition"
                    title="Retirer le copropriétaire"
                  >
                    🗑️
                  </button>
                {/if}
              </div>
            </div>
          </div>
        {/each}

        <!-- Total -->
        <div class="mt-2 p-3 bg-gray-50 border border-gray-200 rounded-lg">
          <div class="flex justify-between items-center">
            <span class="font-semibold text-gray-700">Total</span>
            <span class="text-xl font-bold" class:text-green-600={totalPercentage === 1} class:text-red-600={totalPercentage !== 1}>
              {formatPercentage(totalPercentage)}
            </span>
          </div>
          {#if totalPercentage !== 1}
            <p class="text-xs text-red-600 mt-1">
              ⚠️ La somme des quotes-parts devrait être 100%
            </p>
          {/if}
        </div>
      </div>
    {:else}
      <div class="text-center py-4">
        <p class="text-gray-600 mb-3">
          Aucun copropriétaire actif
        </p>
        {#if canModifyOwnership}
          <button
            on:click={() => showAddModal = true}
            class="px-4 py-2 text-sm font-medium text-white bg-primary-600 rounded-lg hover:bg-primary-700 transition"
          >
            + Ajouter un copropriétaire
          </button>
        {/if}
      </div>
    {/if}

    <!-- Historical Owners (if showHistory is true) -->
    {#if showHistory && inactiveOwners.length > 0}
      <div class="space-y-2 mt-6">
        <h4 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
          Historique
        </h4>

        {#each inactiveOwners as unitOwner (unitOwner.id)}
          <div class="bg-gray-50 border border-gray-200 rounded-lg p-3 opacity-75">
            <div class="flex justify-between items-start">
              <div class="flex-1">
                {#if unitOwner.owner}
                  <h5 class="font-medium text-gray-700">
                    {unitOwner.owner.first_name} {unitOwner.owner.last_name}
                  </h5>
                  <p class="text-sm text-gray-600 mt-1">
                    📧 {unitOwner.owner.email}
                  </p>
                {:else}
                  <p class="text-gray-500 italic">Chargement des détails...</p>
                {/if}
                <p class="text-xs text-gray-500 mt-1">
                  {formatDate(unitOwner.start_date)} → {unitOwner.end_date ? formatDate(unitOwner.end_date) : 'En cours'}
                </p>
              </div>

              <div class="ml-4 text-right">
                <p class="text-lg font-semibold text-gray-600">
                  {formatPercentage(unitOwner.ownership_percentage)}
                </p>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>

<!-- Edit Modal -->
<UnitOwnerEditModal
  bind:open={showEditModal}
  unitOwner={selectedUnitOwner}
  on:updated={loadUnitOwners}
  on:close={() => {
    showEditModal = false;
    selectedUnitOwner = null;
  }}
/>

<!-- Add Owner Modal -->
<UnitOwnerAddModal
  bind:open={showAddModal}
  unitId={unitId}
  on:added={loadUnitOwners}
  on:close={() => showAddModal = false}
/>

<!-- Delete Confirmation Modal -->
{#if showDeleteConfirm && unitOwnerToDelete}
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
          <h3 class="text-xl font-bold text-gray-900 mb-2">Retirer le copropriétaire</h3>
          {#if unitOwnerToDelete.owner}
            <p class="text-gray-600">
              Êtes-vous sûr de vouloir retirer
              <strong>{unitOwnerToDelete.owner.first_name} {unitOwnerToDelete.owner.last_name}</strong>
              de ce lot ?
            </p>
          {:else}
            <p class="text-gray-600">
              Êtes-vous sûr de vouloir retirer ce copropriétaire du lot ?
            </p>
          {/if}
          <p class="text-sm text-gray-500 mt-2">
            Cette action marquera la relation comme inactive et enregistrera la date de fin.
          </p>
        </div>

        <div class="flex gap-2">
          <Button variant="danger" on:click={confirmDelete}>
            Retirer
          </Button>
          <Button variant="outline" on:click={cancelDelete}>
            Annuler
          </Button>
        </div>
      </div>
    </div>
  </div>
{/if}
