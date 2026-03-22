<script lang="ts">
  import { _ } from 'svelte-i18n';
  import { onMount } from "svelte";
  import {
    localExchangesApi,
    type SelStatistics,
    exchangeTypeLabels,
    exchangeTypeIcons,
  } from "../../lib/api/local-exchanges";

  export let buildingId: string;

  let stats: SelStatistics | null = null;
  let loading: boolean = true;
  let error: string | null = null;

  async function loadStatistics() {
    try {
      loading = true;
      error = null;
      stats = await localExchangesApi.getStatistics(buildingId);
    } catch (err: any) {
      error = err.message || $_('exchanges.stats_load_error');
      console.error("Error loading statistics:", err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadStatistics();
  });
</script>

<div class="bg-white shadow rounded-lg p-6">
  <h3 class="text-lg font-semibold text-gray-900 mb-4">
    📊 {$_('exchanges.statistics_title')}
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
  {:else if stats}
    <div class="grid grid-cols-2 md:grid-cols-3 gap-4">
      <!-- Total Exchanges -->
      <div class="bg-blue-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-blue-700 mb-1">{$_('exchanges.stats_total')}</p>
        <p class="text-3xl font-bold text-blue-900">{stats.total_exchanges}</p>
      </div>

      <!-- Active Exchanges -->
      <div class="bg-green-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-green-700 mb-1">{$_('exchanges.stats_active')}</p>
        <p class="text-3xl font-bold text-green-900">
          {stats.active_exchanges}
        </p>
      </div>

      <!-- Completed Exchanges -->
      <div class="bg-purple-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-purple-700 mb-1">{$_('exchanges.stats_completed')}</p>
        <p class="text-3xl font-bold text-purple-900">
          {stats.completed_exchanges}
        </p>
      </div>

      <!-- Total Credits Exchanged -->
      <div class="bg-yellow-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-yellow-700 mb-1">
          {$_('exchanges.stats_credits')}
        </p>
        <p class="text-3xl font-bold text-yellow-900">
          {stats.total_credits_exchanged}h
        </p>
      </div>

      <!-- Active Participants -->
      <div class="bg-indigo-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-indigo-700 mb-1">{$_('exchanges.stats_participants')}</p>
        <p class="text-3xl font-bold text-indigo-900">
          {stats.active_participants}
        </p>
      </div>

      <!-- Average Rating -->
      <div class="bg-pink-50 p-4 rounded-lg">
        <p class="text-xs font-medium text-pink-700 mb-1">{$_('exchanges.stats_rating')}</p>
        <p class="text-3xl font-bold text-pink-900">
          {stats.average_exchange_rating
            ? `${stats.average_exchange_rating.toFixed(1)} ⭐`
            : "N/A"}
        </p>
      </div>
    </div>

    <!-- Most Popular Type -->
    {#if stats.most_popular_exchange_type}
      <div class="mt-6 p-4 bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg">
        <p class="text-sm font-medium text-gray-700 mb-2">
          {$_('exchanges.stats_popular_type')}
        </p>
        <div class="flex items-center gap-2">
          <span class="text-3xl">
            {exchangeTypeIcons[stats.most_popular_exchange_type]}
          </span>
          <span class="text-xl font-semibold text-gray-900">
            {exchangeTypeLabels[stats.most_popular_exchange_type]}
          </span>
        </div>
      </div>
    {/if}

    <!-- Impact Message -->
    <div class="mt-6 p-4 bg-green-50 border-l-4 border-green-400 text-sm text-green-800">
      <p>
        🌱 <strong>{$_('exchanges.impact_title')}</strong> {$_('exchanges.impact_message', { hours: stats.total_credits_exchanged })}
      </p>
    </div>
  {/if}
</div>
