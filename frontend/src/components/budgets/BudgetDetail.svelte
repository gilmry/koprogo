<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import { budgetsApi, type Budget, type BudgetVariance } from '../../lib/api/budgets';
  import BudgetStatusBadge from './BudgetStatusBadge.svelte';
  import { withLoadingState, withErrorHandling } from '../../lib/utils/error.utils';
  import { formatDate } from '../../lib/utils/date.utils';
  import { formatCurrency } from '../../lib/utils/finance.utils';
  import { toast } from '../../stores/toast';

  let budget: Budget | null = null;
  let variance: BudgetVariance | null = null;
  let loading = true;
  let error = '';
  let actionLoading = false;
  let budgetId = '';

  let showApproveModal = false;
  let meetingId = '';

  let showRejectModal = false;
  let rejectReason = '';

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    budgetId = params.get('id') || '';
    if (budgetId) loadBudget();
  });

  async function loadBudget() {
    await withLoadingState({
      action: async () => {
        const b = await budgetsApi.getById(budgetId);
        let v: BudgetVariance | null = null;
        if (b?.status === 'approved') {
          try { v = await budgetsApi.getVariance(budgetId); } catch { /* not available */ }
        }
        return { b, v };
      },
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      errorMessage: $_('budgets.errors.loadingFailed'),
      onSuccess: (result) => {
        budget = result.b;
        variance = result.v;
      },
    });
  }

  async function submitBudget() {
    if (!confirm($_('budgets.confirms.submitForApproval'))) return;
    const result = await withErrorHandling({
      action: () => budgetsApi.submit(budgetId),
      setLoading: (v) => actionLoading = v,
      errorMessage: $_('budgets.errors.submit'),
    });
    if (result) budget = result;
  }

  async function approveBudget() {
    if (!meetingId.trim()) {
      toast.error($_('budgets.errors.meetingIdRequired'));
      return;
    }
    const result = await withErrorHandling({
      action: () => budgetsApi.approve(budgetId, meetingId),
      setLoading: (v) => actionLoading = v,
      errorMessage: $_('budgets.errors.approve'),
    });
    if (result) {
      budget = result;
      showApproveModal = false;
    }
  }

  async function rejectBudget() {
    const result = await withErrorHandling({
      action: () => budgetsApi.reject(budgetId, rejectReason || undefined),
      setLoading: (v) => actionLoading = v,
      errorMessage: $_('budgets.errors.reject'),
    });
    if (result) {
      budget = result;
      showRejectModal = false;
    }
  }

  async function archiveBudget() {
    if (!confirm($_('budgets.confirms.archiveBudget'))) return;
    const result = await withErrorHandling({
      action: () => budgetsApi.archive(budgetId),
      setLoading: (v) => actionLoading = v,
      errorMessage: $_('budgets.errors.archive'),
    });
    if (result) budget = result;
  }

  async function deleteBudget() {
    if (!confirm($_('budgets.confirms.deleteBudget'))) return;
    await withErrorHandling({
      action: () => budgetsApi.delete(budgetId),
      errorMessage: $_('budgets.errors.delete'),
      onSuccess: () => { window.location.href = '/budgets'; },
    });
  }
</script>

