<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import type { Expense, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';
  import InvoiceForm from './InvoiceForm.svelte';
  import { formatDate } from '../lib/utils/date.utils';
  import { formatCurrency } from '../lib/utils/finance.utils';
  import { withLoadingState } from '../lib/utils/error.utils';

  let { buildingId = null }: {
    buildingId?: string | null;
  } = $props();

  // Modal state for creating new invoice
  let showCreateModal = $state(false);

  let expenses = $state<Expense[]>([]);
  let loading = $state(true);
  let error = $state('');

  // Pagination state
  let currentPage = $state(1);
  let perPage = $state(20);
  let totalItems = $state(0);
  let totalPages = $state(0);

  $effect(() => {
    loadExpenses();

    // Listen for page show events to reload data when navigating back (client-side only)
    if (typeof window !== 'undefined') {
      window.addEventListener('pageshow', handlePageShow);
      window.addEventListener('focus', handleWindowFocus);
    }

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('pageshow', handlePageShow);
        window.removeEventListener('focus', handleWindowFocus);
      }
    };
  });

  function handlePageShow(event: PageTransitionEvent) {
    if (event.persisted) {
      loadExpenses();
    }
  }

  function handleWindowFocus() {
    loadExpenses();
  }

  async function loadExpenses() {
    await withLoadingState({
      action: async () => {
        if (buildingId) {
          const response = await api.get<Expense[]>(`/buildings/${buildingId}/expenses`);
          return { type: 'building' as const, data: response };
        } else {
          const endpoint = `/expenses?page=${currentPage}&per_page=${perPage}`;
          const response = await api.get<PageResponse<Expense>>(endpoint);
          return { type: 'paginated' as const, data: response };
        }
      },
      setLoading: (v: boolean) => loading = v,
      setError: (v: string) => error = v,
      errorMessage: $_('expenses.loadError'),
      onSuccess: (result: any) => {
        if (result.type === 'building') {
          expenses = result.data;
          totalItems = result.data.length;
          totalPages = 1;
          currentPage = 1;
        } else {
          expenses = result.data.data;
          totalItems = result.data.pagination.total_items;
          totalPages = result.data.pagination.total_pages;
          currentPage = result.data.pagination.current_page;
          perPage = result.data.pagination.per_page;
        }
      },
    });
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadExpenses();
  }

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'paid': { class: 'bg-green-100 text-green-800', label: $_('expenses.statuses.paid') },
      'pending': { class: 'bg-yellow-100 text-yellow-800', label: $_('expenses.statuses.pending') },
      'overdue': { class: 'bg-red-100 text-red-800', label: $_('expenses.statuses.overdue') },
      'cancelled': { class: 'bg-gray-100 text-gray-800', label: $_('expenses.statuses.cancelled') }
    };
    return badges[status?.toLowerCase()] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function getApprovalBadge(approvalStatus: string): { class: string; label: string; emoji: string } {
    const badges: Record<string, { class: string; label: string; emoji: string }> = {
      'draft': { class: 'bg-gray-100 text-gray-700', label: $_('expenses.approval.draft'), emoji: '📝' },
      'pending_approval': { class: 'bg-blue-100 text-blue-800', label: $_('expenses.approval.pendingApproval'), emoji: '⏳' },
      'approved': { class: 'bg-green-100 text-green-800', label: $_('expenses.approval.approved'), emoji: '✅' },
      'rejected': { class: 'bg-red-100 text-red-800', label: $_('expenses.approval.rejected'), emoji: '❌' }
    };
    return badges[approvalStatus] || badges['draft'];
  }

  function handleInvoiceSaved(_invoice: any) {
    showCreateModal = false;
    loadExpenses();
  }

  function handleCancel() {
    showCreateModal = false;
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} dépense{totalItems !== 1 ? 's' : ''}
    </p>
    <button
      onclick={() => showCreateModal = true}
      class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition font-medium flex items-center gap-2"
      data-testid="create-button"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4"/>
      </svg>
      {$_('expenses.createInvoice')}
    </button>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8" data-testid="loading-spinner">{$_('common.loading')}</p>
  {:else if expenses.length === 0}
    <p class="text-center text-gray-600 py-8">
      {$_('expenses.noExpenses')}
    </p>
  {:else}
    <div class="grid gap-4">
      {#each expenses as expense (expense.id)}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition" data-testid="expense-card">
          <div class="flex justify-between items-start">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-2 flex-wrap">
                <h3 class="text-lg font-semibold text-gray-900">
                  {expense.description}
                </h3>
                <span class="text-xs px-2 py-1 rounded-full {getStatusBadge(expense.payment_status).class}" data-testid="status-badge">
                  {getStatusBadge(expense.payment_status).label}
                </span>
                {#if expense.approval_status}
                  {@const approvalBadge = getApprovalBadge(expense.approval_status)}
                  <span class="text-xs px-2 py-1 rounded-full {approvalBadge.class}">
                    {approvalBadge.emoji} {approvalBadge.label}
                  </span>
                {/if}
              </div>
              <p class="text-gray-600 text-sm">
                📁 {$_(`invoices.category_${expense.category?.toLowerCase()}`) || expense.category}
              </p>
              <p class="text-gray-500 text-sm">
                📅 {formatDate(expense.expense_date)}
              </p>
              {#if expense.supplier}
                <p class="text-gray-500 text-sm">
                  🏢 {expense.supplier}
                </p>
              {/if}
            </div>
            <div class="text-right">
              <p class="text-xl font-bold text-gray-900">
                {formatCurrency(expense.amount)}
              </p>
              <a href="/expense-detail?id={expense.id}" class="text-primary-600 hover:text-primary-700 text-sm font-medium mt-2 inline-block" data-testid="details-link">
                {$_('common.details')} →
              </a>
            </div>
          </div>
        </div>
      {/each}
    </div>

    {#if totalPages > 1}
      <Pagination
        currentPage={currentPage}
        totalPages={totalPages}
        totalItems={totalItems}
        perPage={perPage}
        onPageChange={handlePageChange}
      />
    {/if}
  {/if}
</div>

<!-- Modal pour créer une facture -->
{#if showCreateModal}
  <button type="button" aria-label={$_('common.closeModal')} class="fixed inset-0 bg-black bg-opacity-50 z-40 cursor-default" onclick={handleCancel}></button>
  <div class="fixed inset-0 flex items-center justify-center z-50 p-4 pointer-events-none" role="dialog" aria-modal="true">
    <div class="bg-white rounded-lg shadow-xl max-w-4xl w-full max-h-[90vh] overflow-y-auto pointer-events-auto" role="presentation">
      <div class="sticky top-0 bg-white border-b border-gray-200 px-6 py-4 flex justify-between items-center">
        <h2 class="text-2xl font-bold text-gray-900">{$_('expenses.createInvoice')}</h2>
        <button
          onclick={handleCancel}
          aria-label={$_('common.close')}
          class="text-gray-400 hover:text-gray-600 transition"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"/>
          </svg>
        </button>
      </div>
      <div class="p-6">
        <InvoiceForm
          buildingId={buildingId || ''}
          onSaved={handleInvoiceSaved}
          onCancel={handleCancel}
        />
      </div>
    </div>
  </div>
{/if}
