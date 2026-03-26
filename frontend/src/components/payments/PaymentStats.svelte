<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { paymentsApi, type PaymentStats } from "../../lib/api/payments";
  import { formatAmount } from "../../lib/utils/finance.utils";
  import { withLoadingState } from "../../lib/utils/error.utils";

  export let ownerId: string | undefined = undefined;
  export let buildingId: string | undefined = undefined;

  let stats: PaymentStats | null = null;
  let loading = true;
  let error = "";

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    await withLoadingState({
      action: async () => {
        if (ownerId) {
          return await paymentsApi.getOwnerStats(ownerId);
        } else if (buildingId) {
          return await paymentsApi.getBuildingStats(buildingId);
        }
        return null;
      },
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => stats = data,
      errorMessage: $_('payments.loadError'),
    });
  }
</script>

<div class="bg-white shadow rounded-lg p-6" data-testid="payment-stats">
  <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('payments.statistics')}</h2>

  {#if loading}
    <div class="text-center py-8 text-gray-500" data-testid="payment-stats-loading">{$_('common.loading')}</div>
  {:else if stats}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Paid -->
      <div class="bg-blue-50 rounded-lg p-4" data-testid="stat-total-paid">
        <dt class="text-sm font-medium text-blue-700 mb-1">{$_('payments.totalPaid')}</dt>
        <dd class="text-xl font-bold text-blue-900">
          {formatAmount(stats.total_paid_cents)}
        </dd>
      </div>

      <!-- Succeeded -->
      <div class="bg-green-50 rounded-lg p-4" data-testid="stat-succeeded">
        <dt class="text-sm font-medium text-green-700 mb-1">{$_('payments.succeeded')}</dt>
        <dd class="text-xl font-bold text-green-900">
          {stats.succeeded_count}
        </dd>
      </div>

      <!-- Failed -->
      <div class="bg-red-50 rounded-lg p-4" data-testid="stat-failed">
        <dt class="text-sm font-medium text-red-700 mb-1">{$_('payments.failed')}</dt>
        <dd class="text-xl font-bold text-red-900">{stats.failed_count}</dd>
      </div>

      <!-- Net Amount (after refunds) -->
      <div class="bg-purple-50 rounded-lg p-4" data-testid="stat-net-amount">
        <dt class="text-sm font-medium text-purple-700 mb-1">{$_('payments.netAmount')}</dt>
        <dd class="text-xl font-bold text-purple-900">
          {formatAmount(stats.net_amount_cents)}
        </dd>
        {#if stats.refunded_count > 0}
          <dd class="text-xs text-purple-600 mt-1">
            ({stats.refunded_count} {$_('payments.refunds')})
          </dd>
        {/if}
      </div>
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">{$_('payments.noStats')}</div>
  {/if}
</div>
