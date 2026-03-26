<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { etatsDatesApi, type EtatDate, type EtatDateStats, EtatDateStatus } from '../../lib/api/etats-dates';
  import EtatDateStatusBadge from './EtatDateStatusBadge.svelte';
  import EtatDateCreateForm from './EtatDateCreateForm.svelte';
  import { formatDate } from "../../lib/utils/date.utils";
  import { formatCurrency } from "../../lib/utils/finance.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  let etatsDates: EtatDate[] = [];
  let stats: EtatDateStats | null = null;
  let loading = true;
  let error = '';
  let showCreateForm = false;

  // Filters
  let filterStatus = '';
  let currentPage = 1;
  let totalPages = 1;

  onMount(async () => {
    await Promise.all([loadEtatsDates(), loadStats()]);
  });

  async function loadEtatsDates() {
    loading = true;
    error = '';
    const statusFilter = filterStatus ? filterStatus as EtatDateStatus : undefined;
    const result = await withErrorHandling({
      action: () => etatsDatesApi.list(currentPage, 20, statusFilter),
      errorMessage: $_('common.loadingError'),
    });
    if (result) {
      etatsDates = result.data;
      totalPages = Math.ceil(result.total / result.per_page);
    } else {
      error = $_('common.loadingError');
    }
    loading = false;
  }

  async function loadStats() {
    const result = await withErrorHandling({
      action: () => etatsDatesApi.getStats(),
    });
    if (result) stats = result;
  }

  function handleCreated() {
    showCreateForm = false;
    loadEtatsDates();
    loadStats();
  }

  function changePage(page: number) {
    currentPage = page;
    loadEtatsDates();
  }
</script>

<div class="space-y-6" data-testid="etat-date-list">
  <!-- Stats -->
  {#if stats}
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">{$_('etatsDate.totalRequests')}</p>
        <p class="text-2xl font-bold text-gray-900">{stats.total_requests}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">{$_('etatsDate.inProgress')}</p>
        <p class="text-2xl font-bold text-yellow-600">{stats.in_progress_count}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">{$_('etatsDate.overdue')}</p>
        <p class="text-2xl font-bold text-red-600">{stats.overdue_count}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-4">
        <p class="text-sm text-gray-600">{$_('etatsDate.averageProcessingDays')}</p>
        <p class="text-2xl font-bold text-gray-900">{stats.average_processing_days.toFixed(1)}j</p>
      </div>
    </div>
  {/if}

  <!-- Filters + Actions -->
  <div class="bg-white rounded-lg shadow p-4">
    <div class="flex flex-wrap items-end gap-4">
      <div>
        <label for="filter-status" class="block text-sm font-medium text-gray-700 mb-1">{$_('common.status')}</label>
        <select
          id="filter-status"
          bind:value={filterStatus}
          on:change={() => { currentPage = 1; loadEtatsDates(); }}
          class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
        >
          <option value="">{$_('common.all')}</option>
          <option value="requested">{$_('etatsDate.status.requested')}</option>
          <option value="in_progress">{$_('etatsDate.status.inProgress')}</option>
          <option value="generated">{$_('etatsDate.status.generated')}</option>
          <option value="delivered">{$_('etatsDate.status.delivered')}</option>
          <option value="expired">{$_('etatsDate.status.expired')}</option>
        </select>
      </div>

      <div class="ml-auto">
        <button
          on:click={() => showCreateForm = !showCreateForm}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium"
        >
          {showCreateForm ? $_('common.close') : '+ ' + $_('etatsDate.newEtatDate')}
        </button>
      </div>
    </div>
  </div>

  <!-- Create Form -->
  {#if showCreateForm}
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">{$_('etatsDate.newEtatDate')}</h3>
      <EtatDateCreateForm on:created={handleCreated} on:cancel={() => showCreateForm = false} />
    </div>
  {/if}

  <!-- Error -->
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-lg p-4">
      <p class="text-red-700">{error}</p>
      <button on:click={loadEtatsDates} class="mt-2 text-sm text-red-600 underline">{$_('common.retry')}</button>
    </div>
  {/if}

  <!-- List -->
  {#if loading}
    <div class="flex justify-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600" data-testid="etat-date-list-spinner"></div>
    </div>
  {:else if etatsDates.length === 0}
    <div class="bg-white rounded-lg shadow p-12 text-center">
      <p class="text-4xl mb-4">📋</p>
      <h3 class="text-xl font-semibold text-gray-900 mb-2">{$_('etatsDate.noEtatDates')}</h3>
      <p class="text-gray-600">{$_('etatsDate.noEtatDatesHint')}</p>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.reference')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.buildingUnit')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.notary')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.refDate')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.balance')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('common.status')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('etatsDate.delay')}</th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('common.actions')}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#each etatsDates as ed}
            <tr class="hover:bg-gray-50 transition {ed.is_overdue ? 'bg-red-50' : ''}" data-testid="etat-date-row">
              <td class="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-900">{ed.reference_number}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm">
                <p class="font-medium text-gray-900">{ed.building_name}</p>
                <p class="text-gray-500 text-xs">{$_('etatsDate.unit')} {ed.unit_number}</p>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">{ed.notary_name}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-700">{formatDate(ed.reference_date)}</td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium {ed.total_balance >= 0 ? 'text-green-600' : 'text-red-600'}">
                {formatCurrency(ed.total_balance)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <EtatDateStatusBadge status={ed.status} />
                {#if ed.is_overdue}
                  <span class="ml-1 px-1.5 py-0.5 bg-red-500 text-white text-xs rounded">{$_('etatsDate.overdue')}</span>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm {ed.days_since_request > 10 ? 'text-red-600 font-bold' : 'text-gray-700'}">
                {ed.days_since_request}j
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <a
                  href="/etat-date-detail?id={ed.id}"
                  class="text-sm text-primary-600 hover:text-primary-700 font-medium"
                >
                  {$_('common.details')}
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
