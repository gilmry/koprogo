<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Building, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  let buildings: Building[] = [];
  let loading = true;
  let error = '';
  let showForm = false;

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  let newBuilding = {
    name: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Belgique',
    total_units: 0,
    construction_year: null as number | null,
    organization_id: '' // Will be overridden by backend from JWT
  };

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    try {
      loading = true;
      // Use the new paginated endpoint with JWT authentication
      const response = await api.get<PageResponse<Building>>(
        `/buildings?page=${currentPage}&per_page=${perPage}`
      );

      buildings = response.data;
      totalItems = response.pagination.total_items;
      totalPages = response.pagination.total_pages;
      currentPage = response.pagination.current_page;
      perPage = response.pagination.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des immeubles';
      console.error('Error loading buildings:', e);
    } finally {
      loading = false;
    }
  }

  async function createBuilding(e: Event) {
    e.preventDefault();
    try {
      // Use api.post with JWT authentication
      // Note: organization_id will be overridden by backend from JWT token (secure!)
      await api.post('/buildings', newBuilding);

      await loadBuildings();
      showForm = false;
      resetForm();
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Échec de la création de l\'immeuble';
      console.error('Error creating building:', e);
    }
  }

  function resetForm() {
    newBuilding = {
      name: '',
      address: '',
      city: '',
      postal_code: '',
      country: 'Belgique',
      total_units: 0,
      construction_year: null,
      organization_id: ''
    };
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadBuildings();
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} immeuble{totalItems !== 1 ? 's' : ''}
    </p>
    <button
      on:click={() => showForm = !showForm}
      class="bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700 transition"
    >
      {showForm ? 'Annuler' : 'Nouvel Immeuble'}
    </button>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if showForm}
    <form on:submit={createBuilding} class="bg-gray-50 p-6 rounded-lg space-y-4">
      <div class="grid grid-cols-2 gap-4">
        <div>
          <label for="building-name" class="block text-sm font-medium text-gray-700 mb-1">
            Nom de l'immeuble *
          </label>
          <input
            id="building-name"
            type="text"
            bind:value={newBuilding.name}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
        <div>
          <label for="building-units" class="block text-sm font-medium text-gray-700 mb-1">
            Nombre de lots *
          </label>
          <input
            id="building-units"
            type="number"
            bind:value={newBuilding.total_units}
            required
            min="1"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
      </div>

      <div>
        <label for="building-address" class="block text-sm font-medium text-gray-700 mb-1">
          Adresse *
        </label>
        <input
          id="building-address"
          type="text"
          bind:value={newBuilding.address}
          required
          class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
        />
      </div>

      <div class="grid grid-cols-3 gap-4">
        <div>
          <label for="building-city" class="block text-sm font-medium text-gray-700 mb-1">
            Ville *
          </label>
          <input
            id="building-city"
            type="text"
            bind:value={newBuilding.city}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
        <div>
          <label for="building-postal" class="block text-sm font-medium text-gray-700 mb-1">
            Code postal *
          </label>
          <input
            id="building-postal"
            type="text"
            bind:value={newBuilding.postal_code}
            required
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
        <div>
          <label for="building-year" class="block text-sm font-medium text-gray-700 mb-1">
            Année de construction
          </label>
          <input
            id="building-year"
            type="number"
            bind:value={newBuilding.construction_year}
            min="1800"
            max="2100"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
      </div>

      <button
        type="submit"
        class="w-full bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700 transition"
      >
        Créer l'immeuble
      </button>
    </form>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if buildings.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucun immeuble enregistré. Créez-en un pour commencer !
    </p>
  {:else}
    <div class="grid gap-4">
      {#each buildings as building}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div>
              <h3 class="text-lg font-semibold text-gray-900">{building.name}</h3>
              <p class="text-gray-600 text-sm mt-1">
                📍 {building.address}, {building.postal_code} {building.city}
              </p>
              <p class="text-gray-500 text-sm mt-1">
                🏠 {building.total_units} lots
                {#if building.construction_year}
                  · Construit en {building.construction_year}
                {/if}
              </p>
            </div>
            <button class="text-primary-600 hover:text-primary-700 text-sm font-medium">
              Détails →
            </button>
          </div>
        </div>
      {/each}
    </div>

    <!-- Pagination component -->
    {#if totalPages > 1}
      <Pagination
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        perPage={perPage}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>
