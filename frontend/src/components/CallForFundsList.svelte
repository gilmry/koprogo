<script lang="ts">
  import { onMount } from 'svelte';
  import { callForFundsApi } from '../lib/api';
  import { showToast } from '../stores/toast';

  export let buildingId: string | undefined = undefined;

  let calls: any[] = [];
  let loading = true;
  let showCreateModal = false;

  onMount(async () => {
    await loadCalls();
  });

  async function loadCalls() {
    try {
      loading = true;
      calls = await callForFundsApi.list(buildingId);
    } catch (error: any) {
      showToast(error.message || 'Erreur lors du chargement des appels de fonds', 'error');
    } finally {
      loading = false;
    }
  }

  async function handleSend(id: string) {
    if (!confirm('Êtes-vous sûr de vouloir envoyer cet appel de fonds ? Cela générera automatiquement les contributions individuelles pour tous les copropriétaires.')) {
      return;
    }

    try {
      const result = await callForFundsApi.send(id);
      showToast(`Appel de fonds envoyé avec succès. ${result.contributions_generated} contributions générées.`, 'success');
      await loadCalls();
    } catch (error: any) {
      showToast(error.message || 'Erreur lors de l\'envoi', 'error');
    }
  }

  async function handleCancel(id: string) {
    if (!confirm('Êtes-vous sûr de vouloir annuler cet appel de fonds ?')) {
      return;
    }

    try {
      await callForFundsApi.cancel(id);
      showToast('Appel de fonds annulé', 'success');
      await loadCalls();
    } catch (error: any) {
      showToast(error.message || 'Erreur lors de l\'annulation', 'error');
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Êtes-vous sûr de vouloir supprimer ce brouillon ?')) {
      return;
    }

    try {
      await callForFundsApi.delete(id);
      showToast('Appel de fonds supprimé', 'success');
      await loadCalls();
    } catch (error: any) {
      showToast(error.message || 'Erreur lors de la suppression', 'error');
    }
  }

  function formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString('fr-BE');
  }

  function formatAmount(amount: number): string {
    return new Intl.NumberFormat('fr-BE', {
      style: 'currency',
      currency: 'EUR',
    }).format(amount);
  }

  function getStatusBadgeClass(status: string): string {
    const classes: Record<string, string> = {
      draft: 'bg-gray-100 text-gray-800',
      sent: 'bg-blue-100 text-blue-800',
      partial: 'bg-yellow-100 text-yellow-800',
      completed: 'bg-green-100 text-green-800',
      cancelled: 'bg-red-100 text-red-800',
    };
    return classes[status] || 'bg-gray-100 text-gray-800';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      draft: 'Brouillon',
      sent: 'Envoyé',
      partial: 'Partiellement payé',
      completed: 'Complété',
      cancelled: 'Annulé',
    };
    return labels[status] || status;
  }

  function getContributionTypeLabel(type: string): string {
    const labels: Record<string, string> = {
      regular: 'Charges régulières',
      extraordinary: 'Charges extraordinaires',
      advance: 'Avance',
      adjustment: 'Régularisation',
    };
    return labels[type] || type;
  }
</script>

<div class="space-y-4">
  <div class="flex justify-between items-center">
    <h2 class="text-2xl font-bold text-gray-900">Appels de Fonds</h2>
    <button
      on:click={() => (showCreateModal = true)}
      class="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700"
    >
      + Nouvel Appel de Fonds
    </button>
  </div>

  {#if loading}
    <div class="text-center py-8">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      <p class="mt-2 text-gray-600">Chargement...</p>
    </div>
  {:else if calls.length === 0}
    <div class="text-center py-12 bg-gray-50 rounded-lg">
      <svg
        class="mx-auto h-12 w-12 text-gray-400"
        fill="none"
        viewBox="0 0 24 24"
        stroke="currentColor"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
        />
      </svg>
      <p class="mt-2 text-gray-600">Aucun appel de fonds</p>
      <button
        on:click={() => (showCreateModal = true)}
        class="mt-4 text-blue-600 hover:text-blue-800"
      >
        Créer le premier appel de fonds
      </button>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Titre
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Type
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Montant
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Date d'appel
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Date d'échéance
            </th>
            <th class="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
              Statut
            </th>
            <th class="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
              Actions
            </th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each calls as call (call.id)}
            <tr class:bg-red-50={call.is_overdue}>
              <td class="px-6 py-4 whitespace-nowrap">
                <div class="text-sm font-medium text-gray-900">{call.title}</div>
                <div class="text-sm text-gray-500">{call.description}</div>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {getContributionTypeLabel(call.contribution_type)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                {formatAmount(call.total_amount)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.call_date)}
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                {formatDate(call.due_date)}
                {#if call.is_overdue}
                  <span class="ml-2 px-2 py-1 text-xs font-semibold rounded-full bg-red-100 text-red-800">
                    En retard
                  </span>
                {/if}
              </td>
              <td class="px-6 py-4 whitespace-nowrap">
                <span class="px-2 inline-flex text-xs leading-5 font-semibold rounded-full {getStatusBadgeClass(call.status)}">
                  {getStatusLabel(call.status)}
                </span>
              </td>
              <td class="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                {#if call.status === 'draft'}
                  <button
                    on:click={() => handleSend(call.id)}
                    class="text-blue-600 hover:text-blue-900"
                    title="Envoyer l'appel de fonds"
                  >
                    Envoyer
                  </button>
                  <button
                    on:click={() => handleDelete(call.id)}
                    class="text-red-600 hover:text-red-900"
                    title="Supprimer le brouillon"
                  >
                    Supprimer
                  </button>
                {:else if call.status === 'sent' || call.status === 'partial'}
                  <button
                    on:click={() => handleCancel(call.id)}
                    class="text-orange-600 hover:text-orange-900"
                    title="Annuler l'appel de fonds"
                  >
                    Annuler
                  </button>
                  <a
                    href="/owner-contributions?call_for_funds_id={call.id}"
                    class="text-green-600 hover:text-green-900"
                    title="Voir les contributions"
                  >
                    Contributions
                  </a>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

{#if showCreateModal}
  <div class="fixed inset-0 bg-gray-500 bg-opacity-75 flex items-center justify-center z-50">
    <div class="bg-white rounded-lg p-6 max-w-2xl w-full mx-4">
      <h3 class="text-lg font-medium text-gray-900 mb-4">Créer un appel de fonds</h3>
      <p class="text-sm text-gray-600 mb-4">
        Utilisez le formulaire ci-dessous pour créer un nouvel appel de fonds.
      </p>
      <div class="flex justify-end space-x-3">
        <button
          on:click={() => (showCreateModal = false)}
          class="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50"
        >
          Fermer
        </button>
      </div>
    </div>
  </div>
{/if}
