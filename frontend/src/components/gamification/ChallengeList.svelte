<script lang="ts">
  import { onMount } from 'svelte';
  import {
    gamificationApi,
    type Challenge,
    type ChallengeProgress,
    ChallengeStatus,
    ChallengeType,
  } from '../../lib/api/gamification';
  import { authStore } from '../../stores/auth';

  export let organizationId: string;

  let challenges: Challenge[] = [];
  let userProgress: Map<string, ChallengeProgress> = new Map();
  let loading = true;
  let error = '';
  let statusFilter: 'active' | 'all' | 'completed' = 'active';

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    try {
      loading = true;
      error = '';

      const [challengeList, userChallenges] = await Promise.all([
        statusFilter === 'active'
          ? gamificationApi.getActiveChallenges(organizationId)
          : statusFilter === 'completed'
            ? gamificationApi.listByStatus(organizationId, ChallengeStatus.Completed)
            : gamificationApi.listChallenges(organizationId),
        $authStore.user?.id
          ? gamificationApi.getUserActiveChallenges($authStore.user.id)
          : Promise.resolve([]),
      ]);

      challenges = challengeList;
      userProgress = new Map(userChallenges.map(p => [p.challenge_id, p]));
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement des challenges';
    } finally {
      loading = false;
    }
  }

  $: if (statusFilter) loadData();

  function getStatusConfig(status: ChallengeStatus): { bg: string; text: string; label: string } {
    switch (status) {
      case ChallengeStatus.Draft: return { bg: 'bg-gray-100', text: 'text-gray-700', label: 'Brouillon' };
      case ChallengeStatus.Active: return { bg: 'bg-green-100', text: 'text-green-700', label: 'Actif' };
      case ChallengeStatus.Completed: return { bg: 'bg-blue-100', text: 'text-blue-700', label: 'Terminé' };
      case ChallengeStatus.Cancelled: return { bg: 'bg-red-100', text: 'text-red-700', label: 'Annulé' };
      default: return { bg: 'bg-gray-100', text: 'text-gray-700', label: status };
    }
  }

  function getTypeLabel(type: ChallengeType): string {
    switch (type) {
      case ChallengeType.Individual: return 'Individuel';
      case ChallengeType.Team: return 'Équipe';
      case ChallengeType.Building: return 'Immeuble';
      default: return type;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: 'short',
      year: 'numeric',
    });
  }

  function getDaysRemaining(endDate: string): number {
    const end = new Date(endDate);
    const now = new Date();
    return Math.max(0, Math.ceil((end.getTime() - now.getTime()) / (1000 * 60 * 60 * 24)));
  }

  function getProgressPercent(progress: ChallengeProgress | undefined, target: number): number {
    if (!progress || target <= 0) return 0;
    return Math.min(100, (progress.current_value / target) * 100);
  }
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <h3 class="text-lg leading-6 font-medium text-gray-900">Challenges</h3>
    <p class="mt-1 text-sm text-gray-500">
      Relevez des défis pour gagner des points.
    </p>
  </div>

  <!-- Status filter -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex gap-1">
      {#each [
        { value: 'active', label: 'Actifs' },
        { value: 'all', label: 'Tous' },
        { value: 'completed', label: 'Terminés' },
      ] as f}
        <button on:click={() => statusFilter = f.value}
          class="px-2 py-1 rounded text-xs font-medium transition-colors
            {statusFilter === f.value ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
          {f.label}
        </button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">Chargement...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadData} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">Réessayer</button>
    </div>
  {:else if challenges.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucun challenge {statusFilter === 'active' ? 'actif' : ''} pour le moment</p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each challenges as challenge (challenge.id)}
        {@const statusCfg = getStatusConfig(challenge.status)}
        {@const progress = userProgress.get(challenge.id)}
        {@const progressPct = getProgressPercent(progress, challenge.target_value)}
        {@const daysLeft = getDaysRemaining(challenge.end_date)}
        <li class="px-4 py-4 sm:px-6">
          <div class="flex items-start gap-3">
            <span class="text-2xl flex-shrink-0">{challenge.icon || '🎯'}</span>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="text-sm font-semibold text-gray-900">{challenge.title}</h4>
                <span class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {statusCfg.bg} {statusCfg.text}">
                  {statusCfg.label}
                </span>
                <span class="text-xs text-gray-500">{getTypeLabel(challenge.challenge_type)}</span>
              </div>

              <p class="text-xs text-gray-600 mb-2">{challenge.description}</p>

              <!-- Progress bar -->
              {#if challenge.status === ChallengeStatus.Active}
                <div class="mb-2">
                  <div class="flex items-center justify-between text-xs text-gray-500 mb-1">
                    <span>{progress?.current_value || 0} / {challenge.target_value} {challenge.target_metric}</span>
                    <span>{progressPct.toFixed(0)}%</span>
                  </div>
                  <div class="w-full bg-gray-200 rounded-full h-2">
                    <div class="h-2 rounded-full transition-all duration-500
                      {progress?.completed ? 'bg-green-500' : 'bg-amber-500'}"
                      style="width: {progressPct}%"></div>
                  </div>
                </div>
              {/if}

              <div class="flex items-center gap-3 text-xs text-gray-500">
                <span>{challenge.reward_points} pts</span>
                <span>{formatDate(challenge.start_date)} - {formatDate(challenge.end_date)}</span>
                {#if challenge.status === ChallengeStatus.Active && daysLeft > 0}
                  <span class="font-medium {daysLeft <= 3 ? 'text-red-600' : 'text-gray-600'}">
                    {daysLeft}j restants
                  </span>
                {/if}
                {#if progress?.completed}
                  <span class="text-green-600 font-medium">Complété !</span>
                {/if}
              </div>
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
