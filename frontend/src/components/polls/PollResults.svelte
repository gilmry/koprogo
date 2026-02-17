<script lang="ts">
  import { type Poll, type PollResults, PollType } from "../../lib/api/polls";

  export let poll: Poll;
  export let results: PollResults;

  function getWinnerColor(optionId: string): string {
    if (results.winning_option && results.winning_option.id === optionId) {
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
    {#if results.winning_option}
      <span class="text-sm text-green-600 font-medium">
        🏆 Gagnant: {results.winning_option.option_text}
      </span>
    {/if}
  </div>

  <!-- Participation Summary -->
  <div class="mb-6 p-4 bg-gray-50 rounded-lg">
    <div class="grid grid-cols-2 gap-4">
      <div>
        <div class="text-xs text-gray-500">Total des votes</div>
        <div class="text-2xl font-bold text-gray-900">
          {results.total_votes_cast}
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
      {#each results.options as optionResult}
        <div class="border-2 rounded-lg p-4 {getWinnerColor(optionResult.id)}">
          <div class="flex items-center justify-between mb-2">
            <span class="font-medium text-gray-900">
              {#if results.winning_option && results.winning_option.id === optionResult.id}
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
              style="width: {optionResult.vote_percentage}%"
            ></div>
          </div>
          <div class="mt-1 text-right text-xs text-gray-500">
            {optionResult.vote_percentage.toFixed(1)}%
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
          {getRatingStars(0, 5)}
        </div>
        <div class="text-3xl font-bold text-gray-900">
          — / 5
        </div>
      </div>

      <!-- Rating Distribution (if available) -->
      {#if results.options && results.options.length > 0}
        <div>
          <h4 class="text-sm font-medium text-gray-700 mb-3">
            Distribution des notes
          </h4>
          <div class="space-y-2">
            {#each results.options as optionResult}
              <div class="flex items-center space-x-3">
                <span class="text-sm text-gray-600 w-20">
                  {optionResult.option_text} ⭐
                </span>
                <div class="flex-1 bg-gray-200 rounded-full h-2">
                  <div
                    class="bg-yellow-400 h-2 rounded-full"
                    style="width: {optionResult.vote_percentage}%"
                  ></div>
                </div>
                <span class="text-sm text-gray-600 w-16 text-right">
                  {optionResult.vote_count} ({optionResult.vote_percentage.toFixed(
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
      <p class="text-sm text-gray-500 italic">
        Les réponses textuelles sont disponibles dans les résultats détaillés.
      </p>
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
