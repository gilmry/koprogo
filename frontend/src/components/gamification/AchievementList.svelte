<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { onMount } from 'svelte';
  import {
    gamificationApi,
    type Achievement,
    type UserAchievement,
    AchievementCategory,
    AchievementTier,
  } from '../../lib/api/gamification';
  import { authStore } from '../../stores/auth';
  import { withLoadingState } from "../../lib/utils/error.utils";

  export let organizationId: string;

  let achievements: Achievement[] = [];
  let userAchievements: UserAchievement[] = [];
  let loading = true;
  let error = '';
  let categoryFilter: AchievementCategory | 'all' = 'all';

  $: earnedIds = new Set(userAchievements.map(ua => ua.achievement_id));

  $: filteredAchievements = achievements.filter(a => {
    if (categoryFilter === 'all') return true;
    return a.category === categoryFilter;
  });

  $: earnedCount = achievements.filter(a => earnedIds.has(a.id)).length;
  $: totalPoints = userAchievements.reduce((sum, ua) => {
    const ach = achievements.find(a => a.id === ua.achievement_id);
    return sum + (ach ? ach.points_value * ua.times_earned : 0);
  }, 0);

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    if (!organizationId) {
      loading = false;
      return;
    }
    await withLoadingState({
      action: () => Promise.all([
        gamificationApi.getVisibleAchievements(organizationId),
        $authStore.user?.id
          ? gamificationApi.getUserAchievements($authStore.user.id)
          : Promise.resolve([]),
      ]),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: ([achList, userAchList]) => {
        achievements = achList;
        userAchievements = userAchList;
      },
      errorMessage: $_('gamification.load_error'),
    });
  }

  function getTierConfig(tier: AchievementTier): { bg: string; text: string; border: string } {
    switch (tier) {
      case AchievementTier.Bronze: return { bg: 'bg-orange-100', text: 'text-orange-800', border: 'border-orange-300' };
      case AchievementTier.Silver: return { bg: 'bg-gray-100', text: 'text-gray-700', border: 'border-gray-300' };
      case AchievementTier.Gold: return { bg: 'bg-yellow-100', text: 'text-yellow-800', border: 'border-yellow-400' };
      case AchievementTier.Platinum: return { bg: 'bg-cyan-100', text: 'text-cyan-800', border: 'border-cyan-400' };
      case AchievementTier.Diamond: return { bg: 'bg-purple-100', text: 'text-purple-800', border: 'border-purple-400' };
      default: return { bg: 'bg-gray-100', text: 'text-gray-700', border: 'border-gray-300' };
    }
  }

  function getCategoryLabel(cat: AchievementCategory): string {
    const labels: Record<AchievementCategory, string> = {
      [AchievementCategory.Community]: $_('gamification.cat_community'),
      [AchievementCategory.Sel]: $_('gamification.cat_sel'),
      [AchievementCategory.Booking]: $_('gamification.cat_booking'),
      [AchievementCategory.Sharing]: $_('gamification.cat_sharing'),
      [AchievementCategory.Skills]: $_('gamification.cat_skills'),
      [AchievementCategory.Notice]: $_('gamification.cat_notice'),
      [AchievementCategory.Governance]: $_('gamification.cat_governance'),
      [AchievementCategory.Milestone]: $_('gamification.cat_milestone'),
    };
    return labels[cat] || cat;
  }

  function getUserAchievement(achievementId: string): UserAchievement | undefined {
    return userAchievements.find(ua => ua.achievement_id === achievementId);
  }
</script>

<div class="bg-white shadow-md rounded-lg" data-testid="achievement-list">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">{$_('gamification.achievements_title')}</h3>
        <p class="mt-1 text-sm text-gray-500">
          {$_('gamification.earned_stats', { earned: earnedCount, total: achievements.length, points: totalPoints })}
        </p>
      </div>
    </div>
  </div>

  <!-- Category filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex flex-wrap gap-1">
      <button on:click={() => categoryFilter = 'all'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors
          {categoryFilter === 'all' ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
        {$_('common.all')}
      </button>
      {#each Object.values(AchievementCategory) as cat}
        <button on:click={() => categoryFilter = cat}
          class="px-2 py-1 rounded text-xs font-medium transition-colors
            {categoryFilter === cat ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
          {getCategoryLabel(cat)}
        </button>
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center" data-testid="achievement-list-loading">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">{$_('common.loading')}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadData} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">{$_('common.retry')}</button>
    </div>
  {:else if filteredAchievements.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_('gamification.no_achievements')}</p>
    </div>
  {:else}
    <div class="p-4 grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
      {#each filteredAchievements as achievement (achievement.id)}
        {@const earned = earnedIds.has(achievement.id)}
        {@const userAch = getUserAchievement(achievement.id)}
        {@const tierCfg = getTierConfig(achievement.tier)}
        <div data-testid="achievement-card" class="relative p-4 rounded-lg border-2 transition-all
          {earned ? tierCfg.border + ' ' + tierCfg.bg : 'border-gray-200 bg-gray-50 opacity-60'}">
          {#if earned}
            <div class="absolute top-2 right-2">
              <span class="text-green-500 text-lg">&#10003;</span>
            </div>
          {/if}
          <div class="flex items-start gap-3">
            <span class="text-2xl">{achievement.icon || '🏅'}</span>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2 mb-1">
                <h4 class="text-sm font-semibold text-gray-900 truncate">{achievement.name}</h4>
                <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs font-medium {tierCfg.bg} {tierCfg.text}">
                  {achievement.tier}
                </span>
              </div>
              <p class="text-xs text-gray-600 line-clamp-2">{achievement.description}</p>
              <div class="mt-2 flex items-center gap-2 text-xs text-gray-500">
                <span>{achievement.points_value} pts</span>
                <span>{getCategoryLabel(achievement.category)}</span>
                {#if achievement.is_repeatable}
                  <span class="text-purple-600">{$_('gamification.repeatable')}</span>
                {/if}
                {#if userAch && userAch.times_earned > 1}
                  <span class="text-amber-600 font-medium">x{userAch.times_earned}</span>
                {/if}
              </div>
            </div>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
