<script lang="ts">
  /**
   * Example: Translated Building List Component
   *
   * This component demonstrates:
   * - Using svelte-i18n for translations
   * - Using the api helper with automatic Accept-Language headers
   * - Translating all UI text
   * - Proper i18n integration
   */

  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { api } from '../lib/api';

  interface Building {
    id: string;
    name: string;
    address: string;
    city: string;
    postal_code: string;
    country: string;
    total_units: number;
    construction_year?: number;
  }

  let buildings: Building[] = [];
  let loading = true;
  let error = '';
  let showForm = false;

  let newBuilding = {
    name: '',
    address: '',
    city: '',
    postal_code: '',
    country: 'Belgium',
    total_units: 0,
    construction_year: null as number | null
  };

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    try {
      loading = true;
      // Using api helper - automatically adds Accept-Language header
      buildings = await api.get<Building[]>('/buildings');
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : $_('common.error');
    } finally {
      loading = false;
    }
  }

  async function createBuilding(e: Event) {
    e.preventDefault();
    try {
      // Using api helper with POST - automatically adds Accept-Language header
      await api.post('/buildings', newBuilding);

      await loadBuildings();
      showForm = false;
      resetForm();
    } catch (e) {
      error = e instanceof Error ? e.message : $_('common.error');
    }
  }

  function resetForm() {
    newBuilding = {
      name: '',
      address: '',
      city: '',
      postal_code: '',
      country: 'Belgium',
      total_units: 0,
      construction_year: null
    };
  }
</script>

<div class="space-y-4">
  <!-- Header with count and create button -->
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">
      {$_('buildings.title')}
    </h2>
    <button
      on:click={() => showForm = !showForm}
      class="bg-primary-600 text-white px-4 py-2 rounded-lg hover:bg-primary-700 transition font-medium"
    >
      {showForm ? $_('common.cancel') : $_('buildings.create')}
    </button>
  </div>

  <!-- Building count -->
  <p class="text-gray-600">
    {buildings.length} {$_('buildings.title').toLowerCase()}
  </p>

  <!-- Error message -->
  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {$_('common.error')}: {error}
    </div>
  {/if}

  <!-- Create form -->
  {#if showForm}
    <form on:submit={createBuilding} class="bg-white p-6 rounded-lg shadow-md space-y-4">
      <h3 class="text-lg font-semibold text-gray-900">
        {$_('buildings.create')}
      </h3>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <!-- Name -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.name')} *
          </label>
          <input
            type="text"
            bind:value={newBuilding.name}
            required
            placeholder={$_('buildings.name')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- Address -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.address')} *
          </label>
          <input
            type="text"
            bind:value={newBuilding.address}
            required
            placeholder={$_('buildings.address')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- City -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.city')} *
          </label>
          <input
            type="text"
            bind:value={newBuilding.city}
            required
            placeholder={$_('buildings.city')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- Postal Code -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.postalCode')} *
          </label>
          <input
            type="text"
            bind:value={newBuilding.postal_code}
            required
            placeholder={$_('buildings.postalCode')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- Country -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.country')} *
          </label>
          <input
            type="text"
            bind:value={newBuilding.country}
            required
            placeholder={$_('buildings.country')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- Total Units -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.totalUnits')} *
          </label>
          <input
            type="number"
            bind:value={newBuilding.total_units}
            required
            min="1"
            placeholder={$_('buildings.totalUnits')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>

        <!-- Construction Year -->
        <div>
          <label class="block text-sm font-medium text-gray-700 mb-1">
            {$_('buildings.constructionYear')}
          </label>
          <input
            type="number"
            bind:value={newBuilding.construction_year}
            min="1800"
            max={new Date().getFullYear()}
            placeholder={$_('buildings.constructionYear')}
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
      </div>

      <div class="flex gap-2 justify-end">
        <button
          type="button"
          on:click={() => { showForm = false; resetForm(); }}
          class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition font-medium"
        >
          {$_('common.cancel')}
        </button>
        <button
          type="submit"
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
        >
          {$_('common.save')}
        </button>
      </div>
    </form>
  {/if}

  <!-- Loading state -->
  {#if loading}
    <div class="text-center py-8">
      <p class="text-gray-600">{$_('common.loading')}</p>
    </div>
  {:else}
    <!-- Buildings list -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each buildings as building}
        <div class="bg-white p-4 rounded-lg shadow-md hover:shadow-lg transition">
          <h3 class="text-lg font-semibold text-gray-900 mb-2">
            {building.name}
          </h3>
          <div class="text-sm text-gray-600 space-y-1">
            <p>📍 {building.address}, {building.city}</p>
            <p>📮 {building.postal_code}, {building.country}</p>
            <p>🚪 {building.total_units} {$_('units.title').toLowerCase()}</p>
            {#if building.construction_year}
              <p>🏗️ {building.construction_year}</p>
            {/if}
          </div>
          <div class="mt-4 flex gap-2">
            <button
              class="flex-1 px-3 py-1 text-sm bg-primary-600 text-white rounded hover:bg-primary-700 transition"
            >
              {$_('common.edit')}
            </button>
            <button
              class="flex-1 px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 transition"
            >
              {$_('buildings.details')}
            </button>
          </div>
        </div>
      {/each}
    </div>

    {#if buildings.length === 0}
      <div class="text-center py-12 bg-white rounded-lg shadow-md">
        <p class="text-gray-600 mb-4">
          {$_('buildings.list')} - 0 {$_('buildings.title').toLowerCase()}
        </p>
        <button
          on:click={() => showForm = true}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
        >
          {$_('buildings.create')}
        </button>
      </div>
    {/if}
  {/if}
</div>
