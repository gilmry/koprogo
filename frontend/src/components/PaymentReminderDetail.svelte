<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';

  export let reminderId: string;
  export let onUpdated: ((reminder: any) => void) | null = null;

  let reminder: any = null;
  let loading = false;
  let error = '';

  // Modals
  let showCancelModal = false;
  let cancelReason = '';

  let showTrackingModal = false;
  let trackingNumber = '';

  onMount(async () => {
    await loadReminder();
  });

  async function loadReminder() {
    try {
      loading = true;
      error = '';
      reminder = await api.get(`/payment-reminders/${reminderId}`);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement';
      console.error('Error loading reminder:', err);
    } finally {
      loading = false;
    }
  }

  async function markAsSent() {
    if (!confirm('Marquer cette relance comme envoyée ?')) return;

    try {
      loading = true;
      const updated = await api.put(`/payment-reminders/${reminderId}/mark-sent`, {
        pdf_path: null
      });
      reminder = updated;
      if (onUpdated) onUpdated(updated);
      alert('Relance marquée comme envoyée');
    } catch (err: any) {
      alert('Erreur: ' + (err.message || 'Impossible de marquer comme envoyée'));
    } finally {
      loading = false;
    }
  }

  async function markAsPaid() {
    if (!confirm('Marquer cette relance comme payée ?')) return;

    try {
      loading = true;
      const updated = await api.put(`/payment-reminders/${reminderId}/mark-paid`, {});
      reminder = updated;
      if (onUpdated) onUpdated(updated);
      alert('Relance marquée comme payée');
    } catch (err: any) {
      alert('Erreur: ' + (err.message || 'Impossible de marquer comme payée'));
    } finally {
      loading = false;
    }
  }

  async function escalate() {
    if (!confirm('Escalader vers le niveau supérieur de relance ?')) return;

    try {
      loading = true;
      const updated = await api.post(`/payment-reminders/${reminderId}/escalate`, {
        reason: null
      });
      reminder = await api.get(`/payment-reminders/${reminderId}`);
      if (onUpdated) onUpdated(reminder);
      alert('Relance escaladée avec succès');
    } catch (err: any) {
      alert('Erreur: ' + (err.message || 'Impossible d\'escalader'));
    } finally {
      loading = false;
    }
  }

  function openCancelModal() {
    showCancelModal = true;
    cancelReason = '';
  }

  async function confirmCancel() {
    if (!cancelReason.trim()) {
      alert('Veuillez fournir une raison d\'annulation');
      return;
    }

    try {
      loading = true;
      const updated = await api.put(`/payment-reminders/${reminderId}/cancel`, {
        reason: cancelReason
      });
      reminder = updated;
      showCancelModal = false;
      if (onUpdated) onUpdated(updated);
      alert('Relance annulée');
    } catch (err: any) {
      alert('Erreur: ' + (err.message || 'Impossible d\'annuler'));
    } finally {
      loading = false;
    }
  }

  function openTrackingModal() {
    showTrackingModal = true;
    trackingNumber = '';
  }

  async function confirmAddTracking() {
    if (!trackingNumber.trim()) {
      alert('Veuillez fournir un numéro de suivi');
      return;
    }

    try {
      loading = true;
      const updated = await api.put(`/payment-reminders/${reminderId}/tracking-number`, {
        tracking_number: trackingNumber
      });
      reminder = updated;
      showTrackingModal = false;
      if (onUpdated) onUpdated(updated);
      alert('Numéro de suivi ajouté');
    } catch (err: any) {
      alert('Erreur: ' + (err.message || 'Impossible d\'ajouter le numéro'));
    } finally {
      loading = false;
    }
  }

  function getLevelInfo(level: string) {
    const levels: Record<string, { emoji: string; label: string; description: string; class: string }> = {
      'FirstReminder': {
        emoji: '📧',
        label: 'Rappel Aimable',
        description: 'Premier rappel courtois (J+15)',
        class: 'bg-yellow-100 text-yellow-800 border-yellow-200'
      },
      'SecondReminder': {
        emoji: '⚠️',
        label: 'Relance Ferme',
        description: 'Deuxième relance avec ton plus ferme (J+30)',
        class: 'bg-orange-100 text-orange-800 border-orange-200'
      },
      'FormalNotice': {
        emoji: '🚨',
        label: 'Mise en Demeure',
        description: 'Lettre recommandée avec AR - ton juridique (J+60)',
        class: 'bg-red-100 text-red-800 border-red-200'
      },
      'LegalAction': {
        emoji: '⚖️',
        label: 'Procédure Huissier',
        description: 'Procédure de recouvrement judiciaire',
        class: 'bg-purple-100 text-purple-800 border-purple-200'
      }
    };
    return levels[level] || levels['FirstReminder'];
  }

  function getStatusBadge(status: string) {
    const badges: Record<string, { class: string; label: string }> = {
      'Pending': { class: 'bg-blue-100 text-blue-800', label: 'En attente d\'envoi' },
      'Sent': { class: 'bg-indigo-100 text-indigo-800', label: 'Envoyée' },
      'Opened': { class: 'bg-purple-100 text-purple-800', label: 'Ouverte par destinataire' },
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
      month: 'long',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  }
</script>

{#if loading && !reminder}
  <div class="flex justify-center items-center py-12">
    <div class="text-center">
      <svg class="animate-spin h-12 w-12 text-primary-600 mx-auto mb-4" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
      </svg>
      <p class="text-gray-600">Chargement...</p>
    </div>
  </div>
{:else if error}
  <div class="bg-red-50 border-l-4 border-red-400 p-4">
    <p class="text-sm text-red-700">{error}</p>
  </div>
{:else if reminder}
  {@const levelInfo = getLevelInfo(reminder.level)}
  {@const statusBadge = getStatusBadge(reminder.status)}

  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white rounded-lg shadow overflow-hidden">
      <div class="border-l-4 {levelInfo.class} border-4 p-6">
        <div class="flex items-start justify-between">
          <div>
            <div class="flex items-center space-x-3 mb-2">
              <span class="text-4xl">{levelInfo.emoji}</span>
              <div>
                <h2 class="text-2xl font-bold text-gray-900">{levelInfo.label}</h2>
                <p class="text-gray-600">{levelInfo.description}</p>
              </div>
            </div>
          </div>
          <span class="inline-flex px-4 py-2 rounded-full text-sm font-medium {statusBadge.class}">
            {statusBadge.label}
          </span>
        </div>
      </div>
    </div>

    <!-- Amount Details -->
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Montants</h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
          <p class="text-sm text-gray-600 mb-1">Montant Dû</p>
          <p class="text-2xl font-bold text-blue-600">{formatCurrency(reminder.amount_owed)}</p>
        </div>
        <div class="bg-red-50 border border-red-200 rounded-lg p-4">
          <p class="text-sm text-gray-600 mb-1">Pénalités (8% annuel)</p>
          <p class="text-2xl font-bold text-red-600">+{formatCurrency(reminder.penalty_amount)}</p>
        </div>
        <div class="bg-purple-50 border border-purple-200 rounded-lg p-4">
          <p class="text-sm text-gray-600 mb-1">Total à Payer</p>
          <p class="text-2xl font-bold text-purple-600">{formatCurrency(reminder.total_amount)}</p>
        </div>
      </div>
    </div>

    <!-- Timing Information -->
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Chronologie</h3>
      <div class="space-y-3">
        <div class="flex items-center justify-between py-2 border-b">
          <span class="text-gray-600">Date d'échéance originale</span>
          <span class="font-medium text-gray-900">{formatDate(reminder.due_date)}</span>
        </div>
        <div class="flex items-center justify-between py-2 border-b">
          <span class="text-gray-600">Jours de retard</span>
          <span class="font-bold text-red-600 text-xl">{reminder.days_overdue} jours</span>
        </div>
        <div class="flex items-center justify-between py-2 border-b">
          <span class="text-gray-600">Date d'envoi</span>
          <span class="font-medium text-gray-900">{formatDate(reminder.sent_date)}</span>
        </div>
        {#if reminder.opened_date}
          <div class="flex items-center justify-between py-2 border-b">
            <span class="text-gray-600">Date d'ouverture</span>
            <span class="font-medium text-gray-900">{formatDate(reminder.opened_date)}</span>
          </div>
        {/if}
        <div class="flex items-center justify-between py-2">
          <span class="text-gray-600">Créée le</span>
          <span class="font-medium text-gray-900">{formatDate(reminder.created_at)}</span>
        </div>
      </div>
    </div>

    <!-- Delivery Method & Tracking -->
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Méthode d'Envoi</h3>
      <div class="space-y-3">
        <div class="flex items-center justify-between py-2">
          <span class="text-gray-600">Méthode</span>
          <span class="font-medium text-gray-900">
            {#if reminder.delivery_method === 'Email'}
              📧 Email
            {:else if reminder.delivery_method === 'RegisteredLetter'}
              📮 Lettre Recommandée avec AR
            {:else if reminder.delivery_method === 'Bailiff'}
              ⚖️ Huissier de Justice
            {:else}
              {reminder.delivery_method}
            {/if}
          </span>
        </div>
        {#if reminder.tracking_number}
          <div class="flex items-center justify-between py-2 bg-yellow-50 px-3 rounded">
            <span class="text-gray-600">Numéro de suivi</span>
            <span class="font-mono font-bold text-gray-900">{reminder.tracking_number}</span>
          </div>
        {:else if reminder.delivery_method === 'RegisteredLetter'}
          <button
            on:click={openTrackingModal}
            class="text-sm text-primary-600 hover:text-primary-700"
          >
            + Ajouter un numéro de suivi
          </button>
        {/if}
        {#if reminder.pdf_path}
          <div class="flex items-center justify-between py-2">
            <span class="text-gray-600">Lettre PDF</span>
            <a href={reminder.pdf_path} class="text-primary-600 hover:text-primary-700">
              📄 Télécharger
            </a>
          </div>
        {/if}
      </div>
    </div>

    <!-- Notes -->
    {#if reminder.notes}
      <div class="bg-white rounded-lg shadow p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-3">Notes</h3>
        <p class="text-gray-700 whitespace-pre-wrap">{reminder.notes}</p>
      </div>
    {/if}

    <!-- Actions -->
    <div class="bg-white rounded-lg shadow p-6">
      <h3 class="text-lg font-semibold text-gray-900 mb-4">Actions</h3>
      <div class="flex flex-wrap gap-3">
        {#if reminder.status === 'Pending'}
          <button
            on:click={markAsSent}
            disabled={loading}
            class="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors disabled:opacity-50"
          >
            📧 Marquer comme Envoyée
          </button>
        {/if}

        {#if reminder.status === 'Sent' || reminder.status === 'Opened'}
          <button
            on:click={markAsPaid}
            disabled={loading}
            class="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50"
          >
            ✅ Marquer comme Payée
          </button>

          <button
            on:click={escalate}
            disabled={loading}
            class="px-4 py-2 bg-orange-600 text-white rounded-lg hover:bg-orange-700 transition-colors disabled:opacity-50"
          >
            ⬆️ Escalader
          </button>
        {/if}

        {#if reminder.status !== 'Paid' && reminder.status !== 'Cancelled'}
          <button
            on:click={openCancelModal}
            disabled={loading}
            class="px-4 py-2 bg-gray-600 text-white rounded-lg hover:bg-gray-700 transition-colors disabled:opacity-50"
          >
            ❌ Annuler
          </button>
        {/if}

        <a
          href="/expenses/{reminder.expense_id}"
          class="px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
        >
          📄 Voir la Facture
        </a>

        <a
          href="/owners/{reminder.owner_id}"
          class="px-4 py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-colors"
        >
          👤 Voir le Propriétaire
        </a>
      </div>
    </div>
  </div>

  <!-- Cancel Modal -->
  {#if showCancelModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Annuler la Relance</h3>
        <div class="mb-4">
          <label for="cancel-reason" class="block text-sm font-medium text-gray-700 mb-2">
            Raison de l'annulation
          </label>
          <textarea
            id="cancel-reason"
            bind:value={cancelReason}
            rows="4"
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder="Expliquez pourquoi cette relance est annulée..."
          ></textarea>
        </div>
        <div class="flex justify-end space-x-3">
          <button
            on:click={() => showCancelModal = false}
            class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Annuler
          </button>
          <button
            on:click={confirmCancel}
            class="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700"
          >
            Confirmer l'Annulation
          </button>
        </div>
      </div>
    </div>
  {/if}

  <!-- Tracking Number Modal -->
  {#if showTrackingModal}
    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div class="bg-white rounded-lg shadow-xl max-w-md w-full p-6">
        <h3 class="text-lg font-semibold text-gray-900 mb-4">Ajouter un Numéro de Suivi</h3>
        <div class="mb-4">
          <label for="tracking-number" class="block text-sm font-medium text-gray-700 mb-2">
            Numéro de suivi (Lettre Recommandée)
          </label>
          <input
            id="tracking-number"
            type="text"
            bind:value={trackingNumber}
            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-primary-500"
            placeholder="Ex: RR123456789BE"
          />
        </div>
        <div class="flex justify-end space-x-3">
          <button
            on:click={() => showTrackingModal = false}
            class="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
          >
            Annuler
          </button>
          <button
            on:click={confirmAddTracking}
            class="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700"
          >
            Ajouter
          </button>
        </div>
      </div>
    </div>
  {/if}
{/if}
