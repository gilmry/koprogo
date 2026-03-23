<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { _ } from '../../lib/i18n';
  import {
    gamificationApi,
    type Achievement,
    AchievementCategory,
    AchievementTier,
  } from '../../lib/api/gamification';
  import { toast } from '../../stores/toast';

  export let organizationId: string;
  export let achievement: Achievement | null = null;

  const dispatch = createEventDispatcher();
  let saving = false;

  let title = achievement?.name || '';
  let description = achievement?.description || '';
  let category: AchievementCategory = achievement?.category || AchievementCategory.Community;
  let tier: AchievementTier = achievement?.tier || AchievementTier.Bronze;
  let icon = achievement?.icon || '';
  let pointsValue = achievement?.points_value || 10;
  let isSecret = achievement?.is_secret || false;
  let isRepeatable = achievement?.is_repeatable || false;
  let displayOrder = achievement?.display_order || 0;

  const categoryLabels: Record<AchievementCategory, string> = {
    [AchievementCategory.Community]: $_('gamification.category.community'),
    [AchievementCategory.Sel]: $_('gamification.category.sel'),
    [AchievementCategory.Booking]: $_('gamification.category.booking'),
    [AchievementCategory.Sharing]: $_('gamification.category.sharing'),
    [AchievementCategory.Skills]: $_('gamification.category.skills'),
    [AchievementCategory.Notice]: $_('gamification.category.notice'),
    [AchievementCategory.Governance]: $_('gamification.category.governance'),
    [AchievementCategory.Milestone]: $_('gamification.category.milestone'),
  };

  const tierLabels: Record<AchievementTier, string> = {
    [AchievementTier.Bronze]: $_('gamification.tier.bronze'),
    [AchievementTier.Silver]: $_('gamification.tier.silver'),
    [AchievementTier.Gold]: $_('gamification.tier.gold'),
    [AchievementTier.Platinum]: $_('gamification.tier.platinum'),
    [AchievementTier.Diamond]: $_('gamification.tier.diamond'),
  };

  async function handleSubmit() {
    if (!title.trim()) {
      toast.error($_('gamification.nameRequired'));
      return;
    }
    if (title.trim().length < 3) {
      toast.error($_('gamification.nameMinLength'));
      return;
    }
    if (!description.trim()) {
      toast.error($_('gamification.descriptionRequired'));
      return;
    }
    if (description.trim().length < 10) {
      toast.error($_('gamification.descriptionMinLength'));
      return;
    }
    if (pointsValue < 0 || pointsValue > 1000) {
      toast.error($_('gamification.pointsRange'));
      return;
    }

    try {
      saving = true;
      const data = {
        organization_id: organizationId,
        name: title.trim(),
        description: description.trim(),
        category,
        tier,
        icon: icon || '🏅',
        points_value: pointsValue,
        is_secret: isSecret,
        is_repeatable: isRepeatable,
        display_order: displayOrder,
        requirements: '{}',
      };

      let result: Achievement;
      if (achievement) {
        result = await gamificationApi.updateAchievement(achievement.id, data);
        toast.success($_('gamification.updateSuccess', { values: { name: title.trim() } }));
      } else {
        result = await gamificationApi.createAchievement(data);
        toast.success($_('gamification.createSuccess', { values: { name: title.trim() } }));
      }
      dispatch('saved', result);
    } catch (err: any) {
      toast.error(err.message || $_('gamification.saveError'));
    } finally {
      saving = false;
    }
  }

  function handleCancel() {
    dispatch('cancel');
  }
</script>

<form on:submit|preventDefault={handleSubmit} class="space-y-4">
  <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
    <div class="md:col-span-2">
      <label for="ach-name" class="block text-sm font-medium text-gray-700">{$_('common.name')} *</label>
      <input id="ach-name" type="text" bind:value={title} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder={$_('gamification.namePlaceholder')} />
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.nameHelp')}</p>
    </div>

    <div class="md:col-span-2">
      <label for="ach-desc" class="block text-sm font-medium text-gray-700">{$_('common.description')} *</label>
      <textarea id="ach-desc" bind:value={description} rows="2"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder={$_('gamification.descriptionPlaceholder')}></textarea>
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.descriptionHelp')}</p>
    </div>

    <div>
      <label for="ach-category" class="block text-sm font-medium text-gray-700">{$_('gamification.category')} *</label>
      <select id="ach-category" bind:value={category}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm">
        {#each Object.values(AchievementCategory) as cat}
          <option value={cat}>{categoryLabels[cat]}</option>
        {/each}
      </select>
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.categoryHelp')}</p>
    </div>

    <div>
      <label for="ach-tier" class="block text-sm font-medium text-gray-700">{$_('gamification.tier')} *</label>
      <select id="ach-tier" bind:value={tier}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm">
        {#each Object.values(AchievementTier) as t}
          <option value={t}>{tierLabels[t]}</option>
        {/each}
      </select>
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.tierHelp')}</p>
    </div>

    <div>
      <label for="ach-icon" class="block text-sm font-medium text-gray-700">{$_('gamification.icon')}</label>
      <input id="ach-icon" type="text" bind:value={icon}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="🏅" />
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.iconHelp')}</p>
    </div>

    <div>
      <label for="ach-points" class="block text-sm font-medium text-gray-700">{$_('gamification.points')} *</label>
      <input id="ach-points" type="number" bind:value={pointsValue} min="0" max="1000"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.pointsHelp')}</p>
    </div>

    <div>
      <label for="ach-order" class="block text-sm font-medium text-gray-700">{$_('gamification.displayOrder')}</label>
      <input id="ach-order" type="number" bind:value={displayOrder} min="0"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
      <p class="mt-1 text-xs text-gray-500">{$_('gamification.displayOrderHelp')}</p>
    </div>

    <div class="flex items-center gap-6">
      <label class="flex items-center gap-2 text-sm text-gray-700">
        <input type="checkbox" bind:checked={isSecret} class="rounded border-gray-300 text-amber-600 focus:ring-amber-500" />
        {$_('gamification.secret')}
      </label>
      <label class="flex items-center gap-2 text-sm text-gray-700">
        <input type="checkbox" bind:checked={isRepeatable} class="rounded border-gray-300 text-amber-600 focus:ring-amber-500" />
        {$_('gamification.repeatable')}
      </label>
    </div>
  </div>

  <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
    <button type="button" on:click={handleCancel}
      class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
      {$_('common.cancel')}
    </button>
    <button type="submit" disabled={saving}
      class="px-4 py-2 text-sm font-medium text-white bg-amber-600 border border-transparent rounded-md hover:bg-amber-700 disabled:opacity-50">
      {#if saving}
        {$_('common.saving')}
      {:else}
        {achievement ? $_('common.edit') : $_('common.create')}
      {/if}
    </button>
  </div>
</form>
