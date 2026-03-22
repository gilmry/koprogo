<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { api } from '../lib/api';
  import type { Unit } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let unit: Unit | null = null;
  export let totalTantiemes: number = 1000;

  const dispatch = createEventDispatcher();

  let unitNumber = '';
  let floor = 0;
  let surfaceArea = 0;
  let quota = 0;
  let unitType: 'Apartment' | 'Parking' | 'Cellar' = 'Apartment';
  let loading = false;
  let error = '';

  // Initialize form when unit changes
  $: if (unit && open) {
    unitNumber = unit.unit_number;
    floor = unit.floor;
    surfaceArea = unit.surface_area;
    quota = unit.quota;
    unitType = unit.unit_type;
    error = '';
  }

  function handleClose() {
    dispatch('close');
  }

  async function handleSubmit() {
    error = '';

    if (!unitNumber.trim()) {
      error = $_('units.unit_number_required');
      return;
    }

    if (surfaceArea <= 0) {
      error = $_('units.surface_must_be_positive');
      return;
    }

    if (quota <= 0) {
      error = $_('units.quota_must_be_positive');
      return;
    }

    if (quota > totalTantiemes) {
      error = $_('units.quota_exceeds_max', { values: { max: totalTantiemes } });
      return;
    }

    if (!unit) {
      error = $_('units.no_unit_selected');
      return;
    }

    try {
      loading = true;

      await api.put(`/units/${unit.id}`, {
        unit_number: unitNumber.trim(),
        floor: floor,
        surface_area: surfaceArea,
        quota: quota,
        unit_type: unitType,
      });

      dispatch('updated');
      open = false;
    } catch (e) {
      error = e instanceof Error ? e.message : $_('units.error_updating_unit');
      console.error('Error updating unit:', e);
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
          <h2 class="text-xl font-bold text-gray-900">{$_('units.edit_unit')}</h2>
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
              {$_('units.unit_number')} *
            </label>
            <input
              id="unitNumber"
              type="text"
              bind:value={unitNumber}
              placeholder={$_('units.unit_number_example')}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Unit Type -->
          <div>
            <label for="unitType" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('units.unit_type')} *
            </label>
            <select
              id="unitType"
              bind:value={unitType}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            >
              <option value="Apartment">{$_('units.apartment')}</option>
              <option value="Parking">{$_('units.parking')}</option>
              <option value="Cellar">{$_('units.cellar')}</option>
            </select>
          </div>

          <!-- Floor -->
          <div>
            <label for="floor" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('units.floor')} *
            </label>
            <input
              id="floor"
              type="number"
              bind:value={floor}
              placeholder={$_('units.floor_example')}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Surface Area -->
          <div>
            <label for="surfaceArea" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('units.surface_area')} *
            </label>
            <input
              id="surfaceArea"
              type="number"
              step="0.01"
              min="0.01"
              bind:value={surfaceArea}
              placeholder={$_('units.surface_area_example')}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
          </div>

          <!-- Quota (Tantièmes) -->
          <div>
            <label for="quota" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('units.quota')} * <span class="text-sm text-gray-500">/ {totalTantiemes}</span>
            </label>
            <input
              id="quota"
              type="number"
              min="1"
              max={totalTantiemes}
              bind:value={quota}
              placeholder={$_('units.quota_example')}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
            />
            <p class="text-xs text-gray-500 mt-1">
              {$_('units.quota_description')}
            </p>
          </div>

          <!-- Actions -->
          <div class="flex gap-2 pt-4">
            <Button type="submit" variant="primary" disabled={loading}>
              {loading ? $_('common.updating') : $_('common.save')}
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
