<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';
  import type { Building, Unit, Expense } from '../../lib/types';

  $: user = $authStore.user;

  interface OwnerStats {
    total_buildings: number;
    total_units: number;
    total_owners: number;
    pending_expenses_count: number;
    pending_expenses_amount: number;
    next_meeting: {
      id: string;
      date: string;
      building_name: string;
    } | null;
  }

  interface BoardMandate {
    id: string;
    building_id: string;
    building_name: string;
    building_address: string;
    position: string;
    mandate_start: string;
    mandate_end: string;
    days_remaining: number;
    expires_soon: boolean;
  }

  let stats: OwnerStats | null = null;
  let recentBuildings: Building[] = [];
  let recentUnits: Unit[] = [];
  let boardMandates: BoardMandate[] = [];
  let loading = true;
  let error: string | null = null;

  onMount(async () => {
    await loadDashboardData();
  });

  async function loadDashboardData() {
    try {
      loading = true;
      const [statsData, buildingsData, unitsData, mandatesData] = await Promise.all([
        api.get<OwnerStats>('/stats/owner'),
        api.get<{ data: Building[] }>('/buildings?page=1&per_page=3'),
        api.get<{ data: Unit[] }>('/units?page=1&per_page=5'),
        api.get<{ mandates: BoardMandate[] }>('/board-members/my-mandates'),
      ]);
      stats = statsData;
      recentBuildings = buildingsData.data;
      recentUnits = unitsData.data;
      boardMandates = mandatesData.mandates;
      loading = false;
    } catch (err) {
      error = err instanceof Error ? err.message : 'Erreur lors du chargement des données';
      loading = false;
      console.error('Error fetching owner dashboard data:', err);
    }
  }

  function formatDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', { day: 'numeric', month: 'short' });
  }

  function formatAmount(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function getUnitTypeIcon(type: string): string {
    const icons: Record<string, string> = {
      'Apartment': '🏠',
      'Parking': '🚗',
      'Storage': '📦'
    };
    return icons[type] || '📋';
  }

  function getUnitTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      'Apartment': 'Appartement',
      'Parking': 'Parking',
      'Storage': 'Cave'
    };
    return labels[type] || type;
  }

  function getPositionLabel(position: string): string {
    const labels: Record<string, string> = {
      'president': 'Président',
      'treasurer': 'Trésorier',
      'secretary': 'Secrétaire'
    };
    return labels[position] || position;
  }

  function getPositionIcon(position: string): string {
    const icons: Record<string, string> = {
      'president': '👑',
      'treasurer': '💰',
      'secretary': '📝'
    };
    return icons[position] || '🎯';
  }

  function formatFullDate(dateString: string): string {
    const date = new Date(dateString);
    return date.toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }
</script>

