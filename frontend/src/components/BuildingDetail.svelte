<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { Building } from '../lib/types';
  import BuildingForm from './admin/BuildingForm.svelte';
  import Button from './ui/Button.svelte';
  import UnitList from './UnitList.svelte';
  import ExpenseList from './ExpenseList.svelte';
  import MeetingList from './MeetingList.svelte';
  import DocumentList from './DocumentList.svelte';
  import BuildingFinancialReports from './BuildingFinancialReports.svelte';

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

      // Load organization name (only for SuperAdmin)
      if (building && building.organization_id) {
        try {
          const userInfo = await api.get<any>('/auth/me');
          if (userInfo.role === 'superadmin') {
            const response = await api.get<{ data: any[] }>('/organizations?per_page=1000');
            const org = response.data.find((o: any) => o.id === building.organization_id);
            organizationName = org ? org.name : 'Organisation inconnue';
          }
        } catch (e) {
          console.error('Error loading organization:', e);
          organizationName = '';
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
              <div class="flex items-center">
                <span class="text-gray-600">📊 Total tantièmes:</span>
                <span class="ml-2 font-semibold text-gray-900">{building.total_tantiemes} millièmes</span>
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
    <div class="space-y-8">
      <!-- Units Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Lots</h3>
        <UnitList buildingId={buildingId} />
      </div>

      <!-- Expenses Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Dépenses</h3>
        <ExpenseList buildingId={buildingId} />
      </div>

      <!-- Meetings Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Assemblées Générales</h3>
        <MeetingList buildingId={buildingId} />
      </div>

      <!-- Documents Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Documents</h3>
        <DocumentList buildingId={buildingId} />
      </div>

      <!-- Financial Reports Section -->
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">📊 Rapports Financiers PCMN</h3>
        <p class="text-sm text-gray-600 mb-4">
          Consultez les rapports financiers conformes au Plan Comptable Minimum Normalisé belge pour cet immeuble.
        </p>
        <BuildingFinancialReports buildingId={buildingId} buildingName={building.name} />
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
