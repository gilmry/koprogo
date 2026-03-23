<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import {
    pollsApi,
    type Poll,
    type PollResults as PollResultsType,
    PollType,
    PollStatus,
  } from "../../lib/api/polls";
  import PollStatusBadge from "./PollStatusBadge.svelte";
  import PollTypeBadge from "./PollTypeBadge.svelte";
  import PollResults from "./PollResults.svelte";
  import { toast } from "../../stores/toast";

  export let pollId: string;
  export let isAdmin = false;

  let poll: Poll | null = null;
  let results: PollResultsType | null = null;
  let loading = true;
  let error = "";

  // Voting state
  let selectedOptionId: string | null = null;
  let selectedOptions: Set<string> = new Set();
  let ratingValue: number | null = null;
  let openEndedText = "";
  let votingInProgress = false;
  let votingError = "";
  let votingSuccess = false;
  let hasVoted = false;

  onMount(async () => {
    await loadPoll();
  });

  async function loadPoll() {
    try {
      loading = true;
      error = "";
      poll = await pollsApi.getById(pollId);

      // Load results if closed or active (to show live results)
      if (poll.status === PollStatus.Closed || poll.status === PollStatus.Active) {
        try {
          results = await pollsApi.getResults(pollId);
        } catch {
          // Results may not be available yet
        }
      }
    } catch (err: any) {
      error = err.message || $_("polls.detail.loadingError");
      console.error("Failed to load poll:", err);
    } finally {
      loading = false;
    }
  }

  async function handleVote() {
    if (!poll) return;

    votingInProgress = true;
    votingError = "";
    votingSuccess = false;

    try {
      // Build vote data based on poll type
      let voteData: any = { poll_id: poll.id };

      if (poll.poll_type === PollType.YesNo || poll.poll_type === PollType.MultipleChoice) {
        if (poll.allow_multiple_votes) {
          if (selectedOptions.size === 0) {
            throw new Error($_("polls.detail.selectAtLeastOne"));
          }
          voteData.selected_option_ids = Array.from(selectedOptions);
        } else {
          if (!selectedOptionId) {
            throw new Error($_("polls.detail.selectOption"));
          }
          voteData.selected_option_ids = [selectedOptionId];
        }
      } else if (poll.poll_type === PollType.Rating) {
        if (ratingValue === null) {
          throw new Error($_("polls.detail.giveRating"));
        }
        voteData.rating_value = ratingValue;
      } else if (poll.poll_type === PollType.OpenEnded) {
        if (!openEndedText.trim()) {
          throw new Error($_("polls.detail.writeAnswer"));
        }
        voteData.open_text = openEndedText.trim();
      }

      await pollsApi.vote(voteData);
      votingSuccess = true;
      hasVoted = true;

      // Reload poll to update vote counts
      await loadPoll();

      // Reset form after 3 seconds
      setTimeout(() => {
        votingSuccess = false;
      }, 3000);
    } catch (err: any) {
      const msg = err.message || "";
      if (msg.includes("already voted") || msg.includes("déjà voté") || msg.includes("duplicate")) {
        hasVoted = true;
        votingError = $_("polls.detail.alreadyVoted");
      } else {
        votingError = msg || $_("polls.detail.votingError");
      }
      console.error("Failed to vote:", err);
    } finally {
      votingInProgress = false;
    }
  }

  async function handlePublish() {
    if (!poll || !confirm($_("polls.detail.publishConfirm"))) {
      return;
    }

    try {
      poll = await pollsApi.publish(poll.id, {
        starts_at: poll.starts_at,
        ends_at: poll.ends_at,
      });
      toast.success($_("polls.detail.publishSuccess"));
    } catch (err: any) {
      toast.error($_("polls.detail.publishError") + ": " + err.message);
    }
  }

  async function handleClose() {
    if (!poll || !confirm($_("polls.detail.closeConfirm"))) {
      return;
    }

    try {
      poll = await pollsApi.close(poll.id);
      await loadPoll(); // Reload to get results
      toast.success($_("polls.detail.closeSuccess"));
    } catch (err: any) {
      toast.error($_("polls.detail.closeError") + ": " + err.message);
    }
  }

  async function handleCancel() {
    if (!poll || !confirm($_("polls.detail.cancelConfirm"))) {
      return;
    }

    try {
      poll = await pollsApi.cancel(poll.id);
      toast.success($_("polls.detail.cancelSuccess"));
    } catch (err: any) {
      toast.error($_("polls.detail.cancelError") + ": " + err.message);
    }
  }

  function toggleMultipleOption(optionId: string) {
    if (selectedOptions.has(optionId)) {
      selectedOptions.delete(optionId);
    } else {
      selectedOptions.add(optionId);
    }
    selectedOptions = selectedOptions;
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  function calculateParticipationRate(p: Poll): number {
    if (p.total_eligible_voters === 0) return 0;
    return (p.total_votes_cast / p.total_eligible_voters) * 100;
  }

  function canVote(): boolean {
    return poll !== null && poll.status === PollStatus.Active && !hasVoted;
  }
</script>

{#if loading}
  <div class="p-8 text-center">
    <div
      class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-indigo-600"
    ></div>
    <p class="mt-4 text-gray-500">{$_("polls.detail.loading")}</p>
  </div>
{:else if error}
  <div class="p-4 bg-red-50 border border-red-200 rounded-md">
    <p class="text-sm text-red-800">❌ {error}</p>
    <button
      on:click={loadPoll}
      class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
    >
      {$_("common.retry")}
    </button>
  </div>
{:else if poll}
  <div class="space-y-6">
    <!-- Header -->
    <div class="bg-white shadow-md rounded-lg p-6">
      <div class="flex items-start justify-between">
        <div class="flex-1">
          <div class="flex items-center space-x-3 mb-2">
            <PollTypeBadge type={poll.poll_type} />
            <PollStatusBadge status={poll.status} />
            {#if poll.is_anonymous}
              <span
                class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-200 text-gray-700"
              >
                🔒 Anonyme
              </span>
            {/if}
          </div>
          <h2 class="text-2xl font-bold text-gray-900 mb-2">
            {poll.title}
          </h2>
          {#if poll.description}
            <p class="text-sm text-gray-600 mb-4">{poll.description}</p>
          {/if}

          <!-- Metadata -->
          <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
            <div class="p-3 bg-blue-50 rounded-lg">
              <div class="text-xs text-blue-600 font-medium">{$_("polls.detail.period")}</div>
              <div class="text-sm text-blue-900">
                {#if poll.starts_at && poll.ends_at}
                  {formatDate(poll.starts_at)} → {formatDate(poll.ends_at)}
                {:else}
                  {$_("polls.detail.notDefined")}
                {/if}
              </div>
            </div>
            <div class="p-3 bg-green-50 rounded-lg">
              <div class="text-xs text-green-600 font-medium">{$_("polls.detail.participation")}</div>
              <div class="text-sm text-green-900">
                {poll.total_votes_cast}/{poll.total_eligible_voters} {$_("polls.detail.votes")}
                ({calculateParticipationRate(poll).toFixed(1)}%)
              </div>
            </div>
            <div class="p-3 bg-purple-50 rounded-lg">
              <div class="text-xs text-purple-600 font-medium">{$_("common.created")}</div>
              <div class="text-sm text-purple-900">
                {formatDate(poll.created_at)}
              </div>
            </div>
          </div>
        </div>
        <a
          href="/polls"
          class="text-sm text-gray-600 hover:text-gray-800 underline ml-4"
        >
          ← {$_("common.back")}
        </a>
      </div>

      <!-- Admin Actions -->
      {#if isAdmin}
        <div class="mt-4 flex items-center space-x-3 pt-4 border-t border-gray-200">
          {#if poll.status === PollStatus.Draft}
            <button
              on:click={handlePublish}
              class="px-4 py-2 bg-green-600 text-white text-sm font-medium rounded-md hover:bg-green-700"
            >
              🚀 {$_("polls.detail.publish")}
            </button>
          {/if}
          {#if poll.status === PollStatus.Active}
            <button
              on:click={handleClose}
              class="px-4 py-2 bg-blue-600 text-white text-sm font-medium rounded-md hover:bg-blue-700"
            >
              ✅ {$_("polls.detail.close")}
            </button>
          {/if}
          {#if poll.status === PollStatus.Draft || poll.status === PollStatus.Active}
            <button
              on:click={handleCancel}
              class="px-4 py-2 bg-red-600 text-white text-sm font-medium rounded-md hover:bg-red-700"
            >
              ❌ {$_("common.cancel")}
            </button>
          {/if}
        </div>
      {/if}
    </div>

    <!-- Voting Section -->
    {#if canVote()}
      <div class="bg-white shadow-md rounded-lg p-6">
        <h3 class="text-lg font-medium text-gray-900 mb-4">🗳️ {$_("polls.detail.yourVote")}</h3>

        {#if votingSuccess}
          <div class="mb-4 p-4 bg-green-50 border border-green-200 rounded-md">
            <p class="text-sm text-green-800">
              ✅ {$_("polls.detail.voteSuccess")}
            </p>
          </div>
        {/if}

        {#if votingError}
          <div class="mb-4 p-4 bg-red-50 border border-red-200 rounded-md">
            <p class="text-sm text-red-800">❌ {votingError}</p>
          </div>
        {/if}

        <!-- Vote form based on poll type -->
        {#if poll.poll_type === PollType.YesNo || poll.poll_type === PollType.MultipleChoice}
          <div class="space-y-3">
            {#each poll.options as option}
              <label class="flex items-center p-3 border-2 rounded-lg cursor-pointer hover:bg-gray-50 {(poll.allow_multiple_votes ? selectedOptions.has(option.id) : selectedOptionId === option.id) ? 'border-indigo-500 bg-indigo-50' : 'border-gray-200'}">
                <input
                  type={poll.allow_multiple_votes ? "checkbox" : "radio"}
                  name="poll_option"
                  value={option.id}
                  checked={poll.allow_multiple_votes ? selectedOptions.has(option.id) : selectedOptionId === option.id}
                  on:change={() => {
                    if (poll.allow_multiple_votes) {
                      toggleMultipleOption(option.id);
                    } else {
                      selectedOptionId = option.id;
                    }
                  }}
                  class="mr-3"
                />
                <span class="text-sm font-medium text-gray-900">
                  {option.option_text}
                </span>
              </label>
            {/each}
          </div>
        {:else if poll.poll_type === PollType.Rating}
          <div>
            <div class="flex items-center justify-center space-x-2 mb-4">
              {#each Array(5) as _, i}
                {@const value = i + 1}
                <button
                  type="button"
                  on:click={() => (ratingValue = value)}
                  class="text-4xl transition-all {ratingValue !== null && ratingValue >= value ? 'text-yellow-400' : 'text-gray-300'} hover:text-yellow-300"
                >
                  ⭐
                </button>
              {/each}
            </div>
            {#if ratingValue !== null}
              <p class="text-center text-sm text-gray-600">
                Votre note: {ratingValue}/5
              </p>
            {/if}
          </div>
        {:else if poll.poll_type === PollType.OpenEnded}
          <div>
            <textarea
              bind:value={openEndedText}
              rows="5"
              placeholder="Écrivez votre réponse ici..."
              class="w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500"
            ></textarea>
            <p class="mt-1 text-xs text-gray-500">
              Partagez votre avis, suggestions ou commentaires.
            </p>
          </div>
        {/if}

        <button
          on:click={handleVote}
          disabled={votingInProgress}
          class="mt-4 w-full px-4 py-3 bg-indigo-600 text-white font-medium rounded-md hover:bg-indigo-700 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if votingInProgress}
            <span class="inline-block animate-spin mr-2">⏳</span>
            {$_("polls.detail.submitting")}
          {:else}
            ✅ {$_("polls.detail.vote")}
          {/if}
        </button>
      </div>
    {:else if hasVoted}
      <div class="bg-white shadow-md rounded-lg p-6">
        <div class="text-center py-4">
          <p class="text-lg text-green-600 font-medium">
            ✅ {$_("polls.detail.alreadyVoted")}
          </p>
          <p class="text-sm text-gray-500 mt-2">
            {$_("polls.detail.thankYou")}
          </p>
        </div>
      </div>
    {:else if poll.status === PollStatus.Draft}
      <div class="bg-white shadow-md rounded-lg p-6">
        <div class="text-center py-4">
          <p class="text-lg text-yellow-600 font-medium">
            📝 {$_("polls.detail.draftStatus")}
          </p>
          <p class="text-sm text-gray-500 mt-2">
            {$_("polls.detail.draftMessage")}
          </p>
        </div>
      </div>
    {:else if poll.status === PollStatus.Cancelled}
      <div class="bg-white shadow-md rounded-lg p-6">
        <div class="text-center py-4">
          <p class="text-lg text-red-600 font-medium">
            ❌ {$_("polls.detail.cancelledStatus")}
          </p>
        </div>
      </div>
    {/if}

    <!-- Results Section (only if closed) -->
    {#if poll.status === PollStatus.Closed && results}
      <PollResults {poll} {results} />
    {/if}
  </div>
{/if}
