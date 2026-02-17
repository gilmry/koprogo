<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import {
    gamificationApi,
    type Challenge,
    ChallengeType,
  } from '../../lib/api/gamification';
  import { toast } from '../../stores/toast';

  export let organizationId: string;
  export let challenge: Challenge | null = null;
  export let buildingId: string = '';

  const dispatch = createEventDispatcher();
  let saving = false;

  let title = challenge?.title || '';
  let description = challenge?.description || '';
  let challengeType: ChallengeType = challenge?.challenge_type || ChallengeType.Individual;
  let icon = challenge?.icon || '';
  let startDate = challenge?.start_date ? challenge.start_date.slice(0, 16) : '';
  let endDate = challenge?.end_date ? challenge.end_date.slice(0, 16) : '';
  let targetMetric = challenge?.target_metric || '';
  let targetValue = challenge?.target_value || 1;
  let rewardPoints = challenge?.reward_points || 100;

  const typeLabels: Record<ChallengeType, string> = {
    [ChallengeType.Individual]: 'Individuel',
    [ChallengeType.Team]: 'Equipe',
    [ChallengeType.Building]: 'Immeuble',
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
      toast.error('Le titre est obligatoire');
      return;
    }
    if (!startDate || !endDate) {
      toast.error('Les dates de debut et fin sont obligatoires');
      return;
    }
    if (new Date(startDate) >= new Date(endDate)) {
      toast.error('La date de fin doit etre apres la date de debut');
      return;
    }
    if (!targetMetric.trim()) {
      toast.error('La metrique cible est obligatoire');
      return;
    }
    if (targetValue < 1) {
      toast.error('La valeur cible doit etre au moins 1');
      return;
    }

    try {
      saving = true;
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

      const result = await gamificationApi.createChallenge(data);
      toast.success('Challenge cree');
      dispatch('saved', result);
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la creation');
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
      <label for="ch-title" class="block text-sm font-medium text-gray-700">Titre *</label>
      <input id="ch-title" type="text" bind:value={title} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="Challenge du mois : 5 echanges SEL" />
    </div>

    <div class="md:col-span-2">
      <label for="ch-desc" class="block text-sm font-medium text-gray-700">Description</label>
      <textarea id="ch-desc" bind:value={description} rows="2"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="Completez 5 echanges dans le SEL ce mois-ci"></textarea>
    </div>

    <div>
      <label for="ch-type" class="block text-sm font-medium text-gray-700">Type</label>
      <select id="ch-type" bind:value={challengeType}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm">
        {#each Object.values(ChallengeType) as t}
          <option value={t}>{typeLabels[t]}</option>
        {/each}
      </select>
    </div>

    <div>
      <label for="ch-icon" class="block text-sm font-medium text-gray-700">Icone (emoji)</label>
      <input id="ch-icon" type="text" bind:value={icon}
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="🎯" />
    </div>

    <div>
      <label for="ch-start" class="block text-sm font-medium text-gray-700">Debut *</label>
      <input id="ch-start" type="datetime-local" bind:value={startDate} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-end" class="block text-sm font-medium text-gray-700">Fin *</label>
      <input id="ch-end" type="datetime-local" bind:value={endDate} required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-metric" class="block text-sm font-medium text-gray-700">Metrique cible *</label>
      <input id="ch-metric" type="text" bind:value={targetMetric} required list="metric-suggestions"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm"
        placeholder="exchanges_completed" />
      <datalist id="metric-suggestions">
        {#each metricSuggestions as suggestion}
          <option value={suggestion} />
        {/each}
      </datalist>
    </div>

    <div>
      <label for="ch-target" class="block text-sm font-medium text-gray-700">Valeur cible *</label>
      <input id="ch-target" type="number" bind:value={targetValue} min="1" required
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>

    <div>
      <label for="ch-reward" class="block text-sm font-medium text-gray-700">Points de recompense</label>
      <input id="ch-reward" type="number" bind:value={rewardPoints} min="0" max="10000"
        class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-amber-500 focus:ring-amber-500 sm:text-sm" />
    </div>
  </div>

  <div class="flex justify-end gap-3 pt-4 border-t border-gray-200">
    <button type="button" on:click={handleCancel}
      class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50">
      Annuler
    </button>
    <button type="submit" disabled={saving}
      class="px-4 py-2 text-sm font-medium text-white bg-amber-600 border border-transparent rounded-md hover:bg-amber-700 disabled:opacity-50">
      {#if saving}
        Sauvegarde...
      {:else}
        Creer le challenge
      {/if}
    </button>
  </div>
</form>
