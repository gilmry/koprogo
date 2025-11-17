<script lang="ts">
  import { onMount } from "svelte";
  import { paymentsApi, type PaymentStats } from "../../lib/api/payments";
  import { addToast } from "../../stores/toast";

  export let ownerId: string | undefined = undefined;
  export let buildingId: string | undefined = undefined;

  let stats: PaymentStats | null = null;
  let loading = true;

  onMount(async () => {
    await loadStats();
  });

  async function loadStats() {
    try {
      loading = true;
      if (ownerId) {
        stats = await paymentsApi.getOwnerStats(ownerId);
      } else if (buildingId) {
        stats = await paymentsApi.getBuildingStats(buildingId);
      }
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to load statistics",
        type: "error",
      });
    } finally {
      loading = false;
    }
  }

  function formatAmount(amountCents: number): string {
    const amount = amountCents / 100;
    return new Intl.NumberFormat("nl-BE", {
      style: "currency",
      currency: "EUR",
    }).format(amount);
  }
</script>

<div class="bg-white shadow rounded-lg p-6">
  <h2 class="text-lg font-semibold text-gray-900 mb-4">Payment Statistics</h2>

  {#if loading}
    <div class="text-center py-8 text-gray-500">Loading statistics...</div>
  {:else if stats}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Paid -->
      <div class="bg-blue-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-blue-700 mb-1">Total Paid</dt>
        <dd class="text-xl font-bold text-blue-900">
          {formatAmount(stats.total_paid_cents)}
        </dd>
      </div>

      <!-- Succeeded -->
      <div class="bg-green-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-green-700 mb-1">Succeeded</dt>
        <dd class="text-xl font-bold text-green-900">
          {stats.succeeded_count}
        </dd>
      </div>

      <!-- Failed -->
      <div class="bg-red-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-red-700 mb-1">Failed</dt>
        <dd class="text-xl font-bold text-red-900">{stats.failed_count}</dd>
      </div>

      <!-- Net Amount (after refunds) -->
      <div class="bg-purple-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-purple-700 mb-1">Net Amount</dt>
        <dd class="text-xl font-bold text-purple-900">
          {formatAmount(stats.net_amount_cents)}
        </dd>
        {#if stats.refunded_count > 0}
          <dd class="text-xs text-purple-600 mt-1">
            ({stats.refunded_count} refunds)
          </dd>
        {/if}
      </div>
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">No statistics available</div>
  {/if}
</div>
