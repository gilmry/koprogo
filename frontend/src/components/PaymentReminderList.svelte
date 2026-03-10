<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';

  export let ownerId: string | null = null;
  export let expenseId: string | null = null;

  let reminders: any[] = [];
  let loading = true;
  let error = '';

  // Filters
  let filterStatus: string = 'all';
  let filterLevel: string = 'all';

  // Stats
  let stats: any = null;

  onMount(async () => {
    await Promise.all([loadReminders(), loadStats()]);
  });

  async function loadReminders() {
    try {
      loading = true;
      error = '';

      let endpoint = '/payment-reminders';
      if (ownerId) {
        endpoint = `/owners/${ownerId}/payment-reminders`;
      } else if (expenseId) {
        endpoint = `/expenses/${expenseId}/payment-reminders`;
      }

      reminders = await api.get(endpoint);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des relances';
      console.error('Error loading reminders:', err);
    } finally {
      loading = false;
    }
  }

  async function loadStats() {
    if (ownerId || expenseId) return; // Stats only for organization view

    try {
      stats = await api.get('/payment-reminders/stats');
    } catch (err: any) {
      console.error('Error loading stats:', err);
    }
  }

  function getLevelBadge(level: string): { class: string; label: string; emoji: string } {
    const badges: Record<string, { class: string; label: string; emoji: string }> = {
      'FirstReminder': { class: 'bg-yellow-100 text-yellow-800', label: 'Rappel Aimable', emoji: '📧' },
      'SecondReminder': { class: 'bg-orange-100 text-orange-800', label: 'Relance Ferme', emoji: '⚠️' },
      'FormalNotice': { class: 'bg-red-100 text-red-800', label: 'Mise en Demeure', emoji: '🚨' },
      'LegalAction': { class: 'bg-purple-100 text-purple-800', label: 'Procédure Huissier', emoji: '⚖️' }
    };
    return badges[level] || { class: 'bg-gray-100 text-gray-800', label: level, emoji: '📄' };
  }

  function getStatusBadge(status: string): { class: string; label: string } {
    const badges: Record<string, { class: string; label: string }> = {
      'Pending': { class: 'bg-blue-100 text-blue-800', label: 'En attente' },
      'Sent': { class: 'bg-indigo-100 text-indigo-800', label: 'Envoyée' },
      'Opened': { class: 'bg-purple-100 text-purple-800', label: 'Ouverte' },
      'Paid': { class: 'bg-green-100 text-green-800', label: 'Payée' },
      'Escalated': { class: 'bg-orange-100 text-orange-800', label: 'Escaladée' },
      'Cancelled': { class: 'bg-gray-100 text-gray-800', label: 'Annulée' }
    };
    return badges[status] || { class: 'bg-gray-100 text-gray-800', label: status };
  }

  function formatCurrency(amount: number): string {
    return new Intl.NumberFormat('fr-BE', { style: 'currency', currency: 'EUR' }).format(amount);
  }

  function formatDate(dateString: string | null): string {
    if (!dateString) return '-';
    return new Date(dateString).toLocaleDateString('fr-BE', {
      year: 'numeric',
      month: 'short',
      day: 'numeric'
    });
  }

  $: filteredReminders = reminders.filter(r => {
    if (filterStatus !== 'all' && r.status !== filterStatus) return false;
    if (filterLevel !== 'all' && r.level !== filterLevel) return false;
    return true;
  });

  function bulkCreateReminders() {
    if (confirm('Créer automatiquement des relances pour toutes les charges impayées ?')) {
      createBulkReminders();
    }
  }

  async function createBulkReminders() {
    try {
      loading = true;
      const response = await api.post('/payment-reminders/bulk-create', {
        min_days_overdue: 15
      });
      toast.success(`${response.created_count} relances créées, ${response.skipped_count} ignorées`);
      await loadReminders();
      await loadStats();
    } catch (err: any) {
      toast.error('Erreur: ' + (err.message || 'Impossible de créer les relances'));
    } finally {
      loading = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Stats Cards (only for organization view) -->
  {#if stats && !ownerId && !expenseId}
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Total Impayés</span>
          <span class="text-2xl">💰</span>
        </div>
        <p class="text-3xl font-bold text-gray-900">{formatCurrency(stats.total_owed)}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Pénalités</span>
          <span class="text-2xl">📊</span>
        </div>
        <p class="text-3xl font-bold text-red-600">{formatCurrency(stats.total_penalties)}</p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Relances Actives</span>
          <span class="text-2xl">📧</span>
        </div>
        <p class="text-3xl font-bold text-blue-600">
          {stats.status_counts.find((s: any) => s.status === 'Sent')?.count || 0}
        </p>
      </div>

      <div class="bg-white rounded-lg shadow p-6">
        <div class="flex items-center justify-between mb-2">
          <span class="text-gray-600 text-sm font-medium">Taux Récupération</span>
          <span class="text-2xl">✅</span>
        </div>
        <p class="text-3xl font-bold text-green-600">
          {stats.status_counts.find((s: any) => s.status === 'Paid')?.count || 0}
        </p>
      </div>
    </div>
  {/if}

  <!-- Filters and Actions -->
  <div class="bg-white rounded-lg shadow p-6">
    <div class="flex flex-wrap items-center justify-between gap-4">
      <div class="flex items-center space-x-4">
        <div>
          <label for="filter-status" class="block text-sm font-medium text-gray-700 mb-1">
            Statut
          </label>
          <select
            id="filter-status"
            bind:value={filterStatus}
            class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          >
            <option value="all">Tous</option>
            <option value="Pending">En attente</option>
            <option value="Sent">Envoyée</option>
            <option value="Opened">Ouverte</option>
            <option value="Paid">Payée</option>
            <option value="Escalated">Escaladée</option>
            <option value="Cancelled">Annulée</option>
          </select>
        </div>

        <div>
          <label for="filter-level" class="block text-sm font-medium text-gray-700 mb-1">
            Niveau
          </label>
          <select
            id="filter-level"
            bind:value={filterLevel}
            class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
          >
            <option value="all">Tous</option>
            <option value="FirstReminder">Rappel Aimable</option>
            <option value="SecondReminder">Relance Ferme</option>
            <option value="FormalNotice">Mise en Demeure</option>
          </select>
        </div>
      </div>

      {#if !ownerId && !expenseId}
        <button
          on:click={bulkCreateReminders}
          class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors font-medium"
        >
          🤖 Créer Relances Automatiques
        </button>
      {/if}
    </div>
  </div>

  <!-- Error Message -->
  {#if error}
    <div class="bg-red-50 border-l-4 border-red-400 p-4">
      <p class="text-sm text-red-700">{error}</p>
    </div>
  {/if}

  <!-- Reminders List -->
  {#if loading}
    <div class="flex justify-center items-center py-12">
      <div class="text-center">
        <svg class="animate-spin h-12 w-12 text-primary-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        <p class="text-gray-600">Chargement des relances...</p>
      </div>
    </div>
  {:else if filteredReminders.length === 0}
    <div class="bg-white rounded-lg shadow p-12 text-center">
      <div class="text-6xl mb-4">📭</div>
      <h3 class="text-xl font-semibold text-gray-900 mb-2">Aucune relance</h3>
      <p class="text-gray-600">Aucune relance de paiement trouvée pour les critères sélectionnés.</p>
    </div>
  {:else}
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Niveau
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Propriétaire
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Montant
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Pénalités
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Jours Retard
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Statut
            </th>
            <th scope="col" class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Date Envoi
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredReminders as reminder}
            {@const levelBadge = getLevelBadge(reminder.level)}
            {@const statusBadge = getStatusBadge(reminder.status)}
            <tr class="hover:bg-gray-50 transition-colors">
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {levelBadge.class}">
                  <span class="mr-1">{levelBadge.emoji}</span>
                  {levelBadge.label}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                {#if reminder.owner_name}
                  <a href="/owners/{reminder.owner_id}" class="text-primary-600 hover:text-primary-700">
                    {reminder.owner_name}
                  </a>
                  {#if reminder.owner_email}
                    <br><span class="text-xs text-gray-500">{reminder.owner_email}</span>
                  {/if}
                {:else}
                  <a href="/owners/{reminder.owner_id}" class="text-primary-600 hover:text-primary-700">
                    Propriétaire #{reminder.owner_id.substring(0, 8)}
                  </a>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {formatCurrency(reminder.amount_owed)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-red-600">
                +{formatCurrency(reminder.penalty_amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                <span class="font-bold">{reminder.days_overdue}</span> jours
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="inline-flex px-3 py-1 rounded-full text-sm font-medium {statusBadge.class}">
                  {statusBadge.label}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                {formatDate(reminder.sent_date)}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>

    <div class="bg-white rounded-lg shadow p-4">
      <p class="text-sm text-gray-600">
        Total: {filteredReminders.length} relance{filteredReminders.length > 1 ? 's' : ''}
      </p>
    </div>
  {/if}
</div>
