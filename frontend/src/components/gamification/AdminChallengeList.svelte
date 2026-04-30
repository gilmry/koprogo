<script lang="ts">
  // Svelte 5 runes mode
  import { _ } from '../../lib/i18n';
  import {
    gamificationApi,
    type Challenge,
    ChallengeStatus,
    ChallengeType,
  } from '../../lib/api/gamification';
  import ChallengeForm from './ChallengeForm.svelte';
  import { formatDateShort } from "../../lib/utils/date.utils";
  import { withLoadingState, withErrorHandling } from "../../lib/utils/error.utils";

  let { organizationId, buildingId = '' }: {
    organizationId: string;
    buildingId?: string;
  } = $props();

  let challenges = $state<Challenge[]>([]);
  let loading = $state(true);
  let error = $state('');
  let showForm = $state(false);
  let statusFilter = $state<ChallengeStatus | 'all'>('all');

  let filteredChallenges = $derived(challenges.filter(c => {
    if (statusFilter === 'all') return true;
    return c.status === statusFilter;
  }));

  $effect(() => {
    loadData();
  });

  async function loadData() {
    if (!organizationId) {
      loading = false;
      return;
    }
    await withLoadingState({
      action: () => gamificationApi.listChallenges(organizationId),
      setLoading: (v: boolean) => loading = v,
      setError: (v: string) => error = v,
      onSuccess: (data) => challenges = data,
      errorMessage: $_('common.load_error'),
    });
  }

  async function handleActivate(challenge: Challenge) {
    if (!confirm($_('gamification.confirm_activate', { values: { title: challenge.title } }))) return;
    await withErrorHandling({
      action: () => gamificationApi.activateChallenge(challenge.id),
      successMessage: $_('gamification.activate_success'),
      errorMessage: $_('gamification.activate_error'),
      onSuccess: () => loadData(),
    });
  }

  async function handleComplete(challenge: Challenge) {
    if (!confirm($_('gamification.confirm_complete', { values: { title: challenge.title } }))) return;
    await withErrorHandling({
      action: () => gamificationApi.completeChallenge(challenge.id),
      successMessage: $_('gamification.complete_success'),
      errorMessage: $_('common.error'),
      onSuccess: () => loadData(),
    });
  }

  async function handleCancelChallenge(challenge: Challenge) {
    if (!confirm($_('gamification.confirm_cancel', { values: { title: challenge.title } }))) return;
    await withErrorHandling({
      action: () => gamificationApi.cancelChallenge(challenge.id),
      successMessage: $_('gamification.cancel_success'),
      errorMessage: $_('common.error'),
      onSuccess: () => loadData(),
    });
  }

  async function handleDelete(challenge: Challenge) {
    if (!confirm($_('gamification.confirm_delete_challenge', { values: { title: challenge.title } }))) return;
    await withErrorHandling({
      action: () => gamificationApi.deleteChallenge(challenge.id),
      successMessage: $_('gamification.delete_success'),
      errorMessage: $_('gamification.delete_error'),
      onSuccess: () => loadData(),
    });
  }

  function handleSaved() {
    showForm = false;
    loadData();
  }

  function getStatusConfig(status: ChallengeStatus): { bg: string; text: string; label: string } {
    switch (status) {
      case ChallengeStatus.Draft: return { bg: 'bg-gray-100', text: 'text-gray-700', label: $_('gamification.status_draft') };
      case ChallengeStatus.Active: return { bg: 'bg-green-100', text: 'text-green-700', label: $_('gamification.status_active') };
      case ChallengeStatus.Completed: return { bg: 'bg-blue-100', text: 'text-blue-700', label: $_('gamification.status_completed') };
      case ChallengeStatus.Cancelled: return { bg: 'bg-red-100', text: 'text-red-700', label: $_('gamification.status_cancelled') };
      default: return { bg: 'bg-gray-100', text: 'text-gray-700', label: status };
    }
  }

  const typeLabels: Record<ChallengeType, string> = {
    [ChallengeType.Individual]: $_('gamification.type_individual'),
    [ChallengeType.Team]: $_('gamification.type_team'),
    [ChallengeType.Building]: $_('gamification.type_building'),
  };

</script>

