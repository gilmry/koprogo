<script lang="ts">
  // Svelte 5 runes mode — migrated from legacy (STORY-P7-603)
  import { _ } from '../../lib/i18n';
  import { api } from '../../lib/api';
  import { toast } from '../../stores/toast';
  import { withErrorHandling } from '../../lib/utils/error.utils';
  import type { Meeting } from '../../lib/types';

  let {
    meeting = $bindable(),
    canManage = false,
  }: {
    meeting: Meeting;
    canManage?: boolean;
  } = $props();

  let presentQuotas = $state<number>((meeting as any).present_quotas ?? 0);
  let totalQuotas = $state<number>((meeting as any).total_quotas ?? 1000);
  let submitting = $state(false);

  let quorumValidated = $derived((meeting as any).quorum_validated === true);
  let quorumPercentage = $derived(totalQuotas > 0 ? (presentQuotas / totalQuotas) * 100 : 0);
  let displayPercentage = $derived((meeting as any).quorum_percentage ?? quorumPercentage);

  async function handleValidateQuorum() {
    if (presentQuotas <= 0 || totalQuotas <= 0) {
      toast.error($_('meetings.quorum.invalidValues'));
      return;
    }
    const result = await withErrorHandling({
      action: () => api.post<{ quorum_reached: boolean; meeting: Meeting; message: string }>(
        `/meetings/${meeting.id}/validate-quorum`,
        { present_quotas: presentQuotas, total_quotas: totalQuotas },
      ),
      setLoading: (v: boolean) => submitting = v,
      successMessage: $_('meetings.quorum.validated_success'),
      errorMessage: $_('meetings.quorum.validation_failed'),
    });
    if (result) {
      meeting = result.meeting;
      if (!result.quorum_reached) {
        toast.warning($_('meetings.quorum.second_convocation_scheduled'));
      }
    }
  }
</script>

<div class="bg-white rounded-lg shadow-lg overflow-hidden mb-8" data-testid="quorum-panel">
  <div class="bg-gradient-to-r from-indigo-600 to-indigo-700 px-6 py-4">
    <h2 class="text-xl font-semibold text-white">
      🗳️ {$_('meetings.quorum.title')}
    </h2>
  </div>

  <div class="p-6">
    {#if quorumValidated}
      <div class="flex items-center gap-3 p-4 bg-green-50 border border-green-200 rounded-lg" data-testid="quorum-validated-badge">
        <span class="text-2xl">✅</span>
        <div>
          <p class="text-green-900 font-semibold">
            {$_('meetings.quorum.validated')}
          </p>
          <p class="text-green-700 text-sm">
            {presentQuotas} / {totalQuotas} {$_('meetings.quorum.thousandths')}
            ({displayPercentage.toFixed(1)}%)
          </p>
        </div>
      </div>
    {:else if canManage}
      <p class="text-sm text-gray-600 mb-4">
        {$_('meetings.quorum.legal_notice')}
      </p>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
        <div>
          <label for="present-quotas" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('meetings.quorum.presentQuotas')} *
          </label>
          <input
            id="present-quotas"
            type="number"
            bind:value={presentQuotas}
            min="0"
            max={totalQuotas}
            step="0.01"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500"
            data-testid="quorum-present-input"
          />
        </div>
        <div>
          <label for="total-quotas" class="block text-sm font-medium text-gray-700 mb-1">
            {$_('meetings.quorum.totalQuotas')} *
          </label>
          <input
            id="total-quotas"
            type="number"
            bind:value={totalQuotas}
            min="1"
            step="0.01"
            class="w-full px-3 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-indigo-500"
            data-testid="quorum-total-input"
          />
        </div>
      </div>
      <div class="flex items-center justify-between">
        <div class="text-sm text-gray-600">
          {$_('meetings.quorum.preview')}: <strong>{quorumPercentage.toFixed(1)}%</strong>
          {#if quorumPercentage > 50}
            <span class="text-green-700">✓ {$_('meetings.quorum.quorum_ok')}</span>
          {:else}
            <span class="text-orange-700">⚠ {$_('meetings.quorum.quorum_not_reached')}</span>
          {/if}
        </div>
        <button
          type="button"
          onclick={handleValidateQuorum}
          disabled={submitting}
          class="px-4 py-2 bg-indigo-600 text-white rounded-md hover:bg-indigo-700 disabled:opacity-50"
          data-testid="quorum-validate-btn"
        >
          {submitting ? $_('meetings.quorum.validating') : $_('meetings.quorum.validate')}
        </button>
      </div>
    {:else}
      <p class="text-sm text-gray-500 italic">
        {$_('meetings.quorum.readonly')}
      </p>
    {/if}
  </div>
</div>
