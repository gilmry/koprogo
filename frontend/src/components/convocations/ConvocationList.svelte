<script lang="ts">
  import { onMount } from 'svelte';
  import {
    convocationsApi,
    type Convocation,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';

  export let buildingId: string;

  let convocations: Convocation[] = [];
  let filteredConvocations: Convocation[] = [];
  let loading = true;
  let error = '';
  let statusFilter: ConvocationStatus | 'all' = 'all';

  onMount(async () => {
    await loadConvocations();
  });

  async function loadConvocations() {
    try {
      loading = true;
      error = '';
      convocations = await convocationsApi.listByBuilding(buildingId);
      applyFilters();
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des convocations';
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredConvocations = convocations.filter(c => {
      if (statusFilter === 'all') return true;
      return c.status === statusFilter;
    });
  }

  $: if (statusFilter) applyFilters();

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
    });
  }

  function getStatusConfig(status: ConvocationStatus): { bg: string; text: string; label: string; icon: string } {
    const config: Record<ConvocationStatus, { bg: string; text: string; label: string; icon: string }> = {
      [ConvocationStatus.Draft]: { bg: 'bg-gray-100', text: 'text-gray-800', label: 'Brouillon', icon: '📝' },
      [ConvocationStatus.Scheduled]: { bg: 'bg-blue-100', text: 'text-blue-800', label: 'Planifiée', icon: '📅' },
      [ConvocationStatus.Sent]: { bg: 'bg-green-100', text: 'text-green-800', label: 'Envoyée', icon: '✅' },
      [ConvocationStatus.Cancelled]: { bg: 'bg-red-100', text: 'text-red-800', label: 'Annulée', icon: '❌' },
    };
    return config[status] || config[ConvocationStatus.Draft];
  }

  function getMeetingTypeLabel(type: MeetingType): string {
    switch (type) {
      case MeetingType.Ordinary: return 'AG Ordinaire';
      case MeetingType.Extraordinary: return 'AG Extraordinaire';
      case MeetingType.SecondConvocation: return '2e Convocation';
      default: return type;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <h3 class="text-lg leading-6 font-medium text-gray-900">
      📨 Convocations
    </h3>
    <p class="mt-1 text-sm text-gray-500">
      Gérez les convocations aux assemblées générales avec respect des délais légaux belges.
    </p>
  </div>

  <!-- Filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex items-center space-x-4">
      <label class="text-sm font-medium text-gray-700">Statut:</label>
      <select
        bind:value={statusFilter}
        class="text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
      >
        <option value="all">Tous</option>
        <option value={ConvocationStatus.Draft}>Brouillon</option>
        <option value={ConvocationStatus.Scheduled}>Planifiée</option>
        <option value={ConvocationStatus.Sent}>Envoyée</option>
        <option value={ConvocationStatus.Cancelled}>Annulée</option>
      </select>
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">Chargement des convocations...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadConvocations} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
        Réessayer
      </button>
    </div>
  {:else if filteredConvocations.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucune convocation trouvée</p>
      <p class="mt-2 text-sm text-gray-400">
        Les convocations sont créées depuis la page de détail d'une assemblée.
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredConvocations as convocation (convocation.id)}
        {@const statusCfg = getStatusConfig(convocation.status)}
        <li class="hover:bg-gray-50">
          <a href="/convocation-detail?id={convocation.id}" class="block px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-3 mb-2">
                  <h4 class="text-sm font-medium text-amber-700">
                    {getMeetingTypeLabel(convocation.meeting_type)}
                  </h4>
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}">
                    <span class="mr-1">{statusCfg.icon}</span>
                    {statusCfg.label}
                  </span>
                  {#if !convocation.respects_legal_deadline}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-700">
                      ⚠️ Délai non respecté
                    </span>
                  {/if}
                </div>

                <div class="mt-2 flex items-center text-sm text-gray-500 flex-wrap gap-x-4 gap-y-1">
                  <span>📅 AG le {formatDate(convocation.meeting_date)}</span>
                  <span>📧 {convocation.total_recipients} destinataire{convocation.total_recipients > 1 ? 's' : ''}</span>
                  {#if convocation.opened_count > 0}
                    <span>👁️ {convocation.opened_count} ouvert{convocation.opened_count > 1 ? 's' : ''}</span>
                  {/if}
                  {#if convocation.will_attend_count > 0}
                    <span>✅ {convocation.will_attend_count} présent{convocation.will_attend_count > 1 ? 's' : ''}</span>
                  {/if}
                  <span class="text-xs text-gray-400">Créée le {formatDate(convocation.created_at)}</span>
                </div>
              </div>

              <div class="ml-4 flex flex-col items-center gap-2">
                <svg class="h-5 w-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </div>
            </div>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</div>
