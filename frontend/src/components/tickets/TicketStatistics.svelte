<script lang="ts">
  import { onMount } from "svelte";
  import { _ } from '../../lib/i18n';
  import { ticketsApi, type TicketStatistics } from "../../lib/api/tickets";
  import { withLoadingState } from "../../lib/utils/error.utils";

  let stats: TicketStatistics | null = null;
  let loading = true;
  let error = "";

  onMount(async () => {
    await loadStatistics();
  });

  async function loadStatistics() {
    await withLoadingState({
      action: () => ticketsApi.getStatistics(),
      setLoading: (v) => loading = v,
      setError: (v) => error = v,
      onSuccess: (data) => stats = data,
      errorMessage: $_("tickets.statistics_load_failed"),
    });
  }

  function formatResolutionTime(hours: number | undefined): string {
    if (!hours) return $_("common.notAvailable");
    if (hours < 1) return `${Math.round(hours * 60)} ${$_("common.minutes")}`;
    if (hours < 24) return `${Math.round(hours)} ${$_("common.hours")}`;
    return `${Math.round(hours / 24)} ${$_("common.days")}`;
  }
</script>

<div class="bg-white shadow rounded-lg p-6" data-testid="ticket-statistics">
  <div class="flex items-center justify-between mb-4">
    <h2 class="text-lg font-semibold text-gray-900">{$_("tickets.statistics")}</h2>
    <button
      on:click={loadStatistics}
      class="text-sm text-blue-600 hover:text-blue-700"
      data-testid="ticket-stats-refresh-btn"
    >
      {$_("common.refresh")}
    </button>
  </div>

  {#if loading}
    <div class="text-center py-8 text-gray-500" data-testid="loading-spinner">{$_("tickets.loadingStatistics")}</div>
  {:else if stats}
    <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
      <!-- Total Tickets -->
      <div class="bg-gray-50 rounded-lg p-4" data-testid="ticket-stat-total">
        <dt class="text-sm font-medium text-gray-500 mb-1">{$_("tickets.totalTickets")}</dt>
        <dd class="text-2xl font-bold text-gray-900">{stats.total_tickets}</dd>
      </div>

      <!-- Open Tickets -->
      <div class="bg-blue-50 rounded-lg p-4" data-testid="ticket-stat-open">
        <dt class="text-sm font-medium text-blue-700 mb-1">{$_("tickets.statusOpen")}</dt>
        <dd class="text-2xl font-bold text-blue-900">{stats.open_tickets}</dd>
      </div>

      <!-- In Progress -->
      <div class="bg-yellow-50 rounded-lg p-4" data-testid="ticket-stat-in-progress">
        <dt class="text-sm font-medium text-yellow-700 mb-1">{$_("tickets.statusInProgress")}</dt>
        <dd class="text-2xl font-bold text-yellow-900">
          {stats.in_progress_tickets}
        </dd>
      </div>

      <!-- Resolved -->
      <div class="bg-green-50 rounded-lg p-4" data-testid="ticket-stat-resolved">
        <dt class="text-sm font-medium text-green-700 mb-1">{$_("tickets.statusResolved")}</dt>
        <dd class="text-2xl font-bold text-green-900">
          {stats.resolved_tickets}
        </dd>
      </div>

      <!-- Closed -->
      <div class="bg-gray-50 rounded-lg p-4" data-testid="ticket-stat-closed">
        <dt class="text-sm font-medium text-gray-500 mb-1">{$_("tickets.statusClosed")}</dt>
        <dd class="text-2xl font-bold text-gray-900">{stats.closed_tickets}</dd>
      </div>

      <!-- Assigned -->
      <div class="bg-purple-50 rounded-lg p-4" data-testid="ticket-stat-assigned">
        <dt class="text-sm font-medium text-purple-700 mb-1">{$_("tickets.statusAssigned")}</dt>
        <dd class="text-2xl font-bold text-purple-900">
          {stats.assigned_tickets}
        </dd>
      </div>

      <!-- Overdue -->
      <div class="bg-red-50 rounded-lg p-4" data-testid="ticket-stat-overdue">
        <dt class="text-sm font-medium text-red-700 mb-1">⚠️ {$_("tickets.overdue")}</dt>
        <dd class="text-2xl font-bold text-red-900">{stats.overdue_tickets}</dd>
      </div>

      <!-- Average Resolution Time -->
      <div class="bg-indigo-50 rounded-lg p-4" data-testid="ticket-stat-avg-resolution">
        <dt class="text-sm font-medium text-indigo-700 mb-1">
          {$_("tickets.avgResolution")}
        </dt>
        <dd class="text-2xl font-bold text-indigo-900">
          {formatResolutionTime(stats.average_resolution_time_hours)}
        </dd>
      </div>
    </dl>
  {:else}
    <div class="text-center py-8 text-gray-500">{$_("tickets.noStatistics")}</div>
  {/if}
</div>
