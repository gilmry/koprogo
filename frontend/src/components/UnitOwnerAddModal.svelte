<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Owner, PageResponse } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let unitId: string;

  const dispatch = createEventDispatcher();

  let owners: Owner[] = [];
  let loadingOwners = false;
  let selectedOwnerId = '';
  let ownershipPercentage = 0;
  let isPrimaryContact = false;
  let loading = false;
  let error = '';
  let searchQuery = '';

  // Load all owners when modal opens
  $: if (open) {
    loadOwners();
  }

  async function loadOwners() {
    try {
      loadingOwners = true;
      const response = await api.get<PageResponse<Owner>>('/owners?per_page=100');
      owners = response.data;
    } catch (e) {
      console.error('Error loading owners:', e);
      error = 'Erreur lors du chargement des copropriétaires';
    } finally {
      loadingOwners = false;
    }
  }

  function handleClose() {
    resetForm();
    dispatch('close');
  }

  function resetForm() {
    selectedOwnerId = '';
    ownershipPercentage = 0;
    isPrimaryContact = false;
    error = '';
    searchQuery = '';
  }

  // Filter owners based on search query
  $: filteredOwners = owners.filter(owner => {
    if (!searchQuery.trim()) return true;
    const query = searchQuery.toLowerCase();
    return (
      owner.first_name.toLowerCase().includes(query) ||
      owner.last_name.toLowerCase().includes(query) ||
      owner.email.toLowerCase().includes(query)
    );
  });

  async function handleSubmit() {
    error = '';

    if (!selectedOwnerId) {
      error = 'Veuillez sélectionner un copropriétaire';
      return;
    }

    if (ownershipPercentage <= 0 || ownershipPercentage > 100) {
      error = 'Le pourcentage doit être entre 0.01% et 100%';
      return;
    }

    try {
      loading = true;

      // Convert percentage to decimal (0.0 - 1.0)
      const percentageDecimal = ownershipPercentage / 100;

      await api.post(`/units/${unitId}/owners`, {
        owner_id: selectedOwnerId,
        ownership_percentage: percentageDecimal,
        is_primary_contact: isPrimaryContact,
      });

      dispatch('added');
      resetForm();
      open = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de l\'ajout du copropriétaire';
      console.error('Error adding owner to unit:', e);
    } finally {
      loading = false;
    }
  }
</script>

{#if open}
  <div class="fixed inset-0 z-50 overflow-y-auto">
    <div class="flex min-h-screen items-center justify-center p-4">
      <!-- Backdrop -->
      <div
        class="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
        on:click={handleClose}
        aria-hidden="true"
      ></div>

      <!-- Modal -->
      <div class="relative bg-white rounded-lg shadow-xl max-w-md w-full p-6 z-10" role="dialog" aria-modal="true" aria-labelledby="add-owner-title">
        <div class="flex justify-between items-center mb-4">
          <h2 id="add-owner-title" class="text-xl font-bold text-gray-900">Ajouter un copropriétaire</h2>
          <button
            on:click={handleClose}
            class="text-gray-400 hover:text-gray-500"
            aria-label="Fermer"
          >
            <span class="text-2xl" aria-hidden="true">&times;</span>
          </button>
        </div>

        {#if error}
          <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
            {error}
          </div>
        {/if}

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
          <!-- Owner Selection -->
          <div>
            <label for="ownerId" class="block text-sm font-medium text-gray-700 mb-1">
              Copropriétaire *
            </label>
            {#if loadingOwners}
              <p class="text-sm text-gray-500">Chargement...</p>
            {:else}
              <!-- Search field -->
              <label for="owner-search" class="sr-only">Rechercher par nom ou email</label>
              <input
                id="owner-search"
                type="text"
                bind:value={searchQuery}
                placeholder="Rechercher par nom ou email..."
                class="w-full px-3 py-2 mb-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />

              <select
                id="ownerId"
                bind:value={selectedOwnerId}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                size="5"
              >
                <option value="">-- Sélectionner un copropriétaire --</option>
                {#each filteredOwners as owner (owner.id)}
                  <option value={owner.id}>
                    {owner.first_name} {owner.last_name} ({owner.email})
                  </option>
                {/each}
              </select>
              {#if filteredOwners.length === 0 && searchQuery}
                <p class="text-xs text-gray-500 mt-1">
                  Aucun copropriétaire trouvé pour "{searchQuery}"
                </p>
              {:else}
                <p class="text-xs text-gray-500 mt-1">
                  {filteredOwners.length} copropriétaire{filteredOwners.length !== 1 ? 's' : ''} trouvé{filteredOwners.length !== 1 ? 's' : ''}
                </p>
              {/if}
            {/if}
            <p class="text-xs text-gray-500 mt-1">
              Choisissez un copropriétaire existant ou créez-en un nouveau depuis la page des copropriétaires
            </p>
          </div>

          <!-- Ownership Percentage -->
          <div>
            <label for="ownershipPercentage" class="block text-sm font-medium text-gray-700 mb-1">
              Quote-part de propriété (%) *
            </label>
            <input
              id="ownershipPercentage"
              type="number"
              step="0.01"
              min="0.01"
              max="100"
              bind:value={ownershipPercentage}
              placeholder="Ex: 50.00"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
            <p class="text-xs text-gray-500 mt-1">
              Pourcentage de propriété de ce copropriétaire sur ce lot (la somme de tous les copropriétaires doit faire 100%)
            </p>
          </div>

          <!-- Primary Contact -->
          <div class="flex items-center">
            <input
              id="isPrimaryContact"
              type="checkbox"
              bind:checked={isPrimaryContact}
              class="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
            />
            <label for="isPrimaryContact" class="ml-2 block text-sm text-gray-700">
              Contact principal pour ce lot
            </label>
          </div>
          <p class="text-xs text-gray-500 -mt-2 ml-6">
            Le contact principal reçoit toutes les communications concernant ce lot
          </p>

          <!-- Actions -->
          <div class="flex gap-2 pt-4">
            <Button type="submit" variant="primary" disabled={loading || loadingOwners}>
              {loading ? 'Ajout...' : 'Ajouter'}
            </Button>
            <Button type="button" variant="outline" on:click={handleClose}>
              Annuler
            </Button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}