<div class="bg-white shadow-md rounded-lg" data-testid="admin-challenge-list">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">{$_('gamification.challenges_management')}</h3>
        <p class="mt-1 text-sm text-gray-500">{$_('gamification.challenge_count', { values: { count: challenges.length } })}</p>
      </div>
      <button onclick={() => showForm = !showForm}
        data-testid="challenge-create-btn"
        class="px-4 py-2 text-sm font-medium text-white bg-amber-600 rounded-md hover:bg-amber-700">
        {showForm ? $_('common.close') : '+ ' + $_('common.new')}
      </button>
    </div>
  </div>

  {#if showForm}
    <div class="p-4 bg-amber-50 border-b border-amber-200">
      <h4 class="text-sm font-medium text-amber-800 mb-3">{$_('gamification.create_challenge')}</h4>
      <ChallengeForm
        {organizationId}
        {buildingId}
        onsaved={handleSaved}
        oncancel={() => showForm = false}
      />
    </div>
  {/if}

  <!-- Status filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex flex-wrap gap-1">
      {#each [
        { value: 'all' as ChallengeStatus | 'all', label: $_('common.all') },
        { value: ChallengeStatus.Draft as ChallengeStatus | 'all', label: $_('gamification.status_drafts') },
        { value: ChallengeStatus.Active as ChallengeStatus | 'all', label: $_('gamification.status_actives') },
        { value: ChallengeStatus.Completed as ChallengeStatus | 'all', label: $_('gamification.status_completed_plural') },
        { value: ChallengeStatus.Cancelled as ChallengeStatus | 'all', label: $_('gamification.status_cancelled_plural') },
      ] as f}
        {@const count = f.value === 'all' ? challenges.length : challenges.filter(c => c.status === f.value).length}
        <button onclick={() => statusFilter = f.value}
          class="px-2 py-1 rounded text-xs font-medium transition-colors
            {statusFilter === f.value ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
          {f.label} ({count})
        </button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center" data-testid="admin-challenge-loading">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">{$_('common.loading')}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md" data-testid="admin-challenge-error">
      <p class="text-sm text-red-800">{error}</p>
      <button onclick={loadData} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">{$_('common.retry')}</button>
    </div>
  {:else if filteredChallenges.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_('gamification.no_challenges')}</p>
      <button onclick={() => showForm = true} class="mt-2 text-sm text-amber-600 hover:text-amber-800 underline">
        {$_('gamification.create_first')}
      </button>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredChallenges as challenge (challenge.id)}
        {@const statusCfg = getStatusConfig(challenge.status)}
        <li class="px-4 py-4 sm:px-6 hover:bg-gray-50" data-testid="admin-challenge-row">
          <div class="flex items-start gap-3">
            <span class="text-2xl flex-shrink-0">{challenge.icon || '🎯'}</span>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1 flex-wrap">
                <h4 class="text-sm font-semibold text-gray-900">{challenge.title}</h4>
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}">
                  {statusCfg.label}
                </span>
                <span class="text-xs text-gray-500">{typeLabels[challenge.challenge_type]}</span>
              </div>
              {#if challenge.description}
                <p class="text-xs text-gray-600 mb-2">{challenge.description}</p>
              {/if}
              <div class="flex items-center gap-3 text-xs text-gray-500">
                <span>{challenge.reward_points} pts</span>
                <span>{formatDateShort(challenge.start_date)} - {formatDateShort(challenge.end_date)}</span>
                <span>Objectif: {challenge.target_value} {challenge.target_metric}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-1 flex-shrink-0">
              {#if challenge.status === ChallengeStatus.Draft}
                <button onclick={() => handleActivate(challenge)}
                  class="px-2 py-1 text-xs text-green-600 hover:bg-green-50 rounded">{$_('gamification.activate')}</button>
                <button onclick={() => handleDelete(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">{$_('common.delete')}</button>
              {:else if challenge.status === ChallengeStatus.Active}
                <button onclick={() => handleComplete(challenge)}
                  class="px-2 py-1 text-xs text-blue-600 hover:bg-blue-50 rounded">{$_('gamification.complete')}</button>
                <button onclick={() => handleCancelChallenge(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">{$_('common.cancel')}</button>
              {:else}
                <button onclick={() => handleDelete(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">{$_('common.delete')}</button>
              {/if}
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
