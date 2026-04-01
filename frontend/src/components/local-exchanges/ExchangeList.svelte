<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { onMount } from "svelte";
  import {
    localExchangesApi,
    type LocalExchange,
    ExchangeType,
    ExchangeStatus,
    formatCredits,
    formatRating,
  } from "../../lib/api/local-exchanges";
  import ExchangeStatusBadge from "./ExchangeStatusBadge.svelte";
  import ExchangeTypeBadge from "./ExchangeTypeBadge.svelte";
  import { toast } from "../../stores/toast";
  import { formatDateShort } from "../../lib/utils/date.utils";
  import { withLoadingState, withErrorHandling } from "../../lib/utils/error.utils";

  export let buildingId: string;
  export let currentOwnerId: string;
  export let showOnlyAvailable: boolean = false;
  export let showFilters: boolean = true;

  let exchanges: LocalExchange[] = [];
  let filteredExchanges: LocalExchange[] = [];
  let loading: boolean = true;
  let error: string | null = null;

  // Filters
  let filterType: ExchangeType | "all" = "all";
  let filterStatus: ExchangeStatus | "all" = "all";
  let searchQuery: string = "";

  async function loadExchanges() {
    await withLoadingState({
      action: () => showOnlyAvailable
        ? localExchangesApi.listAvailable(buildingId)
        : localExchangesApi.listByBuilding(buildingId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => { exchanges = data; applyFilters(); },
      errorMessage: $_('exchanges.load_error'),
    });
  }

  function applyFilters() {
    filteredExchanges = exchanges.filter((exchange) => {
      // Type filter
      if (filterType !== "all" && exchange.exchange_type !== filterType) {
        return false;
      }

      // Status filter
      if (filterStatus !== "all" && exchange.status !== filterStatus) {
        return false;
      }

      // Search query (title or description)
      if (searchQuery.trim()) {
        const query = searchQuery.toLowerCase();
        const titleMatch = exchange.title.toLowerCase().includes(query);
        const descMatch = exchange.description.toLowerCase().includes(query);
        if (!titleMatch && !descMatch) {
          return false;
        }
      }

      return true;
    });
  }

  function handleFilterChange() {
    applyFilters();
  }

  function isMyOffer(exchange: LocalExchange): boolean {
    return exchange.provider_id === currentOwnerId;
  }

  function canRequest(exchange: LocalExchange): boolean {
    return (
      exchange.status === ExchangeStatus.Offered &&
      !isMyOffer(exchange) &&
      !exchange.requester_id
    );
  }

  async function handleRequest(exchangeId: string) {
    await withErrorHandling({
      action: () => localExchangesApi.request(exchangeId),
      errorMessage: $_('common.error'),
      onSuccess: () => loadExchanges(),
    });
  }

  onMount(() => {
    loadExchanges();
  });
</script>

<div class="space-y-4" data-testid="exchange-list">
  <!-- Filters (optional) -->
  {#if showFilters}
    <div class="bg-white p-4 rounded-lg shadow" data-testid="exchange-list-filters">
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <!-- Search -->
        <div>
          <label
            for="search"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            {$_('common.search')}
          </label>
          <input
            id="search"
            type="text"
            bind:value={searchQuery}
            on:input={handleFilterChange}
            placeholder={$_('exchanges.search_placeholder')}
            data-testid="exchange-search-input"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          />
        </div>

        <!-- Type Filter -->
        <div>
          <label
            for="filter-type"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            {$_('exchanges.type')}
          </label>
          <select
            id="filter-type"
            bind:value={filterType}
            on:change={handleFilterChange}
            data-testid="exchange-filter-type"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">{$_('exchanges.all_types')}</option>
            <option value={ExchangeType.Service}>🛠️ {$_('exchanges.type_service')}</option>
            <option value={ExchangeType.ObjectLoan}>📦 {$_('exchanges.type_loan')}</option>
            <option value={ExchangeType.SharedPurchase}
              >🛒 {$_('exchanges.type_purchase')}</option
            >
          </select>
        </div>

        <!-- Status Filter -->
        <div>
          <label
            for="filter-status"
            class="block text-sm font-medium text-gray-700 mb-1"
          >
            {$_('common.status')}
          </label>
          <select
            id="filter-status"
            bind:value={filterStatus}
            on:change={handleFilterChange}
            data-testid="exchange-filter-status"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
          >
            <option value="all">{$_('exchanges.all_statuses')}</option>
            <option value={ExchangeStatus.Offered}>{$_('exchanges.status_offered')}</option>
            <option value={ExchangeStatus.Requested}>{$_('exchanges.status_requested')}</option>
            <option value={ExchangeStatus.InProgress}>{$_('exchanges.status_in_progress')}</option>
            <option value={ExchangeStatus.Completed}>{$_('exchanges.status_completed')}</option>
            <option value={ExchangeStatus.Cancelled}>{$_('exchanges.status_cancelled')}</option>
          </select>
        </div>
      </div>
    </div>
  {/if}

  <!-- Loading State -->
  {#if loading}
    <div class="text-center py-12" data-testid="exchange-list-loading">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-gray-900"
      ></div>
      <p class="mt-2 text-gray-600">{$_('common.loading')}</p>
    </div>
  {/if}

  <!-- Error State -->
  {#if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4" data-testid="exchange-list-error">
      <p class="text-red-800">❌ {error}</p>
    </div>
  {/if}

  <!-- Empty State -->
  {#if !loading && !error && filteredExchanges.length === 0}
    <div class="text-center py-12 bg-gray-50 rounded-lg" data-testid="exchange-list-empty">
      <svg
        class="mx-auto h-12 w-12 text-gray-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
        />
      </svg>
      <h3 class="mt-2 text-sm font-medium text-gray-900">
        {$_('exchanges.no_found')}
      </h3>
      <p class="mt-1 text-sm text-gray-500">
        {$_('exchanges.create_new')}
      </p>
    </div>
  {/if}

  <!-- Exchanges List -->
  {#if !loading && !error && filteredExchanges.length > 0}
    <div class="bg-white shadow rounded-lg overflow-hidden">
      <ul class="divide-y divide-gray-200">
        {#each filteredExchanges as exchange (exchange.id)}
          <li class="p-6 hover:bg-gray-50 transition-colors" data-testid="exchange-list-row">
            <div class="flex items-start justify-between">
              <div class="flex-1">
                <!-- Title + Type Badge -->
                <div class="flex items-center gap-2 mb-2">
                  <h3 class="text-lg font-semibold text-gray-900">
                    {exchange.title}
                  </h3>
                  <ExchangeTypeBadge type={exchange.exchange_type} />
                  <ExchangeStatusBadge status={exchange.status} />
                  {#if isMyOffer(exchange)}
                    <span
                      class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-800"
                    >
                      {$_('exchanges.my_offer')}
                    </span>
                  {/if}
                </div>

                <!-- Description -->
                <p class="text-sm text-gray-700 mb-3 line-clamp-2">
                  {exchange.description}
                </p>

                <!-- Meta Info -->
                <div class="flex items-center gap-4 text-sm text-gray-500">
                  <span class="font-medium text-blue-600">
                    ⏱️ {formatCredits(exchange.credits)}
                  </span>
                  <span>👤 {exchange.provider_name}</span>
                  {#if exchange.requester_name}
                    <span>➡️ {exchange.requester_name}</span>
                  {/if}
                  <span>📅 {formatDateShort(exchange.offered_at)}</span>
                  {#if exchange.provider_rating || exchange.requester_rating}
                    <span>
                      {formatRating(exchange.provider_rating || exchange.requester_rating)}
                    </span>
                  {/if}
                </div>
              </div>

              <!-- Actions -->
              <div class="ml-4 flex-shrink-0 space-y-2">
                <a
                  href={`/exchange-detail?id=${exchange.id}`}
                  data-testid="exchange-view-btn"
                  class="block w-full text-center px-4 py-2 border border-gray-300 rounded-md text-sm font-medium text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {$_('common.view_details')}
                </a>

                {#if canRequest(exchange)}
                  <button
                    type="button"
                    on:click={() => handleRequest(exchange.id)}
                    data-testid="exchange-request-btn"
                    class="w-full px-4 py-2 border border-transparent rounded-md text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
                  >
                    {$_('exchanges.request_exchange')}
                  </button>
                {/if}
              </div>
            </div>
          </li>
        {/each}
      </ul>
    </div>

    <!-- Results Count -->
    <p class="text-sm text-gray-600 text-center">
      {$_('exchanges.results_count', { values: { count: filteredExchanges.length } })}
    </p>
  {/if}
</div>
