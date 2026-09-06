<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import { authStore } from '../../stores/auth';
  import { api } from '../../lib/api';
  import { formatDate } from "../../lib/utils/date.utils";
  import { formatCurrency } from "../../lib/utils/finance.utils";
  import { withErrorHandling } from "../../lib/utils/error.utils";

  let user = $derived($authStore.user);

  // Dashboard data
  let stats = $state<any>(null);
  let transactions = $state<any[]>([]);
  let loading = $state(true);
  let error = $state('');

  // Load dashboard data
  async function loadDashboardData() {
    loading = true;
    error = '';
    const result = await withErrorHandling({
      action: async () => {
        const [statsData, transactionsData] = await Promise.all([
          api.get('/dashboard/accountant/stats'),
          api.get('/dashboard/accountant/transactions?limit=5')
        ]);
        return { statsData, transactionsData };
      },
      errorMessage: $_('common.error.loadData'),
    });
    if (result) {
      stats = result.statsData;
      transactions = result.transactionsData;
    } else {
      error = $_('common.error.loadData');
    }
    loading = false;
  }

  $effect(() => {
    loadDashboardData();
  });

  function getTransactionIcon(type: string): string {
    return type === 'paymentreceived' ? '✅' : '💸';
  }

  function getTransactionLabel(type: string): string {
    return type === 'paymentreceived' ? $_('dashboards.accountant.transaction.received') : $_('dashboards.accountant.transaction.made');
  }

  /**
   * Les classes d'une transaction, en CHAÎNES COMPLÈTES.
   *
   * Cette fonction rendait `'green'` ou `'red'`, et le gabarit composait
   * `bg-{...}-50 border-{...}-200` et `text-{...}-600`. Tailwind ne fait pas
   * d'analyse dynamique : il scanne le source à la recherche de classes
   * entières. `bg-green-50` n'était donc **jamais générée** et n'entrait pas
   * dans la feuille de style livrée.
   *
   * Les transactions s'affichaient sans fond, sans bordure et sans couleur de
   * montant. Le code semblait juste, le rendu ne l'était pas, et rien ne le
   * signalait — ni la compilation, ni les tests. Relevé par la revue de
   * design du 2026-09-06, point 0.3, issue #788.
   */
  const TON_TRANSACTION = {
    recu: {
      cadre: 'bg-green-50 border-green-200',
      montant: 'text-green-600',
    },
    sorti: {
      cadre: 'bg-red-50 border-red-200',
      montant: 'text-red-600',
    },
  } as const;

  function tonDeLaTransaction(type: string) {
    return type === 'paymentreceived'
      ? TON_TRANSACTION.recu
      : TON_TRANSACTION.sorti;
  }
</script>

<div data-testid="accountant-dashboard">
  <div class="mb-8">
    <h1 class="text-3xl font-bold text-gray-900 mb-2">
      {$_('common.welcome')}, {user?.first_name} 👋
    </h1>
    <p class="text-gray-600">
      {$_('dashboards.accountant.title')} - {$_('dashboards.accountant.subtitle')}
    </p>
  </div>

  <!-- Loading State -->
  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
      <span class="ml-3 text-gray-600">{$_('common.loading')}</span>
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
          <button onclick={loadDashboardData} class="mt-2 text-sm font-medium text-red-700 hover:text-red-600">
            {$_('common.retry')}
          </button>
        </div>
      </div>
    </div>
  {:else if stats}
    <!-- Stats Cards -->
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.accountant.stats.totalExpenses')}</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{formatCurrency(stats.total_expenses_current_month)}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.accountant.stats.thisMonth')}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.accountant.stats.paid')}</span>
          <span class="text-2xl">✅</span>
        </div>
        <p class="text-3xl font-bold text-green-600">{formatCurrency(stats.total_paid)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.paid_percentage.toFixed(0)}% {$_('dashboards.accountant.stats.collected')}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.accountant.stats.pending')}</span>
          <span class="text-2xl">⏳</span>
        </div>
        <p class="text-3xl font-bold text-orange-600">{formatCurrency(stats.total_pending)}</p>
        <p class="text-sm text-gray-500 mt-1">{stats.pending_percentage.toFixed(0)}% {$_('dashboards.accountant.stats.remaining')}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">{$_('dashboards.accountant.stats.overdue')}</span>
          <span class="text-2xl">🚨</span>
        </div>
        <p class="text-3xl font-bold text-red-600">{stats.owners_with_overdue}</p>
        <p class="text-sm text-gray-500 mt-1">{$_('dashboards.accountant.stats.owners')}</p>
      </div>
    </div>
  {/if}

  <!-- Main Content -->
  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Recent Transactions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.accountant.recentTransactions')}</h2>
      </div>
      <div class="p-6">
        {#if loading}
          <div class="flex justify-center py-4">
            <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600"></div>
          </div>
        {:else if transactions.length > 0}
          <div class="space-y-4">
            {#each transactions as transaction}
              <div class="flex items-center justify-between p-4 border rounded-lg {tonDeLaTransaction(transaction.transaction_type).cadre}">
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
                <p class="text-lg font-bold {tonDeLaTransaction(transaction.transaction_type).montant}">
                  {transaction.amount >= 0 ? '+' : ''}{formatCurrency(transaction.amount)}
                </p>
              </div>
            {/each}
          </div>
        {:else}
          <p class="text-center text-gray-500 py-4">{$_('dashboards.accountant.noRecentTransactions')}</p>
        {/if}
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="bg-white rounded-lg shadow">
      <div class="p-6 border-b border-gray-200">
        <h2 class="text-lg font-semibold text-gray-900">{$_('dashboards.accountant.quickActions')}</h2>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-2 gap-4">
          <a href="/expenses" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">💰</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.expenses')}</span>
          </a>
          <a href="/reports" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📈</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.reports')}</span>
          </a>
          <a href="/invoices" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">📄</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.invoices')}</span>
          </a>
          <a href="/buildings" class="flex flex-col items-center justify-center p-6 border-2 border-gray-200 rounded-lg hover:border-primary-500 hover:bg-primary-50 transition group">
            <span class="text-4xl mb-2 group-hover:scale-110 transition">🏢</span>
            <span class="text-sm font-medium text-gray-700">{$_('navigation.buildings')}</span>
          </a>
        </div>
      </div>
    </div>
  </div>
</div>