{#if loading}
  <div class="flex justify-center py-12">
    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
  </div>
{:else if error}
  <div class="bg-red-50 border border-red-200 rounded-lg p-4">
    <p class="text-red-700">{error}</p>
    <button on:click={loadBudget} class="mt-2 text-sm text-red-600 underline">{$_('common.retry')}</button>
  </div>
{:else if budget}
  <div class="space-y-6" data-testid="budget-detail">
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-white">{$_('budgets.budget')} {budget.fiscal_year}</h1>
            <p class="text-primary-100 mt-1">{$_('budgets.building')}: {budget.building_id.substring(0, 8)}...</p>
          </div>
          <BudgetStatusBadge status={budget.status} />
        </div>
      </div>
    </div>

    <!-- Amounts -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4" data-testid="budget-info">
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">{$_('budgets.ordinaryBudget')}</p>
        <p class="text-2xl font-bold text-gray-900">{formatCurrency(budget.ordinary_budget)}</p>
        <p class="text-xs text-gray-500 mt-1">{$_('budgets.ordinaryBudgetHint')}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">{$_('budgets.extraordinaryBudget')}</p>
        <p class="text-2xl font-bold text-gray-900">{formatCurrency(budget.extraordinary_budget)}</p>
        <p class="text-xs text-gray-500 mt-1">{$_('budgets.extraordinaryBudgetHint')}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">{$_('budgets.totalBudget')}</p>
        <p class="text-2xl font-bold text-primary-600">{formatCurrency(budget.total_budget)}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">{$_('budgets.monthlyProvision')}</p>
        <p class="text-2xl font-bold text-green-600">{formatCurrency(budget.monthly_provision_amount)}</p>
        <p class="text-xs text-gray-500 mt-1">{formatCurrency(budget.total_budget)} / 12</p>
      </div>
    </div>

    <!-- Timeline -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('budgets.timeline')}</h2>
      <div class="space-y-3">
        <div class="flex justify-between py-2 border-b">
          <span class="text-gray-600">{$_('budgets.createdOn')}</span>
          <span class="font-medium">{formatDate(budget.created_at)}</span>
        </div>
        {#if budget.submitted_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">{$_('budgets.submittedOn')}</span>
            <span class="font-medium">{formatDate(budget.submitted_date)}</span>
          </div>
        {/if}
        {#if budget.approved_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">{$_('budgets.approvedOn')}</span>
            <span class="font-medium text-green-600">{formatDate(budget.approved_date)}</span>
          </div>
        {/if}
        {#if budget.notes}
          <div class="flex justify-between py-2">
            <span class="text-gray-600">{$_('budgets.notes')}</span>
            <span class="font-medium text-right max-w-md">{budget.notes}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Variance Analysis (only for approved budgets) -->
    {#if variance}
      <div class="bg-white rounded-lg shadow p-6" data-testid="budget-variance">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">
          {$_('budgets.varianceAnalysis')}
          {#if variance.has_overruns}
            <span class="ml-2 px-2 py-0.5 bg-red-100 text-red-800 text-xs rounded-full">{$_('budgets.overrun')}</span>
          {/if}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          <div>
            <p class="text-sm text-gray-600">{$_('budgets.ordinary')}</p>
            <p class="text-lg font-bold {variance.variance_ordinary >= 0 ? 'text-green-600' : 'text-red-600'}">
              {variance.variance_ordinary >= 0 ? '+' : ''}{formatCurrency(variance.variance_ordinary)}
              <span class="text-sm font-normal">({variance.variance_ordinary_pct.toFixed(1)}%)</span>
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600">{$_('budgets.extraordinary')}</p>
            <p class="text-lg font-bold {variance.variance_extraordinary >= 0 ? 'text-green-600' : 'text-red-600'}">
              {variance.variance_extraordinary >= 0 ? '+' : ''}{formatCurrency(variance.variance_extraordinary)}
              <span class="text-sm font-normal">({variance.variance_extraordinary_pct.toFixed(1)}%)</span>
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600">{$_('budgets.total')}</p>
            <p class="text-lg font-bold {variance.variance_total >= 0 ? 'text-green-600' : 'text-red-600'}">
              {variance.variance_total >= 0 ? '+' : ''}{formatCurrency(variance.variance_total)}
              <span class="text-sm font-normal">({variance.variance_total_pct.toFixed(1)}%)</span>
            </p>
          </div>
        </div>

        <!-- Progress bars -->
        <div class="space-y-3">
          <div>
            <div class="flex justify-between text-sm text-gray-600 mb-1">
              <span>{$_('budgets.ordinary')}: {formatCurrency(variance.actual_ordinary)} / {formatCurrency(variance.budgeted_ordinary)}</span>
              <span>{Math.round((variance.actual_ordinary / variance.budgeted_ordinary) * 100)}%</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="h-2 rounded-full {Math.min(100, (variance.actual_ordinary / variance.budgeted_ordinary) * 100) > 100 ? 'bg-red-500' : 'bg-green-500'}" style="width: {Math.min(100, (variance.actual_ordinary / variance.budgeted_ordinary) * 100)}%"></div>
            </div>
          </div>
          <div>
            <div class="flex justify-between text-sm text-gray-600 mb-1">
              <span>{$_('budgets.extraordinary')}: {formatCurrency(variance.actual_extraordinary)} / {formatCurrency(variance.budgeted_extraordinary)}</span>
              <span>{Math.round((variance.actual_extraordinary / variance.budgeted_extraordinary) * 100)}%</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="h-2 rounded-full {Math.min(100, (variance.actual_extraordinary / variance.budgeted_extraordinary) * 100) > 100 ? 'bg-red-500' : 'bg-blue-500'}" style="width: {Math.min(100, (variance.actual_extraordinary / variance.budgeted_extraordinary) * 100)}%"></div>
            </div>
          </div>
        </div>

        <div class="mt-4 p-3 bg-gray-50 rounded-lg text-sm text-gray-600">
          <p>{$_('budgets.monthsElapsed', { values: { months: variance.months_elapsed } })} | {$_('budgets.yearEndProjection')}: {formatCurrency(variance.projected_year_end_total)}</p>
        </div>
      </div>
    {/if}

    <!-- Actions -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">{$_('common.actions')}</h2>
      <div class="flex flex-wrap gap-3">
        {#if budget.status === 'draft' || budget.status === 'rejected'}
          <button
            on:click={submitBudget}
            disabled={actionLoading}
            data-testid="submit-budget-button"
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
          >
            {$_('budgets.actions.submitForApproval')}
          </button>
        {/if}

        {#if budget.status === 'submitted'}
          <button
            on:click={() => showApproveModal = true}
            disabled={actionLoading}
            data-testid="approve-budget-button"
            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition disabled:opacity-50"
          >
            {$_('budgets.actions.approve')}
          </button>
          <button
            on:click={() => showRejectModal = true}
            disabled={actionLoading}
            data-testid="reject-budget-button"
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition disabled:opacity-50"
          >
            {$_('budgets.actions.reject')}
          </button>
        {/if}

        {#if budget.status === 'approved'}
          <button
            on:click={archiveBudget}
            disabled={actionLoading}
            data-testid="archive-budget-button"
            class="px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition disabled:opacity-50"
          >
            {$_('budgets.actions.archive')}
          </button>
        {/if}

        {#if budget.status === 'draft'}
          <button
            on:click={deleteBudget}
            data-testid="delete-budget-button"
            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition"
          >
            {$_('common.delete')}
          </button>
        {/if}

        <a href="/budgets" class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition">
          {$_('common.backToList')}
        </a>
      </div>
    </div>
  </div>

  <!-- Approve Modal -->
  {#if showApproveModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">{$_('budgets.approveBudget')}</h3>
        <p class="text-sm text-gray-600 mb-4">
          {$_('budgets.approveBudgetDescription')}
        </p>
        <div class="mb-4">
          <label for="meeting-id" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('budgets.meetingId')}
          </label>
          <input
            id="meeting-id"
            type="text"
            bind:value={meetingId}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder="UUID AG"
          />
        </div>
        <div class="flex justify-end space-x-3">
          <button on:click={() => showApproveModal = false} class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
            {$_('common.cancel')}
          </button>
          <button on:click={approveBudget} disabled={actionLoading} class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50">
            {$_('budgets.actions.approve')}
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Reject Modal -->
  {#if showRejectModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">{$_('budgets.rejectBudget')}</h3>
        <div class="mb-4">
          <label for="reject-reason" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('budgets.rejectionReason')}
          </label>
          <textarea
            id="reject-reason"
            bind:value={rejectReason}
            rows="3"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder={$_('budgets.rejectionReasonPlaceholder')}
          ></textarea>
        </div>
        <div class="flex justify-end space-x-3">
          <button on:click={() => showRejectModal = false} class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
            {$_('common.cancel')}
          </button>
          <button on:click={rejectBudget} disabled={actionLoading} class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50">
            {$_('budgets.actions.reject')}
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}
