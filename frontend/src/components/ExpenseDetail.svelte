<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import type { Expense, Building } from '../lib/types';
  import Button from './ui/Button.svelte';
  import ExpenseDocuments from './ExpenseDocuments.svelte';

  let expense: Expense | null = null;
  let building: Building | null = null;
  let loading = true;
  let error = '';
  let expenseId: string = '';

  onMount(() => {
    const urlParams = new URLSearchParams(window.location.search);
    expenseId = urlParams.get('id') || '';

    if (!expenseId) {
      error = 'ID de la dépense manquant';
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

      // Load building info
      if (expense && expense.building_id) {
        try {
          building = await api.get<Building>(`/buildings/${expense.building_id}`);
        } catch (e) {
          console.error('Error loading building:', e);
        }
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement de la dépense';
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
      alert('Dépense marquée comme payée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de la mise à jour';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error marking as paid:', e);
    }
  };

  const handleMarkOverdue = async () => {
    if (!expense) return;

    try {
      await api.post(`/expenses/${expense.id}/mark-overdue`, {});
      await loadExpense();
      alert('Dépense marquée comme en retard avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de la mise à jour';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error marking as overdue:', e);
    }
  };

  const handleCancel = async () => {
    if (!expense) return;

    if (!confirm('Êtes-vous sûr de vouloir annuler cette dépense ?')) {
      return;
    }

    try {
      await api.post(`/expenses/${expense.id}/cancel`, {});
      await loadExpense();
      alert('Dépense annulée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de l\'annulation';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error cancelling expense:', e);
    }
  };

  const handleReactivate = async () => {
    if (!expense) return;

    try {
      await api.post(`/expenses/${expense.id}/reactivate`, {});
      await loadExpense();
      alert('Dépense réactivée avec succès');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de la réactivation';
      alert(`Erreur: ${errorMsg}`);
      console.error('Error reactivating expense:', e);
    }
  };

  const handleUnpay = async () => {
    if (!expense) return;

    if (!confirm('Êtes-vous sûr de vouloir annuler le paiement de cette dépense ?')) {
      return;
    }

    try {
      await api.post(`/expenses/${expense.id}/unpay`, {});
      await loadExpense();
      alert('Paiement annulé avec succès - dépense remise en attente');
    } catch (e) {
      const errorMsg = e instanceof Error ? e.message : 'Erreur lors de l\'annulation du paiement';
      alert(`Erreur: ${errorMsg}`);
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
      'Paid': { class: 'bg-green-100 text-green-800', label: 'Payée' },
      'Pending': { class: 'bg-yellow-100 text-yellow-800', label: 'En attente' },
      'Overdue': { class: 'bg-red-100 text-red-800', label: 'En retard' },
      'Cancelled': { class: 'bg-gray-100 text-gray-800', label: 'Annulée' }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function getCategoryLabel(category: string): string {
    const labels: Record<string, string> = {
      'Maintenance': 'Entretien',
      'Repair': 'Réparation',
      'Insurance': 'Assurance',
      'Utilities': 'Charges',
      'Management': 'Gestion',
      'Other': 'Autre'
    };
    return labels[category] || category;
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement...</p>
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
            Retour
          </button>
          <h1 class="text-3xl font-bold text-gray-900">Détail de la dépense</h1>
        </div>
        <div class="flex gap-2">
          {#if expense.payment_status === 'Pending'}
            <Button variant="primary" on:click={handleMarkPaid}>
              Marquer comme payée
            </Button>
            <Button variant="outline" on:click={handleMarkOverdue}>
              Marquer en retard
            </Button>
            <Button variant="outline" on:click={handleCancel}>
              Annuler
            </Button>
          {:else if expense.payment_status === 'Overdue'}
            <Button variant="primary" on:click={handleMarkPaid}>
              Marquer comme payée
            </Button>
            <Button variant="outline" on:click={handleCancel}>
              Annuler
            </Button>
          {:else if expense.payment_status === 'Paid'}
            <Button variant="outline" on:click={handleUnpay}>
              Annuler le paiement
            </Button>
          {:else if expense.payment_status === 'Cancelled'}
            <Button variant="primary" on:click={handleReactivate}>
              Réactiver
            </Button>
          {/if}
        </div>
      </div>
    </div>

    <!-- Main Info Card -->
    <div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8">
      <div class="bg-gradient-to-r from-primary-600 to-primary-700 px-6 py-4">
        <div class="flex items-center justify-between">
          <h2 class="text-xl font-semibold text-white">Informations générales</h2>
          <span class="px-3 py-1 rounded-full text-sm font-medium {getStatusBadge(expense.payment_status).class}">
            {getStatusBadge(expense.payment_status).label}
          </span>
        </div>
      </div>
      <div class="p-6">
        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          <!-- Description -->
          <div class="md:col-span-2">
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Description</h3>
            <p class="text-lg text-gray-900">{expense.description}</p>
          </div>

          <!-- Amount -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Montant</h3>
            <p class="text-2xl font-bold text-gray-900">{formatCurrency(expense.amount)}</p>
          </div>

          <!-- Category -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Catégorie</h3>
            <p class="text-lg text-gray-900">{getCategoryLabel(expense.category)}</p>
          </div>

          <!-- Expense Date -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Date de la dépense</h3>
            <p class="text-lg text-gray-900">{formatDate(expense.expense_date)}</p>
          </div>

          <!-- Due Date -->
          <div>
            <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Date d'échéance</h3>
            <p class="text-lg text-gray-900">{formatDate(expense.due_date)}</p>
          </div>

          {#if expense.paid_date}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Date de paiement</h3>
              <p class="text-lg text-gray-900">{formatDate(expense.paid_date)}</p>
            </div>
          {/if}

          <!-- Building -->
          {#if building}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Immeuble</h3>
              <a href="/building-detail?id={building.id}" class="text-lg text-primary-600 hover:text-primary-700 hover:underline">
                {building.name}
              </a>
              <p class="text-sm text-gray-600">{building.address}</p>
            </div>
          {/if}

          {#if expense.supplier}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Fournisseur</h3>
              <p class="text-lg text-gray-900">{expense.supplier}</p>
            </div>
          {/if}

          {#if expense.invoice_number}
            <div>
              <h3 class="text-sm font-medium text-gray-500 uppercase tracking-wider mb-2">Numéro de facture</h3>
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
  {/if}
</div>
