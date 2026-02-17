<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  // Report type selection
  let reportType: 'balance-sheet' | 'income-statement' = 'balance-sheet';

  // Date range for income statement
  let periodStart = '';
  let periodEnd = '';

  // Report data
  let balanceSheet: any = null;
  let incomeStatement: any = null;

  // Loading states
  let loading = false;
  let error = '';

  onMount(() => {
    // Set default period to current year
    const now = new Date();
    const yearStart = new Date(now.getFullYear(), 0, 1);
    periodStart = yearStart.toISOString().split('T')[0];
    periodEnd = now.toISOString().split('T')[0];
  });

  async function loadBalanceSheet() {
    try {
      loading = true;
      error = '';
      balanceSheet = await api.get('/reports/balance-sheet');
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement du bilan';
      console.error('Error loading balance sheet:', err);
    } finally {
      loading = false;
    }
  }

  async function loadIncomeStatement() {
    if (!periodStart || !periodEnd) {
      error = 'Veuillez sélectionner une période';
      return;
    }

    try {
      loading = true;
      error = '';
      // Convert YYYY-MM-DD to ISO 8601 format (YYYY-MM-DDTHH:MM:SSZ)
      const startISO = `${periodStart}T00:00:00Z`;
      const endISO = `${periodEnd}T23:59:59Z`;
      incomeStatement = await api.get(
        `/reports/income-statement?period_start=${startISO}&period_end=${endISO}`
      );
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement du compte de résultats';
      console.error('Error loading income statement:', err);
    } finally {
      loading = false;
    }
  }

  function handleReportTypeChange() {
    error = '';
    balanceSheet = null;
    incomeStatement = null;
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', {
      style: 'currency',
      currency: 'EUR',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2
    }).format(amount);
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function exportToPDF() {
    const data = reportType === 'balance-sheet' ? balanceSheet : incomeStatement;
    if (!data) {
      alert('Veuillez d\'abord charger un rapport');
      return;
    }
    window.print();
  }

  function exportToExcel() {
    const data = reportType === 'balance-sheet' ? balanceSheet : incomeStatement;
    if (!data) {
      alert('Veuillez d\'abord charger un rapport');
      return;
    }

    let csv = '';
    const title = reportType === 'balance-sheet' ? 'Bilan_Comptable' : 'Compte_de_Resultats';
    csv += `${title.replace(/_/g, ' ')}\n`;
    csv += `Date export;${new Date().toLocaleDateString('fr-BE')}\n\n`;

    if (reportType === 'balance-sheet' && balanceSheet) {
      csv += 'Section;Code;Compte;Montant EUR\n';
      for (const account of balanceSheet.assets?.accounts || []) {
        csv += `Actif;${account.code};${account.label};${account.amount.toFixed(2)}\n`;
      }
      csv += `;;Total Actif;${balanceSheet.total_assets.toFixed(2)}\n`;
      for (const account of balanceSheet.liabilities?.accounts || []) {
        csv += `Passif;${account.code};${account.label};${account.amount.toFixed(2)}\n`;
      }
      csv += `;;Total Passif;${balanceSheet.total_liabilities.toFixed(2)}\n`;
      for (const account of balanceSheet.equity?.accounts || []) {
        csv += `Capitaux Propres;${account.code};${account.label};${account.amount.toFixed(2)}\n`;
      }
      csv += `;;Total Capitaux Propres;${balanceSheet.total_equity.toFixed(2)}\n`;
    } else if (incomeStatement) {
      csv += 'Section;Code;Compte;Montant EUR\n';
      for (const account of incomeStatement.expenses?.accounts || []) {
        csv += `Charges;${account.code};${account.label};${account.amount.toFixed(2)}\n`;
      }
      csv += `;;Total Charges;${incomeStatement.total_expenses.toFixed(2)}\n`;
      for (const account of incomeStatement.revenue?.accounts || []) {
        csv += `Produits;${account.code};${account.label};${account.amount.toFixed(2)}\n`;
      }
      csv += `;;Total Produits;${incomeStatement.total_revenue.toFixed(2)}\n`;
      csv += `;;Resultat Net;${incomeStatement.net_result.toFixed(2)}\n`;
    }

    const blob = new Blob(['\ufeff' + csv], { type: 'text/csv;charset=utf-8;' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${title}_${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  }
</script>

<div class="space-y-6">
  <!-- Report Type Selection -->
  <div class="bg-white rounded-lg shadow p-6">
    <h2 class="text-xl font-semibold text-gray-900 mb-4">Type de Rapport</h2>
    <div class="flex space-x-4">
      <button
        class="px-6 py-3 rounded-lg font-medium transition-colors {reportType === 'balance-sheet'
          ? 'bg-primary-600 text-white'
          : 'bg-gray-200 text-gray-700 hover:bg-gray-300'}"
        on:click={() => { reportType = 'balance-sheet'; handleReportTypeChange(); }}
      >
        📊 Bilan Comptable
      </button>
      <button
        class="px-6 py-3 rounded-lg font-medium transition-colors {reportType === 'income-statement'
          ? 'bg-primary-600 text-white'
          : 'bg-gray-200 text-gray-700 hover:bg-gray-300'}"
        on:click={() => { reportType = 'income-statement'; handleReportTypeChange(); }}
      >
        📈 Compte de Résultats
      </button>
    </div>
  </div>

  <!-- Period Selection for Income Statement -->
  {#if reportType === 'income-statement'}
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Période</h3>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div>
          <label for="period-start" class="block text-sm font-medium text-gray-700 mb-2">
            Date de début
          </label>
          <input
            id="period-start"
            type="date"
            bind:value={periodStart}
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
        <div>
          <label for="period-end" class="block text-sm font-medium text-gray-700 mb-2">
            Date de fin
          </label>
          <input
            id="period-end"
            type="date"
            bind:value={periodEnd}
            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500 focus:border-primary-500"
          />
        </div>
      </div>
    </div>
  {/if}

  <!-- Generate Report Button -->
  <div class="flex justify-center">
    <button
      class="px-8 py-3 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
      on:click={() => reportType === 'balance-sheet' ? loadBalanceSheet() : loadIncomeStatement()}
      disabled={loading}
    >
      {#if loading}
        <span class="flex items-center">
          <svg class="animate-spin -ml-1 mr-3 h-5 w-5 text-white" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
          </svg>
          Chargement...
        </span>
      {:else}
        Générer le Rapport
      {/if}
    </button>
  </div>

  <!-- Error Message -->
  {#if error}
    <div class="bg-red-50 border-l-4 border-red-400 p-4">
      <div class="flex">
        <div class="flex-shrink-0">
          <svg class="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clip-rule="evenodd"/>
          </svg>
        </div>
        <div class="ml-3">
          <p class="text-sm text-red-700">{error}</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Balance Sheet Report -->
  {#if balanceSheet && reportType === 'balance-sheet'}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <!-- Report Header -->
      <div class="bg-primary-600 text-white p-6">
        <h2 class="text-2xl font-bold">Bilan Comptable</h2>
        <p class="text-primary-100 mt-1">Date: {formatDate(balanceSheet.report_date)}</p>
        <div class="mt-4 flex space-x-4">
          <button
            on:click={exportToPDF}
            class="px-4 py-2 bg-white text-primary-600 rounded hover:bg-primary-50 transition-colors text-sm font-medium"
          >
            📄 Export PDF
          </button>
          <button
            on:click={exportToExcel}
            class="px-4 py-2 bg-white text-primary-600 rounded hover:bg-primary-50 transition-colors text-sm font-medium"
          >
            📊 Export CSV
          </button>
        </div>
      </div>

      <div class="p-6">
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <!-- Left Column: Assets -->
          <div>
            <h3 class="text-lg font-semibold text-gray-900 mb-4 border-b-2 border-primary-600 pb-2">
              ACTIF (Classes 2, 3, 4 & 5)
            </h3>
            <div class="space-y-2">
              {#each balanceSheet.assets.accounts as account}
                <div class="flex justify-between py-2 hover:bg-gray-50 px-2 rounded">
                  <div class="flex items-start">
                    <span class="text-sm text-gray-600 mr-2 font-mono">{account.code}</span>
                    <span class="text-sm text-gray-900">{account.label}</span>
                  </div>
                  <span class="text-sm font-medium text-gray-900 font-mono">
                    {formatCurrency(account.amount)}
                  </span>
                </div>
              {/each}
            </div>
            <div class="mt-4 pt-4 border-t-2 border-gray-300 flex justify-between items-center bg-green-50 p-3 rounded-lg">
              <span class="font-bold text-gray-900">Total Actif</span>
              <span class="text-xl font-bold text-green-600 font-mono">
                {formatCurrency(balanceSheet.total_assets)}
              </span>
            </div>
          </div>

          <!-- Right Column: Liabilities + Equity -->
          <div class="space-y-6">
            <!-- Liabilities Section -->
            <div>
              <h3 class="text-lg font-semibold text-gray-900 mb-4 border-b-2 border-primary-600 pb-2">
                PASSIF (Classes 1 & 4)
              </h3>
              <div class="space-y-2">
                {#each balanceSheet.liabilities.accounts as account}
                  <div class="flex justify-between py-2 hover:bg-gray-50 px-2 rounded">
                    <div class="flex items-start">
                      <span class="text-sm text-gray-600 mr-2 font-mono">{account.code}</span>
                      <span class="text-sm text-gray-900">{account.label}</span>
                    </div>
                    <span class="text-sm font-medium text-gray-900 font-mono">
                      {formatCurrency(account.amount)}
                    </span>
                  </div>
                {/each}
              </div>
              <div class="mt-4 pt-4 border-t-2 border-gray-300 flex justify-between items-center bg-blue-50 p-3 rounded-lg">
                <span class="font-bold text-gray-900">Total Passif</span>
                <span class="text-xl font-bold text-blue-600 font-mono">
                  {formatCurrency(balanceSheet.total_liabilities)}
                </span>
              </div>
            </div>

            <!-- Equity Section -->
            <div>
              <h3 class="text-lg font-semibold text-gray-900 mb-4 border-b-2 border-purple-600 pb-2">
                CAPITAUX PROPRES
              </h3>
              <div class="space-y-2">
                {#each balanceSheet.equity.accounts as account}
                  <div class="flex justify-between py-2 hover:bg-gray-50 px-2 rounded">
                    <div class="flex items-start">
                      <span class="text-sm text-gray-600 mr-2 font-mono">{account.code}</span>
                      <span class="text-sm text-gray-900">{account.label}</span>
                    </div>
                    <span class="text-sm font-medium {account.amount >= 0 ? 'text-green-600' : 'text-red-600'} font-mono">
                      {formatCurrency(account.amount)}
                    </span>
                  </div>
                {/each}
              </div>
              <div class="mt-4 pt-4 border-t-2 border-gray-300 flex justify-between items-center bg-purple-50 p-3 rounded-lg">
                <span class="font-bold text-gray-900">Total Capitaux Propres</span>
                <span class="text-xl font-bold {balanceSheet.total_equity >= 0 ? 'text-green-600' : 'text-red-600'} font-mono">
                  {formatCurrency(balanceSheet.total_equity)}
                </span>
              </div>
            </div>
          </div>
        </div>

        <!-- Balance Check (Actif = Passif + Capitaux Propres) -->
        <div class="mt-8 p-4 rounded-lg {Math.abs(balanceSheet.balance) < 0.01 ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'} border-2">
          <div class="flex justify-between items-center">
            <span class="font-bold text-gray-900">Équilibre (Actif - [Passif + Capitaux Propres])</span>
            <span class="text-xl font-bold {Math.abs(balanceSheet.balance) < 0.01 ? 'text-green-600' : 'text-red-600'} font-mono">
              {formatCurrency(balanceSheet.balance)}
            </span>
          </div>
          {#if Math.abs(balanceSheet.balance) < 0.01}
            <p class="text-sm text-green-700 mt-2">✓ Le bilan est équilibré (Actif = Passif + Capitaux Propres)</p>
          {:else}
            <p class="text-sm text-red-700 mt-2">⚠ Le bilan n'est pas équilibré</p>
          {/if}
          <div class="mt-3 text-sm text-gray-700 font-mono">
            <div>Actif: {formatCurrency(balanceSheet.total_assets)}</div>
            <div>Passif: {formatCurrency(balanceSheet.total_liabilities)}</div>
            <div>Capitaux Propres: {formatCurrency(balanceSheet.total_equity)}</div>
            <div class="font-bold mt-2">Passif + Cap. Propres: {formatCurrency(balanceSheet.total_liabilities + balanceSheet.total_equity)}</div>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- Income Statement Report -->
  {#if incomeStatement && reportType === 'income-statement'}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <!-- Report Header -->
      <div class="bg-primary-600 text-white p-6">
        <h2 class="text-2xl font-bold">Compte de Résultats</h2>
        <p class="text-primary-100 mt-1">
          Période: {formatDate(incomeStatement.period_start)} - {formatDate(incomeStatement.period_end)}
        </p>
        <div class="mt-4 flex space-x-4">
          <button
            on:click={exportToPDF}
            class="px-4 py-2 bg-white text-primary-600 rounded hover:bg-primary-50 transition-colors text-sm font-medium"
          >
            📄 Export PDF
          </button>
          <button
            on:click={exportToExcel}
            class="px-4 py-2 bg-white text-primary-600 rounded hover:bg-primary-50 transition-colors text-sm font-medium"
          >
            📊 Export CSV
          </button>
        </div>
      </div>

      <div class="p-6">
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <!-- Expenses Section -->
          <div>
            <h3 class="text-lg font-semibold text-gray-900 mb-4 border-b-2 border-red-600 pb-2">
              CHARGES (Classe 6)
            </h3>
            <div class="space-y-2">
              {#each incomeStatement.expenses.accounts as account}
                <div class="flex justify-between py-2 hover:bg-gray-50 px-2 rounded">
                  <div class="flex items-start">
                    <span class="text-sm text-gray-600 mr-2 font-mono">{account.code}</span>
                    <span class="text-sm text-gray-900">{account.label}</span>
                  </div>
                  <span class="text-sm font-medium text-gray-900 font-mono">
                    {formatCurrency(account.amount)}
                  </span>
                </div>
              {/each}
            </div>
            <div class="mt-4 pt-4 border-t-2 border-gray-300 flex justify-between items-center bg-red-50 p-3 rounded-lg">
              <span class="font-bold text-gray-900">Total Charges</span>
              <span class="text-xl font-bold text-red-600 font-mono">
                {formatCurrency(incomeStatement.total_expenses)}
              </span>
            </div>
          </div>

          <!-- Revenue Section -->
          <div>
            <h3 class="text-lg font-semibold text-gray-900 mb-4 border-b-2 border-green-600 pb-2">
              PRODUITS (Classe 7)
            </h3>
            <div class="space-y-2">
              {#each incomeStatement.revenue.accounts as account}
                <div class="flex justify-between py-2 hover:bg-gray-50 px-2 rounded">
                  <div class="flex items-start">
                    <span class="text-sm text-gray-600 mr-2 font-mono">{account.code}</span>
                    <span class="text-sm text-gray-900">{account.label}</span>
                  </div>
                  <span class="text-sm font-medium text-gray-900 font-mono">
                    {formatCurrency(account.amount)}
                  </span>
                </div>
              {/each}
            </div>
            <div class="mt-4 pt-4 border-t-2 border-gray-300 flex justify-between items-center bg-green-50 p-3 rounded-lg">
              <span class="font-bold text-gray-900">Total Produits</span>
              <span class="text-xl font-bold text-green-600 font-mono">
                {formatCurrency(incomeStatement.total_revenue)}
              </span>
            </div>
          </div>
        </div>

        <!-- Net Result -->
        <div class="mt-8 p-4 rounded-lg {incomeStatement.net_result >= 0 ? 'bg-green-50 border-green-200' : 'bg-red-50 border-red-200'} border-2">
          <div class="flex justify-between items-center">
            <span class="font-bold text-gray-900">Résultat Net (Produits - Charges)</span>
            <span class="text-2xl font-bold {incomeStatement.net_result >= 0 ? 'text-green-600' : 'text-red-600'} font-mono">
              {formatCurrency(incomeStatement.net_result)}
            </span>
          </div>
          {#if incomeStatement.net_result >= 0}
            <p class="text-sm text-green-700 mt-2">✓ Résultat excédentaire (bénéfice)</p>
          {:else}
            <p class="text-sm text-red-700 mt-2">⚠ Résultat déficitaire (perte)</p>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  /* Ensure font-mono for numbers */
  .font-mono {
    font-family: 'Courier New', Courier, monospace;
  }
</style>
