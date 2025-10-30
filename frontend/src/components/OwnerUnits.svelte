<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { UnitOwner, Unit } from '../lib/types';

  export let ownerId: string;
  export let showHistory = false;

  let ownerUnits: (UnitOwner & { unit?: Unit })[] = [];
  let loading = true;
  let error = '';

  onMount(async () => {
    await loadOwnerUnits();
  });

  async function loadOwnerUnits() {
    try {
      loading = true;
      const endpoint = showHistory
        ? `/owners/${ownerId}/units/history`
        : `/owners/${ownerId}/units`;

      const response = await api.get<UnitOwner[]>(endpoint);

      // Fetch unit details for each unit_owner
      ownerUnits = await Promise.all(
        response.map(async (uo) => {
          try {
            const unit = await api.get<Unit>(`/units/${uo.unit_id}`);
            return { ...uo, unit };
          } catch (e) {
            console.error(`Failed to load unit ${uo.unit_id}:`, e);
            return uo;
          }
        })
      );

      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des lots';
      console.error('Error loading owner units:', e);
    } finally {
      loading = false;
    }
  }

  function formatPercentage(percentage: number): string {
    return `${(percentage * 100).toFixed(2)}%`;
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', {
      day: 'numeric',
      month: 'short',
      year: 'numeric'
    });
  }

  function getUnitTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'Apartment': 'Appartement',
      'Parking': 'Parking',
      'Storage': 'Cave'
    };
    return labels[type] || type;
  }

  function getUnitTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      'Apartment': '🏠',
      'Parking': '🚗',
      'Storage': '📦'
    };
    return icons[type] || '📋';
  }

  $: activeUnits = ownerUnits.filter(uo => uo.is_active);
  $: inactiveUnits = ownerUnits.filter(uo => !uo.is_active);
</script>

<div class="space-y-4">
  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-4">Chargement...</p>
  {:else}
    <!-- Active Units -->
    {#if activeUnits.length > 0}
      <div class="space-y-2">
        <h4 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
          Lots actuels ({activeUnits.length})
        </h4>

        {#each activeUnits as ownerUnit (ownerUnit.id)}
          <div class="bg-white border border-gray-200 rounded-lg p-3 hover:shadow-md transition">
            <div class="flex justify-between items-start">
              <div class="flex-1">
                {#if ownerUnit.unit}
                  <div class="flex items-center gap-2">
                    <span class="text-2xl">{getUnitTypeIcon(ownerUnit.unit.unit_type)}</span>
                    <div>
                      <h5 class="font-semibold text-gray-900">
                        Lot {ownerUnit.unit.unit_number}
                      </h5>
                      <p class="text-sm text-gray-600">
                        {getUnitTypeLabel(ownerUnit.unit.unit_type)} - Étage {ownerUnit.unit.floor}
                      </p>
                      <p class="text-xs text-gray-500">
                        {ownerUnit.unit.surface_area} m² • {Math.round(ownerUnit.unit.quota)}/1000èmes
                      </p>
                    </div>
                  </div>
                  {#if ownerUnit.is_primary_contact}
                    <span class="inline-block mt-2 px-2 py-0.5 text-xs font-medium bg-primary-100 text-primary-800 rounded-full">
                      Contact principal
                    </span>
                  {/if}
                {:else}
                  <p class="text-gray-500 italic">Chargement des détails...</p>
                {/if}
                <p class="text-xs text-gray-500 mt-1">
                  Depuis le {formatDate(ownerUnit.start_date)}
                </p>
              </div>

              <div class="ml-4 text-right">
                <p class="text-2xl font-bold text-primary-600">
                  {formatPercentage(ownerUnit.ownership_percentage)}
                </p>
                <p class="text-xs text-gray-500">Quote-part</p>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {:else}
      <p class="text-center text-gray-600 py-4">
        Aucun lot actuel
      </p>
    {/if}

    <!-- Historical Units (if showHistory is true) -->
    {#if showHistory && inactiveUnits.length > 0}
      <div class="space-y-2 mt-6">
        <h4 class="text-sm font-semibold text-gray-700 uppercase tracking-wide">
          Historique ({inactiveUnits.length})
        </h4>

        {#each inactiveUnits as ownerUnit (ownerUnit.id)}
          <div class="bg-gray-50 border border-gray-200 rounded-lg p-3 opacity-75">
            <div class="flex justify-between items-start">
              <div class="flex-1">
                {#if ownerUnit.unit}
                  <div class="flex items-center gap-2">
                    <span class="text-xl opacity-50">{getUnitTypeIcon(ownerUnit.unit.unit_type)}</span>
                    <div>
                      <h5 class="font-medium text-gray-700">
                        Lot {ownerUnit.unit.unit_number}
                      </h5>
                      <p class="text-sm text-gray-600">
                        {getUnitTypeLabel(ownerUnit.unit.unit_type)}
                      </p>
                    </div>
                  </div>
                {:else}
                  <p class="text-gray-500 italic">Chargement des détails...</p>
                {/if}
                <p class="text-xs text-gray-500 mt-1">
                  {formatDate(ownerUnit.start_date)} → {ownerUnit.end_date ? formatDate(ownerUnit.end_date) : 'En cours'}
                </p>
              </div>

              <div class="ml-4 text-right">
                <p class="text-lg font-semibold text-gray-600">
                  {formatPercentage(ownerUnit.ownership_percentage)}
                </p>
              </div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
</div>
