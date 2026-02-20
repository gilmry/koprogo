<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";
  import type { Unit, Building } from "../lib/types";

  export let buildingIds: string[] = [];

  let units: Unit[] = [];
  let buildings: Record<string, Building> = {};
  let loading = false;
  let error = "";
  let selectedBuildingId = "";

  const unitTypeLabels: Record<string, string> = {
    Apartment: "Appartement",
    Parking: "Parking",
    Cellar: "Cave",
  };

  async function loadBuildings() {
    for (const bid of buildingIds) {
      try {
        const b = await api.get<Building>(`/buildings/${bid}`);
        buildings[bid] = b;
        buildings = buildings;
      } catch {
        // skip
      }
    }
  }

  async function loadUnits() {
    loading = true;
    error = "";
    try {
      if (selectedBuildingId) {
        const resp = await api.get<Unit[]>(`/buildings/${selectedBuildingId}/units`);
        units = Array.isArray(resp) ? resp : [];
      } else {
        const resp = await api.get<{ data: Unit[] }>("/units?page=1&per_page=100");
        units = resp?.data ?? (Array.isArray(resp) ? resp : []);
      }
    } catch (e: any) {
      error = e.message || "Erreur lors du chargement des lots";
    } finally {
      loading = false;
    }
  }

  $: if (selectedBuildingId !== undefined) loadUnits();

  onMount(async () => {
    await loadBuildings();
    await loadUnits();
  });

  function getBuildingName(bid: string): string {
    return buildings[bid]?.name ?? bid.slice(0, 8) + "…";
  }

  function formatArea(area: number | null | undefined): string {
    if (!area) return "-";
    return `${area} m²`;
  }

  function formatQuota(quota: number | null | undefined): string {
    if (!quota && quota !== 0) return "-";
    return `${quota}/1000`;
  }
</script>

<div class="space-y-4">
  {#if buildingIds.length > 1}
    <div class="flex items-center gap-3">
      <label for="unit-building-filter" class="text-sm font-medium text-gray-700 whitespace-nowrap">
        Filtrer par immeuble
      </label>
      <select
        id="unit-building-filter"
        bind:value={selectedBuildingId}
        class="flex-1 px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500"
      >
        <option value="">Tous les immeubles</option>
        {#each buildingIds as bid}
          <option value={bid}>{getBuildingName(bid)}</option>
        {/each}
      </select>
    </div>
  {/if}

  {#if loading}
    <div class="flex items-center justify-center gap-2 text-gray-400 py-8">
      <div class="animate-spin w-5 h-5 border-2 border-primary-500 border-t-transparent rounded-full"></div>
      <span class="text-sm">Chargement des lots...</span>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
      {error}
      <button on:click={loadUnits} class="ml-2 underline">Réessayer</button>
    </div>
  {:else if units.length === 0}
    <div class="bg-gray-50 border border-gray-200 rounded-lg p-6 text-center text-gray-500">
      <p class="text-lg mb-1">Aucun lot trouvé</p>
      <p class="text-sm">Aucun lot n'est associé à {selectedBuildingId ? "cet immeuble" : "vos immeubles"}.</p>
    </div>
  {:else}
    <!-- Summary bar -->
    <div class="bg-white rounded-lg border border-gray-200 p-4 flex flex-wrap gap-6 text-sm">
      <div>
        <span class="text-gray-500">Total lots</span>
        <span class="ml-1 font-semibold text-gray-900">{units.length}</span>
      </div>
      <div>
        <span class="text-gray-500">Surface totale</span>
        <span class="ml-1 font-semibold text-gray-900">
          {units.reduce((sum, u) => sum + (u.surface_area || 0), 0)} m²
        </span>
      </div>
      <div>
        <span class="text-gray-500">Tantièmes cumulés</span>
        <span class="ml-1 font-semibold text-gray-900">
          {units.reduce((sum, u) => sum + (u.quota || 0), 0)}/1000
        </span>
      </div>
    </div>

    <!-- Unit cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
      {#each units as unit (unit.id)}
        <div class="bg-white rounded-lg border border-gray-200 hover:shadow-md transition-shadow p-5">
          <div class="flex items-start justify-between mb-3">
            <div>
              <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium
                {unit.unit_type === 'Apartment' ? 'bg-blue-100 text-blue-800' :
                 unit.unit_type === 'Parking' ? 'bg-gray-100 text-gray-800' :
                 'bg-amber-100 text-amber-800'}">
                {unitTypeLabels[unit.unit_type] || unit.unit_type}
              </span>
            </div>
            <span class="text-xs text-gray-400">
              {unit.id.slice(0, 8)}…
            </span>
          </div>

          <h3 class="text-lg font-semibold text-gray-900 mb-2">
            Lot {unit.unit_number}
          </h3>

          <dl class="space-y-1.5 text-sm">
            <div class="flex justify-between">
              <dt class="text-gray-500">Étage</dt>
              <dd class="font-medium text-gray-900">
                {unit.floor === 0 ? "RDC" : unit.floor}
              </dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-gray-500">Surface</dt>
              <dd class="font-medium text-gray-900">{formatArea(unit.surface_area)}</dd>
            </div>
            <div class="flex justify-between">
              <dt class="text-gray-500">Tantièmes</dt>
              <dd class="font-medium text-gray-900">{formatQuota(unit.quota)}</dd>
            </div>
            {#if !selectedBuildingId && unit.building_id}
              <div class="flex justify-between">
                <dt class="text-gray-500">Immeuble</dt>
                <dd class="font-medium text-gray-900 truncate max-w-[160px]" title={getBuildingName(unit.building_id)}>
                  {getBuildingName(unit.building_id)}
                </dd>
              </div>
            {/if}
          </dl>

          {#if unit.owners && unit.owners.length > 0}
            <div class="mt-3 pt-3 border-t border-gray-100">
              <p class="text-xs text-gray-400 uppercase tracking-wider mb-1">Copropriétaires</p>
              <ul class="space-y-1">
                {#each unit.owners as uo}
                  <li class="text-sm flex justify-between">
                    <span class="text-gray-700">
                      {uo.owner?.first_name ?? ""} {uo.owner?.last_name ?? ""}
                      {#if uo.is_primary_contact}
                        <span class="text-xs text-primary-600">(principal)</span>
                      {/if}
                    </span>
                    <span class="text-gray-500 font-medium">
                      {(uo.ownership_percentage * 100).toFixed(1)}%
                    </span>
                  </li>
                {/each}
              </ul>
            </div>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>