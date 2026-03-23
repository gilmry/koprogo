<script lang="ts">
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { onMount } from 'svelte';

  export let ownerId: string | null = null;
  export let organizationId: string;

  let contributions: any[] = [];
  let loading = true;
  let error = '';
  let showPaymentModal = false;
  let selectedContribution: any = null;
  let paymentData = {
    payment_date: new Date().toISOString().split('T')[0],
    payment_method: 'bank_transfer',
    payment_reference: ''
  };

  async function loadContributions() {
    try {
      loading = true;
      if (ownerId) {
        contributions = await api.get(`/owner-contributions?owner_id=${ownerId}`);
      } else {
        // Load all contributions for organization
        contributions = await api.get('/owner-contributions');
      }
    } catch (err: any) {
      error = err.message;
    } finally {
      loading = false;
    }
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', {
      style: 'currency',
      currency: 'EUR'
    }).format(amount);
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric'
    });
  }

  function getStatusBadge(status: string): string {
    const badges = {
      pending: 'bg-yellow-100 text-yellow-800',
      paid: 'bg-green-100 text-green-800',
      partial: 'bg-orange-100 text-orange-800',
      cancelled: 'bg-gray-100 text-gray-800'
    };
    return badges[status] || 'bg-gray-100 text-gray-800';
  }

  function getStatusLabel(status: string): string {
    const labels = {
      pending: $_('contributions.pending'),
      paid: $_('contributions.paid'),
      partial: $_('contributions.partial'),
      cancelled: $_('contributions.cancelled')
    };
    return labels[status] || status;
  }

  function getTypeLabel(type: string): string {
    const labels = {
      regular: $_('contributions.typeRegular'),
      extraordinary: $_('contributions.typeExtraordinary'),
      advance: $_('contributions.typeAdvance'),
      adjustment: $_('contributions.typeAdjustment')
    };
    return labels[type] || type;
  }

  function openPaymentModal(contribution: any) {
    selectedContribution = contribution;
    paymentData = {
      payment_date: new Date().toISOString().split('T')[0],
      payment_method: 'bank_transfer',
      payment_reference: ''
    };
    showPaymentModal = true;
  }

  function closePaymentModal() {
    showPaymentModal = false;
    selectedContribution = null;
  }

  async function recordPayment() {
    if (!selectedContribution) return;

    try {
      const payload = {
        payment_date: new Date(paymentData.payment_date).toISOString(),
        payment_method: paymentData.payment_method,
        payment_reference: paymentData.payment_reference || null
      };

      await api.put(`/owner-contributions/${selectedContribution.id}/mark-paid`, payload);

      // Reload contributions
      await loadContributions();
      closePaymentModal();
    } catch (err: any) {
      error = err.message || 'Erreur lors de l\'enregistrement du paiement';
    }
  }

  onMount(() => {
    loadContributions();
  });
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-6 py-4 border-b border-gray-200">
    <h3 class="text-lg font-semibold text-gray-900">
      {$_('contributions.title')}
    </h3>
  </div>

  {#if loading}
    <div class="p-8 text-center text-gray-500">
      {$_('common.loading')}
    </div>
  {:else if error}
    <div class="p-4 bg-red-50 border border-red-200 text-red-700">
      {error}
    </div>
  {:else if contributions.length === 0}
    <div class="p-8 text-center text-gray-500">
      {$_('contributions.empty')}
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('contributions.date')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('common.description')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('contributions.type')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('contributions.amount')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('contributions.status')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('common.actions')}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each contributions as contribution}
            <tr class="hover:bg-gray-50">
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {formatDate(contribution.contribution_date)}
              </td>
              <td class="px-6 py-4 text-sm text-gray-900">
                {contribution.description}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                {getTypeLabel(contribution.contribution_type)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {formatCurrency(contribution.amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 py-1 inline-flex text-xs leading-5 font-semibold rounded-full {getStatusBadge(contribution.payment_status)}">
                  {getStatusLabel(contribution.payment_status)}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm">
                {#if contribution.payment_status === 'pending'}
                  <button
                    on:click={() => openPaymentModal(contribution)}
                    class="text-blue-600 hover:text-blue-900 font-medium"
                  >
                    {$_('contributions.recordPayment')}
                  </button>
                {:else if contribution.payment_date}
                  <span class="text-green-600 text-xs">
                    {$_('contributions.paidOn')} {formatDate(contribution.payment_date)}
                  </span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<!-- Payment Modal -->
{#if showPaymentModal && selectedContribution}
  <div class="fixed inset-0 bg-gray-600 bg-opacity-50 overflow-y-auto h-full w-full z-50">
    <div class="relative top-20 mx-auto p-5 border w-96 shadow-lg rounded-md bg-white">
      <div class="mt-3">
        <h3 class="text-lg font-medium text-gray-900 mb-4">
          {$_('contributions.recordPaymentTitle')}
        </h3>

        <div class="mb-4 p-3 bg-gray-50 rounded">
          <p class="text-sm text-gray-600">{selectedContribution.description}</p>
          <p class="text-lg font-bold text-gray-900 mt-1">
            {formatCurrency(selectedContribution.amount)}
          </p>
        </div>

        <form on:submit|preventDefault={recordPayment} class="space-y-4">
          <!-- Payment Date -->
          <div>
            <label for="payment_date" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('contributions.paymentDate')}
            </label>
            <input
              type="date"
              id="payment_date"
              bind:value={paymentData.payment_date}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <!-- Payment Method -->
          <div>
            <label for="payment_method" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('contributions.paymentMethod')}
            </label>
            <select
              id="payment_method"
              bind:value={paymentData.payment_method}
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="bank_transfer">{$_('contributions.methodBank')}</option>
              <option value="cash">{$_('contributions.methodCash')}</option>
              <option value="check">{$_('contributions.methodCheck')}</option>
              <option value="domiciliation">{$_('contributions.methodDomicile')}</option>
            </select>
          </div>

          <!-- Payment Reference -->
          <div>
            <label for="payment_reference" class="block text-sm font-medium text-gray-700 mb-1">
              {$_('contributions.reference')}
            </label>
            <input
              type="text"
              id="payment_reference"
              bind:value={paymentData.payment_reference}
              placeholder="Ex: REF-2025-001"
              class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            />
          </div>

          <!-- Buttons -->
          <div class="flex justify-end space-x-3 pt-4">
            <button
              type="button"
              on:click={closePaymentModal}
              class="px-4 py-2 bg-gray-200 text-gray-800 rounded-md hover:bg-gray-300"
            >
              {$_('common.cancel')}
            </button>
            <button
              type="submit"
              class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
            >
              {$_('contributions.recordButton')}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
{/if}
