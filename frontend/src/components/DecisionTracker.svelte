<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { BoardDecisionResponse } from '../lib/types';

  export let buildingId: string = '';
  export let filterStatus: string = '';

  let decisions: BoardDecisionResponse[] = [];
  let loading = true;
  let error = '';
  let statusFilter = filterStatus || 'all';

  $: statusOptions = [
    { value: 'all', label: $_('common.all') },
    { value: 'pending', label: $_('board.status.pending') },
    { value: 'in_progress', label: $_('board.status.inProgress') },
    { value: 'completed', label: $_('board.status.completed') },
    { value: 'overdue', label: $_('board.status.overdue') },
    { value: 'cancelled', label: $_('board.status.cancelled') }
  ];

  onMount(() => {
    if (!buildingId) {
      error = $_('board.error.buildingIdMissing');
      loading = false;
      return;
    }
    loadDecisions();
  });

  async function loadDecisions() {
    try {
      loading = true;
      error = '';

      let endpoint = `/board-decisions/building/${buildingId}`;
      if (statusFilter !== 'all') {
        endpoint = `/board-decisions/building/${buildingId}/status/${statusFilter}`;
      }

      decisions = await api.get<BoardDecisionResponse[]>(endpoint);
    } catch (e) {
      error = e instanceof Error ? e.message : $_('board.error.loadDecisions');
      console.error('Error loading decisions:', e);
      toast.error(error);
    } finally {
      loading = false;
    }
  }

  async function updateDecisionStatus(decisionId: string, newStatus: string) {
    try {
      await api.put(`/board-decisions/${decisionId}/status`, { status: newStatus });
      toast.success($_('board.success.statusUpdated'));
      loadDecisions();
    } catch (e) {
      toast.error($_('board.error.updateStatus'));
      console.error('Error updating status:', e);
    }
  }

  async function completeDecision(decisionId: string) {
    try {
      await api.put(`/board-decisions/${decisionId}/complete`, {});
      toast.success($_('board.success.decisionCompleted'));
      loadDecisions();
    } catch (e) {
      toast.error($_('board.error.completeDecision'));
      console.error('Error completing decision:', e);
    }
  }

  function getStatusColor(status: string): string {
    const colors: Record<string, string> = {
      'pending': 'bg-blue-100 text-blue-800 border-blue-300',
      'in_progress': 'bg-yellow-100 text-yellow-800 border-yellow-300',
      'completed': 'bg-green-100 text-green-800 border-green-300',
      'overdue': 'bg-red-100 text-red-800 border-red-300',
      'cancelled': 'bg-gray-100 text-gray-800 border-gray-300'
    };
    return colors[status] || 'bg-gray-100 text-gray-800';
  }

  function getStatusLabel(status: string): string {
    const labels: Record<string, string> = {
      'pending': $_('board.status.pending'),
      'in_progress': $_('board.status.inProgress'),
      'completed': $_('board.status.completed'),
      'overdue': $_('board.status.overdue'),
      'cancelled': $_('board.status.cancelled')
    };
    return labels[status] || status;
  }

  function getStatusIcon(status: string): string {
    const icons: Record<string, string> = {
      'pending': '⏳',
      'in_progress': '🔄',
      'completed': '✅',
      'overdue': '🚨',
      'cancelled': '❌'
    };
    return icons[status] || '📋';
  }

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function isOverdue(decision: BoardDecisionResponse): boolean {
    if (!decision.deadline) return false;
    const deadline = new Date(decision.deadline);
    return deadline < new Date() && decision.status !== 'completed' && decision.status !== 'cancelled';
  }

  function getDaysUntilDeadline(deadlineStr: string): number {
    const deadline = new Date(deadlineStr);
    const today = new Date();
    const diffTime = deadline.getTime() - today.getTime();
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  function handleStatusFilterChange() {
    loadDecisions();
  }
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
  <div class="px-6 py-4 border-b border-gray-200">
    <div class="flex items-center justify-between">
      <div>
        <h2 class="text-xl font-semibold text-gray-900">
          {$_('board.decisionTracker')}
        </h2>
        <p class="mt-1 text-sm text-gray-600">
          {$_('board.decisionCount', { values: { count: decisions.length } })}
        </p>
      </div>
      <div class="flex items-center space-x-4">
        <label class="text-sm font-medium text-gray-700">
          {$_('board.filterByStatus')}:
        </label>
        <select
          bind:value={statusFilter}
          on:change={handleStatusFilterChange}
          class="rounded-md border-gray-300 shadow-sm focus:border-primary-500 focus:ring-primary-500"
        >
          {#each statusOptions as option}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">{$_('common.loading')}</p>
      </div>
    </div>
  {:else if error}
    <div class="p-6">
      <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded relative" role="alert">
        <strong class="font-bold">Erreur :</strong>
        <span class="block sm:inline">{error}</span>
      </div>
    </div>
  {:else if decisions.length === 0}
    <div class="p-12 text-center">
      <span class="text-6xl">📋</span>
      <h3 class="mt-4 text-lg font-medium text-gray-900">{$_('board.noDecisions')}</h3>
      <p class="mt-2 text-sm text-gray-600">
        {statusFilter === 'all'
          ? $_('board.noDecisionsToTrack')
          : $_('board.noDecisionsWithStatus', { values: { status: getStatusLabel(statusFilter) } })}
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each decisions as decision}
        <li class="px-6 py-4 hover:bg-gray-50">
          <div class="flex items-start justify-between">
            <div class="flex items-start flex-1">
              <span class="text-2xl mr-4">{getStatusIcon(decision.status)}</span>
              <div class="flex-1">
                <div class="flex items-center">
                  <h3 class="text-lg font-medium text-gray-900">
                    {decision.subject}
                  </h3>
                  <span class="ml-3 inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium {getStatusColor(decision.status)}">
                    {getStatusLabel(decision.status)}
                  </span>
                </div>
                <p class="mt-2 text-sm text-gray-700">
                  {decision.decision_text}
                </p>
                <div class="mt-3 flex flex-wrap items-center gap-4 text-sm text-gray-600">
                  <p>
                    <strong>AG :</strong> {formatDate(decision.meeting_date || decision.created_at)}
                  </p>
                  {#if decision.deadline}
                    <p class:text-red-600={isOverdue(decision)}>
                      <strong>Deadline :</strong> {formatDate(decision.deadline)}
                      {#if !isOverdue(decision) && decision.status !== 'completed' && decision.status !== 'cancelled'}
                        <span class="ml-1 text-xs">
                          (dans {getDaysUntilDeadline(decision.deadline)} jours)
                        </span>
                      {/if}
                    </p>
                  {/if}
                  {#if decision.completed_at}
                    <p class="text-green-600">
                      <strong>Terminée le :</strong> {formatDate(decision.completed_at)}
                    </p>
                  {/if}
                </div>

                {#if decision.notes}
                  <div class="mt-3 bg-gray-50 border border-gray-200 rounded-md p-3">
                    <p class="text-xs font-medium text-gray-700 mb-1">Notes de suivi :</p>
                    <p class="text-sm text-gray-600">{decision.notes}</p>
                  </div>
                {/if}
              </div>
            </div>

            <div class="ml-4 flex-shrink-0 flex flex-col space-y-2">
              {#if decision.status === 'pending'}
                <button
                  on:click={() => updateDecisionStatus(decision.id, 'in_progress')}
                  class="px-3 py-1.5 text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 rounded-md"
                >
                  Démarrer
                </button>
              {:else if decision.status === 'in_progress'}
                <button
                  on:click={() => completeDecision(decision.id)}
                  class="px-3 py-1.5 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md"
                >
                  Terminer
                </button>
              {/if}
            </div>
          </div>

          {#if isOverdue(decision)}
            <div class="mt-3 bg-red-50 border border-red-200 rounded-md p-3">
              <div class="flex">
                <span class="text-lg mr-2">🚨</span>
                <p class="text-sm text-red-800">
                  <strong>Attention :</strong> Cette décision est en retard.
                  La deadline était le {formatDate(decision.deadline)}.
                </p>
              </div>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  <div class="px-6 py-4 bg-gray-50 border-t border-gray-200">
    <p class="text-sm text-gray-600">
      <strong>Rôle du conseil :</strong> Le conseil de copropriété surveille l'exécution des décisions de l'AG par le syndic et peut demander des comptes.
    </p>
  </div>
</div>
