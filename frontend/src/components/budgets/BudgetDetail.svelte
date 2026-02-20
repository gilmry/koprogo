<script lang="ts">
  import { onMount } from 'svelte';
  import { budgetsApi, type Budget, type BudgetVariance } from '../../lib/api/budgets';
  import BudgetStatusBadge from './BudgetStatusBadge.svelte';
  import { toast } from '../../stores/toast';

  let budget: Budget | null = null;
  let variance: BudgetVariance | null = null;
  let loading = true;
  let error = '';
  let actionLoading = false;
  let budgetId = '';

  // Approve modal
  let showApproveModal = false;
  let meetingId = '';

  // Reject modal
  let showRejectModal = false;
  let rejectReason = '';

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    budgetId = params.get('id') || '';
    if (budgetId) loadBudget();
  });

  async function loadBudget() {
    try {
      loading = true;
      error = '';
      budget = await budgetsApi.getById(budgetId);

      if (budget?.status === 'approved') {
        try {
          variance = await budgetsApi.getVariance(budgetId);
        } catch {
          // Variance not available yet
        }
      }
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement du budget';
    } finally {
      loading = false;
    }
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function formatDate(dateString: string | null): string {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric', month: 'long', day: 'numeric'
    });
  }

  async function submitBudget() {
    if (!confirm('Soumettre ce budget pour approbation en AG ?')) return;
    try {
      actionLoading = true;
      budget = await budgetsApi.submit(budgetId);
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible de soumettre'));
    } finally {
      actionLoading = false;
    }
  }

  async function approveBudget() {
    if (!meetingId.trim()) {
      toast.error('Veuillez entrer l\'ID de l\'assemblée générale');
      return;
    }
    try {
      actionLoading = true;
      budget = await budgetsApi.approve(budgetId, meetingId);
      showApproveModal = false;
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible d\'approuver'));
    } finally {
      actionLoading = false;
    }
  }

  async function rejectBudget() {
    try {
      actionLoading = true;
      budget = await budgetsApi.reject(budgetId, rejectReason || undefined);
      showRejectModal = false;
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible de rejeter'));
    } finally {
      actionLoading = false;
    }
  }

  async function archiveBudget() {
    if (!confirm('Archiver ce budget ?')) return;
    try {
      actionLoading = true;
      budget = await budgetsApi.archive(budgetId);
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible d\'archiver'));
    } finally {
      actionLoading = false;
    }
  }

  async function deleteBudget() {
    if (!confirm('Supprimer ce budget ? Cette action est irreversible.')) return;
    try {
      await budgetsApi.delete(budgetId);
      window.location.href = '/budgets';
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible de supprimer'));
    }
  }
</script>

