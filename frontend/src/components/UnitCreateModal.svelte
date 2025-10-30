<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { api } from '../lib/api';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let buildingId: string;
  export let organizationId: string;
  export let totalTantiemes: number = 1000;

  const dispatch = createEventDispatcher();

  let unitNumber = '';
  let floor = 0;
  let surfaceArea = 0;
  let quota = 0;
  let unitType: 'Apartment' | 'Parking' | 'Storage' = 'Apartment';
  let loading = false;
  let error = '';

  function resetForm() {
    unitNumber = '';
    floor = 0;
    surfaceArea = 0;
    quota = 0;
    unitType = 'Apartment';
    error = '';
  }

  function handleClose() {
    resetForm();
    dispatch('close');
  }

  async function handleSubmit() {
    error = '';

    if (!unitNumber.trim()) {
      error = 'Le numéro de lot est obligatoire';
      return;
    }

    if (surfaceArea <= 0) {
      error = 'La surface doit être supérieure à 0';
      return;
    }

    if (quota <= 0) {
      error = 'La quote-part doit être supérieure à 0';
      return;
    }

    if (quota > totalTantiemes) {
      error = `La quote-part ne peut pas dépasser ${totalTantiemes} millièmes`;
      return;
    }

    try {
      loading = true;

      await api.post('/units', {
        organization_id: organizationId,
        building_id: buildingId,
        unit_number: unitNumber.trim(),
        floor: floor,
        surface_area: surfaceArea,
        quota: quota,
        unit_type: unitType,
      });

      dispatch('created');
      resetForm();
      open = false;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors de la création du lot';
      console.error('Error creating unit:', e);
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
      <div class="relative bg-white rounded-lg shadow-xl max-w-lg w-full p-6 z-10">
        <div class="flex justify-between items-center mb-4">
          <h2 class="text-xl font-bold text-gray-900">Ajouter un lot</h2>
          <button
            on:click={handleClose}
            class="text-gray-400 hover:text-gray-500"
          >
            <span class="text-2xl">&times;</span>
          </button>
        </div>

        {#if error}
          <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-4">
            {error}
          </div>
        {/if}

        <form on:submit|preventDefault={handleSubmit} class="space-y-4">
          <!-- Unit Number -->
          <div>
            <label for="unitNumber" class="block text-sm font-medium text-gray-700 mb-1">
              Numéro de lot *
            </label>
            <input
              id="unitNumber"
              type="text"
              bind:value={unitNumber}
              placeholder="Ex: 101, A, B3, etc."
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Unit Type -->
          <div>
            <label for="unitType" class="block text-sm font-medium text-gray-700 mb-1">
              Type de lot *
            </label>
            <select
              id="unitType"
              bind:value={unitType}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            >
              <option value="Apartment">Appartement</option>
              <option value="Parking">Parking</option>
              <option value="Storage">Cave/Storage</option>
            </select>
          </div>

          <!-- Floor -->
          <div>
            <label for="floor" class="block text-sm font-medium text-gray-700 mb-1">
              Étage *
            </label>
            <input
              id="floor"
              type="number"
              bind:value={floor}
              placeholder="Ex: 0 (RDC), 1, 2, -1 (sous-sol)"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Surface Area -->
          <div>
            <label for="surfaceArea" class="block text-sm font-medium text-gray-700 mb-1">
              Surface (m²) *
            </label>
            <input
              id="surfaceArea"
              type="number"
              step="0.01"
              min="0.01"
              bind:value={surfaceArea}
              placeholder="Ex: 75.50"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Quota (Tantièmes) -->
          <div>
            <label for="quota" class="block text-sm font-medium text-gray-700 mb-1">
              Quote-part (millièmes) * <span class="text-sm text-gray-500">/ {totalTantiemes}</span>
            </label>
            <input
              id="quota"
              type="number"
              min="1"
              max={totalTantiemes}
              bind:value={quota}
              placeholder="Ex: 350"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
            <p class="text-xs text-gray-500 mt-1">
              Quote-part du lot dans l'immeuble (en millièmes)
            </p>
          </div>

          <!-- Actions -->
          <div class="flex gap-2 pt-4">
            <Button type="submit" variant="primary" disabled={loading}>
              {loading ? 'Création...' : 'Créer le lot'}
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
