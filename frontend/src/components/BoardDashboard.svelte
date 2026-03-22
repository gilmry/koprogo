<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from 'svelte-i18n';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { BoardDashboardResponse, DeadlineUrgency } from '../lib/types';

  export let buildingId: string = '';

  let dashboard: BoardDashboardResponse | null = null;
  let loading = true;
  let error = '';

  onMount(() => {
    // If buildingId is not provided as prop, try to get it from URL
    if (!buildingId) {
      const urlParams = new URLSearchParams(window.location.search);
      buildingId = urlParams.get('building_id') || '';
    }

    if (!buildingId) {
      error = $_('board.error.buildingIdMissing');
      loading = false;
      return;
    }
    loadDashboard();
  });

  async function loadDashboard() {
    try {
      loading = true;
      error = '';
      dashboard = await api.get<BoardDashboardResponse>(
        `/board-members/dashboard?building_id=${buildingId}`
      );
    } catch (e) {
      error = e instanceof Error ? e.message : $_('board.error.loadDashboard');
      console.error('Error loading board dashboard:', e);
      toast.error(error);
    } finally {
      loading = false;
    }
  }

  function getUrgencyColor(urgency: DeadlineUrgency): string {
    switch (urgency) {
      case 'critical':
        return 'bg-red-100 text-red-800 border-red-300';
      case 'high':
        return 'bg-orange-100 text-orange-800 border-orange-300';
      case 'medium':
        return 'bg-yellow-100 text-yellow-800 border-yellow-300';
    }
  }

  function getUrgencyIcon(urgency: DeadlineUrgency): string {
    switch (urgency) {
      case 'critical':
        return '🔴';
      case 'high':
        return '🟠';
      case 'medium':
        return '🟡';
    }
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR');
  }

  function formatDaysRemaining(days: number): string {
    if (days === 0) return $_('common.today');
    if (days === 1) return $_('common.tomorrow');
    if (days < 0) return `${$_('common.daysAgo', { values: { count: Math.abs(days) } })}`;
    return `${$_('common.daysLeft', { values: { count: days } })}`;
  }
</script>

