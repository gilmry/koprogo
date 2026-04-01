<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    quotesApi,
    type Quote,
    type CreateQuoteDto,
    QuoteStatus,
  } from '../../lib/api/quotes';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import QuoteStatusBadge from './QuoteStatusBadge.svelte';
  import QuoteDetail from './QuoteDetail.svelte';
  import { withLoadingState, withErrorHandling } from '../../lib/utils/error.utils';
  import { formatDate } from '../../lib/utils/date.utils';
  import { formatAmount } from '../../lib/utils/finance.utils';

  export let buildingId: string;

  let quotes: Quote[] = [];
  let filteredQuotes: Quote[] = [];
  let loading = true;
  let error = '';
  let statusFilter: QuoteStatus | 'all' = 'all';
  let expandedId: string | null = null;
  let showCreateForm = false;
  let createLoading = false;

  let compareMode = false;
  let selectedForCompare: Set<string> = new Set();

  let newContractorId = '';
  let newProjectTitle = '';
  let newProjectDescription = '';
  let newWorkCategory = '';

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;

  onMount(async () => {
    await loadQuotes();
  });

  async function loadQuotes() {
    await withLoadingState({
      action: () => quotesApi.listByBuilding(buildingId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      errorMessage: $_("quotes.list.loadingError"),
      onSuccess: (data) => {
        quotes = data;
        applyFilters();
      },
    });
  }

  function applyFilters() {
    filteredQuotes = quotes.filter(q => {
      if (statusFilter === 'all') return true;
      return q.status === statusFilter;
    });
  }

  $: if (statusFilter) applyFilters();

  function formatQuoteAmount(amountCents: number | undefined): string {
    if (!amountCents) return '-';
    return formatAmount(amountCents);
  }

  function toggleExpand(id: string) {
    expandedId = expandedId === id ? null : id;
  }

  function toggleCompareSelect(id: string) {
    if (selectedForCompare.has(id)) {
      selectedForCompare.delete(id);
    } else {
      selectedForCompare.add(id);
    }
    selectedForCompare = selectedForCompare;
  }

  function goToCompare() {
    if (selectedForCompare.size < 2) {
      toast.error($_("quotes.list.selectAtLeast2"));
      return;
    }
    const ids = Array.from(selectedForCompare).join(',');
    window.location.href = `/quotes/compare?ids=${ids}`;
  }

  async function handleCreate() {
    if (!newContractorId || !newProjectTitle || !newWorkCategory) {
      toast.error($_("quotes.list.fillRequired"));
      return;
    }

    await withErrorHandling({
      action: async () => {
        const data: CreateQuoteDto = {
          building_id: buildingId,
          contractor_id: newContractorId,
          project_title: newProjectTitle,
          project_description: newProjectDescription,
          work_category: newWorkCategory,
        };
        return quotesApi.create(data);
      },
      setLoading: (v) => createLoading = v,
      successMessage: $_("quotes.list.createSuccess"),
      errorMessage: $_("quotes.list.createError"),
      onSuccess: async () => {
        showCreateForm = false;
        newContractorId = '';
        newProjectTitle = '';
        newProjectDescription = '';
        newWorkCategory = '';
        await loadQuotes();
      },
    });
  }

  function handleQuoteUpdated(event: CustomEvent<Quote>) {
    const updated = event.detail;
    quotes = quotes.map(q => q.id === updated.id ? updated : q);
    applyFilters();
  }

  function handleQuoteDeleted(quoteId: string) {
    quotes = quotes.filter(q => q.id !== quoteId);
    expandedId = null;
    applyFilters();
  }

  // Status counts
  $: statusCounts = {
    total: quotes.length,
    requested: quotes.filter(q => q.status === QuoteStatus.Requested).length,
    received: quotes.filter(q => q.status === QuoteStatus.Received).length,
    underReview: quotes.filter(q => q.status === QuoteStatus.UnderReview).length,
    accepted: quotes.filter(q => q.status === QuoteStatus.Accepted).length,
    rejected: quotes.filter(q => q.status === QuoteStatus.Rejected).length,
  };
</script>

<div class="bg-white shadow-md rounded-lg" data-testid="quote-list">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">
          {$_("quotes.list.title")}
        </h3>
        <p class="mt-1 text-sm text-gray-500">
          {$_("quotes.list.description")}
        </p>
      </div>
      {#if isAdmin}
        <div class="flex gap-2">
          {#if compareMode}
            <button on:click={goToCompare}
              class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors disabled:opacity-50"
              disabled={selectedForCompare.size < 2}>
              {$_("quotes.list.compare")} ({selectedForCompare.size})
            </button>
            <button on:click={() => { compareMode = false; selectedForCompare = new Set(); }}
              class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
              {$_("common.cancel")}
            </button>
          {:else}
            <button on:click={() => compareMode = true}
              data-testid="compare-quotes-button"
              class="px-3 py-1.5 bg-blue-100 text-blue-700 rounded-lg text-sm font-medium hover:bg-blue-200 transition-colors"
              disabled={quotes.length < 2}>
              {$_("quotes.list.compare")}
            </button>
            <button on:click={() => showCreateForm = !showCreateForm}
              data-testid="request-quote-button"
              class="px-3 py-1.5 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 transition-colors">
              + {$_("quotes.list.requestQuote")}
            </button>
          {/if}
        </div>
      {/if}
    </div>
  </div>

  <!-- Status summary -->
  {#if quotes.length > 0}
    <div class="px-4 py-2 bg-gray-50 border-b border-gray-200 flex flex-wrap gap-3 text-xs text-gray-600">
      <span>{statusCounts.total} {$_("common.total")}</span>
      {#if statusCounts.requested > 0}<span class="text-blue-600">{statusCounts.requested} {$_("quotes.status.requested").toLowerCase()}</span>{/if}
      {#if statusCounts.received > 0}<span class="text-purple-600">{statusCounts.received} {$_("quotes.status.received").toLowerCase()}</span>{/if}
      {#if statusCounts.underReview > 0}<span class="text-yellow-600">{statusCounts.underReview} {$_("quotes.status.underReview").toLowerCase()}</span>{/if}
      {#if statusCounts.accepted > 0}<span class="text-green-600">{statusCounts.accepted} {$_("quotes.status.accepted").toLowerCase()}</span>{/if}
      {#if statusCounts.rejected > 0}<span class="text-red-600">{statusCounts.rejected} {$_("quotes.status.rejected").toLowerCase()}</span>{/if}
    </div>
  {/if}

  <!-- Filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex items-center space-x-4">
      <label class="text-sm font-medium text-gray-700">{$_("common.status")}:</label>
      <select bind:value={statusFilter}
        data-testid="quote-status-filter"
        class="text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
        <option value="all">{$_("common.all")}</option>
        <option value={QuoteStatus.Requested}>{$_("quotes.status.requested")}</option>
        <option value={QuoteStatus.Received}>{$_("quotes.status.received")}</option>
        <option value={QuoteStatus.UnderReview}>{$_("quotes.status.underReview")}</option>
        <option value={QuoteStatus.Accepted}>{$_("quotes.status.accepted")}</option>
        <option value={QuoteStatus.Rejected}>{$_("quotes.status.rejected")}</option>
        <option value={QuoteStatus.Expired}>{$_("quotes.status.expired")}</option>
        <option value={QuoteStatus.Withdrawn}>{$_("quotes.status.withdrawn")}</option>
      </select>
    </div>
  </div>

  <!-- Create form -->
  {#if showCreateForm}
    <div class="p-4 border-b border-gray-200 bg-amber-50">
      <h4 class="text-sm font-semibold text-gray-900 mb-3">{$_("quotes.list.newRequest")}</h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <div>
          <label for="contractorId" class="block text-xs text-gray-600 mb-1">{$_("quotes.list.contractorId")} *</label>
          <input id="contractorId" type="text" bind:value={newContractorId} placeholder={$_("quotes.list.contractorUUID")}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div>
          <label for="workCategory" class="block text-xs text-gray-600 mb-1">{$_("quotes.list.workCategory")} *</label>
          <select id="workCategory" bind:value={newWorkCategory}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500">
            <option value="">{$_("common.select")}</option>
            <option value="plumbing">{$_("quotes.list.plumbing")}</option>
            <option value="electrical">{$_("quotes.list.electrical")}</option>
            <option value="heating">{$_("quotes.list.heating")}</option>
            <option value="painting">{$_("quotes.list.painting")}</option>
            <option value="roofing">{$_("quotes.list.roofing")}</option>
            <option value="facade">{$_("quotes.list.facade")}</option>
            <option value="elevator">{$_("quotes.list.elevator")}</option>
            <option value="general">{$_("common.general")}</option>
          </select>
        </div>
        <div class="md:col-span-2">
          <label for="projectTitle" class="block text-xs text-gray-600 mb-1">{$_("quotes.list.projectTitle")} *</label>
          <input id="projectTitle" type="text" bind:value={newProjectTitle} placeholder={$_("quotes.list.projectTitlePlaceholder")}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500" />
        </div>
        <div class="md:col-span-2">
          <label for="projectDesc" class="block text-xs text-gray-600 mb-1">{$_("quotes.list.projectDescription")}</label>
          <textarea id="projectDesc" rows="2" bind:value={newProjectDescription} placeholder={$_("quotes.list.workDetails")}
            class="w-full text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"></textarea>
        </div>
      </div>
      <div class="mt-3 flex gap-2">
        <button on:click={handleCreate} disabled={createLoading}
          class="px-4 py-2 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors">
          {createLoading ? $_("quotes.list.creating") : $_("quotes.list.createRequest")}
        </button>
        <button on:click={() => showCreateForm = false}
          class="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors">
          {$_("common.cancel")}
        </button>
      </div>
      <p class="mt-2 text-xs text-gray-400">
        {$_("quotes.list.bestPractice")}
      </p>
    </div>
  {/if}

  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">{$_("quotes.list.loading")}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadQuotes} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
        {$_("common.retry")}
      </button>
    </div>
  {:else if filteredQuotes.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_("quotes.list.notFound")}</p>
      {#if isAdmin}
        <p class="mt-2 text-sm text-gray-400">
          {$_("quotes.list.emptyMessage")}
        </p>
      {/if}
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredQuotes as quote (quote.id)}
        <li class="hover:bg-gray-50" data-testid="quote-row">
          <div class="px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between cursor-pointer" on:click={() => toggleExpand(quote.id)}>
              <div class="flex items-center space-x-3 flex-1 min-w-0">
                {#if compareMode}
                  <input type="checkbox" checked={selectedForCompare.has(quote.id)}
                    on:click|stopPropagation={() => toggleCompareSelect(quote.id)}
                    class="h-4 w-4 text-amber-600 border-gray-300 rounded focus:ring-amber-500" />
                {/if}
                <div class="flex-1 min-w-0">
                  <div class="flex items-center space-x-3 mb-1">
                    <h4 class="text-sm font-medium text-gray-900 truncate">{quote.project_title}</h4>
                    <QuoteStatusBadge status={quote.status} />
                  </div>
                  <div class="flex items-center text-sm text-gray-500 flex-wrap gap-x-4 gap-y-1">
                    <span>{quote.contractor_name || quote.contractor_id.slice(0, 8)}</span>
                    <span>{quote.work_category}</span>
                    {#if quote.amount_incl_vat_cents}
                      <span class="font-medium text-gray-700">{formatQuoteAmount(quote.amount_incl_vat_cents)}</span>
                    {/if}
                    <span class="text-xs text-gray-400">{formatDate(quote.created_at)}</span>
                  </div>
                </div>
              </div>
              <div class="ml-4">
                <svg class="h-5 w-5 text-gray-400 transition-transform {expandedId === quote.id ? 'rotate-90' : ''}"
                  fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </div>
            </div>

            {#if expandedId === quote.id}
              <div class="mt-3">
                <QuoteDetail {quote}
                  on:updated={handleQuoteUpdated}
                  on:deleted={() => handleQuoteDeleted(quote.id)} />
              </div>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}

  <!-- Belgian law notice -->
  {#if quotes.length > 0 && quotes.length < 3}
    <div class="px-4 py-3 bg-yellow-50 border-t border-yellow-200">
      <p class="text-xs text-yellow-800">
        <strong>{$_("quotes.list.bestPracticeTitle")}:</strong> {$_("quotes.list.bestPracticeMessage", { values: { count: quotes.length } })}
      </p>
    </div>
  {/if}
</div>
