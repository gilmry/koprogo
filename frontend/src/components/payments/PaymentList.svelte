<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    paymentsApi,
    PaymentStatus,
    type Payment,
  } from "../../lib/api/payments";
  import { formatDateTime } from "../../lib/utils/date.utils";
  import { formatAmount } from "../../lib/utils/finance.utils";
  import { withLoadingState } from "../../lib/utils/error.utils";
  import PaymentStatusBadge from "./PaymentStatusBadge.svelte";

  export let ownerId: string | undefined = undefined;
  export let buildingId: string | undefined = undefined;
  export let expenseId: string | undefined = undefined;

  let payments: Payment[] = [];
  let loading = true;
  let error = "";
  let statusFilter: PaymentStatus | "all" = "all";
  let searchQuery = "";

  onMount(async () => {
    await loadPayments();
  });

  async function loadPayments() {
    await withLoadingState({
      action: async () => {
        if (ownerId) {
          return await paymentsApi.listByOwner(ownerId);
        } else if (buildingId) {
          return await paymentsApi.listByBuilding(buildingId);
        } else if (expenseId) {
          return await paymentsApi.listByExpense(expenseId);
        } else {
          return [];
        }
      },
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => payments = data,
      errorMessage: $_('payments.loadError'),
    });
  }

  $: filteredPayments = payments.filter((payment) => {
    if (statusFilter !== "all" && payment.status !== statusFilter) return false;
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      return (
        payment.id.toLowerCase().includes(query) ||
        payment.stripe_payment_intent_id?.toLowerCase().includes(query) ||
        payment.idempotency_key.toLowerCase().includes(query)
      );
    }
    return true;
  });

  function getPaymentUrl(paymentId: string): string {
    return `/payment-detail?id=${paymentId}`;
  }
</script>

<div class="bg-white shadow rounded-lg" data-testid="payment-list">
  <!-- Header with filters -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-gray-900">
        {$_('payments.title')}
        <span class="ml-2 text-sm text-gray-500">
          ({filteredPayments.length})
        </span>
      </h2>
      <button
        on:click={loadPayments}
        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
      >
        {$_('common.refresh')}
      </button>
    </div>

    <!-- Filters -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Search -->
      <div>
        <label for="payment-search" class="sr-only">{$_('payments.search')}</label>
        <input
          id="payment-search"
          type="text"
          bind:value={searchQuery}
          placeholder={$_('payments.searchPlaceholder')}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      <!-- Status filter -->
      <div>
        <select
          bind:value={statusFilter}
          data-testid="payment-status-filter"
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">{$_('payments.allStatuses')}</option>
          <option value={PaymentStatus.Pending}>{$_('payments.pending')}</option>
          <option value={PaymentStatus.Processing}>{$_('payments.processing')}</option>
          <option value={PaymentStatus.RequiresAction}>{$_('payments.requiresAction')}</option>
          <option value={PaymentStatus.Succeeded}>{$_('payments.succeeded')}</option>
          <option value={PaymentStatus.Failed}>{$_('payments.failed')}</option>
          <option value={PaymentStatus.Refunded}>{$_('payments.refunded')}</option>
        </select>
      </div>
    </div>
  </div>

  <!-- Payments table -->
  <div class="overflow-x-auto">
    {#if loading}
      <div class="px-6 py-12 text-center text-gray-500" data-testid="payment-list-loading">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">{$_('common.loading')}</p>
      </div>
    {:else if filteredPayments.length === 0}
      <div class="px-6 py-12 text-center text-gray-500">
        {$_('payments.noPayments')}
      </div>
    {:else}
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_('payments.date')}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_('payments.amount')}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_('payments.status')}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_('payments.method')}
            </th>
            <th
              scope="col"
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              {$_('payments.id')}
            </th>
            <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('common.actions')}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredPayments as payment (payment.id)}
            <tr class="hover:bg-gray-50" data-testid="payment-row">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatDateTime(payment.created_at)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">
                  {formatAmount(payment.amount_cents)}
                </div>
                {#if payment.refunded_amount_cents > 0}
                  <div class="text-xs text-purple-600">
                    {$_('payments.refunded')}: {formatAmount(payment.refunded_amount_cents)}
                  </div>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <PaymentStatusBadge status={payment.status} />
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {payment.payment_method_type}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-xs text-gray-500 font-mono">
                  {payment.id.slice(0, 8)}
                </div>
                {#if payment.stripe_payment_intent_id}
                  <div class="text-xs text-gray-400 font-mono">
                    {payment.stripe_payment_intent_id.slice(0, 12)}...
                  </div>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm">
                <a
                  href={getPaymentUrl(payment.id)}
                  class="text-blue-600 hover:text-blue-900"
                >
                  {$_('common.view')}
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
