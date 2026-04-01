<script lang="ts">
  import { _ } from '../../lib/i18n';
  import { onMount } from 'svelte';
  import {
    gamificationApi,
    type Achievement,
    AchievementCategory,
    AchievementTier,
  } from '../../lib/api/gamification';
  import AchievementForm from './AchievementForm.svelte';
  import { toast } from '../../stores/toast';
  import { withLoadingState, withErrorHandling } from "../../lib/utils/error.utils";

  export let organizationId: string;

  let achievements: Achievement[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let editingAchievement: Achievement | null = null;
  let categoryFilter: AchievementCategory | 'all' = 'all';

  $: filteredAchievements = achievements.filter(a => {
    if (categoryFilter === 'all') return true;
    return a.category === categoryFilter;
  });

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    if (!organizationId) {
      loading = false;
      return;
    }
    await withLoadingState({
      action: () => gamificationApi.listAchievements(organizationId),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => achievements = data,
      errorMessage: $_('common.load_error'),
    });
  }

  function handleCreate() {
    editingAchievement = null;
    showForm = true;
  }

  function handleEdit(achievement: Achievement) {
    editingAchievement = achievement;
    showForm = true;
  }

  async function handleDelete(achievement: Achievement) {
    if (!confirm($_('gamification.confirm_delete', { name: achievement.name }))) return;
    await withErrorHandling({
      action: () => gamificationApi.deleteAchievement(achievement.id),
      successMessage: $_('gamification.delete_success'),
      errorMessage: $_('gamification.delete_error'),
      onSuccess: () => loadData(),
    });
  }

  function handleSaved() {
    showForm = false;
    editingAchievement = null;
    loadData();
  }

  function handleCancel() {
    showForm = false;
    editingAchievement = null;
  }

  function getTierColor(tier: AchievementTier): string {
    switch (tier) {
      case AchievementTier.Bronze: return 'bg-orange-100 text-orange-800';
      case AchievementTier.Silver: return 'bg-gray-100 text-gray-700';
      case AchievementTier.Gold: return 'bg-yellow-100 text-yellow-800';
      case AchievementTier.Platinum: return 'bg-cyan-100 text-cyan-800';
      case AchievementTier.Diamond: return 'bg-purple-100 text-purple-800';
      default: return 'bg-gray-100 text-gray-700';
    }
  }

  const categoryLabels: Record<AchievementCategory, string> = {
    [AchievementCategory.Community]: $_('gamification.cat_community'),
    [AchievementCategory.Sel]: $_('gamification.cat_sel'),
    [AchievementCategory.Booking]: $_('gamification.cat_booking'),
    [AchievementCategory.Sharing]: $_('gamification.cat_sharing'),
    [AchievementCategory.Skills]: $_('gamification.cat_skills'),
    [AchievementCategory.Notice]: $_('gamification.cat_notice'),
    [AchievementCategory.Governance]: $_('gamification.cat_governance'),
    [AchievementCategory.Milestone]: $_('gamification.cat_milestone'),
  };
</script>

