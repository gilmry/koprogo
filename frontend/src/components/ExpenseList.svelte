<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Expense, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  let expenses: Expense[] = [];
  let loading = true;
  let error = '';

  // Pagination state
  let currentPage = 1;
  let perPage = 20;
  let totalItems = 0;
  let totalPages = 0;

  onMount(async () => {
    await loadExpenses();
  });

  async function loadExpenses() {
    try {
      loading = true;
      const response = await api.get<PageResponse<Expense>>(
        `/expenses?page=${currentPage}&per_page=${perPage}`
      );

      expenses = response.data;
      totalItems = response.total;
      totalPages = response.total_pages;
      currentPage = response.page;
      perPage = response.per_page;
      error = '';
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des dépenses';
      console.error('Error loading expenses:', e);
    } finally {
      loading = false;
    }
  }

  async function handlePageChange(page: number) {
    currentPage = page;
    await loadExpenses();
  }

  function getStatusBadge(status: string): string {
    const badges: Record<string, string> = {
      'Paid': 'bg-green-100 text-green-800',
      'Pending': 'bg-yellow-100 text-yellow-800',
      'Overdue': 'bg-red-100 text-red-800',
      'Cancelled': 'bg-gray-100 text-gray-800'
    };
    return badges[status] || 'bg-gray-100 text-gray-800';
  }

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
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <p class="text-gray-600">
      {totalItems} dépense{totalItems !== 1 ? 's' : ''}
    </p>
  </div>

  {#if error}
    <div class="bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded">
      {error}
    </div>
  {/if}

  {#if loading}
    <p class="text-center text-gray-600 py-8">Chargement...</p>
  {:else if expenses.length === 0}
    <p class="text-center text-gray-600 py-8">
      Aucune dépense enregistrée.
    </p>
  {:else}
    <div class="grid gap-4">
      {#each expenses as expense}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">
                  {expense.description}
                </h3>
                <span class="text-xs px-2 py-1 rounded-full {getStatusBadge(expense.payment_status)}">
                  {expense.payment_status}
                </span>
              </div>
              <p class="text-gray-600 text-sm">
                📁 {expense.category}
              </p>
              <p class="text-gray-500 text-sm">
                📅 {formatDate(expense.expense_date)}
              </p>
            </div>
            <div class="text-right">
              <p class="text-xl font-bold text-gray-900">
                {formatCurrency(expense.amount)}
              </p>
              <button class="text-primary-600 hover:text-primary-700 text-sm font-medium mt-2">
                Détails →
              </button>
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