{#if loading}
  <div class="flex justify-center py-12">
    <div class="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
  </div>
{:else if error}
  <div class="bg-red-50 border border-red-200 rounded-lg p-4">
    <p class="text-red-700">{error}</p>
    <button on:click={loadBudget} class="mt-2 text-sm text-red-600 underline">Reessayer</button>
  </div>
{:else if budget}
  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <div>
            <h1 class="text-2xl font-bold text-white">Budget {budget.fiscal_year}</h1>
            <p class="text-primary-100 mt-1">Immeuble: {budget.building_id.substring(0, 8)}...</p>
          </div>
          <BudgetStatusBadge status={budget.status} />
        </div>
      </div>
    </div>

    <!-- Amounts -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">Budget Ordinaire</p>
        <p class="text-2xl font-bold text-gray-900">{formatCurrency(budget.ordinary_budget)}</p>
        <p class="text-xs text-gray-500 mt-1">Charges courantes</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">Budget Extraordinaire</p>
        <p class="text-2xl font-bold text-gray-900">{formatCurrency(budget.extraordinary_budget)}</p>
        <p class="text-xs text-gray-500 mt-1">Gros travaux</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">Budget Total</p>
        <p class="text-2xl font-bold text-primary-600">{formatCurrency(budget.total_budget)}</p>
      </div>
      <div class="bg-white rounded-lg shadow p-6">
        <p class="text-sm text-gray-600 mb-1">Provision Mensuelle</p>
        <p class="text-2xl font-bold text-green-600">{formatCurrency(budget.monthly_provision_amount)}</p>
        <p class="text-xs text-gray-500 mt-1">{formatCurrency(budget.total_budget)} / 12</p>
      </div>
    </div>

    <!-- Timeline -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">Chronologie</h2>
      <div class="space-y-3">
        <div class="flex justify-between py-2 border-b">
          <span class="text-gray-600">Cree le</span>
          <span class="font-medium">{formatDate(budget.created_at)}</span>
        </div>
        {#if budget.submitted_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">Soumis le</span>
            <span class="font-medium">{formatDate(budget.submitted_date)}</span>
          </div>
        {/if}
        {#if budget.approved_date}
          <div class="flex justify-between py-2 border-b">
            <span class="text-gray-600">Approuve le</span>
            <span class="font-medium text-green-600">{formatDate(budget.approved_date)}</span>
          </div>
        {/if}
        {#if budget.notes}
          <div class="flex justify-between py-2">
            <span class="text-gray-600">Notes</span>
            <span class="font-medium text-right max-w-md">{budget.notes}</span>
          </div>
        {/if}
      </div>
    </div>

    <!-- Variance Analysis (only for approved budgets) -->
    {#if variance}
      <div class="bg-white rounded-lg shadow p-6">
        <h2 class="text-lg font-semibold text-gray-900 mb-4">
          Analyse des Ecarts
          {#if variance.has_overruns}
            <span class="ml-2 px-2 py-0.5 bg-red-100 text-red-800 text-xs rounded-full">Depassement</span>
          {/if}
        </h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-4">
          <div>
            <p class="text-sm text-gray-600">Ordinaire</p>
            <p class="text-lg font-bold {variance.variance_ordinary >= 0 ? 'text-green-600' : 'text-red-600'}">
              {variance.variance_ordinary >= 0 ? '+' : ''}{formatCurrency(variance.variance_ordinary)}
              <span class="text-sm font-normal">({variance.variance_ordinary_pct.toFixed(1)}%)</span>
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600">Extraordinaire</p>
            <p class="text-lg font-bold {variance.variance_extraordinary >= 0 ? 'text-green-600' : 'text-red-600'}">
              {variance.variance_extraordinary >= 0 ? '+' : ''}{formatCurrency(variance.variance_extraordinary)}
              <span class="text-sm font-normal">({variance.variance_extraordinary_pct.toFixed(1)}%)</span>
            </p>
          </div>
          <div>
            <p class="text-sm text-gray-600">Total</p>
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
              <span>Ordinaire: {formatCurrency(variance.actual_ordinary)} / {formatCurrency(variance.budgeted_ordinary)}</span>
              <span>{Math.round((variance.actual_ordinary / variance.budgeted_ordinary) * 100)}%</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="h-2 rounded-full {Math.min(100, (variance.actual_ordinary / variance.budgeted_ordinary) * 100) > 100 ? 'bg-red-500' : 'bg-green-500'}" style="width: {Math.min(100, (variance.actual_ordinary / variance.budgeted_ordinary) * 100)}%"></div>
            </div>
          </div>
          <div>
            <div class="flex justify-between text-sm text-gray-600 mb-1">
              <span>Extraordinaire: {formatCurrency(variance.actual_extraordinary)} / {formatCurrency(variance.budgeted_extraordinary)}</span>
              <span>{Math.round((variance.actual_extraordinary / variance.budgeted_extraordinary) * 100)}%</span>
            </div>
            <div class="w-full bg-gray-200 rounded-full h-2">
              <div class="h-2 rounded-full {Math.min(100, (variance.actual_extraordinary / variance.budgeted_extraordinary) * 100) > 100 ? 'bg-red-500' : 'bg-blue-500'}" style="width: {Math.min(100, (variance.actual_extraordinary / variance.budgeted_extraordinary) * 100)}%"></div>
            </div>
          </div>
        </div>

        <div class="mt-4 p-3 bg-gray-50 rounded-lg text-sm text-gray-600">
          <p>Mois ecoules: {variance.months_elapsed}/12 | Projection fin d'annee: {formatCurrency(variance.projected_year_end_total)}</p>
        </div>
      </div>
    {/if}

    <!-- Actions -->
    <div class="bg-white rounded-lg shadow p-6">
      <h2 class="text-lg font-semibold text-gray-900 mb-4">Actions</h2>
      <div class="flex flex-wrap gap-3">
        {#if budget.status === 'draft' || budget.status === 'rejected'}
          <button
            on:click={submitBudget}
            disabled={actionLoading}
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition disabled:opacity-50"
          >
            Soumettre pour approbation AG
          </button>
        {/if}

        {#if budget.status === 'submitted'}
          <button
            on:click={() => showApproveModal = true}
            disabled={actionLoading}
            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition disabled:opacity-50"
          >
            Approuver
          </button>
          <button
            on:click={() => showRejectModal = true}
            disabled={actionLoading}
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition disabled:opacity-50"
          >
            Rejeter
          </button>
        {/if}

        {#if budget.status === 'approved'}
          <button
            on:click={archiveBudget}
            disabled={actionLoading}
            class="px-4 py-2 bg-yellow-600 text-white rounded-lg hover:bg-yellow-700 transition disabled:opacity-50"
          >
            Archiver
          </button>
        {/if}

        {#if budget.status === 'draft'}
          <button
            on:click={deleteBudget}
            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition"
          >
            Supprimer
          </button>
        {/if}

        <a href="/budgets" class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50 transition">
          Retour a la liste
        </a>
      </div>
    </div>
  </div>

  <!-- Approve Modal -->
  {#if showApproveModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Approuver le Budget</h3>
        <p class="text-sm text-gray-600 mb-4">
          Indiquez l'assemblee generale qui a approuve ce budget.
        </p>
        <div class="mb-4">
          <label for="meeting-id" class="block text-sm font-medium text-gray-700 mb-1">
            ID de l'Assemblee Generale
          </label>
          <input
            id="meeting-id"
            type="text"
            bind:value={meetingId}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder="UUID de l'AG"
          />
        </div>
        <div class="flex justify-end space-x-3">
          <button on:click={() => showApproveModal = false} class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
            Annuler
          </button>
          <button on:click={approveBudget} disabled={actionLoading} class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 disabled:opacity-50">
            Approuver
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Reject Modal -->
  {#if showRejectModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Rejeter le Budget</h3>
        <div class="mb-4">
          <label for="reject-reason" class="block text-sm font-medium text-gray-700 mb-1">
            Raison du rejet (optionnel)
          </label>
          <textarea
            id="reject-reason"
            bind:value={rejectReason}
            rows="3"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder="Expliquez la raison du rejet..."
          ></textarea>
        </div>
        <div class="flex justify-end space-x-3">
          <button on:click={() => showRejectModal = false} class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50">
            Annuler
          </button>
          <button on:click={rejectBudget} disabled={actionLoading} class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 disabled:opacity-50">
            Rejeter
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}
