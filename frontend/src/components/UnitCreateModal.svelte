<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
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
      error = 'Erreur lors du chargement des organisations';
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
      error = 'Erreur lors du chargement des immeubles';
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
      error = 'Veuillez sélectionner une organisation';
      return;
    }

    if (!finalBuildingId) {
      error = 'Veuillez sélectionner un immeuble';
      return;
    }

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
          <!-- Organization Selection (SuperAdmin only, when not provided) -->
          {#if needsSelection}
            <div>
              <label for="organizationSelect" class="block text-sm font-medium text-gray-700 mb-1">
                Organisation *
              </label>
              <select
                id="organizationSelect"
                bind:value={selectedOrganizationId}
                disabled={loadingOrgs}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="">-- Sélectionner une organisation --</option>
                {#each organizations as org}
                  <option value={org.id}>{org.name}</option>
                {/each}
              </select>
            </div>

            <!-- Building Selection (SuperAdmin only, when not provided) -->
            <div>
              <label for="buildingSelect" class="block text-sm font-medium text-gray-700 mb-1">
                Immeuble *
              </label>
              <select
                id="buildingSelect"
                bind:value={selectedBuildingId}
                disabled={!selectedOrganizationId || loadingBuildings}
                required
                class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
              >
                <option value="">-- Sélectionner un immeuble --</option>
                {#each buildings as building}
                  <option value={building.id}>{building.name} - {building.address}</option>
                {/each}
              </select>
              {#if loadingBuildings}
                <p class="text-xs text-gray-500 mt-1">Chargement des immeubles...</p>
              {/if}
            </div>
          {/if}

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
              <option value="Cellar">Cave</option>
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
