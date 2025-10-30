<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api';
  import type { UnitOwner, Owner } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let unitOwner: (UnitOwner & { owner?: Owner }) | null = null;

  const dispatch = createEventDispatcher();

  let ownershipPercentage = 0;
  let isPrimaryContact = false;
  let loading = false;
  let error = '';

  // Initialize form when unitOwner changes
  $: if (unitOwner && open) {
    ownershipPercentage = unitOwner.ownership_percentage * 100; // Convert to percentage
    isPrimaryContact = unitOwner.is_primary_contact;
    error = '';
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleSubmit() {
    error = '';

    if (ownershipPercentage <= 0 || ownershipPercentage > 100) {
      error = 'Le pourcentage doit être entre 0.01% et 100%';
      return;
    }

    if (!unitOwner) {
      error = 'Aucune relation sélectionnée';
      return;
    }

    try {
      loading = true;

      // Convert percentage back to decimal (0.0 - 1.0)
      const percentageDecimal = ownershipPercentage / 100;

      await api.put(`/unit-owners/${unitOwner.id}`, {
        ownership_percentage: percentageDecimal,
        is_primary_contact: isPrimaryContact,
      });

      dispatch('updated');
      open = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de la modification';
      console.error('Error updating unit owner:', e);
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
      ></div>

      <!-- Modal -->
      <div class="relative bg-white rounded-lg shadow-xl max-w-md w-full p-6 z-10">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-bold text-gray-900">Modifier la quote-part</h2>
          <button
            on:click={handleClose}
            class="text-gray-400 hover:text-gray-500"
          >
            <span class="text-2xl">&times;</span>
          </button>
        </div>

        {#if unitOwner?.owner}
          <div class="mb-4 p-3 bg-gray-50 rounded-lg">
            <p class="font-semibold text-gray-900">
              {unitOwner.owner.first_name} {unitOwner.owner.last_name}
            </p>
            <p class="text-sm text-gray-600">{unitOwner.owner.email}</p>
          </div>
        {/if}

        {#if error}
          <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
            {error}
          </div>
        {/if}

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
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
            <Button type="submit" variant="primary" disabled={loading}>
              {loading ? 'Enregistrement...' : 'Enregistrer'}
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
