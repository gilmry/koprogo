<script lang="ts">
  import { onMount } from 'svelte';
  import {
    gamificationApi,
    type Challenge,
    ChallengeStatus,
    ChallengeType,
  } from '../../lib/api/gamification';
  import ChallengeForm from './ChallengeForm.svelte';
  import { toast } from '../../stores/toast';

  export let organizationId: string;
  export let buildingId: string = '';

  let challenges: Challenge[] = [];
  let loading = true;
  let error = '';
  let showForm = false;
  let statusFilter: ChallengeStatus | 'all' = 'all';

  $: filteredChallenges = challenges.filter(c => {
    if (statusFilter === 'all') return true;
    return c.status === statusFilter;
  });

  onMount(async () => {
    await loadData();
  });

  async function loadData() {
    if (!organizationId) {
      loading = false;
      return;
    }
    try {
      loading = true;
      error = '';
      challenges = await gamificationApi.listChallenges(organizationId);
    } catch (err: any) {
      error = err.message || 'Erreur lors du chargement';
    } finally {
      loading = false;
    }
  }

  async function handleActivate(challenge: Challenge) {
    if (!confirm(`Activer le challenge "${challenge.title}" ?`)) return;
    try {
      await gamificationApi.activateChallenge(challenge.id);
      toast.success('Challenge active');
      await loadData();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de l\'activation');
    }
  }

  async function handleComplete(challenge: Challenge) {
    if (!confirm(`Marquer le challenge "${challenge.title}" comme termine ?`)) return;
    try {
      await gamificationApi.completeChallenge(challenge.id);
      toast.success('Challenge termine');
      await loadData();
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    }
  }

  async function handleCancel(challenge: Challenge) {
    if (!confirm(`Annuler le challenge "${challenge.title}" ?`)) return;
    try {
      await gamificationApi.cancelChallenge(challenge.id);
      toast.success('Challenge annule');
      await loadData();
    } catch (err: any) {
      toast.error(err.message || 'Erreur');
    }
  }

  async function handleDelete(challenge: Challenge) {
    if (!confirm(`Supprimer le challenge "${challenge.title}" ?`)) return;
    try {
      await gamificationApi.deleteChallenge(challenge.id);
      toast.success('Challenge supprime');
      await loadData();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la suppression');
    }
  }

  function handleSaved() {
    showForm = false;
    loadData();
  }

  function getStatusConfig(status: ChallengeStatus): { bg: string; text: string; label: string } {
    switch (status) {
      case ChallengeStatus.Draft: return { bg: 'bg-gray-100', text: 'text-gray-700', label: 'Brouillon' };
      case ChallengeStatus.Active: return { bg: 'bg-green-100', text: 'text-green-700', label: 'Actif' };
      case ChallengeStatus.Completed: return { bg: 'bg-blue-100', text: 'text-blue-700', label: 'Termine' };
      case ChallengeStatus.Cancelled: return { bg: 'bg-red-100', text: 'text-red-700', label: 'Annule' };
      default: return { bg: 'bg-gray-100', text: 'text-gray-700', label: status };
    }
  }

  const typeLabels: Record<ChallengeType, string> = {
    [ChallengeType.Individual]: 'Individuel',
    [ChallengeType.Team]: 'Equipe',
    [ChallengeType.Building]: 'Immeuble',
  };

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit', month: 'short', year: 'numeric',
    });
  }
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <div>
        <h3 class="text-lg leading-6 font-medium text-gray-900">Gestion des Challenges</h3>
        <p class="mt-1 text-sm text-gray-500">{challenges.length} challenge{challenges.length > 1 ? 's' : ''}</p>
      </div>
      <button on:click={() => showForm = !showForm}
        class="px-4 py-2 text-sm font-medium text-white bg-amber-600 rounded-md hover:bg-amber-700">
        {showForm ? 'Fermer' : '+ Nouveau'}
      </button>
    </div>
  </div>

  {#if showForm}
    <div class="p-4 bg-amber-50 border-b border-amber-200">
      <h4 class="text-sm font-medium text-amber-800 mb-3">Creer un challenge</h4>
      <ChallengeForm
        {organizationId}
        {buildingId}
        on:saved={handleSaved}
        on:cancel={() => showForm = false}
      />
    </div>
  {/if}

  <!-- Status filters -->
  <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
    <div class="flex flex-wrap gap-1">
      {#each [
        { value: 'all', label: 'Tous' },
        { value: ChallengeStatus.Draft, label: 'Brouillons' },
        { value: ChallengeStatus.Active, label: 'Actifs' },
        { value: ChallengeStatus.Completed, label: 'Termines' },
        { value: ChallengeStatus.Cancelled, label: 'Annules' },
      ] as f}
        {@const count = f.value === 'all' ? challenges.length : challenges.filter(c => c.status === f.value).length}
        <button on:click={() => statusFilter = f.value}
          class="px-2 py-1 rounded text-xs font-medium transition-colors
            {statusFilter === f.value ? 'bg-amber-600 text-white' : 'bg-white text-gray-600 hover:bg-gray-100 border border-gray-200'}">
          {f.label} ({count})
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
      <button on:click={loadData} class="mt-2 text-sm text-red-600 hover:text-red-800 underline">Reessayer</button>
    </div>
  {:else if filteredChallenges.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucun challenge</p>
      <button on:click={() => showForm = true} class="mt-2 text-sm text-amber-600 hover:text-amber-800 underline">
        Creer le premier
      </button>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredChallenges as challenge (challenge.id)}
        {@const statusCfg = getStatusConfig(challenge.status)}
        <li class="px-4 py-4 sm:px-6 hover:bg-gray-50">
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
                <span>{formatDate(challenge.start_date)} - {formatDate(challenge.end_date)}</span>
                <span>Objectif: {challenge.target_value} {challenge.target_metric}</span>
              </div>
            </div>

            <!-- Actions -->
            <div class="flex items-center gap-1 flex-shrink-0">
              {#if challenge.status === ChallengeStatus.Draft}
                <button on:click={() => handleActivate(challenge)}
                  class="px-2 py-1 text-xs text-green-600 hover:bg-green-50 rounded">Activer</button>
                <button on:click={() => handleDelete(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">Supprimer</button>
              {:else if challenge.status === ChallengeStatus.Active}
                <button on:click={() => handleComplete(challenge)}
                  class="px-2 py-1 text-xs text-blue-600 hover:bg-blue-50 rounded">Terminer</button>
                <button on:click={() => handleCancel(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">Annuler</button>
              {:else}
                <button on:click={() => handleDelete(challenge)}
                  class="px-2 py-1 text-xs text-red-600 hover:bg-red-50 rounded">Supprimer</button>
              {/if}
            </div>
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>
