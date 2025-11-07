<script lang="ts">
  import { onMount } from 'svelte';
  import { api } from '../lib/api';
  import { toast } from '../stores/toast';
  import type { BoardMemberResponse } from '../lib/types';

  export let buildingId: string = '';
  export let showInactive: boolean = false;

  let members: BoardMemberResponse[] = [];
  let loading = true;
  let error = '';

  onMount(() => {
    if (!buildingId) {
      error = 'ID de l\'immeuble manquant';
      loading = false;
      return;
    }
    loadMembers();
  });

  async function loadMembers() {
    try {
      loading = true;
      error = '';

      const endpoint = showInactive
        ? `/board-members/building/${buildingId}/all`
        : `/board-members/building/${buildingId}`;

      members = await api.get<BoardMemberResponse[]>(endpoint);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Erreur lors du chargement des membres';
      console.error('Error loading board members:', e);
      toast.error(error);
    } finally {
      loading = false;
    }
  }

  function getPositionLabel(position: string): string {
    const labels: Record<string, string> = {
      'president': 'Président',
      'treasurer': 'Trésorier',
      'secretary': 'Secrétaire',
      'member': 'Membre'
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

  function formatDate(dateStr: string): string {
    return new Date(dateStr).toLocaleDateString('fr-FR', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    });
  }

  function getMandateStatusColor(member: BoardMemberResponse): string {
    if (!member.is_active) return 'bg-gray-100 text-gray-800 border-gray-300';
    if (member.expires_soon) return 'bg-orange-100 text-orange-800 border-orange-300';
    return 'bg-green-100 text-green-800 border-green-300';
  }

  function getMandateStatusText(member: BoardMemberResponse): string {
    if (!member.is_active) return 'Inactif';
    if (member.expires_soon) return `Expire dans ${member.days_remaining} jours`;
    return 'Actif';
  }
</script>

<div class="bg-white shadow rounded-lg overflow-hidden">
  <div class="px-6 py-4 border-b border-gray-200">
    <h2 class="text-xl font-semibold text-gray-900">
      Membres du Conseil de Copropriété
    </h2>
    <p class="mt-1 text-sm text-gray-600">
      {members.length} membre{members.length > 1 ? 's' : ''} {showInactive ? 'au total' : 'actif(s)'}
    </p>
  </div>

  {#if loading}
    <div class="flex items-center justify-center py-12">
      <div class="text-center">
        <div class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-primary-600"></div>
        <p class="mt-4 text-gray-600">Chargement des membres...</p>
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
      <h3 class="mt-4 text-lg font-medium text-gray-900">Aucun membre du conseil</h3>
      <p class="mt-2 text-sm text-gray-600">
        Le conseil de copropriété n'a pas encore été élu.
        {#if !showInactive}
        <br />
        Les immeubles de plus de 20 lots doivent obligatoirement avoir un conseil.
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
                    <strong>Mandat :</strong>
                    {formatDate(member.mandate_start)} → {formatDate(member.mandate_end)}
                  </p>
                  <p>
                    <strong>Élu lors de :</strong> AG du {formatDate(member.elected_at)}
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
                  Le mandat expire bientôt. Pensez à organiser une nouvelle élection lors de la prochaine AG.
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
        <strong>Note légale :</strong> Le conseil de copropriété est obligatoire pour les immeubles de plus de 20 lots (Article 577-8/4 Code Civil belge).
      </p>
      <button
        on:click={() => { showInactive = !showInactive; loadMembers(); }}
        class="text-sm text-primary-600 hover:text-primary-800 font-medium"
      >
        {showInactive ? 'Masquer' : 'Afficher'} les anciens membres
      </button>
    </div>
  </div>
</div>