<div class="bg-white shadow-md rounded-lg" data-testid="admin-achievement-list">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">{$_('gamification.management_title')}</h3>
        <p class="mt-1 text-sm text-gray-500">{$_('gamification.achievement_count', { values: { count: achievements.length } })}</p>
      </div>
      <button on:click={handleCreate}
        data-testid="achievement-create-btn"
        class="px-4 py-2 text-sm font-medium text-white bg-amber-600 rounded-md hover:bg-amber-700">
        + {$_('common.new')}
      </button>
    </div>
  </div>

  {#if showForm}
    <div class="p-4 bg-amber-50 border-b border-amber-200">
      <h4 class="text-sm font-medium text-amber-800 mb-3">
        {editingAchievement ? $_('common.edit') : $_('common.create')} {$_('gamification.achievement_singular')}
      </h4>
      <AchievementForm
        {organizationId}
        achievement={editingAchievement}
        on:saved={handleSaved}
        on:cancel={handleCancel}
      />
    </div>
  {/if}

  <!-- Category filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex flex-wrap gap-1">
      <button on:click={() => categoryFilter = 'all'}
        class="px-2 py-1 rounded text-xs font-medium transition-colors
          {categoryFilter === 'all' ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
        {$_('common.all')} ({achievements.length})
      </button>
      {#each Object.values(AchievementCategory) as cat}
        {@const count = achievements.filter(a => a.category === cat).length}
        {#if count > 0}
          <button on:click={() => categoryFilter = cat}
            class="px-2 py-1 rounded text-xs font-medium transition-colors
              {categoryFilter === cat ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
            {categoryLabels[cat]} ({count})
          </button>
        {/if}
      {/each}
    </div>
  </div>

  {#if loading}
    <div class="p-8 text-center" data-testid="admin-achievement-loading">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-amber-600"></div>
      <p class="mt-2 text-sm text-gray-500">{$_('common.loading')}</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md" data-testid="admin-achievement-error">
      <p class="text-sm text-red-800">{error}</p>
      <button on:click={loadData} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">{$_('common.retry')}</button>
    </div>
  {:else if filteredAchievements.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">{$_('gamification.no_achievements')}</p>
      <button on:click={handleCreate} class="mt-2 text-sm text-amber-600 hover:text-amber-800 underline">
        {$_('gamification.create_first')}
      </button>
    </div>
  {:else}
    <div class="overflow-x-auto">
      <table class="min-w-full divide-y divide-gray-200">
        <thead class="bg-gray-50">
          <tr>
            <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('gamification.achievement_singular')}</th>
            <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('gamification.category')}</th>
            <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('gamification.tier')}</th>
            <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('gamification.points')}</th>
            <th scope="col" class="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase">{$_('gamification.flags')}</th>
            <th scope="col" class="px-4 py-3 text-right text-xs font-medium text-gray-500 uppercase">{$_('common.actions')}</th>
          </tr>
        </thead>
        <tbody class="bg-white divide-y divide-gray-200">
          {#each filteredAchievements as achievement (achievement.id)}
            <tr class="hover:bg-gray-50" data-testid="admin-achievement-row">
              <td class="px-4 py-3">
                <div class="flex items-center gap-2">
                  <span class="text-lg">{achievement.icon || '🏅'}</span>
                  <div>
                    <p class="text-sm font-medium text-gray-900">{achievement.name}</p>
                    {#if achievement.description}
                      <p class="text-xs text-gray-500 truncate max-w-xs">{achievement.description}</p>
                    {/if}
                  </div>
                </div>
              </td>
              <td class="px-4 py-3">
                <span class="text-xs text-gray-600">{categoryLabels[achievement.category]}</span>
              </td>
              <td class="px-4 py-3">
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getTierColor(achievement.tier)}">
                  {achievement.tier}
                </span>
              </td>
              <td class="px-4 py-3 text-sm text-gray-900">{achievement.points_value}</td>
              <td class="px-4 py-3">
                <div class="flex gap-1">
                  {#if achievement.is_secret}
                    <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs bg-gray-800 text-white">{$_('gamification.secret')}</span>
                  {/if}
                  {#if achievement.is_repeatable}
                    <span class="inline-flex items-center px-1.5 py-0.5 rounded text-xs bg-purple-100 text-purple-800">{$_('gamification.repeatable')}</span>
                  {/if}
                </div>
              </td>
              <td class="px-4 py-3 text-right">
                <div class="flex justify-end gap-1">
                  <button on:click={() => handleEdit(achievement)}
                    data-testid="achievement-edit-btn"
                    class="px-2 py-1 text-xs text-blue-600 hover:bg-blue-50 rounded">
                    {$_('common.edit')}
                  </button>
                  <button on:click={() => handleDelete(achievement)}
                    data-testid="achievement-delete-btn"
                    class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">
                    {$_('common.delete')}
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
