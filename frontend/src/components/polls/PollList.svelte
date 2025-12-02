<script lang="ts">
  import { onMount } from "svelte";
  import {
    pollsApi,
    type Poll,
    PollStatus,
    PollType,
  } from "../../lib/api/polls";
  import PollStatusBadge from "./PollStatusBadge.svelte";
  import PollTypeBadge from "./PollTypeBadge.svelte";

  export let buildingId: string;
  export let showOnlyActive = false;

  let polls: Poll[] = [];
  let filteredPolls: Poll[] = [];
  let loading = true;
  let error = "";
  let statusFilter: PollStatus | "all" = "all";

  onMount(async () => {
    await loadPolls();
  });

  async function loadPolls() {
    try {
      loading = true;
      error = "";
      if (showOnlyActive) {
        polls = await pollsApi.listActive(buildingId);
      } else {
        polls = await pollsApi.list({ building_id: buildingId });
      }
      applyFilters();
    } catch (err: any) {
      error = err.message || "Erreur lors du chargement des sondages";
      console.error("Failed to load polls:", err);
    } finally {
      loading = false;
    }
  }

  function applyFilters() {
    filteredPolls = polls.filter((poll) => {
      if (statusFilter === "all") return true;
      return poll.status === statusFilter;
    });
  }

  $: if (statusFilter) applyFilters();

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString("fr-BE", {
      day: "2-digit",
      month: "2-digit",
      year: "numeric",
    });
  }

  function getParticipationColor(rate: number): string {
    if (rate >= 50) return "text-green-600";
    if (rate >= 30) return "text-yellow-600";
    return "text-red-600";
  }

  function calculateParticipationRate(poll: Poll): number {
    if (poll.total_eligible_voters === 0) return 0;
    return (poll.total_votes_cast / poll.total_eligible_voters) * 100;
  }
</script>

<div class="bg-white shadow-md rounded-lg">
  <div class="px-4 py-5 border-b border-gray-200 sm:px-6">
    <div class="flex items-center justify-between">
      <h3 class="text-lg leading-6 font-medium text-gray-900">
        🗳️ Sondages (Consultations)
      </h3>
      <a
        href="/polls/new?building={buildingId}"
        class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700"
      >
        <span class="mr-2">➕</span>
        Nouveau sondage
      </a>
    </div>
    <p class="mt-1 text-sm text-gray-500">
      Consultez les copropriétaires entre les assemblées générales (Art.
      577-8/4 §4 Code Civil Belge).
    </p>
  </div>

  <!-- Filters -->
  {#if !showOnlyActive}
    <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
      <div class="flex items-center space-x-4">
        <label class="text-sm font-medium text-gray-700">Statut:</label>
        <select
          bind:value={statusFilter}
          class="text-sm rounded-md border-gray-300 focus:border-indigo-500 focus:ring-indigo-500"
        >
          <option value="all">Tous</option>
          <option value={PollStatus.Draft}>Brouillon</option>
          <option value={PollStatus.Active}>En cours</option>
          <option value={PollStatus.Closed}>Clôturé</option>
          <option value={PollStatus.Cancelled}>Annulé</option>
        </select>
      </div>
    </div>
  {/if}

  {#if loading}
    <div class="p-8 text-center">
      <div
        class="inline-block animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-600"
      ></div>
      <p class="mt-2 text-sm text-gray-500">Chargement des sondages...</p>
    </div>
  {:else if error}
    <div class="p-4 m-4 bg-red-50 border border-red-200 rounded-md">
      <p class="text-sm text-red-800">❌ {error}</p>
      <button
        on:click={loadPolls}
        class="mt-2 text-sm text-red-600 hover:text-red-800 underline"
      >
        Réessayer
      </button>
    </div>
  {:else if filteredPolls.length === 0}
    <div class="p-8 text-center">
      <p class="text-gray-500">Aucun sondage trouvé</p>
      <p class="mt-2 text-sm text-gray-400">
        Créez un sondage pour consulter les copropriétaires sur une décision
        entre les assemblées générales.
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each filteredPolls as poll}
        <li class="hover:bg-gray-50">
          <a href="/polls/{poll.id}" class="block px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between">
              <div class="flex-1 min-w-0">
                <div class="flex items-center space-x-3 mb-2">
                  <h4
                    class="text-sm font-medium text-indigo-600 truncate max-w-md"
                  >
                    {poll.question}
                  </h4>
                  <PollTypeBadge type={poll.poll_type} />
                  <PollStatusBadge status={poll.status} />
                  {#if poll.is_anonymous}
                    <span
                      class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-200 text-gray-700"
                      title="Vote anonyme"
                    >
                      🔒 Anonyme
                    </span>
                  {/if}
                </div>

                {#if poll.description}
                  <p class="mt-1 text-sm text-gray-500 truncate max-w-2xl">
                    {poll.description}
                  </p>
                {/if}

                <div class="mt-2 flex items-center text-sm text-gray-500">
                  <!-- Participation -->
                  <span
                    class="font-medium {getParticipationColor(
                      calculateParticipationRate(poll),
                    )}"
                  >
                    📊 {poll.total_votes_cast}/{poll.total_eligible_voters}
                    votes ({calculateParticipationRate(poll).toFixed(1)}%)
                  </span>

                  <!-- Dates -->
                  {#if poll.starts_at && poll.ends_at}
                    <span class="mx-2">•</span>
                    <span>
                      📅 {formatDate(poll.starts_at)} → {formatDate(
                        poll.ends_at,
                      )}
                    </span>
                  {/if}

                  <!-- Created by -->
                  <span class="mx-2">•</span>
                  <span class="text-xs text-gray-400">
                    Créé le {formatDate(poll.created_at)}
                  </span>
                </div>

                <!-- Options preview (for multiple choice) -->
                {#if poll.poll_type === PollType.MultipleChoice && poll.options.length > 0}
                  <div class="mt-2 flex items-center space-x-2">
                    <span class="text-xs text-gray-500">Options:</span>
                    {#each poll.options.slice(0, 3) as option}
                      <span
                        class="inline-flex items-center px-2 py-0.5 rounded text-xs bg-gray-100 text-gray-700"
                      >
                        {option.option_text}
                      </span>
                    {/each}
                    {#if poll.options.length > 3}
                      <span class="text-xs text-gray-400">
                        +{poll.options.length - 3} autres
                      </span>
                    {/if}
                  </div>
                {/if}
              </div>

              <!-- Arrow -->
              <div class="ml-4">
                <svg
                  class="h-5 w-5 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M9 5l7 7-7 7"
                  />
                </svg>
              </div>
            </div>
          </a>
        </li>
      {/each}
    </ul>
  {/if}
</div>
