<script lang="ts">
  import { onMount } from 'svelte';
  import {
    convocationsApi,
    type Convocation,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import ConvocationTrackingSummary from './ConvocationTrackingSummary.svelte';
  import ConvocationRecipientList from './ConvocationRecipientList.svelte';

  export let meetingId: string;
  export let meetingStatus: string = 'Scheduled';
  export let buildingId: string = '';

  let convocation: Convocation | null = null;
  let loading = true;
  let error = '';
  let showRecipients = false;
  let actionLoading = false;

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;

  onMount(async () => {
    await loadConvocation();
  });

  async function loadConvocation() {
    try {
      loading = true;
      error = '';
      convocation = await convocationsApi.getByMeetingId(meetingId);
    } catch (err: any) {
      // 404 = pas de convocation pour ce meeting, c'est normal
      if (err.message?.includes('404') || err.message?.includes('not found') || err.message?.includes('Not Found')) {
        convocation = null;
      } else {
        error = err.message || 'Erreur lors du chargement';
      }
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    try {
      actionLoading = true;
      convocation = await convocationsApi.create({
        meeting_id: meetingId,
        building_id: buildingId,
        meeting_type: MeetingType.Ordinary,
        meeting_date: '', // Le backend le calculera depuis le meeting
        language: 'fr',
      });
      toast.success('Convocation créée');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la création');
    } finally {
      actionLoading = false;
    }
  }

  async function handleSend() {
    if (!convocation) return;
    if (!confirm('Envoyer la convocation à tous les copropriétaires ?')) return;

    try {
      actionLoading = true;
      convocation = await convocationsApi.send(convocation.id);
      toast.success('Convocation envoyée');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'envoi');
    } finally {
      actionLoading = false;
    }
  }

  async function handleCancel() {
    if (!convocation) return;
    if (!confirm('Annuler cette convocation ?')) return;

    try {
      actionLoading = true;
      convocation = await convocationsApi.cancel(convocation.id);
      toast.success('Convocation annulée');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'annulation');
    } finally {
      actionLoading = false;
    }
  }

  async function handleSendReminders() {
    if (!convocation) return;

    try {
      actionLoading = true;
      await convocationsApi.sendReminders(convocation.id);
      toast.success('Rappels envoyés');
      await loadConvocation();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'envoi des rappels');
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (!convocation) return;
    if (!confirm('Supprimer cette convocation ? Cette action est irréversible.')) return;

    try {
      actionLoading = true;
      await convocationsApi.delete(convocation.id);
      convocation = null;
      toast.success('Convocation supprimée');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    } finally {
      actionLoading = false;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
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
      case MeetingType.Ordinary: return 'AG Ordinaire (15j min)';
      case MeetingType.Extraordinary: return 'AG Extraordinaire (8j min)';
      case MeetingType.SecondConvocation: return '2e Convocation (8j min)';
      default: return type;
    }
  }
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden">
  <div class="bg-gradient-to-r from-amber-600 to-amber-700 px-6 py-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-white">Convocation</h2>
      {#if convocation}
        {@const statusCfg = getStatusConfig(convocation.status)}
        <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}">
          <span class="mr-1">{statusCfg.icon}</span>
          {statusCfg.label}
        </span>
      {/if}
    </div>
  </div>

  <div class="p-6">
    {#if loading}
      <div class="py-6 text-center">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
        <p class="mt-2 text-sm text-gray-500">Chargement...</p>
      </div>

    {:else if error}
      <div class="p-4 bg-red-50 border border-red-200 rounded-md">
        <p class="text-sm text-red-800">{error}</p>
        <button on:click={loadConvocation} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
          Réessayer
        </button>
      </div>

    {:else if !convocation}
      <!-- No convocation yet -->
      <div class="py-6 text-center">
        <p class="text-gray-500 mb-2">Aucune convocation n'a été créée pour cette assemblée.</p>
        {#if isAdmin && meetingStatus === 'Scheduled'}
          <button
            on:click={handleCreate}
            disabled={actionLoading}
            class="inline-flex items-center px-4 py-2 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors"
          >
            {actionLoading ? 'Création...' : '📨 Créer la convocation'}
          </button>
          <p class="mt-3 text-xs text-gray-400">
            Les délais légaux seront vérifiés automatiquement (15j pour AG ordinaire, 8j pour AG extraordinaire).
          </p>
        {:else}
          <p class="text-sm text-gray-400">Le syndic n'a pas encore créé de convocation.</p>
        {/if}
      </div>

    {:else}
      <!-- Convocation details -->
      <div class="space-y-4">
        <!-- Info grid -->
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div>
            <p class="text-xs text-gray-500 mb-1">Type</p>
            <p class="text-sm font-medium text-gray-900">{getMeetingTypeLabel(convocation.meeting_type)}</p>
          </div>
          <div>
            <p class="text-xs text-gray-500 mb-1">Date AG</p>
            <p class="text-sm font-medium text-gray-900">{formatDate(convocation.meeting_date)}</p>
          </div>
          <div>
            <p class="text-xs text-gray-500 mb-1">Date limite d'envoi</p>
            <p class="text-sm font-medium text-gray-900">{formatDate(convocation.minimum_send_date)}</p>
          </div>
          <div>
            <p class="text-xs text-gray-500 mb-1">Délai légal</p>
            {#if convocation.respects_legal_deadline}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                ✅ Respecté
              </span>
            {:else}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800">
                ⚠️ Non respecté
              </span>
            {/if}
          </div>
        </div>

        <!-- Recipients summary -->
        <div class="flex items-center gap-4 text-sm text-gray-600">
          <span>📧 {convocation.total_recipients} destinataire{convocation.total_recipients > 1 ? 's' : ''}</span>
          {#if convocation.opened_count > 0}
            <span>👁️ {convocation.opened_count} ouvert{convocation.opened_count > 1 ? 's' : ''}</span>
          {/if}
          {#if convocation.will_attend_count > 0}
            <span>✅ {convocation.will_attend_count} présent{convocation.will_attend_count > 1 ? 's' : ''}</span>
          {/if}
        </div>

        <!-- Tracking Summary (if sent) -->
        {#if convocation.status === ConvocationStatus.Sent}
          <ConvocationTrackingSummary convocationId={convocation.id} />
        {/if}

        <!-- Admin Actions -->
        {#if isAdmin}
          <div class="flex flex-wrap gap-2 pt-2 border-t border-gray-100">
            {#if convocation.status === ConvocationStatus.Draft}
              <button
                on:click={handleSend}
                disabled={actionLoading}
                class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
              >
                📨 Envoyer
              </button>
              <button
                on:click={handleDelete}
                disabled={actionLoading}
                class="px-3 py-1.5 bg-red-100 text-red-700 rounded-lg text-sm font-medium hover:bg-red-200 disabled:opacity-50 transition-colors"
              >
                Supprimer
              </button>
            {:else if convocation.status === ConvocationStatus.Scheduled}
              <button
                on:click={handleSend}
                disabled={actionLoading}
                class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
              >
                📨 Envoyer maintenant
              </button>
              <button
                on:click={handleCancel}
                disabled={actionLoading}
                class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors"
              >
                Annuler
              </button>
            {:else if convocation.status === ConvocationStatus.Sent}
              <button
                on:click={handleSendReminders}
                disabled={actionLoading}
                class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
              >
                🔔 Envoyer rappels (J-3)
              </button>
            {/if}

            {#if convocation.status === ConvocationStatus.Sent || convocation.total_recipients > 0}
              <button
                on:click={() => showRecipients = !showRecipients}
                class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors"
              >
                {showRecipients ? 'Masquer' : 'Voir'} les destinataires ({convocation.total_recipients})
              </button>
            {/if}
          </div>
        {/if}

        <!-- Recipient list (expandable) -->
        {#if showRecipients && convocation}
          <div class="mt-4">
            <ConvocationRecipientList convocationId={convocation.id} />
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
