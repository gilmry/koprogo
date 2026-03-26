<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    convocationsApi,
    type ConvocationRecipient,
    AttendanceStatus,
  } from '../../lib/api/convocations';
  import { formatDateTime } from '../../lib/utils/date.utils';
  import { withErrorHandling, withLoadingState } from '../../lib/utils/error.utils';

  export let convocationId: string;

  let recipients: ConvocationRecipient[] = [];
  let filteredRecipients: ConvocationRecipient[] = [];
  let loading = true;
  let error = '';
  let filter: 'all' | 'confirmed' | 'pending' | 'absent' = 'all';

  onMount(async () => {
    await loadRecipients();
  });

  async function loadRecipients() {
    await withLoadingState({
      action: () => convocationsApi.getRecipients(convocationId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => { recipients = data; applyFilter(); },
      errorMessage: $_('convocations.errors.loadingRecipientsFailed'),
    });
  }

  function applyFilter() {
    switch (filter) {
      case 'confirmed':
        filteredRecipients = recipients.filter(r =>
          r.attendance_status === AttendanceStatus.WillAttend ||
          r.attendance_status === AttendanceStatus.Attended
        );
        break;
      case 'pending':
        filteredRecipients = recipients.filter(r =>
          r.attendance_status === AttendanceStatus.Pending
        );
        break;
      case 'absent':
        filteredRecipients = recipients.filter(r =>
          r.attendance_status === AttendanceStatus.WillNotAttend ||
          r.attendance_status === AttendanceStatus.DidNotAttend
        );
        break;
      default:
        filteredRecipients = recipients;
    }
  }

  $: if (filter) applyFilter();

  async function updateAttendance(recipientId: string, status: AttendanceStatus) {
    await withErrorHandling({
      action: () => convocationsApi.updateAttendance(recipientId, status),
      successMessage: $_('convocations.messages.attendanceUpdated'),
      errorMessage: $_('common.updateFailed'),
      onSuccess: () => { loadRecipients(); },
    });
  }

  function getAttendanceConfig(status: AttendanceStatus): { bg: string; text: string; label: string } {
    const config: Record<AttendanceStatus, { bg: string; text: string; label: string }> = {
      [AttendanceStatus.Pending]: { bg: 'bg-gray-100', text: 'text-gray-700', label: $_('common.pending') },
      [AttendanceStatus.WillAttend]: { bg: 'bg-green-100', text: 'text-green-700', label: $_('convocations.attendance.willAttend') },
      [AttendanceStatus.WillNotAttend]: { bg: 'bg-red-100', text: 'text-red-700', label: $_('convocations.attendance.willNotAttend') },
      [AttendanceStatus.Attended]: { bg: 'bg-green-200', text: 'text-green-800', label: $_('convocations.attendance.attended') },
      [AttendanceStatus.DidNotAttend]: { bg: 'bg-red-200', text: 'text-red-800', label: $_('convocations.attendance.didNotAttend') },
    };
    return config[status] || config[AttendanceStatus.Pending];
  }
</script>

<div class="bg-gray-50 border border-gray-200 rounded-lg" data-testid="recipient-list">
  <div class="px-4 py-3 border-b border-gray-200">
    <div class="flex items-center justify-between">
      <h4 class="text-sm font-semibold text-gray-900">
        {$_('convocations.recipients')} ({recipients.length})
      </h4>
      <div class="flex gap-1">
        {#each [
          { value: 'all', label: $_('common.all') },
          { value: 'confirmed', label: $_('convocations.filters.present') },
          { value: 'pending', label: $_('common.pending') },
          { value: 'absent', label: $_('convocations.filters.absent') },
        ] as f}
          <button
            on:click={() => filter = f.value}
            class="px-2 py-1 rounded text-xs font-medium transition-colors
              {filter === f.value
                ? 'bg-amber-600 text-white'
                : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}"
          >
            {f.label}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if loading}
    <div class="p-6 text-center">
      <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-amber-600"></div>
    </div>
  {:else if error}
    <div class="p-4">
      <p class="text-sm text-red-600">{error}</p>
    </div>
  {:else if filteredRecipients.length === 0}
    <div class="p-6 text-center text-sm text-gray-500">
      {$_('convocations.noRecipientsInCategory')}
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-left text-xs text-gray-500 uppercase border-b border-gray-200">
            <th scope="col" class="px-4 py-2">{$_('common.owner')}</th>
            <th scope="col" class="px-4 py-2">{$_('common.email')}</th>
            <th scope="col" class="px-4 py-2">{$_('convocations.sent')}</th>
            <th scope="col" class="px-4 py-2">{$_('convocations.opened')}</th>
            <th scope="col" class="px-4 py-2">{$_('convocations.attendance.title')}</th>
            <th scope="col" class="px-4 py-2">{$_('convocations.proxy')}</th>
            <th scope="col" class="px-4 py-2">{$_('common.actions')}</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-100">
          {#each filteredRecipients as recipient (recipient.id)}
            {@const attendCfg = getAttendanceConfig(recipient.attendance_status)}
            <tr class="hover:bg-white" data-testid="recipient-row-{recipient.id}">
              <td class="px-4 py-2">
                <span class="font-medium text-gray-900">{recipient.owner_name || recipient.owner_id.slice(0, 8)}</span>
              </td>
              <td class="px-4 py-2 text-gray-600">{recipient.owner_email}</td>
              <td class="px-4 py-2">
                {#if recipient.email_failed}
                  <span class="text-red-600 text-xs">❌ {$_('common.failed')}</span>
                {:else if recipient.email_sent_at}
                  <span class="text-green-600 text-xs">✅ {formatDateTime(recipient.email_sent_at)}</span>
                {:else}
                  <span class="text-gray-400 text-xs">-</span>
                {/if}
              </td>
              <td class="px-4 py-2">
                {#if recipient.email_opened_at}
                  <span class="text-blue-600 text-xs">👁️ {formatDateTime(recipient.email_opened_at)}</span>
                {:else}
                  <span class="text-gray-400 text-xs">-</span>
                {/if}
              </td>
              <td class="px-4 py-2">
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {attendCfg.bg} {attendCfg.text}" data-testid="recipient-status-{recipient.id}">
                  {attendCfg.label}
                </span>
              </td>
              <td class="px-4 py-2">
                {#if recipient.proxy_owner_id}
                  <span class="text-xs text-purple-600">
                    🤝 {recipient.proxy_owner_name || recipient.proxy_owner_id.slice(0, 8)}
                  </span>
                {:else}
                  <span class="text-gray-400 text-xs">-</span>
                {/if}
              </td>
              <td class="px-4 py-2">
                <div class="flex gap-1">
                  {#if recipient.attendance_status === AttendanceStatus.Pending}
                    <button
                      on:click={() => updateAttendance(recipient.id, AttendanceStatus.WillAttend)}
                      class="text-xs text-green-600 hover:text-green-800 underline"
                      title={$_('convocations.markAsPresent')}
                      data-testid="recipient-btn-attend-{recipient.id}"
                    >
                      ✅
                    </button>
                    <button
                      on:click={() => updateAttendance(recipient.id, AttendanceStatus.WillNotAttend)}
                      class="text-xs text-red-600 hover:text-red-800 underline"
                      title={$_('convocations.markAsAbsent')}
                      data-testid="recipient-btn-absent-{recipient.id}"
                    >
                      ❌
                    </button>
                  {/if}
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
