<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from 'svelte-i18n';
  import { api } from "../lib/api";

  export let selectedBuildingId = "";
  export let label = "Immeuble";
  export let required = true;
  export let disabled = false;
  export let onSelect: ((buildingId: string) => void) | undefined = undefined;
  export let onSelectBuilding: ((building: Building) => void) | undefined = undefined;

  interface Building {
    id: string;
    name: string;
    address: string;
    city?: string;
    postal_code?: string;
    organization_id?: string;
  }

  let buildings: Building[] = [];
  let loading = true;
  let error = "";

  onMount(async () => {
    await loadBuildings();
  });

  async function loadBuildings() {
    try {
      loading = true;
      error = "";
      const response = await api.get("/buildings?per_page=100");

      // Le backend retourne { data: [...], pagination: {...} }
      // Gérer tous les formats possibles de manière robuste
      let list: Building[];
      if (Array.isArray(response)) {
        list = response;
      } else if (response && Array.isArray(response.data)) {
        list = response.data;
      } else if (response && Array.isArray(response.buildings)) {
        list = response.buildings;
      } else {
        list = [];
      }
      buildings = list;

      // Auto-sélection si un seul immeuble
      if (buildings.length === 1 && !selectedBuildingId) {
        selectedBuildingId = buildings[0].id;
        // Utiliser tick() pour laisser Svelte mettre à jour avant d'appeler le callback
        setTimeout(() => {
          if (onSelect) onSelect(selectedBuildingId);
          if (onSelectBuilding) onSelectBuilding(buildings[0]);
        }, 0);
      } else if (buildings.length > 1 && selectedBuildingId) {
        // Si un ID était déjà sélectionné, notifier le parent
        const selected = buildings.find(b => b.id === selectedBuildingId);
        setTimeout(() => {
          if (onSelect) onSelect(selectedBuildingId);
          if (selected && onSelectBuilding) onSelectBuilding(selected);
        }, 0);
      }
    } catch (err: any) {
      error = $_('buildings.loadError');
      console.error("Failed to load buildings:", err);
    } finally {
      loading = false;
    }
  }
</script>

{#if loading}
  <div class="text-sm text-gray-500 py-2">{$_('buildings.loading')}</div>
{:else if error}
  <div class="p-3 bg-red-50 border border-red-200 rounded-md">
    <p class="text-sm text-red-800">{error}</p>
    <button
      on:click={loadBuildings}
      class="mt-2 text-sm text-red-700 underline hover:text-red-900"
    >
      {$_('common.retry')}
    </button>
  </div>
{:else if buildings.length === 0}
  <div class="p-3 bg-red-50 border border-red-200 rounded-md">
    <p class="text-sm text-red-800">
      {$_('buildings.noBuildings')}
    </p>
  </div>
{:else if buildings.length === 1}
  <div>
    <label class="block text-sm font-medium text-gray-700">{label}</label>
    <div class="mt-1 px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-sm text-gray-700">
      {buildings[0].name} — {buildings[0].address}{#if buildings[0].city}, {buildings[0].postal_code} {buildings[0].city}{/if}
    </div>
  </div>
{:else}
  <div>
    <label for="building-selector" class="block text-sm font-medium text-gray-700">
      {label} {#if required}<span class="text-red-500">*</span>{/if}
    </label>
    <select
      id="building-selector"
      bind:value={selectedBuildingId}
      on:change={() => {
        if (selectedBuildingId) {
          if (onSelect) onSelect(selectedBuildingId);
          const selected = buildings.find(b => b.id === selectedBuildingId);
          if (selected && onSelectBuilding) onSelectBuilding(selected);
        }
      }}
      {required}
      {disabled}
      class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
    >
      <option value="">{$_('buildings.selectBuilding')}</option>
      {#each buildings as building}
        <option value={building.id}>
          {building.name} — {building.address}{#if building.city}, {building.postal_code} {building.city}{/if}
        </option>
      {/each}
    </select>
  </div>
{/if}
