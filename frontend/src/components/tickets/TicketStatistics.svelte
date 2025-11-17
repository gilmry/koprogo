<script lang="ts">
  import { onMount } from "svelte";
  import { ticketsApi, type TicketStatistics } from "../../lib/api/tickets";
  import { addToast } from "../../stores/toast";

  let stats: TicketStatistics | null = null;
  let loading = true;

  onMount(async () => {
    await loadStatistics();
  });

  async function loadStatistics() {
    try {
      loading = true;
      stats = await ticketsApi.getStatistics();
    } catch (err: any) {
      addToast({
        message: err.message || "Failed to load ticket statistics",
        type: "error",
      });
    } finally {
      loading = false;
    }
  }

  function formatResolutionTime(hours: number | undefined): string {
    if (!hours) return "N/A";
    if (hours < 1) return `${Math.round(hours * 60)} minutes`;
    if (hours < 24) return `${Math.round(hours)} hours`;
    return `${Math.round(hours / 24)} days`;
  }
</script>

<div class="bg-white shadow rounded-lg p-6">
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-lg font-semibold text-gray-900">Ticket Statistics</h2>
    <button
      on:click={loadStatistics}
      class="text-sm text-blue-600 hover:text-blue-700"
    >
      Refresh
    </button>
  </div>

  {#if loading}
    <div class="text-center py-8 text-gray-500">Loading statistics...</div>
  {:else if stats}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Tickets -->
      <div class="bg-gray-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-gray-500 mb-1">Total Tickets</dt>
        <dd class="text-2xl font-bold text-gray-900">{stats.total_tickets}</dd>
      </div>

      <!-- Open Tickets -->
      <div class="bg-blue-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-blue-700 mb-1">Open</dt>
        <dd class="text-2xl font-bold text-blue-900">{stats.open_tickets}</dd>
      </div>

      <!-- In Progress -->
      <div class="bg-yellow-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-yellow-700 mb-1">In Progress</dt>
        <dd class="text-2xl font-bold text-yellow-900">
          {stats.in_progress_tickets}
        </dd>
      </div>

      <!-- Resolved -->
      <div class="bg-green-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-green-700 mb-1">Resolved</dt>
        <dd class="text-2xl font-bold text-green-900">
          {stats.resolved_tickets}
        </dd>
      </div>

      <!-- Closed -->
      <div class="bg-gray-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-gray-500 mb-1">Closed</dt>
        <dd class="text-2xl font-bold text-gray-900">{stats.closed_tickets}</dd>
      </div>

      <!-- Assigned -->
      <div class="bg-purple-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-purple-700 mb-1">Assigned</dt>
        <dd class="text-2xl font-bold text-purple-900">
          {stats.assigned_tickets}
        </dd>
      </div>

      <!-- Overdue -->
      <div class="bg-red-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-red-700 mb-1">⚠️ Overdue</dt>
        <dd class="text-2xl font-bold text-red-900">{stats.overdue_tickets}</dd>
      </div>

      <!-- Average Resolution Time -->
      <div class="bg-indigo-50 rounded-lg p-4">
        <dt class="text-sm font-medium text-indigo-700 mb-1">
          Avg Resolution
        </dt>
        <dd class="text-2xl font-bold text-indigo-900">
          {formatResolutionTime(stats.average_resolution_time_hours)}
        </dd>
      </div>
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">No statistics available</div>
  {/if}
</div>
