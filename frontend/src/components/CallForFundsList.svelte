<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { callForFundsApi } from '../lib/api';
  import { toast } from '../stores/toast';
  import { formatDate } from "../lib/utils/date.utils";
  import { formatCurrency } from "../lib/utils/finance.utils";
  import { withErrorHandling } from "../lib/utils/error.utils";

  export let buildingId: string | undefined = undefined;
  export let statusFilter: string | undefined = undefined;
  export let onCreate: () => void = () => {};

  let calls: any[] = [];
  let filteredCalls: any[] = [];
  let loading = true;

  $: {
    if (statusFilter && statusFilter !== 'all') {
      if (statusFilter === 'overdue') {
        filteredCalls = calls.filter(c => c.is_overdue);
      } else {
        filteredCalls = calls.filter(c => c.status === statusFilter);
      }
    } else {
      filteredCalls = calls;
    }
  }

  onMount(async () => {
    await loadCalls();
  });

  async function loadCalls() {
    loading = true;
    const result = await withErrorHandling({
      action: () => callForFundsApi.list(buildingId),
      errorMessage: $_('callForFunds.loadError'),
    });
    if (result) calls = result;
    loading = false;
  }

  async function handleSend(id: string) {
    if (!confirm($_('callForFunds.sendConfirm'))) return;
    const result = await withErrorHandling({
      action: () => callForFundsApi.send(id),
      successMessage: $_('callForFunds.sendSuccess', { values: { count: 0 } }),
      errorMessage: $_('callForFunds.sendError'),
    });
    if (result) await loadCalls();
  }

  async function handleCancel(id: string) {
    if (!confirm($_('callForFunds.cancelConfirm'))) return;
    const result = await withErrorHandling({
      action: () => callForFundsApi.cancel(id),
      successMessage: $_('callForFunds.cancelled'),
      errorMessage: $_('callForFunds.cancelError'),
    });
    if (result !== undefined) await loadCalls();
  }

  async function handleDelete(id: string) {
    if (!confirm($_('callForFunds.deleteConfirm'))) return;
    const result = await withErrorHandling({
      action: () => callForFundsApi.delete(id),
      successMessage: $_('callForFunds.deleted'),
      errorMessage: $_('callForFunds.deleteError'),
    });
    if (result !== undefined) await loadCalls();
  }

  function getStatusBadgeClass(status: string): string {
    const classes: Record<string, string> = {
      draft: 'bg-gray-100 text-gray-800',
      sent: 'bg-blue-100 text-blue-800',
      partial: 'bg-yellow-100 text-yellow-800',
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-red-100 text-red-800',
    };
    return classes[status] || 'bg-gray-100 text-gray-800';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      draft: 'Brouillon',
      sent: 'Envoyé',
      partial: 'Partiellement payé',
      completed: 'Complété',
      cancelled: 'Annulé',
    };
    return labels[status] || status;
  }

  function getContributionTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      regular: 'Charges régulières',
      extraordinary: 'Charges extraordinaires',
      advance: 'Avance',
      adjustment: 'Régularisation',
    };
    return labels[type] || type;
  }
</script>

<div class="space-y-4" data-testid="call-for-funds-list">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">{$_('callForFunds.title')}</h2>
    <button
      on:click={onCreate}
      class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
    >
      + {$_('callForFunds.new')}
    </button>
  </div>

  {#if loading}
    <div class="text-center py-8">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      <p class="mt-2 text-gray-600">{$_('common.loading')}</p>
    </div>
  {:else if filteredCalls.length === 0}
    <div class="text-center py-12 bg-gray-50 rounded-lg">
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
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <p class="mt-2 text-gray-600">{$_('callForFunds.none')}</p>
      <button
        on:click={onCreate}
        class="mt-4 text-blue-600 hover:text-blue-800"
      >
        {$_('callForFunds.createFirst')}
      </button>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.title')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.type')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.amount')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.callDate')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.dueDate')}
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('callForFunds.status')}
            </th>
            <th scope="col" class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              {$_('common.actions')}
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredCalls as call (call.id)}
            <tr class:bg-red-50={call.is_overdue}>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">{call.title}</div>
                <div class="text-sm text-gray-500">{call.description}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {getContributionTypeLabel(call.contribution_type)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {formatCurrency(call.total_amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.call_date)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.due_date)}
                {#if call.is_overdue}
                  <span class="ml-2 px-2 py-1 text-xs font-semibold rounded-full bg-red-100 text-red-800">
                    {$_('callForFunds.overdue')}
                  </span>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getStatusBadgeClass(call.status)}">
                  {getStatusLabel(call.status)}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                {#if call.status === 'draft'}
                  <button
                    on:click={() => handleSend(call.id)}
                    class="text-blue-600 hover:text-blue-900"
                    title={$_('callForFunds.sendTitle')}
                  >
                    {$_('callForFunds.send')}
                  </button>
                  <button
                    on:click={() => handleDelete(call.id)}
                    class="text-red-600 hover:text-red-900"
                    title={$_('callForFunds.deleteTitle')}
                  >
                    {$_('common.delete')}
                  </button>
                {:else if call.status === 'sent' || call.status === 'partial'}
                  <button
                    on:click={() => handleCancel(call.id)}
                    class="text-orange-600 hover:text-orange-900"
                    title={$_('callForFunds.cancelTitle')}
                  >
                    {$_('common.cancel')}
                  </button>
                  <a
                    href="/owner-contributions?call_for_funds_id={call.id}"
                    class="text-green-600 hover:text-green-900"
                    title={$_('callForFunds.viewContributions')}
                  >
                    {$_('callForFunds.contributions')}
                  </a>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
