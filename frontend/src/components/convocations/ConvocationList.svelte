<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    convocationsApi,
    type Convocation,
    ConvocationStatus,
    MeetingType,
  } from '../../lib/api/convocations';
  import { formatDate } from '../../lib/utils/date.utils';
  import { withLoadingState } from '../../lib/utils/error.utils';
  import { extractArray } from '../../lib/utils/response.utils';

  export let buildingId: string;

  let convocations: Convocation[] = [];
  let filteredConvocations: Convocation[] = [];
  let loading = true;
  let error = '';
  let statusFilter: ConvocationStatus | 'all' = 'all';

  onMount(async () => {
    if (buildingId) await loadConvocations();
  });

  // Recharger quand le buildingId change
  $: if (buildingId) loadConvocations();

  async function loadConvocations() {
    await withLoadingState({
      action: () => convocationsApi.listByBuilding(buildingId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => { convocations = extractArray<Convocation>(data, 'convocations'); applyFilters(); },
      errorMessage: $_('convocations.errors.loadingFailed'),
    });
  }

  function applyFilters() {
    filteredConvocations = convocations.filter(c => {
      if (statusFilter === 'all') return true;
      return c.status === statusFilter;
    });
  }

  $: if (statusFilter) applyFilters();

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
      case MeetingType.Ordinary: return $_('convocations.meetingType.ordinary');
      case MeetingType.Extraordinary: return $_('convocations.meetingType.extraordinary');
      case MeetingType.SecondConvocation: return $_('convocations.meetingType.secondConvocation');
      default: return type;
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg" data-testid="convocation-list">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <h3 class="text-lg leading-6 font-medium text-gray-900">
      📨 {$_('convocations.title')}
    </h3>
    <p class="mt-1 text-sm text-gray-500">
      {$_('convocations.description')}
    </p>
  </div>

  <!-- Filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex items-center space-x-4">
      <label class="text-sm font-medium text-gray-700">{$_('common.status')}:</label>
      <select
        bind:value={statusFilter}
        class="text-sm rounded-md border-gray-300 focus:border-amber-500 focus:ring-amber-500"
      >
        <option value="all">{$_('common.all')}</option>
        <option value={ConvocationStatus.Draft}>{$_('convocations.status.draft')}</option>
        <option value={ConvocationStatus.Scheduled}>{$_('convocations.status.scheduled')}</option>
        <option value={ConvocationStatus.Sent}>{$_('convocations.status.sent')}</option>
        <option value={ConvocationStatus.Cancelled}>{$_('convocations.status.cancelled')}</option>
      </select>
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center" data-testid="convocation-list-loading">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600" data-testid="convocation-list-spinner"></div>
      <p class="mt-2 text-sm text-gray-500">{$_('convocations.loading')}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadConvocations} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">
        {$_('common.retry')}
      </button>
    </div>
  {:else if filteredConvocations.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_('convocations.noFound')}</p>
      <p class="mt-2 text-sm text-gray-400">
        {$_('convocations.noFoundHint')}
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200" data-testid="convocation-rows">
      {#each filteredConvocations as convocation (convocation.id)}
        {@const statusCfg = getStatusConfig(convocation.status)}
        <li class="hover:bg-gray-50" data-testid="convocation-row-{convocation.id}">
          <a href="/convocation-detail?id={convocation.id}" class="block px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-3 mb-2">
                  <h4 class="text-sm font-medium text-amber-700">
                    {getMeetingTypeLabel(convocation.meeting_type)}
                  </h4>
                  <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}" data-testid="convocation-status-{convocation.id}">
                    <span class="mr-1">{statusCfg.icon}</span>
                    {statusCfg.label}
                  </span>
                  {#if !convocation.respects_legal_deadline}
                    <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-red-100 text-red-700">
                      ⚠️ {$_('convocations.legalDeadlineNotRespected')}
                    </span>
                  {/if}
                </div>

                <div class="mt-2 flex items-center text-sm text-gray-500 flex-wrap gap-x-4 gap-y-1">
                  <span>📅 {$_('convocations.meetingOn', { values: { date: formatDate(convocation.meeting_date) } })}</span>
                  <span>📧 {convocation.total_recipients} {$_('common.recipient', { values: { count: convocation.total_recipients } })}</span>
                  {#if convocation.opened_count > 0}
                    <span>👁️ {convocation.opened_count} {$_('common.opened', { values: { count: convocation.opened_count } })}</span>
                  {/if}
                  {#if convocation.will_attend_count > 0}
                    <span>✅ {convocation.will_attend_count} {$_('common.present', { values: { count: convocation.will_attend_count } })}</span>
                  {/if}
                  <span class="text-xs text-gray-400">{$_('common.createdOn')} {formatDate(convocation.created_at)}</span>
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
