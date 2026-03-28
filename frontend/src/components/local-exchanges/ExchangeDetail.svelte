<script lang="ts">
  import { _ } from '../../lib/i18n';
  import {
    localExchangesApi,
    type LocalExchange,
    ExchangeStatus,
    exchangeTypeLabels,
    exchangeTypeIcons,
    exchangeStatusLabels,
    exchangeStatusColors,
    formatCredits,
    formatRating,
  } from '../../lib/api/local-exchanges';
  import { toast } from '../../stores/toast';
  import { formatDateTime } from "../../lib/utils/date.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  export let exchange: LocalExchange;
  export let currentUserId: string = '';

  let actionLoading = false;
  let ratingValue = 0;
  let showRatingForm = false;
  let cancelReason = '';
  let showCancelForm = false;

  $: isProvider = exchange.provider_id === currentUserId;
  $: isRequester = exchange.requester_id === currentUserId;
  $: statusColors = exchangeStatusColors[exchange.status];

  async function handleRequest() {
    if (!confirm($_('exchanges.confirm_request'))) return;
    const result = await withErrorHandling({
      action: () => localExchangesApi.request(exchange.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.request_success'),
      errorMessage: $_('exchanges.request_error'),
    });
    if (result) exchange = result;
  }

  async function handleStart() {
    if (!confirm($_('exchanges.confirm_start'))) return;
    const result = await withErrorHandling({
      action: () => localExchangesApi.start(exchange.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.start_success'),
      errorMessage: $_('exchanges.start_error'),
    });
    if (result) exchange = result;
  }

  async function handleComplete() {
    if (!confirm($_('exchanges.confirm_complete'))) return;
    const result = await withErrorHandling({
      action: () => localExchangesApi.complete(exchange.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.complete_success'),
      errorMessage: $_('exchanges.complete_error'),
    });
    if (result) exchange = result;
  }

  async function handleCancel() {
    if (!cancelReason.trim()) {
      toast.error($_('exchanges.reason_required'));
      return;
    }
    const result = await withErrorHandling({
      action: () => localExchangesApi.cancel(exchange.id, { reason: cancelReason }),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.cancel_success'),
      errorMessage: $_('exchanges.cancel_error'),
    });
    if (result) {
      exchange = result;
      showCancelForm = false;
    }
  }

  async function handleRate(asProvider: boolean) {
    if (ratingValue < 1 || ratingValue > 5) {
      toast.error($_('exchanges.rating_required'));
      return;
    }
    const result = await withErrorHandling({
      action: () => asProvider
        ? localExchangesApi.rateRequester(exchange.id, { rating: ratingValue })
        : localExchangesApi.rateProvider(exchange.id, { rating: ratingValue }),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.rating_saved'),
      errorMessage: $_('exchanges.rating_error'),
    });
    if (result) {
      if (asProvider) {
        exchange.requester_rating = ratingValue;
      } else {
        exchange.provider_rating = ratingValue;
      }
      showRatingForm = false;
      ratingValue = 0;
    }
  }

  async function handleDelete() {
    if (!confirm($_('exchanges.confirm_delete'))) return;
    await withErrorHandling({
      action: () => localExchangesApi.delete(exchange.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('exchanges.delete_success'),
      errorMessage: $_('exchanges.delete_error'),
      onSuccess: () => { window.location.href = '/exchanges'; },
    });
  }

  function canRate(): { canRateProvider: boolean; canRateRequester: boolean } {
    if (exchange.status !== ExchangeStatus.Completed) return { canRateProvider: false, canRateRequester: false };
    return {
      canRateProvider: isRequester && !exchange.provider_rating,
      canRateRequester: isProvider && !exchange.requester_rating,
    };
  }
</script>

<div class="space-y-6" data-testid="exchange-detail">
  <!-- Header Card -->
  <div class="bg-white shadow-md rounded-lg p-6" data-testid="exchange-detail-header">
    <div class="flex items-start justify-between">
      <div class="flex items-start gap-4">
        <span class="text-4xl">{exchangeTypeIcons[exchange.exchange_type]}</span>
        <div>
          <div class="flex items-center gap-3 mb-2">
            <h2 class="text-2xl font-bold text-gray-900">{exchange.title}</h2>
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusColors.bg} {statusColors.text}">
              {exchangeStatusLabels[exchange.status]}
            </span>
          </div>
          <p class="text-sm text-gray-500">
            {exchangeTypeLabels[exchange.exchange_type]} - {formatCredits(exchange.credits)}
          </p>
        </div>
      </div>
    </div>

    <p class="mt-4 text-gray-700">{exchange.description}</p>

    <!-- Metadata Grid -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
      <div class="p-3 bg-blue-50 rounded-lg">
        <div class="text-xs text-blue-600 font-medium">{$_('exchanges.provider')}</div>
        <div class="text-sm text-blue-900 font-medium">{exchange.provider_name}</div>
        {#if isProvider}
          <span class="text-xs text-blue-600">({$_('common.you')})</span>
        {/if}
      </div>
      <div class="p-3 bg-green-50 rounded-lg">
        <div class="text-xs text-green-600 font-medium">{$_('exchanges.requester')}</div>
        <div class="text-sm text-green-900 font-medium">
          {exchange.requester_name || $_('exchanges.pending')}
        </div>
        {#if isRequester}
          <span class="text-xs text-green-600">({$_('common.you')})</span>
        {/if}
      </div>
      <div class="p-3 bg-amber-50 rounded-lg">
        <div class="text-xs text-amber-600 font-medium">{$_('exchanges.credits')}</div>
        <div class="text-sm text-amber-900 font-bold">{formatCredits(exchange.credits)}</div>
      </div>
      <div class="p-3 bg-purple-50 rounded-lg">
        <div class="text-xs text-purple-600 font-medium">{$_('exchanges.created_at')}</div>
        <div class="text-sm text-purple-900">{formatDateTime(exchange.created_at)}</div>
      </div>
    </div>

    <!-- Timeline -->
    <div class="mt-6 border-t border-gray-200 pt-4">
      <h4 class="text-sm font-medium text-gray-700 mb-3">{$_('exchanges.history')}</h4>
      <div class="space-y-2 text-sm">
        <div class="flex items-center gap-2">
          <span class="w-2 h-2 rounded-full bg-green-500"></span>
          <span class="text-gray-600">{$_('exchanges.offered_at', { date: formatDate(exchange.offered_at) })}</span>
        </div>
        {#if exchange.requested_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-blue-500"></span>
            <span class="text-gray-600">{$_('exchanges.requested_at', { date: formatDate(exchange.requested_at) })}</span>
          </div>
        {/if}
        {#if exchange.started_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-yellow-500"></span>
            <span class="text-gray-600">{$_('exchanges.started_at', { date: formatDate(exchange.started_at) })}</span>
          </div>
        {/if}
        {#if exchange.completed_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-green-600"></span>
            <span class="text-gray-600">{$_('exchanges.completed_at', { date: formatDate(exchange.completed_at) })}</span>
          </div>
        {/if}
        {#if exchange.cancelled_at}
          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-red-500"></span>
            <span class="text-gray-600">{$_('exchanges.cancelled_at', { date: formatDate(exchange.cancelled_at) })}</span>
            {#if exchange.cancellation_reason}
              <span class="text-gray-500">- {exchange.cancellation_reason}</span>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  </div>

  <!-- Ratings Card (if completed) -->
  {#if exchange.status === ExchangeStatus.Completed}
    <div class="bg-white shadow-md rounded-lg p-6" data-testid="exchange-ratings-card">
      <h3 class="text-lg font-medium text-gray-900 mb-4">{$_('exchanges.ratings')}</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
        <div class="p-4 bg-gray-50 rounded-lg">
          <div class="text-sm font-medium text-gray-700 mb-1">{$_('exchanges.provider_rating')}</div>
          <div class="text-lg">{formatRating(exchange.provider_rating)}</div>
        </div>
        <div class="p-4 bg-gray-50 rounded-lg">
          <div class="text-sm font-medium text-gray-700 mb-1">{$_('exchanges.requester_rating')}</div>
          <div class="text-lg">{formatRating(exchange.requester_rating)}</div>
        </div>
      </div>

      {#if canRate().canRateProvider || canRate().canRateRequester}
        {#if !showRatingForm}
          <button
            on:click={() => { showRatingForm = true; }}
            class="mt-4 px-4 py-2 bg-amber-600 text-white text-sm rounded-md hover:bg-amber-700"
          >
            {$_('exchanges.rate', { type: canRate().canRateProvider ? $_('exchanges.provider') : $_('exchanges.requester') })}
          </button>
        {:else}
          <div class="mt-4 p-4 border border-amber-200 rounded-lg bg-amber-50">
            <div class="text-sm font-medium text-gray-700 mb-2">
              {$_('exchanges.your_rating')} ({canRate().canRateProvider ? $_('exchanges.provider') : $_('exchanges.requester')})
            </div>
            <div class="flex items-center gap-2 mb-3">
              {#each [1, 2, 3, 4, 5] as star}
                <button
                  type="button"
                  on:click={() => ratingValue = star}
                  class="text-3xl transition-colors {ratingValue >= star ? 'text-yellow-400' : 'text-gray-300'} hover:text-yellow-300"
                >
                  &#9733;
                </button>
              {/each}
            </div>
            <div class="flex gap-2">
              <button
                on:click={() => handleRate(canRate().canRateRequester)}
                disabled={actionLoading || ratingValue === 0}
                class="px-4 py-2 bg-amber-600 text-white text-sm rounded-md hover:bg-amber-700 disabled:opacity-50"
              >
                {$_('common.confirm')}
              </button>
              <button
                on:click={() => { showRatingForm = false; ratingValue = 0; }}
                class="px-4 py-2 bg-gray-200 text-gray-700 text-sm rounded-md hover:bg-gray-300"
              >
                {$_('common.cancel')}
              </button>
            </div>
          </div>
        {/if}
      {/if}
    </div>
  {/if}

  <!-- Actions Card -->
  <div class="bg-white shadow-md rounded-lg p-6" data-testid="exchange-actions-card">
    <h3 class="text-lg font-medium text-gray-900 mb-4">{$_('common.actions')}</h3>
    <div class="flex flex-wrap gap-3">
      {#if exchange.status === ExchangeStatus.Offered && !isProvider}
        <button
          on:click={handleRequest}
          disabled={actionLoading}
          data-testid="exchange-request-btn"
          class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
        >
          {$_('exchanges.request_exchange')}
        </button>
      {/if}

      {#if exchange.status === ExchangeStatus.Requested && isProvider}
        <button
          on:click={handleStart}
          disabled={actionLoading}
          data-testid="exchange-start-btn"
          class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
        >
          {$_('exchanges.accept_and_start')}
        </button>
      {/if}

      {#if exchange.status === ExchangeStatus.InProgress && isProvider}
        <button
          on:click={handleComplete}
          disabled={actionLoading}
          data-testid="exchange-complete-btn"
          class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
        >
          {$_('exchanges.mark_completed')}
        </button>
      {/if}

      {#if exchange.status !== ExchangeStatus.Completed && exchange.status !== ExchangeStatus.Cancelled}
        {#if !showCancelForm}
          <button
            on:click={() => showCancelForm = true}
            data-testid="exchange-cancel-btn"
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200"
          >
            {$_('common.cancel')}
          </button>
        {:else}
          <div class="w-full p-4 border border-red-200 rounded-lg bg-red-50">
            <label class="block text-sm font-medium text-red-800 mb-1">{$_('exchanges.cancellation_reason')}</label>
            <textarea
              bind:value={cancelReason}
              rows="2"
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-red-500 focus:ring-red-500 text-sm"
              placeholder={$_('exchanges.reason_placeholder')}
            ></textarea>
            <div class="flex gap-2 mt-2">
              <button
                on:click={handleCancel}
                disabled={actionLoading}
                class="px-4 py-2 bg-red-600 text-white text-sm rounded-md hover:bg-red-700 disabled:opacity-50"
              >
                {$_('exchanges.confirm_cancellation')}
              </button>
              <button
                on:click={() => { showCancelForm = false; cancelReason = ''; }}
                class="px-4 py-2 bg-gray-200 text-gray-700 text-sm rounded-md hover:bg-gray-300"
              >
                {$_('common.back')}
              </button>
            </div>
          </div>
        {/if}
      {/if}

      {#if exchange.status === ExchangeStatus.Offered && isProvider}
        <button
          on:click={handleDelete}
          disabled={actionLoading}
          data-testid="exchange-delete-btn"
          class="px-4 py-2 bg-gray-100 text-gray-700 text-sm font-medium rounded-md hover:bg-gray-200"
        >
          {$_('exchanges.delete_offer')}
        </button>
      {/if}
    </div>
  </div>
</div>
