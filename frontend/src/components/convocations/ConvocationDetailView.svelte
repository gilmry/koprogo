<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    convocationsApi,
    type Convocation,
    type TrackingSummary,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import { formatDateTime, formatDateShort } from '../../lib/utils/date.utils';
  import { withErrorHandling } from '../../lib/utils/error.utils';
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

  function getMeetingTypeLabel(type: MeetingType): string {
    switch (type) {
      case MeetingType.Ordinary: return $_('convocations.meetingType.ordinary');
      case MeetingType.Extraordinary: return $_('convocations.meetingType.extraordinary');
      case MeetingType.SecondConvocation: return $_('convocations.meetingType.secondConvocation');
      default: return type;
    }
  }

  function getStatusConfig(status: ConvocationStatus): { bg: string; text: string; label: string } {
    switch (status) {
      case ConvocationStatus.Draft: return { bg: 'bg-gray-100', text: 'text-gray-700', label: $_('convocations.status.draft') };
      case ConvocationStatus.Scheduled: return { bg: 'bg-blue-100', text: 'text-blue-700', label: $_('convocations.status.scheduled') };
      case ConvocationStatus.Sent: return { bg: 'bg-green-100', text: 'text-green-700', label: $_('convocations.status.sent') };
      case ConvocationStatus.Cancelled: return { bg: 'bg-red-100', text: 'text-red-700', label: $_('convocations.status.cancelled') };
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
    const sendDate = prompt($_('convocations.prompts.scheduledSendDate'));
    if (!sendDate) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.schedule(convocation.id, sendDate),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.scheduled'),
      errorMessage: $_('convocations.errors.schedulingFailed'),
    });
    if (result) convocation = result;
  }

  async function handleSend() {
    if (!confirm($_('convocations.confirms.sendToAll'))) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.send(convocation.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.sent'),
      errorMessage: $_('convocations.errors.sendingFailed'),
    });
    if (result) convocation = result;
  }

  async function handleCancel() {
    if (!confirm($_('convocations.confirms.cancelConvocation'))) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.cancel(convocation.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.cancelled'),
      errorMessage: $_('convocations.errors.cancellationFailed'),
    });
    if (result) convocation = result;
  }

  async function handleSendReminders() {
    if (!confirm($_('convocations.confirms.sendReminders'))) return;
    await withErrorHandling({
      action: () => convocationsApi.sendReminders(convocation.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.remindersEntered'),
      errorMessage: $_('convocations.errors.remindersSendingFailed'),
    });
  }

  async function handleDelete() {
    if (!confirm($_('convocations.confirms.deleteConvocation'))) return;
    await withErrorHandling({
      action: () => convocationsApi.delete(convocation.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.deleted'),
      errorMessage: $_('convocations.errors.deletionFailed'),
      onSuccess: () => { window.location.href = '/convocations'; },
    });
  }
</script>

<div class="space-y-6" data-testid="convocation-detail">
  <div class="bg-white shadow-md rounded-lg p-6">
    <div class="flex items-start justify-between">
      <div>
        <div class="flex items-center gap-3 mb-2">
          <h2 class="text-2xl font-bold text-gray-900">{$_('convocations.title')}</h2>
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusConfig(convocation.status).bg} {getStatusConfig(convocation.status).text}" data-testid="convocation-detail-status">
            {getStatusConfig(convocation.status).label}
          </span>
          <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-indigo-100 text-indigo-700">
            {getMeetingTypeLabel(convocation.meeting_type)}
          </span>
        </div>
        <p class="text-sm text-gray-500">
          {$_('common.language')}: {convocation.language.toUpperCase()} - {$_('common.createdOn')} {formatDateTime(convocation.created_at)}
        </p>
      </div>

      <div class="text-right" data-testid="convocation-detail-legal-deadline">
        {#if convocation.respects_legal_deadline}
          <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-green-100 text-green-700">
            {$_('convocations.legalDeadlineRespected', { values: { days: getLegalDeadlineDays(convocation.meeting_type) } })}
          </span>
        {:else}
          <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium bg-red-100 text-red-700">
            {$_('convocations.legalDeadlineNotRespected')}
          </span>
        {/if}
      </div>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mt-6">
      <div class="p-3 bg-blue-50 rounded-lg">
        <div class="text-xs text-blue-600 font-medium">{$_('convocations.meetingDate')}</div>
        <div class="text-sm text-blue-900 font-medium">{formatDateShort(convocation.meeting_date)}</div>
      </div>
      <div class="p-3 bg-amber-50 rounded-lg">
        <div class="text-xs text-amber-600 font-medium">{$_('convocations.minimumSendDate')}</div>
        <div class="text-sm text-amber-900 font-medium">{formatDateShort(convocation.minimum_send_date)}</div>
      </div>
      <div class="p-3 bg-green-50 rounded-lg" data-testid="convocation-detail-recipients-summary">
        <div class="text-xs text-green-600 font-medium">{$_('convocations.recipients')}</div>
        <div class="text-sm text-green-900 font-bold">{convocation.total_recipients}</div>
      </div>
      <div class="p-3 bg-purple-50 rounded-lg">
        <div class="text-xs text-purple-600 font-medium">{$_('convocations.openings')}</div>
        <div class="text-sm text-purple-900 font-bold">{convocation.opened_count}/{convocation.total_recipients}</div>
      </div>
    </div>
  </div>

  {#if convocation.status === ConvocationStatus.Sent && tracking}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">{$_('convocations.trackingTitle')}</h3>
      <ConvocationTrackingSummary summary={tracking} />
    </div>
  {/if}

  {#if convocation.status === ConvocationStatus.Sent}
    <div class="bg-white shadow-md rounded-lg">
      <div class="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
        <h3 class="text-lg font-medium text-gray-900">{$_('convocations.recipients')}</h3>
        <button
          on:click={() => showRecipients = !showRecipients}
          data-testid="convocation-detail-btn-toggle-recipients"
          class="text-sm text-blue-600 hover:text-blue-800"
        >
          {showRecipients ? $_('common.hide') : $_('common.show')} {$_('common.list')}
        </button>
      </div>
      {#if showRecipients}
        <div class="p-6">
          <ConvocationRecipientList convocationId={convocation.id} />
        </div>
      {/if}
    </div>
  {/if}

  {#if isAdmin}
    <div class="bg-white shadow-md rounded-lg p-6">
      <h3 class="text-lg font-medium text-gray-900 mb-4">{$_('common.actions')}</h3>
      <div class="flex flex-wrap gap-3">
        {#if convocation.status === ConvocationStatus.Draft}
          <button
            on:click={handleSchedule}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-schedule"
            class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700 disabled:opacity-50"
          >
            {$_('convocations.actions.scheduleSend')}
          </button>
          <button
            on:click={handleSend}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send"
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {$_('convocations.actions.sendNow')}
          </button>
          <button
            on:click={handleDelete}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-delete"
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            {$_('common.delete')}
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Scheduled}
          <button
            on:click={handleSend}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send"
            class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700 disabled:opacity-50"
          >
            {$_('convocations.actions.sendNow')}
          </button>
          <button
            on:click={handleCancel}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-cancel"
            class="px-4 py-2 bg-red-100 text-red-700 text-sm font-medium rounded-md hover:bg-red-200 disabled:opacity-50"
          >
            {$_('common.cancel')}
          </button>
        {/if}

        {#if convocation.status === ConvocationStatus.Sent}
          <button
            on:click={handleSendReminders}
            disabled={actionLoading}
            data-testid="convocation-detail-btn-send-reminders"
            class="px-4 py-2 bg-amber-600 text-white text-sm font-medium rounded-md hover:bg-amber-700 disabled:opacity-50"
          >
            {$_('convocations.actions.sendReminders')}
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>
