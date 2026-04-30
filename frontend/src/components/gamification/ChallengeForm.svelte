<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import {
    gamificationApi,
    type Challenge,
    ChallengeType,
  } from '../../lib/api/gamification';
  import { toast } from '../../stores/toast';
  import { withErrorHandling } from "../../lib/utils/error.utils";

  let {
    organizationId,
    challenge = null,
    buildingId = '',
    onsaved,
    oncancel,
  }: {
    organizationId: string;
    challenge?: Challenge | null;
    buildingId?: string;
    onsaved?: (result: any) => void;
    oncancel?: () => void;
  } = $props();

  let saving = $state(false);

  let title = $state('');
  let description = $state('');
  let challengeType = $state<ChallengeType>(ChallengeType.Individual);
  let icon = $state('');
  let startDate = $state('');
  let endDate = $state('');
  let targetMetric = $state('');
  let targetValue = $state(1);
  let rewardPoints = $state(100);

  $effect(() => {
    title = challenge?.title || '';
    description = challenge?.description || '';
    challengeType = challenge?.challenge_type || ChallengeType.Individual;
    icon = challenge?.icon || '';
    startDate = challenge?.start_date ? challenge.start_date.slice(0, 16) : '';
    endDate = challenge?.end_date ? challenge.end_date.slice(0, 16) : '';
    targetMetric = challenge?.target_metric || '';
    targetValue = challenge?.target_value || 1;
    rewardPoints = challenge?.reward_points || 100;
  });

  const typeLabels: Record<ChallengeType, string> = {
    [ChallengeType.Individual]: $_('gamification.type_individual'),
    [ChallengeType.Team]: $_('gamification.type_team'),
    [ChallengeType.Building]: $_('gamification.type_building'),
  };

  const metricSuggestions = [
    'exchanges_completed',
    'bookings_created',
    'notices_posted',
    'skills_shared',
    'objects_shared',
    'polls_voted',
    'meetings_attended',
  ];

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error($_('gamification.title_required'));
      return;
    }
    if (!startDate || !endDate) {
      toast.error($_('gamification.dates_required'));
      return;
    }
    if (new Date(startDate) >= new Date(endDate)) {
      toast.error($_('gamification.end_after_start'));
      return;
    }
    if (!targetMetric.trim()) {
      toast.error($_('gamification.metric_required'));
      return;
    }
    if (targetValue < 1) {
      toast.error($_('gamification.target_min_1'));
      return;
    }

    const data = {
      organization_id: organizationId,
      building_id: buildingId || undefined,
      challenge_type: challengeType,
      title: title.trim(),
      description: description.trim(),
      icon: icon || '🎯',
      start_date: new Date(startDate).toISOString(),
      end_date: new Date(endDate).toISOString(),
      target_metric: targetMetric.trim(),
      target_value: targetValue,
      reward_points: rewardPoints,
    };

    await withErrorHandling({
      action: () => gamificationApi.createChallenge(data),
      setLoading: (v: boolean) => saving = v,
      successMessage: $_('gamification.challenge_created'),
      errorMessage: $_('gamification.creation_error'),
      onSuccess: (result) => onsaved?.(result),
    });
  }

  function handleCancel() {
    oncancel?.();
  }
</script>

<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-4" data-testid="challenge-form">
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="md:col-span-2">
      <label for="ch-title" class="block text-sm font-medium text-gray-700">{$_('gamification.title')} *</label>
      <input id="ch-title" type="text" bind:value={title} required
        data-testid="challenge-title-input"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder={$_('gamification.title_placeholder')} />
    </div>

    <div class="md:col-span-2">
      <label for="ch-desc" class="block text-sm font-medium text-gray-700">{$_('gamification.description')}</label>
      <textarea id="ch-desc" bind:value={description} rows="2"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder={$_('gamification.description_placeholder')}></textarea>
    </div>

    <div>
      <label for="ch-type" class="block text-sm font-medium text-gray-700">{$_('gamification.type')}</label>
      <select id="ch-type" bind:value={challengeType}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm">
        {#each Object.values(ChallengeType) as t}
          <option value={t}>{typeLabels[t]}</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="ch-icon" class="block text-sm font-medium text-gray-700">{$_('gamification.icon')}</label>
      <input id="ch-icon" type="text" bind:value={icon}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="🎯" />
    </div>

    <div>
      <label for="ch-start" class="block text-sm font-medium text-gray-700">{$_('gamification.start_date')} *</label>
      <input id="ch-start" type="datetime-local" bind:value={startDate} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-end" class="block text-sm font-medium text-gray-700">{$_('gamification.end_date')} *</label>
      <input id="ch-end" type="datetime-local" bind:value={endDate} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-metric" class="block text-sm font-medium text-gray-700">{$_('gamification.metric')} *</label>
      <input id="ch-metric" type="text" bind:value={targetMetric} required list="metric-suggestions"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="exchanges_completed" />
      <datalist id="metric-suggestions">
        {#each metricSuggestions as suggestion}
          <option value={suggestion}></option>
        {/each}
      </datalist>
    </div>

    <div>
      <label for="ch-target" class="block text-sm font-medium text-gray-700">{$_('gamification.target_value')} *</label>
      <input id="ch-target" type="number" bind:value={targetValue} min="1" required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-reward" class="block text-sm font-medium text-gray-700">{$_('gamification.reward_points')}</label>
      <input id="ch-reward" type="number" bind:value={rewardPoints} min="0" max="10000"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>
  </div>

  <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
    <button type="button" onclick={handleCancel}
      class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
      {$_('common.cancel')}
    </button>
    <button type="submit" disabled={saving}
      data-testid="challenge-submit-btn"
      class="px-4 py-2 text-sm font-medium text-white bg-amber-600 border border-transparent rounded-md hover:bg-amber-700 disabled:opacity-50">
      {#if saving}
        {$_('common.saving')}
      {:else}
        {$_('gamification.create_challenge')}
      {/if}
    </button>
  </div>
</form>
