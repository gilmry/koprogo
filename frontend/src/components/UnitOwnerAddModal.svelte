<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import type { Owner, PageResponse } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let unitId: string;
  export let currentTotalPercentage = 0; // Total actuel des quotes-parts (0.0 - 1.0)

  const dispatch = createEventDispatcher();

  let owners: Owner[] = [];
  let loadingOwners = false;
  let selectedOwnerId = '';
  let ownershipPercentage = 0;
  let isPrimaryContact = false;
  let loading = false;
  let error = '';
  let searchQuery = '';

  // Calcul du pourcentage disponible
  $: availablePercentage = Math.max(0, (1 - currentTotalPercentage) * 100);
  $: wouldExceed = ownershipPercentage > 0 && ownershipPercentage > availablePercentage + 0.01;
  $: isSubmitDisabled = loading || loadingOwners || wouldExceed || ownershipPercentage <= 0 || !selectedOwnerId;

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
      error = $_('common.error_loading');
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
      error = $_('units.select_owner');
      return;
    }

    if (ownershipPercentage <= 0 || ownershipPercentage > 100) {
      error = $_('units.percentage_must_be_valid');
      return;
    }

    if (ownershipPercentage > availablePercentage + 0.01) {
      error = $_('units.quota_would_exceed', { values: { available: availablePercentage.toFixed(2) } });
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
      error = e instanceof Error ? e.message : $_('units.error_adding_owner');
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
          <h2 id="add-owner-title" class="text-xl font-bold text-gray-900">{$_('units.add_owner')}</h2>
          <button
            on:click={handleClose}
            class="text-gray-400 hover:text-gray-500"
            aria-label={$_('common.close')}
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
              {$_('units.owner')} *
            </label>
            {#if loadingOwners}
              <p class="text-sm text-gray-500">{$_('common.loading')}</p>
            {:else}
              <!-- Search field -->
              <label for="owner-search" class="sr-only">{$_('units.search_owner')}</label>
              <input
                id="owner-search"
                type="text"
                bind:value={searchQuery}
                placeholder={$_('units.search_owner_placeholder')}
                class="w-full px-3 py-2 mb-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              />

              <select
                id="ownerId"
                bind:value={selectedOwnerId}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
                size="5"
              >
                <option value="">{$_('units.select_owner_option')}</option>
                {#each filteredOwners as owner (owner.id)}
                  <option value={owner.id}>
                    {owner.first_name} {owner.last_name} ({owner.email})
                  </option>
                {/each}
              </select>
              {#if filteredOwners.length === 0 && searchQuery}
                <p class="text-xs text-gray-500 mt-1">
                  {$_('units.no_owner_found', { values: { query: searchQuery } })}
                </p>
              {:else}
                <p class="text-xs text-gray-500 mt-1">
                  {$_('units.owners_found', { values: { count: filteredOwners.length } })}
                </p>
              {/if}
            {/if}
            <p class="text-xs text-gray-500 mt-1">
              {$_('units.owner_selection_help')}
            </p>
          </div>

          <!-- Ownership Percentage -->
          <div>
            <label for="ownershipPercentage" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('units.ownership_percentage')} *
            </label>
            <input
              id="ownershipPercentage"
              type="number"
              step="0.01"
              min="0.01"
              max={availablePercentage > 0 ? availablePercentage : 100}
              bind:value={ownershipPercentage}
              placeholder="Ex: 50.00"
              required
              class="w-full px-3 py-2 border rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              class:border-red-500={wouldExceed}
              class:border-gray-300={!wouldExceed}
            />
            <div class="flex justify-between items-center mt-1">
              <p class="text-xs text-gray-500">
                {$_('units.quota_sum_100')}
              </p>
              <p class="text-xs font-semibold" class:text-green-600={availablePercentage > 0} class:text-red-600={availablePercentage <= 0}>
                {$_('units.available', { values: { pct: availablePercentage.toFixed(2) } })}
              </p>
            </div>
            {#if wouldExceed}
              <p class="text-xs text-red-600 mt-1 font-medium">
                {$_('units.quota_would_exceed_detail', { values: { current: (currentTotalPercentage * 100).toFixed(2), added: ownershipPercentage.toFixed(2), total: ((currentTotalPercentage * 100) + ownershipPercentage).toFixed(2) } })}
              </p>
            {/if}
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
              {$_('units.primary_contact')}
            </label>
          </div>
          <p class="text-xs text-gray-500 -mt-2 ml-6">
            {$_('units.primary_contact_help')}
          </p>

          <!-- Actions -->
          <div class="flex gap-2 pt-4">
            <Button type="submit" variant="primary" disabled={isSubmitDisabled}>
              {loading ? $_('common.adding') : $_('common.add')}
            </Button>
            <Button type="button" variant="outline" on:click={handleClose}>
              {$_('common.cancel')}
            </Button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}