<div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
  {#if loading}
    <div class="flex items-center justify-center min-h-screen">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement du tableau de bord...</p>
      </div>
    </div>
  {:else if error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative" role="alert">
      <strong class="font-bold">Erreur :</strong>
      <span class="block sm:inline">{error}</span>
    </div>
  {:else if dashboard}
    <!-- Header -->
    <div class="mb-8">
      <h1 class="text-3xl font-bold text-gray-900">{$_('board.title')}</h1>
      <p class="mt-2 text-gray-600">{$_('board.subtitle')}</p>
    </div>

    <!-- My Mandate Section -->
    {#if dashboard.my_mandate}
      <div class="bg-white shadow rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold text-gray-900 mb-4">{$_('board.myMandate')}</h2>
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <p class="text-sm text-gray-500">{$_('board.mandate.position')}</p>
            <p class="text-lg font-medium text-gray-900 capitalize">{dashboard.my_mandate.position}</p>
          </div>
          <div>
            <p class="text-sm text-gray-500">{$_('board.mandate.startDate')}</p>
            <p class="text-lg font-medium text-gray-900">{formatDate(dashboard.my_mandate.mandate_start)}</p>
          </div>
          <div>
            <p class="text-sm text-gray-500">{$_('board.mandate.endDate')}</p>
            <p class="text-lg font-medium text-gray-900">{formatDate(dashboard.my_mandate.mandate_end)}</p>
          </div>
        </div>

        {#if dashboard.my_mandate.expires_soon}
          <div class="mt-4 bg-orange-50 border border-orange-200 rounded-md p-4">
            <div class="flex">
              <div class="flex-shrink-0">
                <span class="text-2xl">⚠️</span>
              </div>
              <div class="ml-3">
                <h3 class="text-sm font-medium text-orange-800">{$_('board.mandate.expiringWarning')}</h3>
                <p class="mt-1 text-sm text-orange-700">
                  {$_('board.mandate.expiresSoon', { values: { days: dashboard.my_mandate.days_remaining } })}
                </p>
              </div>
            </div>
          </div>
        {:else}
          <div class="mt-4 flex items-center text-sm text-gray-600">
            <span class="text-green-500 mr-2">✓</span>
            {$_('board.mandate.active', { values: { days: dashboard.my_mandate.days_remaining } })}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Decision Statistics -->
    <div class="bg-white shadow rounded-lg p-6 mb-6">
      <h2 class="text-xl font-semibold text-gray-900 mb-4">{$_('board.decisionStats')}</h2>
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-4">
        <div class="text-center">
          <p class="text-3xl font-bold text-gray-900">{dashboard.decisions_stats.total_decisions}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.total')}</p>
        </div>
        <div class="text-center">
          <p class="text-3xl font-bold text-blue-600">{dashboard.decisions_stats.pending}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.pending')}</p>
        </div>
        <div class="text-center">
          <p class="text-3xl font-bold text-yellow-600">{dashboard.decisions_stats.in_progress}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.inProgress')}</p>
        </div>
        <div class="text-center">
          <p class="text-3xl font-bold text-green-600">{dashboard.decisions_stats.completed}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.completed')}</p>
        </div>
        <div class="text-center">
          <p class="text-3xl font-bold text-red-600">{dashboard.decisions_stats.overdue}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.overdue')}</p>
        </div>
        <div class="text-center">
          <p class="text-3xl font-bold text-gray-400">{dashboard.decisions_stats.cancelled}</p>
          <p class="text-sm text-gray-500">{$_('board.stats.cancelled')}</p>
        </div>
      </div>
    </div>

    <!-- Overdue Decisions -->
    {#if dashboard.overdue_decisions.length > 0}
      <div class="bg-red-50 border border-red-200 shadow rounded-lg p-6 mb-6">
        <h2 class="text-xl font-semibold text-red-900 mb-4">
          🚨 {$_('board.overdueDecisions', { values: { count: dashboard.overdue_decisions.length } })}
        </h2>
        <div class="space-y-3">
          {#each dashboard.overdue_decisions as decision}
            <div class="bg-white border border-red-300 rounded-md p-4">
              <h3 class="font-medium text-gray-900">{decision.subject}</h3>
              <p class="text-sm text-gray-600 mt-1">{decision.decision_text}</p>
              {#if decision.deadline}
                <p class="text-sm text-red-600 mt-2">
                  <strong>{$_('board.overdueDeadline')}:</strong> {formatDate(decision.deadline)}
                </p>
              {/if}
              <div class="mt-2">
                <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-red-100 text-red-800">
                  {decision.status}
                </span>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="bg-green-50 border border-green-200 shadow rounded-lg p-6 mb-6">
        <div class="flex items-center">
          <span class="text-2xl mr-3">✅</span>
          <div>
            <h2 class="text-xl font-semibold text-green-900">{$_('board.noOverdueDecisions')}</h2>
            <p class="text-sm text-green-700">{$_('board.excellentWork')}</p>
          </div>
        </div>
      </div>
    {/if}

    <!-- Upcoming Deadlines -->
    {#if dashboard.upcoming_deadlines.length > 0}
      <div class="bg-white shadow rounded-lg p-6">
        <h2 class="text-xl font-semibold text-gray-900 mb-4">
          📅 {$_('board.upcomingDeadlines', { values: { count: dashboard.upcoming_deadlines.length } })}
        </h2>
        <div class="space-y-3">
          {#each dashboard.upcoming_deadlines as alert}
            <div class="border {getUrgencyColor(alert.urgency)} rounded-md p-4">
              <div class="flex items-start">
                <span class="text-2xl mr-3">{getUrgencyIcon(alert.urgency)}</span>
                <div class="flex-1">
                  <h3 class="font-medium text-gray-900">{alert.subject}</h3>
                  <p class="text-sm mt-1">
                    <strong>{$_('board.deadline')}:</strong> {formatDate(alert.deadline)}
                  </p>
                  <p class="text-sm mt-1">
                    <strong>{formatDaysRemaining(alert.days_remaining)}</strong>
                  </p>
                </div>
                <div>
                  <span class="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium uppercase {getUrgencyColor(alert.urgency)}">
                    {alert.urgency}
                  </span>
                </div>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {:else}
      <div class="bg-gray-50 border border-gray-200 shadow rounded-lg p-6">
        <div class="flex items-center">
          <span class="text-2xl mr-3">✨</span>
          <div>
            <h2 class="text-xl font-semibold text-gray-900">{$_('board.noUpcomingDeadlines')}</h2>
            <p class="text-sm text-gray-600">{$_('board.noUrgentDecisions')}</p>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>
