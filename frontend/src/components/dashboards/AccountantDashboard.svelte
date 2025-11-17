<script lang="ts">
  import { onMount } from 'svelte';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';

  $: user = $authStore.user;

  // Dashboard data
  let stats: any = null;
  let transactions: any[] = [];
  let loading = true;
  let error = '';

  // Load dashboard data
  async function loadDashboardData() {
    try {
      loading = true;
      error = '';

      // Load stats and transactions in parallel
      const [statsData, transactionsData] = await Promise.all([
        api.get('/dashboard/accountant/stats'),
        api.get('/dashboard/accountant/transactions?limit=5')
      ]);

      stats = statsData;
      transactions = transactionsData;
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des données';
      console.error('Dashboard loading error:', err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadDashboardData();
  });

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', {
      style: 'currency',
      currency: 'EUR',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0
    }).format(amount);
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  function getTransactionIcon(type: string): string {
    return type === 'paymentreceived' ? '✅' : '💸';
  }

  function getTransactionLabel(type: string): string {
    return type === 'paymentreceived' ? 'Paiement reçu' : 'Paiement effectué';
  }

  function getTransactionColor(type: string): string {
    return type === 'paymentreceived' ? 'green' : 'red';
  }
</script>

<div>
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      Bienvenue, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      Dashboard Comptable - Gestion financière
    </p>
  </div>

  <!-- Loading State -->
  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
      <span class="ml-3 text-gray-600">Chargement des données...</span>
    </div>
  {:else if error}
    <!-- Error State -->
    <div class="bg-red-50 border-l-4 border-red-400 p-4 mb-8">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
          </svg>
        </div>
        <div class="ml-3">
          <p class="text-sm text-red-700">{error}</p>
          <button on:click={loadDashboardData} class="mt-2 text-sm font-medium text-red-700 hover:text-red-600">
            Réessayer
          </button>
        </div>
      </div>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Charges totales</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{formatCurrency(stats.total_expenses_current_month)}</p>
        <p class="text-sm text-gray-500 mt-1">Ce mois</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Payé</span>
          <span class="text-2xl">✅</span>
        </div>
        <p class="text-3xl font-bold text-green-600">{formatCurrency(stats.total_paid)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.paid_percentage.toFixed(0)}% collecté</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">En attente</span>
          <span class="text-2xl">⏳</span>
        </div>
        <p class="text-3xl font-bold text-orange-600">{formatCurrency(stats.total_pending)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.pending_percentage.toFixed(0)}% restant</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">En retard</span>
          <span class="text-2xl">🚨</span>
        </div>
        <p class="text-3xl font-bold text-red-600">{stats.owners_with_overdue}</p>
        <p class="text-sm text-gray-500 mt-1">Copropriétaires</p>
      </div>
    </div>
  {/if}

  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Recent Transactions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Dernières transactions</h2>
      </div>
      <div class="p-6">
        {#if loading}
          <div class="flex justify-center py-4">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>
        {:else if transactions.length > 0}
          <div class="space-y-4">
            {#each transactions as transaction}
              <div class="flex items-center justify-between p-4 bg-{getTransactionColor(transaction.transaction_type)}-50 border border-{getTransactionColor(transaction.transaction_type)}-200 rounded-lg">
                <div class="flex items-center space-x-3">
                  <span class="text-2xl">{getTransactionIcon(transaction.transaction_type)}</span>
                  <div>
                    <p class="text-sm font-medium text-gray-900">{getTransactionLabel(transaction.transaction_type)}</p>
                    <p class="text-sm text-gray-600">{transaction.description}</p>
                    {#if transaction.related_entity}
                      <p class="text-xs text-gray-500">{transaction.related_entity}</p>
                    {/if}
                    <p class="text-xs text-gray-400 mt-1">{formatDate(transaction.date)}</p>
                  </div>
                </div>
                <p class="text-lg font-bold text-{getTransactionColor(transaction.transaction_type)}-600">
                  {transaction.amount >= 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                </p>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-center text-gray-500 py-4">Aucune transaction récente</p>
        {/if}
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">Actions rapides</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
            <span class="text-sm font-medium text-gray-700">Charges</span>
          </a>
          <a href="/reports" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📈</span>
            <span class="text-sm font-medium text-gray-700">Rapports</span>
          </a>
          <a href="/invoices" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📄</span>
            <span class="text-sm font-medium text-gray-700">Factures</span>
          </a>
          <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">Immeubles</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
