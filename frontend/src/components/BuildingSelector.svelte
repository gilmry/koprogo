<script lang="ts">
  import { onMount } from "svelte";
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
      const response = await api.get("/buildings");
      const list = Array.isArray(response)
        ? response
        : response.data ?? response.buildings ?? [];
      buildings = list;

      if (buildings.length === 1 && !selectedBuildingId) {
        selectedBuildingId = buildings[0].id;
        if (onSelect) onSelect(selectedBuildingId);
        if (onSelectBuilding) onSelectBuilding(buildings[0]);
      } else if (buildings.length > 1 && selectedBuildingId) {
        const selected = buildings.find(b => b.id === selectedBuildingId);
        if (onSelect) onSelect(selectedBuildingId);
        if (selected && onSelectBuilding) onSelectBuilding(selected);
      }
    } catch (err: any) {
      error = "Erreur lors du chargement des immeubles";
      console.error("Failed to load buildings:", err);
    } finally {
      loading = false;
    }
  }
</script>

{#if loading}
  <div class="text-sm text-gray-500">Chargement des immeubles...</div>
{:else if error}
  <div class="text-sm text-red-600">{error}</div>
{:else if buildings.length === 0}
  <div class="p-3 bg-red-50 border border-red-200 rounded-md">
    <p class="text-sm text-red-800">
      Aucun immeuble trouvé. Veuillez d'abord créer un immeuble.
    </p>
  </div>
{:else if buildings.length === 1}
  <div>
    <label class="block text-sm font-medium text-gray-700">{label}</label>
    <div class="mt-1 px-3 py-2 bg-gray-50 border border-gray-200 rounded-md text-sm text-gray-700">
      {buildings[0].name} — {buildings[0].address}
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
      <option value="">-- Sélectionner un immeuble --</option>
      {#each buildings as building}
        <option value={building.id}>{building.name} — {building.address}</option>
      {/each}
    </select>
  </div>
{/if}
