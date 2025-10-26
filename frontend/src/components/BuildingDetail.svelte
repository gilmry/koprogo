<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { Building } from '../lib/types';
  import BuildingForm from './admin/BuildingForm.svelte';
  import Button from './ui/Button.svelte';

  let building: Building | null = null;
  let loading = true;
  let error = '';
  let showEditModal = false;
  let buildingId: string = '';
  let organizationName: string = '';

  onMount(() => {
    // Get building ID from URL query params
    const urlParams = new URLSearchParams(window.location.search);
    buildingId = urlParams.get('id') || '';

    if (!buildingId) {
      error = 'ID de l\'immeuble manquant';
      loading = false;
      return;
    }

    loadBuilding();
  });

  async function loadBuilding() {
    try {
      loading = true;
      error = '';
      building = await api.get<Building>(`/buildings/${buildingId}`);

      // Load organization name
      if (building && building.organization_id) {
        try {
          const response = await api.get<{ data: any[] }>('/organizations?per_page=1000');
          const org = response.data.find((o: any) => o.id === building.organization_id);
          organizationName = org ? org.name : 'Organisation inconnue';
        } catch (e) {
          console.error('Error loading organization:', e);
          organizationName = 'Organisation inconnue';
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement de l\'immeuble';
      console.error('Error loading building:', e);
    } finally {
      loading = false;
    }
  }

  const handleEdit = () => {
    showEditModal = true;
  };

  const handleEditSuccess = async () => {
    showEditModal = false;
    await loadBuilding();
  };

  const handleGoBack = () => {
    window.history.back();
  };
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement...</p>
      </div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
      ⚠️ {error}
    </div>
    <div class="mt-4">
      <Button variant="outline" on:click={handleGoBack}>
        ← Retour
      </Button>
    </div>
  {:else if building}
    <!-- Header -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <button
            on:click={handleGoBack}
            class="text-gray-600 hover:text-gray-900"
          >
            ← Retour
          </button>
          <h1 class="text-3xl font-bold text-gray-900">{building.name}</h1>
        </div>
        <Button variant="primary" on:click={handleEdit}>
          ✏️ Modifier
        </Button>
      </div>
    </div>

    <!-- Building Info Card -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <h2 class="text-xl font-semibold text-white">Informations de l'immeuble</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Adresse</h3>
            <p class="text-lg text-gray-900">{building.address}</p>
            <p class="text-gray-600">{building.postal_code} {building.city}</p>
            <p class="text-gray-600">{building.country || 'Belgique'}</p>
          </div>
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Détails</h3>
            <div class="space-y-2">
              {#if organizationName}
                <div class="flex items-center">
                  <span class="text-gray-600">🏛️ Organisation:</span>
                  <span class="ml-2 font-semibold text-gray-900">{organizationName}</span>
                </div>
              {/if}
              <div class="flex items-center">
                <span class="text-gray-600">🏢 Nombre de lots:</span>
                <span class="ml-2 font-semibold text-gray-900">{building.total_units}</span>
              </div>
              {#if building.construction_year}
                <div class="flex items-center">
                  <span class="text-gray-600">🏗️ Année de construction:</span>
                  <span class="ml-2 font-semibold text-gray-900">{building.construction_year}</span>
                </div>
              {/if}
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Related Data Sections -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Units Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900">Lots</h3>
          <Button variant="outline" size="sm">
            ➕ Ajouter
          </Button>
        </div>
        <div class="text-center text-gray-500 py-8">
          <p>Les lots seront affichés ici</p>
          <p class="text-sm mt-2">(À implémenter)</p>
        </div>
      </div>

      <!-- Expenses Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900">Dépenses</h3>
          <Button variant="outline" size="sm">
            ➕ Ajouter
          </Button>
        </div>
        <div class="text-center text-gray-500 py-8">
          <p>Les dépenses seront affichées ici</p>
          <p class="text-sm mt-2">(À implémenter)</p>
        </div>
      </div>

      <!-- Meetings Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900">Assemblées Générales</h3>
          <Button variant="outline" size="sm">
            ➕ Planifier
          </Button>
        </div>
        <div class="text-center text-gray-500 py-8">
          <p>Les AG seront affichées ici</p>
          <p class="text-sm mt-2">(À implémenter)</p>
        </div>
      </div>

      <!-- Documents Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-4">
          <h3 class="text-lg font-semibold text-gray-900">Documents</h3>
          <Button variant="outline" size="sm">
            📎 Télécharger
          </Button>
        </div>
        <div class="text-center text-gray-500 py-8">
          <p>Les documents seront affichés ici</p>
          <p class="text-sm mt-2">(À implémenter)</p>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- Edit Modal -->
{#if building}
  <BuildingForm
    bind:isOpen={showEditModal}
    building={building}
    mode="edit"
    on:success={handleEditSuccess}
    on:close={() => showEditModal = false}
  />
{/if}
