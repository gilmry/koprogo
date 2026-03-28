<script lang="ts">
  import { onMount } from 'svelte';
  import { _ } from '../lib/i18n';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { BoardMemberResponse } from '../lib/types';
  import { formatDate } from "../lib/utils/date.utils";
  import { withErrorHandling } from "../lib/utils/error.utils";

  export let buildingId: string = '';
  export let showInactive: boolean = false;

  let members: BoardMemberResponse[] = [];
  let loading = true;
  let error = '';

  onMount(() => {
    if (!buildingId) {
      error = $_('board.error.buildingIdMissing');
      loading = false;
      return;
    }
    loadMembers();
  });

  async function loadMembers() {
    loading = true;
    error = '';
    const endpoint = showInactive
      ? `/board-members/building/${buildingId}/all`
      : `/board-members/building/${buildingId}`;
    const result = await withErrorHandling({
      action: () => api.get<BoardMemberResponse[]>(endpoint),
      errorMessage: $_('board.error.loadMembers'),
    });
    if (result) {
      members = result;
    } else {
      error = $_('board.error.loadMembers');
    }
    loading = false;
  }

  function getPositionLabel(position: string): string {
    const labels: Record<string, string> = {
      'president': $_('board.position.president'),
      'treasurer': $_('board.position.treasurer'),
      'secretary': $_('board.position.secretary'),
      'member': $_('board.position.member')
    };
    return labels[position] || position;
  }

  function getPositionIcon(position: string): string {
    const icons: Record<string, string> = {
      'president': '👑',
      'treasurer': '💰',
      'secretary': '📝',
      'member': '👤'
    };
    return icons[position] || '👤';
  }

  function getMandateStatusColor(member: BoardMemberResponse): string {
    if (!member.is_active) return 'bg-gray-100 text-gray-800 border-gray-300';
    if (member.expires_soon) return 'bg-orange-100 text-orange-800 border-orange-300';
    return 'bg-green-100 text-green-800 border-green-300';
  }

  function getMandateStatusText(member: BoardMemberResponse): string {
    if (!member.is_active) return $_('board.status.inactive');
    if (member.expires_soon) return $_('board.status.expiresSoon', { values: { days: member.days_remaining } });
    return $_('board.status.active');
  }
</script>

<div class="bg-white shadow rounded-lg overflow-hidden" data-testid="board-member-list">
  <div class="px-6 py-4 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      {$_('board.membersTitle')}
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      {$_('board.memberCount', { values: { count: members.length, status: showInactive ? 'total' : 'active' } })}
    </p>
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
  {:else if members.length === 0}
    <div class="p-12 text-center">
      <span class="text-6xl">🏛️</span>
      <h3 class="mt-4 text-lg font-medium text-gray-900">{$_('board.noMembers')}</h3>
      <p class="mt-2 text-sm text-gray-600">
        {$_('board.notYetElected')}
        {#if !showInactive}
        <br />
        {$_('board.mandatoryNote')}
        {/if}
      </p>
    </div>
  {:else}
    <ul class="divide-y divide-gray-200">
      {#each members as member}
        <li class="px-6 py-4 hover:bg-gray-50">
          <div class="flex items-center justify-between">
            <div class="flex items-center flex-1">
              <span class="text-3xl mr-4">{getPositionIcon(member.position)}</span>
              <div class="flex-1">
                <div class="flex items-center">
                  <h3 class="text-lg font-medium text-gray-900">
                    {member.owner_name}
                  </h3>
                  <span class="ml-3 inline-flex items-center px-3 py-0.5 rounded-full text-sm font-medium bg-primary-100 text-primary-800">
                    {getPositionLabel(member.position)}
                  </span>
                </div>
                <div class="mt-2 text-sm text-gray-600 space-y-1">
                  <p>
                    <strong>{$_('board.mandate.period')}:</strong>
                    {formatDate(member.mandate_start)} → {formatDate(member.mandate_end)}
                  </p>
                  <p>
                    <strong>{$_('board.electedAt')}:</strong> AG du {formatDate(member.elected_at)}
                  </p>
                </div>
              </div>
            </div>
            <div class="ml-4 flex-shrink-0">
              <span class="inline-flex items-center px-3 py-1.5 rounded-md text-sm font-medium border {getMandateStatusColor(member)}">
                {getMandateStatusText(member)}
              </span>
            </div>
          </div>

          {#if member.expires_soon && member.is_active}
            <div class="mt-3 bg-orange-50 border border-orange-200 rounded-md p-3">
              <div class="flex">
                <span class="text-lg mr-2">⚠️</span>
                <p class="text-sm text-orange-800">
                  {$_('board.expiresSoonWarning')}
                </p>
              </div>
            </div>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  <div class="px-6 py-4 bg-gray-50 border-t border-gray-200">
    <div class="flex items-center justify-between">
      <p class="text-sm text-gray-600">
        <strong>{$_('board.legalNote')}:</strong> {$_('board.legalRequirement')}
      </p>
      <button
        on:click={() => { showInactive = !showInactive; loadMembers(); }}
        class="text-sm text-primary-600 hover:text-primary-800 font-medium"
      >
        {showInactive ? $_('board.hideMembers') : $_('board.showMembers')}
      </button>
    </div>
  </div>
</div>
