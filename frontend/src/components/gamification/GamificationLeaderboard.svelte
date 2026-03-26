<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { onMount } from 'svelte';
  import {
    gamificationApi,
    type LeaderboardEntry,
  } from '../../lib/api/gamification';
  import { authStore } from '../../stores/auth';
  import { withLoadingState } from "../../lib/utils/error.utils";

  export let organizationId: string;
  export let buildingId: string = '';
  export let limit: number = 10;

  let leaderboard: LeaderboardEntry[] = [];
  let loading = true;
  let error = '';

  $: currentUserId = $authStore.user?.id;

  onMount(async () => {
    await loadLeaderboard();
  });

  async function loadLeaderboard() {
    if (!organizationId) {
      loading = false;
      return;
    }
    await withLoadingState({
      action: () => gamificationApi.getLeaderboard(organizationId, buildingId || undefined, limit),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => leaderboard = data,
      errorMessage: $_('gamification.leaderboard_error'),
    });
  }

  function getRankDisplay(rank: number): string {
    switch (rank) {
      case 1: return '🥇';
      case 2: return '🥈';
      case 3: return '🥉';
      default: return String(rank);
    }
  }
</script>

<div class="bg-white shadow-md rounded-lg" data-testid="gamification-leaderboard">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <h3 class="text-lg leading-6 font-medium text-gray-900">{$_('gamification.leaderboard')}</h3>
    <p class="mt-1 text-sm text-gray-500">{$_('gamification.leaderboard_description')}</p>
  </div>

  {#if loading}
    <div class="p-8 text-center" data-testid="gamification-leaderboard-loading">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">{$_('common.loading')}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadLeaderboard} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">{$_('common.retry')}</button>
    </div>
  {:else if leaderboard.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_('gamification.no_participants')}</p>
    </div>
  {:else}
    <div class="divide-y divide-gray-100">
      {#each leaderboard as entry (entry.user_id)}
        {@const isMe = entry.user_id === currentUserId}
        <div data-testid="gamification-leaderboard-row" class="flex items-center gap-4 px-4 py-3 {isMe ? 'bg-amber-50' : 'hover:bg-gray-50'}">
          <!-- Rank -->
          <div class="flex-shrink-0 w-10 h-10 rounded-full flex items-center justify-center font-bold text-lg
            {entry.rank <= 3 ? 'bg-gradient-to-br from-yellow-300 to-amber-400 text-white shadow' : 'bg-gray-100 text-gray-600'}">
            {getRankDisplay(entry.rank)}
          </div>

          <!-- User info -->
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-gray-900 truncate">
              {entry.user_name}
              {#if isMe}
                <span class="text-xs text-amber-600 font-medium">({$_('common.you')})</span>
              {/if}
            </p>
            <div class="flex items-center gap-3 text-xs text-gray-500">
              <span>{$_('gamification.achievement_count', { count: entry.achievements_count })}</span>
              <span>{$_('gamification.challenge_count_completed', { count: entry.challenges_completed })}</span>
            </div>
          </div>

          <!-- Points -->
          <div class="flex-shrink-0 text-right">
            <p class="text-lg font-bold text-amber-600">{entry.total_points}</p>
            <p class="text-xs text-gray-500">{$_('gamification.points')}</p>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