<div>
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      Bienvenue, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      Mon Espace Copropriétaire
    </p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4 mb-8">
      <p class="text-red-800 font-medium">Erreur</p>
      <p class="text-red-600 text-sm">{error}</p>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6 mb-8">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Immeubles</span>
          <span class="text-2xl">🏢</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{stats.total_buildings}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.total_units} lots au total</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Charges à payer</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-orange-600">{formatAmount(stats.pending_expenses_amount)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.pending_expenses_count} charges en attente</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Prochaine AG</span>
          <span class="text-2xl">📅</span>
        </div>
        {#if stats.next_meeting}
          <p class="text-xl font-bold text-gray-900">{formatDate(stats.next_meeting.date)}</p>
          <p class="text-sm text-gray-500 mt-1">{stats.next_meeting.building_name}</p>
        {:else}
          <p class="text-lg font-medium text-gray-500">Aucune AG prévue</p>
        {/if}
      </div>
    </div>

    <!-- Board Member Mandates (if applicable) -->
    {#if boardMandates.length > 0}
      <div class="mb-8">
        <div class="bg-gradient-to-r from-primary-50 to-primary-100 border-2 border-primary-300 rounded-lg shadow-lg p-6">
          <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-3">
              <span class="text-4xl">🎖️</span>
              <div>
                <h2 class="text-2xl font-bold text-gray-900">Membre du Conseil</h2>
                <p class="text-sm text-gray-600">Vous faites partie du conseil de copropriété</p>
              </div>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-4">
            {#each boardMandates as mandate}
              <div class="bg-white rounded-lg border-2 border-primary-200 p-4 hover:border-primary-400 transition">
                <div class="flex items-start justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <span class="text-3xl">{getPositionIcon(mandate.position)}</span>
                    <div>
                      <h3 class="font-bold text-gray-900">{getPositionLabel(mandate.position)}</h3>
                      <p class="text-sm text-gray-600">{mandate.building_name}</p>
                    </div>
                  </div>
                  {#if mandate.expires_soon}
                    <span class="px-2 py-1 bg-orange-100 text-orange-800 text-xs font-medium rounded">
                      ⚠️ Expire bientôt
                    </span>
                  {/if}
                </div>

                <p class="text-xs text-gray-500 mb-3">{mandate.building_address}</p>

                <div class="flex items-center justify-between text-sm mb-3">
                  <span class="text-gray-600">Mandat:</span>
                  <span class="font-medium text-gray-900">
                    {formatFullDate(mandate.mandate_start)} - {formatFullDate(mandate.mandate_end)}
                  </span>
                </div>

                <div class="flex items-center justify-between text-sm mb-4">
                  <span class="text-gray-600">Reste:</span>
                  <span class="font-medium {mandate.expires_soon ? 'text-orange-600' : 'text-green-600'}">
                    {mandate.days_remaining} jours
                  </span>
                </div>

                <a
                  href="/board-dashboard?building_id={mandate.building_id}"
                  class="block w-full text-center bg-primary-600 hover:bg-primary-700 text-white font-medium py-2 px-4 rounded transition"
                >
                  📊 Tableau de Bord du Conseil
                </a>
              </div>
            {/each}
          </div>
        </div>
      </div>
    {/if}

    <!-- Main Content -->
    <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
      <!-- Buildings -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-lg font-semibold text-gray-900">Mes immeubles</h2>
          <a href="/buildings" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            Voir tout →
          </a>
        </div>
        <div class="p-6">
          {#if recentBuildings.length > 0}
            <div class="space-y-4">
              {#each recentBuildings as building}
                <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
                  <div class="flex items-center justify-between mb-2">
                    <h3 class="font-semibold text-gray-900">{building.name}</h3>
                    <span class="text-2xl">🏢</span>
                  </div>
                  <p class="text-sm text-gray-600">{building.address}</p>
                  <p class="text-sm text-gray-500 mt-1">{building.city}, {building.postal_code}</p>
                  <p class="text-xs text-gray-400 mt-1">{building.total_units} lots</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">Aucun immeuble</p>
            </div>
          {/if}
        </div>
      </div>

      <!-- Recent Units -->
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200 flex justify-between items-center">
          <h2 class="text-lg font-semibold text-gray-900">Lots récents</h2>
          <a href="/units" class="text-sm text-primary-600 hover:text-primary-700 font-medium">
            Voir tout →
          </a>
        </div>
        <div class="p-6">
          {#if recentUnits.length > 0}
            <div class="space-y-4">
              {#each recentUnits as unit}
                <div class="p-4 border border-gray-200 rounded-lg hover:border-primary-500 transition">
                  <div class="flex items-center justify-between mb-2">
                    <h3 class="font-semibold text-gray-900">Lot {unit.unit_number}</h3>
                    <span class="text-2xl">{getUnitTypeIcon(unit.unit_type)}</span>
                  </div>
                  <p class="text-sm text-gray-600">{getUnitTypeLabel(unit.unit_type)} - Étage {unit.floor}</p>
                  <p class="text-sm text-gray-500 mt-1">{unit.surface_area} m² • {Math.round(unit.quota)}/1000èmes</p>
                </div>
              {/each}
            </div>
          {:else}
            <div class="text-center py-8">
              <p class="text-gray-500">Aucun lot</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="mt-8">
      <div class="bg-white rounded-lg shadow">
        <div class="p-6 border-b border-gray-200">
          <h2 class="text-lg font-semibold text-gray-900">Actions rapides</h2>
        </div>
        <div class="p-6">
          <div class="grid grid-cols-2 md:grid-cols-{boardMandates.length > 0 ? '5' : '4'} gap-4">
            <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
              <span class="text-sm font-medium text-gray-700">Immeubles</span>
            </a>
            <a href="/units" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">🚪</span>
              <span class="text-sm font-medium text-gray-700">Lots</span>
            </a>
            <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
              <span class="text-sm font-medium text-gray-700">Charges</span>
            </a>
            <a href="/meetings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
              <span class="text-4xl mb-2 group-hover:scale-110 transition">📅</span>
              <span class="text-sm font-medium text-gray-700">Assemblées</span>
            </a>
            {#if boardMandates.length > 0}
              <a
                href="/board-dashboard?building_id={boardMandates[0].building_id}"
                class="flex flex-col items-center justify-center p-6 border-2 border-primary-300 bg-primary-50 rounded-lg hover:border-primary-500 hover:bg-primary-100 transition group"
              >
                <span class="text-4xl mb-2 group-hover:scale-110 transition">🎖️</span>
                <span class="text-sm font-medium text-primary-700">Conseil</span>
              </a>
            {/if}
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>
