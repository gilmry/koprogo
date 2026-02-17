<script lang="ts">
  import { onMount } from 'svelte';
  import {
    convocationsApi,
    type Convocation,
    type TrackingSummary,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';
  import { toast } from '../../stores/toast';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import ConvocationTrackingSummary from './ConvocationTrackingSummary.svelte';
  import ConvocationRecipientList from './ConvocationRecipientList.svelte';

  export let convocation: Convocation;

  let tracking: TrackingSummary | null = null;
  let showRecipients = false;
  let actionLoading = false;

  $: isAdmin = $authStore.user?.role === UserRole.SYNDIC || $authStore.user?.role === UserRole.SUPERADMIN;

  onMount(async () => {
    if (convocation.status === ConvocationStatus.Sent) {
      try {
        tracking = await convocationsApi.getTrackingSummary(convocation.id);
      } catch {
        // Non-critical
      }
    }
  });

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'long',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function formatDateShort(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'short',
      year: 'numeric',
    });
  }

  function getMeetingTypeLabel(type: MeetingType): string {
    switch (type) {
      case MeetingType.Ordinary: return 'AG Ordinaire';
      case MeetingType.Extraordinary: return 'AG Extraordinaire';
      case MeetingType.SecondConvocation: return '2e Convocation';
      default: return type;
    }
  }

  function getStatusConfig(status: ConvocationStatus): { bg: string; text: string; label: string } {
    switch (status) {
      case ConvocationStatus.Draft: return { bg: 'bg-gray-100', text: 'text-gray-700', label: 'Brouillon' };
      case ConvocationStatus.Scheduled: return { bg: 'bg-blue-100', text: 'text-blue-700', label: 'Planifiée' };
      case ConvocationStatus.Sent: return { bg: 'bg-green-100', text: 'text-green-700', label: 'Envoyée' };
      case ConvocationStatus.Cancelled: return { bg: 'bg-red-100', text: 'text-red-700', label: 'Annulée' };
      default: return { bg: 'bg-gray-100', text: 'text-gray-700', label: status };
    }
  }

  function getLegalDeadlineDays(type: MeetingType): number {
    switch (type) {
      case MeetingType.Ordinary: return 15;
      case MeetingType.Extraordinary: return 8;
      case MeetingType.SecondConvocation: return 8;
      default: return 15;
    }
  }

  async function handleSchedule() {
    const sendDate = prompt('Date d\'envoi planifiée (YYYY-MM-DD):');
    if (!sendDate) return;
    actionLoading = true;
    try {
      convocation = await convocationsApi.schedule(convocation.id, sendDate);
      toast.success('Convocation planifiée !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la planification');
    } finally {
      actionLoading = false;
    }
  }

  async function handleSend() {
    if (!confirm('Envoyer cette convocation à tous les copropriétaires ? Un PDF sera généré et les emails envoyés.')) return;
    actionLoading = true;
    try {
      convocation = await convocationsApi.send(convocation.id);
      toast.success('Convocation envoyée !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'envoi');
    } finally {
      actionLoading = false;
    }
  }

  async function handleCancel() {
    if (!confirm('Annuler cette convocation ?')) return;
    actionLoading = true;
    try {
      convocation = await convocationsApi.cancel(convocation.id);
      toast.success('Convocation annulée');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'annulation');
    } finally {
      actionLoading = false;
    }
  }

  async function handleSendReminders() {
    if (!confirm('Envoyer des rappels aux copropriétaires qui n\'ont pas ouvert l\'email ?')) return;
    actionLoading = true;
    try {
      await convocationsApi.sendReminders(convocation.id);
      toast.success('Rappels envoyés !');
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'envoi des rappels');
    } finally {
      actionLoading = false;
    }
  }

  async function handleDelete() {
    if (!confirm('Supprimer cette convocation ? Cette action est irréversible.')) return;
    actionLoading = true;
    try {
      await convocationsApi.delete(convocation.id);
      toast.success('Convocation supprimée');
      window.location.href = '/convocations';
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    } finally {
      actionLoading = false;
    }
  }
