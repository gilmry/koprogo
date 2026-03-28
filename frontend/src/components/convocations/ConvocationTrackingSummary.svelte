<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { convocationsApi, type TrackingSummary } from "../../lib/api/convocations";
  import { withLoadingState } from '../../lib/utils/error.utils';

  export let convocationId: string;

  let summary: TrackingSummary | null = null;
  let loading = true;
  let error = '';

  onMount(async () => {
    await loadSummary();
  });

  async function loadSummary() {
    await withLoadingState({
      action: () => convocationsApi.getTrackingSummary(convocationId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => { summary = data; },
      errorMessage: $_('convocations.errors.trackingSummaryFailed'),
    });
  }

  function formatPercentage(rate: number): string {
    return `${(rate * 100).toFixed(1)}%`;
  }
</script>

<div class="bg-white shadow rounded-lg p-6" data-testid="tracking-summary">
  <h3 class="text-lg font-semibold text-gray-900 mb-4">{$_('convocations.trackingSummary')}</h3>

  {#if loading}
    <div class="text-center py-8 text-gray-500" data-testid="tracking-summary-loading">
      <div class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-amber-600" data-testid="tracking-summary-spinner"></div>
      <p class="mt-2">{$_('common.loading')}</p>
    </div>
  {:else if summary}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <div class="bg-gray-50 rounded-lg p-4" data-testid="tracking-stat-total">
        <dt class="text-sm font-medium text-gray-500 mb-1">{$_('convocations.totalRecipients')}</dt>
        <dd class="text-2xl font-bold text-gray-900">{summary.total_recipients}</dd>
      </div>

      <div class="bg-blue-50 rounded-lg p-4" data-testid="tracking-stat-opened">
        <dt class="text-sm font-medium text-blue-700 mb-1">📧 {$_('convocations.opened')}</dt>
        <dd class="text-2xl font-bold text-blue-900">{summary.email_opened}</dd>
        <dd class="text-xs text-blue-600 mt-1">
          {formatPercentage(summary.opening_rate)} {$_('convocations.openingRate')}
        </dd>
      </div>

      <div class="bg-green-50 rounded-lg p-4" data-testid="tracking-stat-attend">
        <dt class="text-sm font-medium text-green-700 mb-1">✅ {$_('convocations.attendance.willAttend')}</dt>
        <dd class="text-2xl font-bold text-green-900">{summary.will_attend}</dd>
        <dd class="text-xs text-green-600 mt-1">
          {formatPercentage(summary.attendance_rate)} {$_('common.confirmed')}
        </dd>
      </div>

      {#if summary.email_failed > 0}
        <div class="bg-red-50 rounded-lg p-4" data-testid="tracking-stat-failed">
          <dt class="text-sm font-medium text-red-700 mb-1">❌ {$_('common.failed')}</dt>
          <dd class="text-2xl font-bold text-red-900">{summary.email_failed}</dd>
        </div>
      {/if}
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">{$_('convocations.noTrackingData')}</div>
  {/if}
</div>
