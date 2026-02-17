<script lang="ts">
  import { onMount } from 'svelte';
  import { budgetsApi, type Budget, type BudgetStats, BudgetStatus } from '../../lib/api/budgets';
  import { api } from '../../lib/api';
  import type { Building } from '../../lib/types';
  import BudgetStatusBadge from './BudgetStatusBadge.svelte';
  import BudgetCreateForm from './BudgetCreateForm.svelte';

  let budgets: Budget[] = [];
  let stats: BudgetStats | null = null;
  let buildings: Building[] = [];
  let loading = true;
  let error = '';
  let showCreateForm = false;

  // Filters
  let filterBuildingId = '';
  let filterStatus = '';
  let filterYear = '';
  let currentPage = 1;
  let totalPages = 1;

  onMount(async () => {
    try {
      const response = await api.get<{ data: Building[] }>('/buildings?page=1&per_page=100');
      buildings = response.data || [];
    } catch (err) {
      console.error('Error loading buildings:', err);
    }
    await Promise.all([loadBudgets(), loadStats()]);
  });

  async function loadBudgets() {
    try {
      loading = true;
      error = '';
      const statusFilter = filterStatus ? filterStatus as BudgetStatus : undefined;
      const buildingFilter = filterBuildingId || undefined;
      const response = await budgetsApi.list(currentPage, 20, buildingFilter, statusFilter);
      budgets = response.data;
      totalPages = Math.ceil(response.total / response.per_page);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des budgets';
    } finally {
      loading = false;
    }
  }

  async function loadStats() {
    try {
      stats = await budgetsApi.getStats();
    } catch (err) {
      console.error('Error loading stats:', err);
    }
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function handleCreated() {
    showCreateForm = false;
    loadBudgets();
    loadStats();
  }

  function changePage(page: number) {
    currentPage = page;
    loadBudgets();
  }

  $: if (filterBuildingId !== undefined || filterStatus !== undefined) {
    // Reset to page 1 on filter change (after mount)
  }
</script>

<div class="space-y-6">
  <!-- Stats -->
  {#if stats}
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">Total budgets</p>
        <p class="text-2xl font-bold text-gray-900">{stats.total_budgets}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">Approuves</p>
        <p class="text-2xl font-bold text-green-600">{stats.approved_count}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">En attente</p>
        <p class="text-2xl font-bold text-blue-600">{stats.submitted_count}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">Budget moyen</p>
        <p class="text-2xl font-bold text-gray-900">{formatCurrency(stats.average_total_budget)}</p>
      </div>
    </div>
  {/if}

  <!-- Filters + Actions -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="flex flex-wrap items-end gap-4">
      <div>
        <label for="filter-building" class="block text-sm font-medium text-gray-700 mb-1">Immeuble</label>
        <select
          id="filter-building"
          bind:value={filterBuildingId}
          on:change={() => { currentPage = 1; loadBudgets(); }}
          class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        >
          <option value="">Tous</option>
          {#each buildings as building}
            <option value={building.id}>{building.name}</option>
          {/each}
        </select>
      </div>

      <div>
        <label for="filter-status" class="block text-sm font-medium text-gray-700 mb-1">Statut</label>
        <select
          id="filter-status"
          bind:value={filterStatus}
          on:change={() => { currentPage = 1; loadBudgets(); }}
          class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        >
          <option value="">Tous</option>
          <option value="draft">Brouillon</option>
          <option value="submitted">Soumis</option>
          <option value="approved">Approuve</option>
          <option value="rejected">Rejete</option>
          <option value="archived">Archive</option>
        </select>
      </div>

      <div class="ml-auto">
        <button
          on:click={() => showCreateForm = !showCreateForm}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
        >
          {showCreateForm ? 'Fermer' : '+ Nouveau Budget'}
        </button>
      </div>
    </div>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Nouveau Budget</h3>
      <BudgetCreateForm on:created={handleCreated} on:cancel={() => showCreateForm = false} />
    </div>
  {/if}

  <!-- Error -->
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-700">{error}</p>
      <button on:click={loadBudgets} class="mt-2 text-sm text-red-600 underline">Reessayer</button>
    </div>
  {/if}

  <!-- Budget List -->
  {#if loading}
    <div class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
    </div>
  {:else if budgets.length === 0}
    <div class="bg-white rounded-lg shadow p-12 text-center">
      <p class="text-4xl mb-4">📊</p>
      <h3 class="text-xl font-semibold text-gray-900 mb-2">Aucun budget</h3>
      <p class="text-gray-600">Creez votre premier budget pour commencer.</p>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Annee</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Immeuble</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Ordinaire</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Extraordinaire</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Total</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Provision/mois</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Statut</th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#each budgets as budget}
            {@const buildingName = buildings.find(b => b.id === budget.building_id)?.name || budget.building_id.substring(0, 8)}
            <tr class="hover:bg-gray-50 transition">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-bold text-gray-900">{budget.fiscal_year}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">{buildingName}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">{formatCurrency(budget.ordinary_budget)}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">{formatCurrency(budget.extraordinary_budget)}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">{formatCurrency(budget.total_budget)}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-primary-600">{formatCurrency(budget.monthly_provision_amount)}</td>
              <td class="px-6 py-4 whitespace-nowrap">
                <BudgetStatusBadge status={budget.status} />
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <a
                  href="/budget-detail?id={budget.id}"
                  class="text-sm text-primary-600 hover:text-primary-700 font-medium"
                >
                  Details
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <!-- Pagination -->
    {#if totalPages > 1}
      <div class="flex justify-center gap-2 mt-4">
        {#each Array(totalPages) as _, i}
          <button
            on:click={() => changePage(i + 1)}
            class="px-3 py-1 rounded-lg text-sm {currentPage === i + 1 ? 'bg-primary-600 text-white' : 'bg-white border border-gray-300 text-gray-700 hover:bg-gray-50'}"
          >
            {i + 1}
          </button>
        {/each}
      </div>
    {/if}
  {/if}
</div>
