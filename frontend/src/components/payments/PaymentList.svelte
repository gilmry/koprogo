<script lang="ts">
  import { onMount } from "svelte";
  import {
    paymentsApi,
    PaymentStatus,
    type Payment,
  } from "../../lib/api/payments";
  import { toast } from "../../stores/toast";
  import PaymentStatusBadge from "./PaymentStatusBadge.svelte";

  export let ownerId: string | undefined = undefined;
  export let buildingId: string | undefined = undefined;
  export let expenseId: string | undefined = undefined;

  let payments: Payment[] = [];
  let loading = true;
  let statusFilter: PaymentStatus | "all" = "all";
  let searchQuery = "";

  onMount(async () => {
    await loadPayments();
  });

  async function loadPayments() {
    try {
      loading = true;
      if (ownerId) {
        payments = await paymentsApi.listByOwner(ownerId);
      } else if (buildingId) {
        payments = await paymentsApi.listByBuilding(buildingId);
      } else if (expenseId) {
        payments = await paymentsApi.listByExpense(expenseId);
      } else {
        payments = [];
      }
    } catch (err: any) {
      toast.error(err.message || "Failed to load payments");
    } finally {
      loading = false;
    }
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

  function formatAmount(amountCents: number, currency: string): string {
    const amount = amountCents / 100;
    return new Intl.NumberFormat("nl-BE", {
      style: "currency",
      currency: currency,
    }).format(amount);
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString("nl-BE", {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function getPaymentUrl(paymentId: string): string {
    return `/payment-detail?id=${paymentId}`;
  }
</script>

<div class="bg-white shadow rounded-lg">
  <!-- Header with filters -->
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between mb-4">
      <h2 class="text-xl font-semibold text-gray-900">
        Payments
        <span class="ml-2 text-sm text-gray-500">
          ({filteredPayments.length})
        </span>
      </h2>
      <button
        on:click={loadPayments}
        class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
      >
        Refresh
      </button>
    </div>

    <!-- Filters -->
    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
      <!-- Search -->
      <div>
        <input
          type="text"
          bind:value={searchQuery}
          placeholder="Search payments..."
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        />
      </div>

      <!-- Status filter -->
      <div>
        <select
          bind:value={statusFilter}
          class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-blue-500 focus:border-blue-500"
        >
          <option value="all">All Statuses</option>
          <option value={PaymentStatus.Pending}>Pending</option>
          <option value={PaymentStatus.Processing}>Processing</option>
          <option value={PaymentStatus.RequiresAction}>Requires Action</option>
          <option value={PaymentStatus.Succeeded}>Succeeded</option>
          <option value={PaymentStatus.Failed}>Failed</option>
          <option value={PaymentStatus.Refunded}>Refunded</option>
        </select>
      </div>
    </div>
  </div>

  <!-- Payments table -->
  <div class="overflow-x-auto">
    {#if loading}
      <div class="px-6 py-12 text-center text-gray-500">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"
        ></div>
        <p class="mt-4">Loading payments...</p>
      </div>
    {:else if filteredPayments.length === 0}
      <div class="px-6 py-12 text-center text-gray-500">
        No payments found matching your filters.
      </div>
    {:else}
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              Date
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              Amount
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              Status
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              Method
            </th>
            <th
              class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
            >
              ID
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredPayments as payment (payment.id)}
            <tr class="hover:bg-gray-50">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatDate(payment.created_at)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">
                  {formatAmount(payment.amount_cents, payment.currency)}
                </div>
                {#if payment.refunded_amount_cents > 0}
                  <div class="text-xs text-purple-600">
                    Refunded: {formatAmount(
                      payment.refunded_amount_cents,
                      payment.currency,
                    )}
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
                  View
                </a>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</div>
