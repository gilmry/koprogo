<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import type { Expense, Building } from '../lib/types';
  import Button from './ui/Button.svelte';
  import ExpenseDocuments from './ExpenseDocuments.svelte';
  import { toast } from '../stores/toast';
  import { paymentsApi, type Payment } from '../lib/api/payments';
  import { chargeDistributionsApi, type ChargeDistribution } from '../lib/api/charge-distributions';

  let expense: Expense | null = null;
  let building: Building | null = null;
  let expensePayments: Payment[] = [];
  let totalPaidCents = 0;
  let distributions: ChargeDistribution[] = [];
  let loading = true;
  let error = '';
  let expenseId: string = '';

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    expenseId = urlParams.get('id') || '';

    if (!expenseId) {
      error = $_('expenses.missing_id');
      loading = false;
      return;
    }

    loadExpense();
  });

  async function loadExpense() {
    try {
      loading = true;
      error = '';
      expense = await api.get<Expense>(`/expenses/${expenseId}`);

      // Load building info and payments in parallel
      if (expense) {
        const promises: Promise<any>[] = [];

        if (expense.building_id) {
          promises.push(
            api.get<Building>(`/buildings/${expense.building_id}`)
              .then(b => { building = b; })
              .catch(e => console.error('Error loading building:', e))
          );
        }

        promises.push(
          paymentsApi.listByExpense(expenseId)
            .then(p => { expensePayments = p; })
            .catch(() => { expensePayments = []; })
        );

        promises.push(
          paymentsApi.getExpenseTotal(expenseId)
            .then(t => { totalPaidCents = t.total_paid_cents; })
            .catch(() => { totalPaidCents = 0; })
        );

        promises.push(
          chargeDistributionsApi.getByExpense(expenseId)
            .then(d => { distributions = d; })
            .catch(() => { distributions = []; })
        );

        await Promise.all(promises);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : $_('expenses.load_error');
      console.error('Error loading expense:', e);
    } finally {
      loading = false;
    }
  }

  const handleGoBack = () => {
    window.history.back();
  };

  const handleMarkPaid = async () => {
    if (!expense) return;

    try {
      await api.put(`/expenses/${expense.id}/mark-paid`, {});
      await loadExpense();
      toast.success($_('expenses.marked_paid'));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : $_('common.update_error');
      toast.error(`${$_('common.error')}: ${errorMsg}`);
      console.error('Error marking as paid:', e);
    }
  };

  const handleMarkOverdue = async () => {
    if (!expense) return;

    try {
      await api.post(`/expenses/${expense.id}/mark-overdue`, {});
      await loadExpense();
      toast.success($_('expenses.marked_overdue'));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : $_('common.update_error');
      toast.error(`${$_('common.error')}: ${errorMsg}`);
      console.error('Error marking as overdue:', e);
    }
  };

  const handleCancel = async () => {
    if (!expense) return;

    if (!confirm($_('expenses.confirm_cancel'))) {
      return;
    }

    try {
      await api.post(`/expenses/${expense.id}/cancel`, {});
      await loadExpense();
      toast.success($_('expenses.cancelled'));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : $_('expenses.cancel_error');
      toast.error(`${$_('common.error')}: ${errorMsg}`);
      console.error('Error cancelling expense:', e);
    }
  };

  const handleReactivate = async () => {
    if (!expense) return;

    try {
      await api.post(`/expenses/${expense.id}/reactivate`, {});
      await loadExpense();
      toast.success($_('expenses.reactivated'));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : $_('expenses.reactivate_error');
      toast.error(`${$_('common.error')}: ${errorMsg}`);
      console.error('Error reactivating expense:', e);
    }
  };

  const handleUnpay = async () => {
    if (!expense) return;

    if (!confirm($_('expenses.confirm_unpay'))) {
      return;
    }

    try {
      await api.post(`/expenses/${expense.id}/unpay`, {});
      await loadExpense();
      toast.success($_('expenses.unpaid'));
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : $_('expenses.unpay_error');
      toast.error(`${$_('common.error')}: ${errorMsg}`);
      console.error('Error unpaying expense:', e);
    }
  };

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'Paid': { class: 'bg-green-100 text-green-800', label: $_('expenses.status_paid') },
      'Pending': { class: 'bg-yellow-100 text-yellow-800', label: $_('expenses.status_pending') },
      'Overdue': { class: 'bg-red-100 text-red-800', label: $_('expenses.status_overdue') },
      'Cancelled': { class: 'bg-gray-100 text-gray-800', label: $_('expenses.status_cancelled') }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function formatCentsToEur(cents: number): string {
    return formatCurrency(cents / 100);
  }

  function getPaymentStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'Pending': { class: 'bg-yellow-100 text-yellow-800', label: $_('expenses.payment_pending') },
      'Processing': { class: 'bg-blue-100 text-blue-800', label: $_('expenses.payment_processing') },
      'RequiresAction': { class: 'bg-orange-100 text-orange-800', label: $_('expenses.payment_action_required') },
      'Succeeded': { class: 'bg-green-100 text-green-800', label: $_('expenses.payment_succeeded') },
      'Failed': { class: 'bg-red-100 text-red-800', label: $_('expenses.payment_failed') },
      'Cancelled': { class: 'bg-gray-100 text-gray-800', label: $_('expenses.payment_cancelled') },
      'Refunded': { class: 'bg-purple-100 text-purple-800', label: $_('expenses.payment_refunded') },
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function getPaymentMethodLabel(type: string): string {
    const labels: Record<string, string> = {
      'Card': $_('expenses.method_card'),
      'SepaDebit': $_('expenses.method_sepa'),
      'BankTransfer': $_('expenses.method_transfer'),
      'Cash': $_('expenses.method_cash'),
    };
    return labels[type] || type;
  }

  function getCategoryLabel(category: string): string {
    const labels: Record<string, string> = {
      'Maintenance': $_('expenses.category_maintenance'),
      'Repair': $_('expenses.category_repair'),
      'Insurance': $_('expenses.category_insurance'),
      'Utilities': $_('expenses.category_utilities'),
      'Management': $_('expenses.category_management'),
      'Other': $_('expenses.category_other')
    };
    return labels[category] || category;
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">{$_('common.loading')}</p>
      </div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
      {error}
    </div>
    <div class="mt-4">
      <Button variant="outline" on:click={handleGoBack}>
        Retour
      </Button>
    </div>
  {:else if expense}
    <!-- Header -->
    <div class="mb-8">
      <div class="flex items-center justify-between">
        <div class="flex items-center space-x-4">
          <button
            on:click={handleGoBack}
            class="text-gray-600 hover:text-gray-900"
          >
            {$_('common.back')}
          </button>
          <h1 class="text-3xl font-bold text-gray-900">{$_('expenses.detail_title')}</h1>
        </div>
        <div class="flex gap-2">
          {#if expense.payment_status === 'Pending'}
            <Button variant="primary" on:click={handleMarkPaid}>
              {$_('expenses.mark_paid')}
            </Button>
            <Button variant="outline" on:click={handleMarkOverdue}>
              {$_('expenses.mark_overdue')}
            </Button>
            <Button variant="outline" on:click={handleCancel}>
              {$_('common.cancel')}
            </Button>
          {:else if expense.payment_status === 'Overdue'}
            <Button variant="primary" on:click={handleMarkPaid}>
              {$_('expenses.mark_paid')}
            </Button>
            <Button variant="outline" on:click={handleCancel}>
              {$_('common.cancel')}
            </Button>
          {:else if expense.payment_status === 'Paid'}
            <Button variant="outline" on:click={handleUnpay}>
              {$_('expenses.cancel_payment')}
            </Button>
          {:else if expense.payment_status === 'Cancelled'}
            <Button variant="primary" on:click={handleReactivate}>
              {$_('expenses.reactivate')}
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Main Info Card -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">{$_('expenses.general_info')}</h2>
          <span class="px-3 py-1 rounded-full text-sm font-medium {getStatusBadge(expense.payment_status).class}">
            {getStatusBadge(expense.payment_status).label}
          </span>
        </div>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Description -->
          <div class="md:col-span-2">
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('common.description')}</h3>
            <p class="text-lg text-gray-900">{expense.description}</p>
          </div>

          <!-- Amount -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('common.amount')}</h3>
            <p class="text-2xl font-bold text-gray-900">{formatCurrency(expense.amount)}</p>
          </div>

          <!-- Category -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('common.category')}</h3>
            <p class="text-lg text-gray-900">{getCategoryLabel(expense.category)}</p>
          </div>

          <!-- Expense Date -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('expenses.date')}</h3>
            <p class="text-lg text-gray-900">{formatDate(expense.expense_date)}</p>
          </div>

          <!-- Due Date -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('expenses.due_date')}</h3>
            <p class="text-lg text-gray-900">{formatDate(expense.due_date)}</p>
          </div>

          {#if expense.paid_date}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('expenses.paid_date')}</h3>
              <p class="text-lg text-gray-900">{formatDate(expense.paid_date)}</p>
            </div>
          {/if}

          <!-- Building -->
          {#if building}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('common.building')}</h3>
              <a href="/building-detail?id={building.id}" class="text-lg text-primary-600 hover:text-primary-700 hover:underline">
                {building.name}
              </a>
              <p class="text-sm text-gray-600">{building.address}</p>
            </div>
          {/if}

          {#if expense.supplier}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('expenses.supplier')}</h3>
              <p class="text-lg text-gray-900">{expense.supplier}</p>
            </div>
          {/if}

          {#if expense.invoice_number}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">{$_('expenses.invoice_number')}</h3>
              <p class="text-lg text-gray-900">{expense.invoice_number}</p>
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Documents Section -->
    <div class="mb-8">
      <ExpenseDocuments expenseId={expenseId} expenseStatus={expense.payment_status} />
    </div>

    <!-- Charge Distribution Section -->
    {#if distributions.length > 0}
      <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
        <div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
          <h2 class="text-xl font-semibold text-white">{$_('expenses.charge_distribution')}</h2>
        </div>
        <div class="p-6">
          <div class="space-y-3">
            {#each distributions as dist}
              <div class="flex items-center justify-between p-3 border border-gray-200 rounded-lg">
                <div class="flex-1">
                  <p class="text-sm font-medium text-gray-900">
                    {$_('expenses.owner')} #{dist.owner_id.substring(0, 8)}
                  </p>
                  <p class="text-xs text-gray-500">
                    {$_('expenses.quota_part')}: {(dist.quota_percentage * 100).toFixed(2)}%
                  </p>
                </div>
                <span class="text-sm font-bold text-indigo-600">
                  {formatCurrency(dist.amount_due)}
                </span>
              </div>
            {/each}
          </div>
          <div class="mt-4 pt-3 border-t text-sm text-gray-500">
            {distributions.length} {$_('expenses.owner_count', { values: { count: distributions.length } })}
          </div>
        </div>
      </div>
    {/if}

    <!-- Payments Section -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-green-600 to-green-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">{$_('expenses.payments')}</h2>
          {#if totalPaidCents > 0}
            <span class="px-3 py-1 rounded-full text-sm font-medium bg-white/20 text-white">
              {$_('expenses.total_paid')}: {formatCentsToEur(totalPaidCents)}
            </span>
          {/if}
        </div>
      </div>
      <div class="p-6">
        {#if expensePayments.length > 0}
          <!-- Payment progress bar -->
          {#if expense.amount > 0}
            {@const paidPercent = Math.min(100, (totalPaidCents / 100 / expense.amount) * 100)}
            <div class="mb-6">
              <div class="flex items-center justify-between text-sm text-gray-600 mb-1">
                <span>{$_('expenses.payment_progress')}</span>
                <span class="font-medium">{Math.round(paidPercent)}%</span>
              </div>
              <div class="w-full bg-gray-200 rounded-full h-2.5">
                <div
                  class="h-2.5 rounded-full {paidPercent >= 100 ? 'bg-green-500' : 'bg-primary-500'}"
                  style="width: {paidPercent}%"
                ></div>
              </div>
              <div class="flex items-center justify-between text-xs text-gray-500 mt-1">
                <span>{formatCentsToEur(totalPaidCents)} {$_('expenses.paid')}</span>
                <span>{formatCurrency(expense.amount)} {$_('common.total')}</span>
              </div>
            </div>
          {/if}

          <!-- Payment list -->
          <div class="space-y-3">
            {#each expensePayments as payment}
              {@const badge = getPaymentStatusBadge(payment.status)}
              <div class="flex items-center justify-between p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition">
                <div class="flex-1">
                  <div class="flex items-center gap-3 mb-1">
                    <span class="text-sm font-medium text-gray-900">{formatCentsToEur(payment.amount_cents)}</span>
                    <span class="px-2 py-0.5 rounded-full text-xs font-medium {badge.class}">{badge.label}</span>
                  </div>
                  <div class="flex items-center gap-2 text-xs text-gray-500">
                    <span>{getPaymentMethodLabel(payment.payment_method_type)}</span>
                    <span>·</span>
                    <span>{formatDate(payment.created_at)}</span>
                    {#if payment.refunded_amount_cents > 0}
                      <span>·</span>
                      <span class="text-purple-600">{$_('expenses.refunded')}: {formatCentsToEur(payment.refunded_amount_cents)}</span>
                    {/if}
                  </div>
                  {#if payment.failure_reason}
                    <p class="text-xs text-red-600 mt-1">{payment.failure_reason}</p>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="text-center py-8">
            <p class="text-gray-500">{$_('expenses.no_payments')}</p>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
