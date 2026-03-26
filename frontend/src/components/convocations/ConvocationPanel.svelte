<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    convocationsApi,
    type Convocation,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';
  import { authStore } from '../../stores/auth';
  import { UserRole } from '../../lib/types';
  import { formatDateTime } from '../../lib/utils/date.utils';
  import { withErrorHandling } from '../../lib/utils/error.utils';
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
      if (err.message?.includes('404') || err.message?.includes('not found') || err.message?.includes('Not Found')) {
        convocation = null;
      } else {
        error = err.message || $_('common.loadingError');
      }
    } finally {
      loading = false;
    }
  }

  async function handleCreate() {
    const result = await withErrorHandling({
      action: () => convocationsApi.create({
        meeting_id: meetingId,
        building_id: buildingId,
        meeting_type: MeetingType.Ordinary,
        meeting_date: '',
        language: 'fr',
      }),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.created'),
      errorMessage: $_('convocations.errors.creationFailed'),
    });
    if (result) convocation = result;
  }

  async function handleSend() {
    if (!convocation) return;
    if (!confirm($_('convocations.confirms.sendToAll'))) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.send(convocation!.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.sent'),
      errorMessage: $_('convocations.errors.sendingFailed'),
    });
    if (result) convocation = result;
  }

  async function handleCancel() {
    if (!convocation) return;
    if (!confirm($_('convocations.confirms.cancelConvocation'))) return;
    const result = await withErrorHandling({
      action: () => convocationsApi.cancel(convocation!.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.cancelled'),
      errorMessage: $_('convocations.errors.cancellationFailed'),
    });
    if (result) convocation = result;
  }

  async function handleSendReminders() {
    if (!convocation) return;
    await withErrorHandling({
      action: () => convocationsApi.sendReminders(convocation!.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.remindersEntered'),
      errorMessage: $_('convocations.errors.remindersSendingFailed'),
      onSuccess: () => { loadConvocation(); },
    });
  }

  async function handleDelete() {
    if (!convocation) return;
    if (!confirm($_('convocations.confirms.deleteConvocation'))) return;
    await withErrorHandling({
      action: () => convocationsApi.delete(convocation!.id),
      setLoading: (v) => actionLoading = v,
      successMessage: $_('convocations.messages.deleted'),
      errorMessage: $_('convocations.errors.deletionFailed'),
      onSuccess: () => { convocation = null; },
    });
  }

  function getStatusConfig(status: ConvocationStatus): { bg: string; text: string; label: string; icon: string } {
    const config: Record<ConvocationStatus, { bg: string; text: string; label: string; icon: string }> = {
      [ConvocationStatus.Draft]: { bg: 'bg-gray-100', text: 'text-gray-800', label: $_('convocations.status.draft'), icon: '📝' },
      [ConvocationStatus.Scheduled]: { bg: 'bg-blue-100', text: 'text-blue-800', label: $_('convocations.status.scheduled'), icon: '📅' },
      [ConvocationStatus.Sent]: { bg: 'bg-green-100', text: 'text-green-800', label: $_('convocations.status.sent'), icon: '✅' },
      [ConvocationStatus.Cancelled]: { bg: 'bg-red-100', text: 'text-red-800', label: $_('convocations.status.cancelled'), icon: '❌' },
    };
    return config[status] || config[ConvocationStatus.Draft];
  }

  function getMeetingTypeLabel(type: MeetingType): string {
    switch (type) {
      case MeetingType.Ordinary: return $_('convocations.meetingType.ordinaryWithDays');
      case MeetingType.Extraordinary: return $_('convocations.meetingType.extraordinaryWithDays');
      case MeetingType.SecondConvocation: return $_('convocations.meetingType.secondConvocationWithDays');
      default: return type;
    }
  }
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden" data-testid="convocation-panel">
  <div class="bg-gradient-to-r from-amber-600 to-amber-700 px-6 py-4">
    <div class="flex items-center justify-between">
      <h2 class="text-xl font-semibold text-white">{$_('convocations.title')}</h2>
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
        <p class="mt-2 text-sm text-gray-500">{$_('common.loading')}</p>
      </div>

    {:else if error}
      <div class="p-4 bg-red-50 border border-red-200 rounded-md">
        <p class="text-sm text-red-800">{error}</p>
        <button on:click={loadConvocation} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
          {$_('common.retry')}
        </button>
      </div>

    {:else if !convocation}
      <div class="py-6 text-center">
        <p class="text-gray-500 mb-2">{$_('convocations.noConvocationCreated')}</p>
        {#if isAdmin && meetingStatus === 'Scheduled'}
          <button
            on:click={handleCreate}
            disabled={actionLoading}
            data-testid="convocation-btn-create"
            class="inline-flex items-center px-4 py-2 bg-amber-600 text-white rounded-lg text-sm font-medium hover:bg-amber-700 disabled:opacity-50 transition-colors"
          >
            {actionLoading ? $_('common.creating') : '📨 ' + $_('convocations.actions.create')}
          </button>
          <p class="mt-3 text-xs text-gray-400">
            {$_('convocations.legalDeadlineHint')}
          </p>
        {:else}
          <p class="text-sm text-gray-400">{$_('convocations.syndicNotCreatedYet')}</p>
        {/if}
      </div>

    {:else}
      <div class="space-y-4">
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
          <div data-testid="convocation-field-type">
            <p class="text-xs text-gray-500 mb-1">{$_('common.type')}</p>
            <p class="text-sm font-medium text-gray-900">{getMeetingTypeLabel(convocation.meeting_type)}</p>
          </div>
          <div data-testid="convocation-field-meeting-date">
            <p class="text-xs text-gray-500 mb-1">{$_('convocations.meetingDate')}</p>
            <p class="text-sm font-medium text-gray-900">{formatDateTime(convocation.meeting_date)}</p>
          </div>
          <div data-testid="convocation-field-send-deadline">
            <p class="text-xs text-gray-500 mb-1">{$_('convocations.sendDeadline')}</p>
            <p class="text-sm font-medium text-gray-900">{formatDateTime(convocation.minimum_send_date)}</p>
          </div>
          <div data-testid="convocation-field-legal-deadline">
            <p class="text-xs text-gray-500 mb-1">{$_('convocations.legalDeadline')}</p>
            {#if convocation.respects_legal_deadline}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
                ✅ {$_('common.respected')}
              </span>
            {:else}
              <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-800">
                ⚠️ {$_('common.notRespected')}
              </span>
            {/if}
          </div>
        </div>

        <div class="flex items-center gap-4 text-sm text-gray-600">
          <span>📧 {convocation.total_recipients} {$_('common.recipient', { count: convocation.total_recipients })}</span>
          {#if convocation.opened_count > 0}
            <span>👁️ {convocation.opened_count} {$_('common.opened', { count: convocation.opened_count })}</span>
          {/if}
          {#if convocation.will_attend_count > 0}
            <span>✅ {convocation.will_attend_count} {$_('common.present', { count: convocation.will_attend_count })}</span>
          {/if}
        </div>

        {#if convocation.status === ConvocationStatus.Sent}
          <ConvocationTrackingSummary convocationId={convocation.id} />
        {/if}

        {#if isAdmin}
          <div class="flex flex-wrap gap-2 pt-2 border-t border-gray-100">
            {#if convocation.status === ConvocationStatus.Draft}
              <button
                on:click={handleSend}
                disabled={actionLoading}
                data-testid="convocation-btn-send"
                class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
              >
                📨 {$_('common.send')}
              </button>
              <button
                on:click={handleDelete}
                disabled={actionLoading}
                data-testid="convocation-btn-delete"
                class="px-3 py-1.5 bg-red-100 text-red-700 rounded-lg text-sm font-medium hover:bg-red-200 disabled:opacity-50 transition-colors"
              >
                {$_('common.delete')}
              </button>
            {:else if convocation.status === ConvocationStatus.Scheduled}
              <button
                on:click={handleSend}
                disabled={actionLoading}
                data-testid="convocation-btn-send"
                class="px-3 py-1.5 bg-green-600 text-white rounded-lg text-sm font-medium hover:bg-green-700 disabled:opacity-50 transition-colors"
              >
                📨 {$_('convocations.actions.sendNow')}
              </button>
              <button
                on:click={handleCancel}
                disabled={actionLoading}
                data-testid="convocation-btn-cancel"
                class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 disabled:opacity-50 transition-colors"
              >
                {$_('common.cancel')}
              </button>
            {:else if convocation.status === ConvocationStatus.Sent}
              <button
                on:click={handleSendReminders}
                disabled={actionLoading}
                data-testid="convocation-btn-send-reminders"
                class="px-3 py-1.5 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 transition-colors"
              >
                🔔 {$_('convocations.actions.sendReminders')}
              </button>
            {/if}

            {#if convocation.status === ConvocationStatus.Sent || convocation.total_recipients > 0}
              <button
                on:click={() => showRecipients = !showRecipients}
                data-testid="convocation-btn-toggle-recipients"
                class="px-3 py-1.5 bg-gray-100 text-gray-700 rounded-lg text-sm font-medium hover:bg-gray-200 transition-colors"
              >
                {showRecipients ? $_('common.hide') : $_('common.view')} {$_('convocations.recipients')} ({convocation.total_recipients})
              </button>
            {/if}
          </div>
        {/if}

        {#if showRecipients && convocation}
          <div class="mt-4">
            <ConvocationRecipientList convocationId={convocation.id} />
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
