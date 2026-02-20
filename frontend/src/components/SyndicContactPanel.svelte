<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "../lib/api";

  export let buildingIds: string[] = [];

  interface BuildingInfo {
    id: string;
    name: string;
    syndic_name: string | null;
    syndic_email: string | null;
    syndic_phone: string | null;
    syndic_address: string | null;
    syndic_office_hours: string | null;
    syndic_emergency_contact: string | null;
  }

  let selectedBuildingId = buildingIds[0] ?? "";
  let building: BuildingInfo | null = null;
  let loading = false;
  let error = "";

  async function loadBuilding(id: string) {
    if (!id) return;
    loading = true;
    error = "";
    try {
      building = await api.get<BuildingInfo>(`/buildings/${id}`);
    } catch (e: any) {
      error = e.message || "Erreur lors du chargement des coordonnées";
    } finally {
      loading = false;
    }
  }

  $: if (selectedBuildingId) loadBuilding(selectedBuildingId);

  onMount(() => {
    if (selectedBuildingId) loadBuilding(selectedBuildingId);
  });

  function hasSyndicInfo(b: BuildingInfo): boolean {
    return !!(b.syndic_name || b.syndic_email || b.syndic_phone);
  }
</script>

<div class="space-y-4">
  {#if buildingIds.length === 0}
    <div class="bg-gray-50 border border-gray-200 rounded-lg p-4 text-sm text-gray-500">
      Aucun immeuble associé à votre compte.
    </div>
  {:else}
    {#if buildingIds.length > 1}
      <div>
        <label for="building-select" class="block text-sm font-medium text-gray-700 mb-1">
          Sélectionner un immeuble
        </label>
        <select
          id="building-select"
          bind:value={selectedBuildingId}
          class="w-full px-3 py-2 border border-gray-300 rounded-lg text-sm focus:ring-2 focus:ring-primary-500"
        >
          {#each buildingIds as bid}
            <option value={bid}>{bid.slice(0, 8)}…</option>
          {/each}
        </select>
      </div>
    {/if}

    {#if loading}
      <div class="flex items-center gap-2 text-gray-500 py-6">
        <div class="animate-spin w-5 h-5 border-2 border-primary-500 border-t-transparent rounded-full"></div>
        <span class="text-sm">Chargement des coordonnées...</span>
      </div>
    {:else if error}
      <div class="bg-red-50 border border-red-200 rounded-lg p-4 text-sm text-red-700">
        {error}
        <button on:click={() => loadBuilding(selectedBuildingId)} class="ml-2 underline">
          Réessayer
        </button>
      </div>
    {:else if building}
      {#if !hasSyndicInfo(building)}
        <div class="bg-gray-50 border border-gray-200 rounded-lg p-5">
          <p class="font-medium text-gray-700 mb-1">{building.name}</p>
          <p class="text-sm text-gray-500">
            Aucune coordonnée syndic configurée pour cet immeuble.
          </p>
          <p class="text-xs text-gray-400 mt-1">
            Contactez votre administrateur pour mettre à jour ces informations.
          </p>
        </div>
      {:else}
        <div class="bg-white rounded-lg border border-gray-200 p-5 space-y-4">
          <p class="text-xs font-semibold text-gray-400 uppercase tracking-wider">
            {building.name}
          </p>

          <ul class="space-y-4">
            {#if building.syndic_name}
              <li class="flex items-start gap-3">
                <span class="text-primary-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-gray-400 uppercase tracking-wider">Syndic</p>
                  <p class="font-medium text-gray-900">{building.syndic_name}</p>
                </div>
              </li>
            {/if}

            {#if building.syndic_email}
              <li class="flex items-start gap-3">
                <span class="text-primary-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-gray-400 uppercase tracking-wider">Email</p>
                  <a href="mailto:{building.syndic_email}" class="text-primary-600 hover:underline">
                    {building.syndic_email}
                  </a>
                </div>
              </li>
            {/if}

            {#if building.syndic_phone}
              <li class="flex items-start gap-3">
                <span class="text-primary-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 5a2 2 0 012-2h3.28a1 1 0 01.948.684l1.498 4.493a1 1 0 01-.502 1.21l-2.257 1.13a11.042 11.042 0 005.516 5.516l1.13-2.257a1 1 0 011.21-.502l4.493 1.498a1 1 0 01.684.949V19a2 2 0 01-2 2h-1C9.716 21 3 14.284 3 6V5z"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-gray-400 uppercase tracking-wider">Téléphone</p>
                  <a href="tel:{building.syndic_phone}" class="text-primary-600 hover:underline">
                    {building.syndic_phone}
                  </a>
                </div>
              </li>
            {/if}

            {#if building.syndic_address}
              <li class="flex items-start gap-3">
                <span class="text-primary-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17.657 16.657L13.414 20.9a1.998 1.998 0 01-2.827 0l-4.244-4.243a8 8 0 1111.314 0z"/>
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 11a3 3 0 11-6 0 3 3 0 016 0z"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-gray-400 uppercase tracking-wider">Adresse</p>
                  <p class="text-gray-900">{building.syndic_address}</p>
                </div>
              </li>
            {/if}

            {#if building.syndic_office_hours}
              <li class="flex items-start gap-3">
                <span class="text-primary-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-gray-400 uppercase tracking-wider">Horaires</p>
                  <p class="text-gray-900">{building.syndic_office_hours}</p>
                </div>
              </li>
            {/if}

            {#if building.syndic_emergency_contact}
              <li class="flex items-start gap-3">
                <span class="text-red-500 mt-0.5 shrink-0">
                  <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"/>
                  </svg>
                </span>
                <div>
                  <p class="text-xs text-red-400 uppercase tracking-wider font-medium">Urgences</p>
                  <p class="text-gray-900 font-medium">{building.syndic_emergency_contact}</p>
                </div>
              </li>
            {/if}
          </ul>
        </div>
      {/if}
    {/if}
  {/if}
</div>