</script>

<div class="space-y-6">
  <!-- Header Card -->
  <div class="bg-white shadow-md rounded-lg p-6">
    <div class="flex items-start justify-between">
      <div>
        <div class="flex items-center gap-3 mb-2">
          <h2 class="text-2xl font-bold text-gray-900">Convocation</h2>
          {@const statusCfg = getStatusConfig(convocation.status)}
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}">
            {statusCfg.label}
          </span>
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-700">
            {getMeetingTypeLabel(convocation.meeting_type)}
          </span>
        </div>
        <p class="text-sm text-gray-500">
          Langue: {convocation.language.toUpperCase()} - Créée le {formatDate(convocation.created_at)}
        </p>
      </div>

      <!-- Legal Deadline Badge -->
      <div class="text-right">
        {#if convocation.respects_legal_deadline}
          <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-700">
            Délai légal respecté ({getLegalDeadlineDays(convocation.meeting_type)}j)
          </span>
        {:else}
          <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-700">
            Délai légal non respecté
          </span>
        {/if}
      </div>
    </div>

    <!-- Key Dates -->
    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
      <div class="p-3 bg-blue-50 rounded-lg">
        <div class="text-xs text-blue-600 font-medium">Date AG</div>
        <div class="text-sm text-blue-900 font-medium">{formatDateShort(convocation.meeting_date)}</div>
      </div>
      <div class="p-3 bg-amber-50 rounded-lg">
        <div class="text-xs text-amber-600 font-medium">Envoi minimum</div>
        <div class="text-sm text-amber-900 font-medium">{formatDateShort(convocation.minimum_send_date)}</div>
      </div>
      <div class="p-3 bg-green-50 rounded-lg">
        <div class="text-xs text-green-600 font-medium">Destinataires</div>
        <div class="text-sm text-green-900 font-bold">{convocation.total_recipients}</div>
      </div>
      <div class="p-3 bg-purple-50 rounded-lg">
        <div class="text-xs text-purple-600 font-medium">Ouvertures</div>
        <div class="text-sm text-purple-900 font-bold">{convocation.opened_count}/{convocation.total_recipients}</div>
      </div>
    </div>
  </div>

  <!-- Tracking Summary (if sent) -->
  {#if convocation.status === ConvocationStatus.Sent && tracking}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">Suivi des envois</h3>
      <ConvocationTrackingSummary summary={tracking} />
    </div>
  {/if}

  <!-- Recipients -->
  {#if convocation.status === ConvocationStatus.Sent}
    <div class="bg-white shadow-md rounded-lg">
      <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
        <h3 class="text-lg font-medium text-gray-900">Destinataires</h3>
        <button
          on:click={() => showRecipients = !showRecipients}
          class="text-sm text-blue-600 hover:text-blue-800"
        >
          {showRecipients ? 'Masquer' : 'Afficher'} la liste
        </button>
      </div>
      {#if showRecipients}
        <div class="p-6">
          <ConvocationRecipientList convocationId={convocation.id} />
        </div>
      {/if}
    </div>
  {/if}

  <!-- Actions (admin only) -->
  {#if isAdmin}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">Actions</h3>
      <div class="flex flex-wrap gap-3">
        {#if convocation.status === ConvocationStatus.Draft}
          <button
            on:click={handleSchedule}
            disabled={actionLoading}
            class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            Planifier l'envoi
          </button>
          <button
            on:click={handleSend}
            disabled={actionLoading}
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            Envoyer maintenant
          </button>
          <button
            on:click={handleDelete}
            disabled={actionLoading}
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            Supprimer
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Scheduled}
          <button
            on:click={handleSend}
            disabled={actionLoading}
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            Envoyer maintenant
          </button>
          <button
            on:click={handleCancel}
            disabled={actionLoading}
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            Annuler
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Sent}
          <button
            on:click={handleSendReminders}
            disabled={actionLoading}
            class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-md hover:bg-amber-700 disabled:opacity-50"
          >
            Envoyer les rappels (J-3)
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
