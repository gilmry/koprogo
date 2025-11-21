<script lang="ts">
  import { type Poll, type PollResults, PollType } from "../../lib/api/polls";

  export let poll: Poll;
  export let results: PollResults;

  function getWinnerColor(optionId: string): string {
    if (results.winner && results.winner.option_id === optionId) {
      return "border-green-500 bg-green-50";
    }
    return "border-gray-200";
  }

  function getRatingStars(rating: number, maxRating: number): string {
    const fullStars = Math.floor(rating);
    const partialStar = rating - fullStars;
    let stars = "";
    for (let i = 0; i < fullStars; i++) {
      stars += "⭐";
    }
    if (partialStar >= 0.5) {
      stars += "⭐";
    }
    for (let i = Math.ceil(rating); i < maxRating; i++) {
      stars += "☆";
    }
    return stars;
  }
</script>

<div class="bg-white shadow-md rounded-lg p-6">
  <div class="flex items-center justify-between mb-4">
    <h3 class="text-lg font-medium text-gray-900">📊 Résultats</h3>
    {#if results.winner}
      <span class="text-sm text-green-600 font-medium">
        🏆 Gagnant: {results.winner.option_text}
      </span>
    {/if}
  </div>

  <!-- Participation Summary -->
  <div class="mb-6 p-4 bg-gray-50 rounded-lg">
    <div class="grid grid-cols-2 gap-4">
      <div>
        <div class="text-xs text-gray-500">Total des votes</div>
        <div class="text-2xl font-bold text-gray-900">
          {results.total_votes}
        </div>
      </div>
      <div>
        <div class="text-xs text-gray-500">Taux de participation</div>
        <div
          class="text-2xl font-bold {results.participation_rate >= 50 ? 'text-green-600' : results.participation_rate >= 30 ? 'text-yellow-600' : 'text-red-600'}"
        >
          {results.participation_rate.toFixed(1)}%
        </div>
      </div>
    </div>
  </div>

  <!-- Results by Poll Type -->
  {#if poll.poll_type === PollType.YesNo || poll.poll_type === PollType.MultipleChoice}
    <div class="space-y-3">
      {#each results.results_by_option as optionResult}
        <div class="border-2 rounded-lg p-4 {getWinnerColor(optionResult.option_id)}">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium text-gray-900">
              {#if results.winner && results.winner.option_id === optionResult.option_id}
                🏆
              {/if}
              {optionResult.option_text}
            </span>
            <span class="text-sm text-gray-600">
              {optionResult.vote_count} votes
            </span>
          </div>
          <div class="w-full bg-gray-200 rounded-full h-2.5">
            <div
              class="bg-indigo-600 h-2.5 rounded-full transition-all duration-300"
              style="width: {optionResult.percentage}%"
            ></div>
          </div>
          <div class="mt-1 text-right text-xs text-gray-500">
            {optionResult.percentage.toFixed(1)}%
          </div>
        </div>
      {/each}
    </div>
  {:else if poll.poll_type === PollType.Rating}
    <div class="space-y-4">
      <!-- Average Rating -->
      <div class="text-center p-6 bg-gradient-to-r from-yellow-50 to-orange-50 rounded-lg">
        <div class="text-sm text-gray-600 mb-2">Note moyenne</div>
        <div class="text-5xl mb-2">
          {getRatingStars(results.average_rating || 0, poll.max_rating || 5)}
        </div>
        <div class="text-3xl font-bold text-gray-900">
          {results.average_rating?.toFixed(2)} / {poll.max_rating}
        </div>
      </div>

      <!-- Rating Distribution (if available) -->
      {#if results.results_by_option && results.results_by_option.length > 0}
        <div>
          <h4 class="text-sm font-medium text-gray-700 mb-3">
            Distribution des notes
          </h4>
          <div class="space-y-2">
            {#each results.results_by_option as optionResult}
              <div class="flex items-center space-x-3">
                <span class="text-sm text-gray-600 w-20">
                  {optionResult.option_text} ⭐
                </span>
                <div class="flex-1 bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-yellow-400 h-2 rounded-full"
                    style="width: {optionResult.percentage}%"
                  ></div>
                </div>
                <span class="text-sm text-gray-600 w-16 text-right">
                  {optionResult.vote_count} ({optionResult.percentage.toFixed(
                    1,
                  )}%)
                </span>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </div>
  {:else if poll.poll_type === PollType.OpenEnded}
    <div>
      <h4 class="text-sm font-medium text-gray-700 mb-3">
        Réponses textuelles ({results.text_responses?.length || 0})
      </h4>
      {#if results.text_responses && results.text_responses.length > 0}
        <div class="space-y-3 max-h-96 overflow-y-auto">
          {#each results.text_responses as response, index}
            <div class="p-3 bg-gray-50 rounded-lg border border-gray-200">
              <div class="text-xs text-gray-500 mb-1">
                Réponse #{index + 1}
              </div>
              <p class="text-sm text-gray-700">{response}</p>
            </div>
          {/each}
        </div>
      {:else}
        <p class="text-sm text-gray-500 italic">
          Aucune réponse textuelle n'a été enregistrée.
        </p>
      {/if}
    </div>
  {/if}

  <!-- Legal Notice -->
  <div class="mt-6 p-4 bg-yellow-50 border border-yellow-200 rounded-md">
    <p class="text-xs text-yellow-800">
      ⚖️ <strong>Cadre légal:</strong> Ces résultats doivent être documentés dans
      le procès-verbal de la prochaine assemblée générale conformément à l'Article
      577-8/4 §4 du Code Civil Belge.
    </p>
  </div>
</div>
