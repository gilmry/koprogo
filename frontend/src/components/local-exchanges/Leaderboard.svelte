<script lang="ts">
  import { onMount } from "svelte";
  import {
    localExchangesApi,
    type OwnerCreditBalance,
    participationLevelLabels,
    participationLevelColors,
    getCreditStatusColor,
  } from "../../lib/api/local-exchanges";

  export let buildingId: string;
  export let limit: number = 10;

  let leaderboard: OwnerCreditBalance[] = [];
  let loading: boolean = true;
  let error: string | null = null;

  async function loadLeaderboard() {
    try {
      loading = true;
      error = null;
      leaderboard = await localExchangesApi.getLeaderboard(buildingId, limit);
    } catch (err: any) {
      error = err.message || "Impossible de charger le classement";
      console.error("Error loading leaderboard:", err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadLeaderboard();
  });
</script>

<div class="bg-white shadow rounded-lg p-6">
  <h3 class="text-lg font-semibold text-gray-900 mb-4">
    🏆 Classement des Contributeurs
  </h3>

  {#if loading}
    <div class="text-center py-8">
      <div
        class="inline-block animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900"
      ></div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 rounded-md p-4">
      <p class="text-red-800">❌ {error}</p>
    </div>
  {:else if leaderboard.length === 0}
    <p class="text-gray-500 text-center py-8">
      Aucun contributeur pour le moment
    </p>
  {:else}
    <div class="space-y-3">
      {#each leaderboard as owner, index (owner.owner_id)}
        {@const rank = index + 1}
        {@const isTopThree = rank <= 3}
        {@const participationConfig = participationLevelColors[owner.participation_level]}
        {@const balanceColor = getCreditStatusColor(owner.credit_status)}

        <div
          class="flex items-center gap-4 p-4 rounded-lg {isTopThree
            ? 'bg-gradient-to-r from-yellow-50 to-amber-50 border border-yellow-200'
            : 'bg-gray-50'}"
        >
          <!-- Rank -->
          <div
            class="flex-shrink-0 w-12 h-12 rounded-full flex items-center justify-center font-bold text-xl {isTopThree
              ? 'bg-gradient-to-br from-yellow-400 to-amber-500 text-white shadow-md'
              : 'bg-gray-200 text-gray-700'}"
          >
            {#if rank === 1}
              🥇
            {:else if rank === 2}
              🥈
            {:else if rank === 3}
              🥉
            {:else}
              {rank}
            {/if}
          </div>

          <!-- Owner Info -->
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2 mb-1">
              <p class="font-semibold text-gray-900 truncate">
                {owner.owner_name}
              </p>
              <span
                class="inline-flex items-center px-2 py-0.5 rounded-full text-xs font-medium {participationConfig.bg} {participationConfig.text}"
              >
                {participationLevelLabels[owner.participation_level]}
              </span>
            </div>
            <div class="flex items-center gap-3 text-sm text-gray-600">
              <span class="font-medium {balanceColor}">
                {owner.balance > 0 ? "+" : ""}{owner.balance}h
              </span>
              <span>📊 {owner.total_exchanges} échanges</span>
              {#if owner.average_rating}
                <span>
                  ⭐ {owner.average_rating.toFixed(1)}
                </span>
              {/if}
            </div>
          </div>

          <!-- Balance Badge -->
          <div class="flex-shrink-0 text-right">
            <p class="text-lg font-bold {balanceColor}">
              {owner.balance > 0 ? "+" : ""}{owner.balance}
            </p>
            <p class="text-xs text-gray-500">crédits</p>
          </div>
        </div>
      {/each}
    </div>

    <p class="mt-4 text-xs text-gray-500 text-center">
      Classement basé sur le solde de crédits (heures données - heures reçues)
    </p>
  {/if}
</div>
