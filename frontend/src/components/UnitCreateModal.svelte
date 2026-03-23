<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { authStore } from '../stores/auth';
  import type { Organization, Building } from '../lib/types';
  import Button from './ui/Button.svelte';

  export let open = false;
  export let buildingId: string | undefined = undefined;
  export let organizationId: string | undefined = undefined;
  export let totalTantiemes: number = 1000;

  const dispatch = createEventDispatcher();

  let unitNumber = '';
  let floor = 0;
  let surfaceArea = 0;
  let quota = 0;
  let unitType: 'Apartment' | 'Parking' | 'Cellar' = 'Apartment';
  let loading = false;
  let error = '';

  // For SuperAdmin: selectable organization and building
  let organizations: Organization[] = [];
  let buildings: Building[] = [];
  let selectedOrganizationId: string = organizationId || '';
  let selectedBuildingId: string = buildingId || '';
  let loadingOrgs = false;
  let loadingBuildings = false;

  $: isSuperAdmin = $authStore.user?.role === 'superadmin';
  $: needsSelection = isSuperAdmin && (!buildingId || !organizationId);

  // Load organizations for SuperAdmin
  onMount(async () => {
    if (isSuperAdmin && !organizationId) {
      await loadOrganizations();
    }
  });

  async function loadOrganizations() {
    try {
      loadingOrgs = true;
      const response = await api.get<Organization[]>('/organizations');
      organizations = response;
    } catch (e) {
      console.error('Error loading organizations:', e);
      error = $_('common.error_loading');
    } finally {
      loadingOrgs = false;
    }
  }

  async function loadBuildings(orgId: string) {
    try {
      loadingBuildings = true;
      buildings = [];
      selectedBuildingId = '';
      const response = await api.get<Building[]>(`/buildings?organization_id=${orgId}`);
      buildings = response;
    } catch (e) {
      console.error('Error loading buildings:', e);
      error = $_('common.error_loading_buildings');
    } finally {
      loadingBuildings = false;
    }
  }

  $: if (selectedOrganizationId && isSuperAdmin && !organizationId) {
    loadBuildings(selectedOrganizationId);
  }

  $: if (selectedBuildingId && buildings.length > 0) {
    const building = buildings.find(b => b.id === selectedBuildingId);
    if (building && building.total_tantiemes) {
      totalTantiemes = building.total_tantiemes;
    }
  }

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

    // Determine which IDs to use
    const finalOrganizationId = organizationId || selectedOrganizationId;
    const finalBuildingId = buildingId || selectedBuildingId;

    if (!finalOrganizationId) {
      error = $_('units.select_organization');
      return;
    }

    if (!finalBuildingId) {
      error = $_('units.select_building');
      return;
    }

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

    try {
      loading = true;

      await api.post('/units', {
        organization_id: finalOrganizationId,
        building_id: finalBuildingId,
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
      error = e instanceof Error ? e.message : $_('units.error_creating_unit');
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
          <h2 class="text-xl font-bold text-gray-900">{$_('units.add_unit')}</h2>
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
          <!-- Organization Selection (SuperAdmin only, when not provided) -->
          {#if needsSelection}
            <div>
              <label for="organizationSelect" class="block text-sm font-medium text-gray-700 mb-1">
                {$_('units.organization')} *
              </label>
              <select
                id="organizationSelect"
                bind:value={selectedOrganizationId}
                disabled={loadingOrgs}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="">{$_('common.select_organization')}</option>
                {#each organizations as org}
                  <option value={org.id}>{org.name}</option>
                {/each}
              </select>
            </div>

            <!-- Building Selection (SuperAdmin only, when not provided) -->
            <div>
              <label for="buildingSelect" class="block text-sm font-medium text-gray-700 mb-1">
                {$_('units.building')} *
              </label>
              <select
                id="buildingSelect"
                bind:value={selectedBuildingId}
                disabled={!selectedOrganizationId || loadingBuildings}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="">{$_('common.select_building')}</option>
                {#each buildings as building}
                  <option value={building.id}>{building.name} - {building.address}</option>
                {/each}
              </select>
              {#if loadingBuildings}
                <p class="text-xs text-gray-500 mt-1">{$_('common.loading_buildings')}</p>
              {/if}
            </div>
          {/if}

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
              {loading ? $_('common.creating') : $_('units.create_unit')}
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
