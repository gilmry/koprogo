<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { api } from '../lib/api';
  import type { Expense, PageResponse } from '../lib/types';
  import Pagination from './Pagination.svelte';

  export let buildingId: string | null = null;

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

    // Listen for page show events to reload data when navigating back (client-side only)
    if (typeof window !== 'undefined') {
      window.addEventListener('pageshow', handlePageShow);
      window.addEventListener('focus', handleWindowFocus);
    }
  });

  onDestroy(() => {
    if (typeof window !== 'undefined') {
      window.removeEventListener('pageshow', handlePageShow);
      window.removeEventListener('focus', handleWindowFocus);
    }
  });

  function handlePageShow(event: PageTransitionEvent) {
    // Reload data when navigating back to this page
    if (event.persisted) {
      loadExpenses();
    }
  }

  function handleWindowFocus() {
    // Reload data when window regains focus
    loadExpenses();
  }

  async function loadExpenses() {
    try {
      loading = true;

      if (buildingId) {
        // Endpoint without pagination for building-specific expenses
        const response = await api.get<Expense[]>(`/buildings/${buildingId}/expenses`);
        expenses = response;
        totalItems = response.length;
        totalPages = 1;
        currentPage = 1;
      } else {
        // Paginated endpoint for all expenses
        const endpoint = `/expenses?page=${currentPage}&per_page=${perPage}`;
        const response = await api.get<PageResponse<Expense>>(endpoint);
        expenses = response.data;
        totalItems = response.pagination.total_items;
        totalPages = response.pagination.total_pages;
        currentPage = response.pagination.current_page;
        perPage = response.pagination.per_page;
      }

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

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'Paid': { class: 'bg-green-100 text-green-800', label: 'Payée' },
      'Pending': { class: 'bg-yellow-100 text-yellow-800', label: 'En attente' },
      'Overdue': { class: 'bg-red-100 text-red-800', label: 'En retard' },
      'Cancelled': { class: 'bg-gray-100 text-gray-800', label: 'Annulée' }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
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
      {#each expenses as expense (expense.id)}
        <div class="bg-white border border-gray-200 rounded-lg p-4 hover:shadow-md transition">
          <div class="flex justify-between items-start">
            <div class="flex-1">
              <div class="flex items-center gap-2 mb-2">
                <h3 class="text-lg font-semibold text-gray-900">
                  {expense.description}
                </h3>
                <span class="text-xs px-2 py-1 rounded-full {getStatusBadge(expense.payment_status).class}">
                  {getStatusBadge(expense.payment_status).label}
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
              <a href="/expense-detail?id={expense.id}" class="text-primary-600 hover:text-primary-700 text-sm font-medium mt-2 inline-block">
                Détails →
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
