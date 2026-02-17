<script lang="ts">
  import { onMount } from 'svelte';
  import {
    resolutionsApi,
    type Resolution,
    type Vote,
    VoteChoice,
    ResolutionStatus,
    MajorityType,
  } from '../../lib/api/resolutions';
  import { toast } from '../../stores/toast';
  import ResolutionStatusBadge from './ResolutionStatusBadge.svelte';

  export let resolution: Resolution;
  export let meetingStatus: string = 'Scheduled';
  export let isAdmin: boolean = false;

  let votes: Vote[] = [];
  let loadingVotes = false;
  let showVotes = false;

  // Vote form state
  let voteChoice: VoteChoice | null = null;
  let votingPower: number = 1;
  let proxyOwnerId: string = '';
  let submittingVote = false;
  let closingVoting = false;

  $: canVote = resolution.status === ResolutionStatus.Pending && meetingStatus === 'Scheduled';
  $: isClosed = resolution.status !== ResolutionStatus.Pending;
  $: totalVotes = resolution.votes_pour + resolution.votes_contre + resolution.votes_abstention;

  function getMajorityLabel(type: MajorityType): string {
    switch (type) {
      case MajorityType.Simple: return 'Majorité simple (50%+1 des votes exprimés)';
      case MajorityType.Absolute: return 'Majorité absolue (50%+1 de tous les votes)';
      case MajorityType.Qualified: return 'Majorité qualifiée (seuil personnalisé)';
      default: return type;
    }
  }

  function getVotePercentage(count: number): number {
    if (totalVotes === 0) return 0;
    return (count / totalVotes) * 100;
  }

  async function loadVotes() {
    try {
      loadingVotes = true;
      votes = await resolutionsApi.getVotes(resolution.id);
      showVotes = true;
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors du chargement des votes');
    } finally {
      loadingVotes = false;
    }
  }

  async function handleVote() {
    if (!voteChoice) {
      toast.error('Veuillez sélectionner votre vote');
      return;
    }

    try {
      submittingVote = true;
      await resolutionsApi.castVote(resolution.id, {
        owner_id: '', // sera rempli par le backend via le token JWT
        choice: voteChoice,
        voting_power: votingPower,
        proxy_owner_id: proxyOwnerId || undefined,
      });
      toast.success('Vote enregistré avec succès');
      // Reload resolution to get updated counts
      resolution = await resolutionsApi.getById(resolution.id);
      voteChoice = null;
      if (showVotes) await loadVotes();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors du vote');
    } finally {
      submittingVote = false;
    }
  }

  async function handleCloseVoting() {
    if (!confirm('Êtes-vous sûr de vouloir clôturer le vote ? Cette action est irréversible.')) return;

    try {
      closingVoting = true;
      resolution = await resolutionsApi.closeVoting(resolution.id);
      toast.success(`Vote clôturé - Résolution ${resolution.status === ResolutionStatus.Adopted ? 'adoptée' : 'rejetée'}`);
      if (showVotes) await loadVotes();
    } catch (err: any) {
      toast.error(err.message || 'Erreur lors de la clôture du vote');
    } finally {
      closingVoting = false;
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-BE', {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  }

  function getChoiceLabel(choice: VoteChoice): string {
    switch (choice) {
      case VoteChoice.Pour: return 'Pour';
      case VoteChoice.Contre: return 'Contre';
      case VoteChoice.Abstention: return 'Abstention';
      default: return choice;
    }
  }

  function getChoiceColor(choice: VoteChoice): string {
    switch (choice) {
      case VoteChoice.Pour: return 'text-green-700 bg-green-100';
      case VoteChoice.Contre: return 'text-red-700 bg-red-100';
      case VoteChoice.Abstention: return 'text-gray-700 bg-gray-100';
      default: return 'text-gray-700 bg-gray-100';
    }
  }
</script>

<div class="border border-gray-200 rounded-lg p-4">
  <!-- Header -->
  <div class="flex items-start justify-between mb-3">
    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2 mb-1">
        <h4 class="text-sm font-semibold text-gray-900 truncate">{resolution.title}</h4>
        <ResolutionStatusBadge status={resolution.status} />
      </div>
      {#if resolution.description}
        <p class="text-sm text-gray-600 mt-1">{resolution.description}</p>
      {/if}
      <p class="text-xs text-gray-400 mt-1">
        {getMajorityLabel(resolution.majority_required)}
      </p>
    </div>
  </div>

  <!-- Vote Results / Progress Bars -->
  <div class="space-y-2 mb-4">
    <!-- Pour -->
    <div>
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-green-700 font-medium">Pour</span>
        <span class="text-gray-600">{resolution.votes_pour} vote{resolution.votes_pour !== 1 ? 's' : ''} ({getVotePercentage(resolution.votes_pour).toFixed(1)}%)</span>
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div class="bg-green-500 h-2.5 rounded-full transition-all" style="width: {getVotePercentage(resolution.votes_pour)}%"></div>
      </div>
    </div>

    <!-- Contre -->
    <div>
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-red-700 font-medium">Contre</span>
        <span class="text-gray-600">{resolution.votes_contre} vote{resolution.votes_contre !== 1 ? 's' : ''} ({getVotePercentage(resolution.votes_contre).toFixed(1)}%)</span>
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div class="bg-red-500 h-2.5 rounded-full transition-all" style="width: {getVotePercentage(resolution.votes_contre)}%"></div>
      </div>
    </div>

    <!-- Abstention -->
    <div>
      <div class="flex items-center justify-between text-sm mb-1">
        <span class="text-gray-700 font-medium">Abstention</span>
        <span class="text-gray-600">{resolution.votes_abstention} vote{resolution.votes_abstention !== 1 ? 's' : ''} ({getVotePercentage(resolution.votes_abstention).toFixed(1)}%)</span>
      </div>
      <div class="w-full bg-gray-100 rounded-full h-2.5">
        <div class="bg-gray-400 h-2.5 rounded-full transition-all" style="width: {getVotePercentage(resolution.votes_abstention)}%"></div>
      </div>
    </div>

    <p class="text-xs text-gray-500 mt-1">
      Total : {totalVotes} vote{totalVotes !== 1 ? 's' : ''}
      {#if resolution.total_voting_power > 0}
        &middot; Pouvoir de vote total : {resolution.total_voting_power} millièmes
      {/if}
    </p>
  </div>

  <!-- Vote Form (if pending) -->
  {#if canVote}
    <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 mb-3">
      <h5 class="text-sm font-semibold text-blue-900 mb-3">Voter sur cette résolution</h5>

      <div class="flex gap-2 mb-3">
        <button
          on:click={() => voteChoice = VoteChoice.Pour}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Pour
              ? 'bg-green-600 text-white border-green-600'
              : 'bg-white text-green-700 border-green-300 hover:bg-green-50'}"
          disabled={submittingVote}
        >
          Pour
        </button>
        <button
          on:click={() => voteChoice = VoteChoice.Contre}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Contre
              ? 'bg-red-600 text-white border-red-600'
              : 'bg-white text-red-700 border-red-300 hover:bg-red-50'}"
          disabled={submittingVote}
        >
          Contre
        </button>
        <button
          on:click={() => voteChoice = VoteChoice.Abstention}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium border-2 transition-colors
            {voteChoice === VoteChoice.Abstention
              ? 'bg-gray-600 text-white border-gray-600'
              : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'}"
          disabled={submittingVote}
        >
          Abstention
        </button>
      </div>

      <div class="grid grid-cols-2 gap-3 mb-3">
        <div>
          <label for="voting-power-{resolution.id}" class="block text-xs font-medium text-gray-700 mb-1">
            Pouvoir de vote (millièmes)
          </label>
          <input
            id="voting-power-{resolution.id}"
            type="number"
            bind:value={votingPower}
            min="1"
            max="1000"
            class="w-full px-2 py-1.5 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
        <div>
          <label for="proxy-{resolution.id}" class="block text-xs font-medium text-gray-700 mb-1">
            Procuration (optionnel)
          </label>
          <input
            id="proxy-{resolution.id}"
            type="text"
            bind:value={proxyOwnerId}
            placeholder="ID du mandataire"
            class="w-full px-2 py-1.5 border border-gray-300 rounded-md text-sm focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
          />
        </div>
      </div>

      <button
        on:click={handleVote}
        disabled={!voteChoice || submittingVote}
        class="w-full py-2 px-4 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {#if submittingVote}
          Enregistrement...
        {:else}
          Enregistrer le vote
        {/if}
      </button>
    </div>
  {/if}

  <!-- Actions -->
  <div class="flex items-center gap-2">
    <!-- View votes -->
    <button
      on:click={showVotes ? () => showVotes = false : loadVotes}
      class="text-xs text-indigo-600 hover:text-indigo-800 underline"
      disabled={loadingVotes}
    >
      {#if loadingVotes}
        Chargement...
      {:else if showVotes}
        Masquer les votes
      {:else}
        Voir les votes ({totalVotes})
      {/if}
    </button>

    <!-- Close voting (admin only) -->
    {#if isAdmin && canVote && totalVotes > 0}
      <button
        on:click={handleCloseVoting}
        disabled={closingVoting}
        class="text-xs text-orange-600 hover:text-orange-800 underline ml-auto"
      >
        {closingVoting ? 'Clôture...' : 'Clôturer le vote'}
      </button>
    {/if}
  </div>

  <!-- Votes list -->
  {#if showVotes && votes.length > 0}
    <div class="mt-3 border-t border-gray-100 pt-3">
      <table class="w-full text-sm">
        <thead>
          <tr class="text-left text-xs text-gray-500 uppercase">
            <th class="pb-2">Votant</th>
            <th class="pb-2">Choix</th>
            <th class="pb-2 text-right">Millièmes</th>
            <th class="pb-2 text-right">Date</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-50">
          {#each votes as vote}
            <tr>
              <td class="py-1.5">
                <span class="text-gray-900">{vote.owner_name || vote.owner_id.slice(0, 8)}</span>
                {#if vote.proxy_owner_id}
                  <span class="text-xs text-gray-400 ml-1">(procuration)</span>
                {/if}
              </td>
              <td class="py-1.5">
                <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {getChoiceColor(vote.choice)}">
                  {getChoiceLabel(vote.choice)}
                </span>
              </td>
              <td class="py-1.5 text-right text-gray-600">{vote.voting_power}</td>
              <td class="py-1.5 text-right text-xs text-gray-400">{formatDate(vote.created_at)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if showVotes && votes.length === 0}
    <p class="mt-3 text-xs text-gray-400 text-center">Aucun vote enregistré</p>
  {/if}
</div>
